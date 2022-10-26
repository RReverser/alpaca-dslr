use ascom_alpaca_rs::api::{Camera, Device};
use ascom_alpaca_rs::{
    ASCOMError, ASCOMErrorCode, ASCOMParams, ASCOMResult, DevicesBuilder, OpaqueResponse,
};
use gphoto2::camera::CameraEvent;
use gphoto2::file::CameraFilePath;
use gphoto2::widget::ToggleWidget;
use send_wrapper::SendWrapper;
use std::net::SocketAddr;
use std::rc::Rc;
use tokio::task::LocalSet;
use tokio::time::sleep;
use tower_http::trace::TraceLayer;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;

struct MyCamera {
    inner: SendWrapper<Rc<gphoto2::Camera>>,
    dimensions: (u32, u32),
    current_exposure: Option<tokio::task::JoinHandle<ASCOMResult<CameraFilePath>>>,
}

impl MyCamera {
    pub fn new(inner: gphoto2::Camera) -> anyhow::Result<Self> {
        let dimensions = {
            let span = tracing::trace_span!("Determine dimensions");
            let _enter = span.enter();

            tracing::trace!("Capturing test image");
            let camera_file_path = inner.capture_image()?;
            let folder = camera_file_path.folder();
            let name = camera_file_path.name();
            let fs = inner.fs();
            tracing::trace!("Downloading test image from the camera");
            let camera_file = fs.download(&folder, &name)?;
            tracing::trace!("Deleting test image from the camera");
            fs.delete_file(&folder, &name)?;

            let mime_type = camera_file.mime_type();
            let img_format = image::ImageFormat::from_mime_type(&mime_type)
                .ok_or_else(|| anyhow::anyhow!("unknown image format {mime_type}"))?;

            tracing::trace!(mime_type, ?img_format, "Test image format");

            image::io::Reader::with_format(
                std::io::Cursor::new(camera_file.get_data()?),
                img_format,
            )
            .into_dimensions()
        }?;

        tracing::info!(
            width = dimensions.0,
            height = dimensions.1,
            "Detected camera dimensions"
        );

        Ok(Self {
            inner: SendWrapper::new(Rc::new(inner)),
            dimensions,
            current_exposure: None,
        })
    }
}

impl std::ops::Deref for MyCamera {
    type Target = gphoto2::Camera;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

fn convert_err(err: impl std::string::ToString) -> ASCOMError {
    // TODO: more granular error codes.
    ASCOMError::new(ASCOMErrorCode::UNSPECIFIED, err.to_string())
}

#[allow(unused_variables)]
impl Device for MyCamera {
    fn ty(&self) -> &'static str {
        <dyn Camera>::TYPE
    }

    fn handle_action(
        &mut self,
        is_mut: bool,
        action: &str,
        params: ASCOMParams,
    ) -> ASCOMResult<OpaqueResponse> {
        <dyn Camera>::handle_action_impl(self, is_mut, action, params)
    }

    fn action(
        &mut self,
        action: String,
        parameters: String,
    ) -> ascom_alpaca_rs::ASCOMResult<String> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn command_blind(&mut self, command: String, raw: String) -> ascom_alpaca_rs::ASCOMResult {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn command_bool(&mut self, command: String, raw: String) -> ascom_alpaca_rs::ASCOMResult<bool> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn command_string(
        &mut self,
        command: String,
        raw: String,
    ) -> ascom_alpaca_rs::ASCOMResult<String> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn connected(&self) -> ascom_alpaca_rs::ASCOMResult<bool> {
        Ok(true)
    }

    fn set_connected(&mut self, connected: bool) -> ascom_alpaca_rs::ASCOMResult {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn description(&self) -> ascom_alpaca_rs::ASCOMResult<String> {
        self.inner.about().map_err(convert_err)
    }

    fn driver_info(&self) -> ascom_alpaca_rs::ASCOMResult<String> {
        Ok(env!("CARGO_PKG_DESCRIPTION").to_owned())
    }

    fn driver_version(&self) -> ascom_alpaca_rs::ASCOMResult<String> {
        Ok(env!("CARGO_PKG_VERSION").to_owned())
    }

    fn interface_version(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn name(&self) -> ascom_alpaca_rs::ASCOMResult<String> {
        Ok(self.inner.abilities().model().into_owned())
    }

    fn supported_actions(&self) -> ascom_alpaca_rs::ASCOMResult<Vec<String>> {
        Ok(vec![])
    }
}

#[allow(unused_variables)]
impl Camera for MyCamera {
    fn bayer_offset_x(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn bayer_offset_y(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn bin_x(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Ok(1)
    }

    fn set_bin_x(&mut self, bin_x: i32) -> ascom_alpaca_rs::ASCOMResult {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn bin_y(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Ok(1)
    }

    fn set_bin_y(&mut self, bin_y: i32) -> ascom_alpaca_rs::ASCOMResult {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn camera_state(
        &self,
    ) -> ascom_alpaca_rs::ASCOMResult<ascom_alpaca_rs::api::CameraStateResponse> {
        Ok(ascom_alpaca_rs::api::CameraStateResponse::Idle)
    }

    fn camera_xsize(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Ok(self.dimensions.0 as i32)
    }

    fn camera_ysize(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Ok(self.dimensions.1 as i32)
    }

    fn can_abort_exposure(&self) -> ascom_alpaca_rs::ASCOMResult<bool> {
        Ok(false)
    }

    fn can_asymmetric_bin(&self) -> ascom_alpaca_rs::ASCOMResult<bool> {
        Ok(false)
    }

    fn can_fast_readout(&self) -> ascom_alpaca_rs::ASCOMResult<bool> {
        Ok(false)
    }

    fn can_get_cooler_power(&self) -> ascom_alpaca_rs::ASCOMResult<bool> {
        Ok(false)
    }

    fn can_pulse_guide(&self) -> ascom_alpaca_rs::ASCOMResult<bool> {
        Ok(false)
    }

    fn can_set_ccdtemperature(&self) -> ascom_alpaca_rs::ASCOMResult<bool> {
        Ok(false)
    }

    fn can_stop_exposure(&self) -> ascom_alpaca_rs::ASCOMResult<bool> {
        Ok(false)
    }

    fn ccdtemperature(&self) -> ascom_alpaca_rs::ASCOMResult<f64> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn cooler_on(&self) -> ascom_alpaca_rs::ASCOMResult<bool> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn set_cooler_on(&mut self, cooler_on: bool) -> ascom_alpaca_rs::ASCOMResult {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn cooler_power(&self) -> ascom_alpaca_rs::ASCOMResult<f64> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn electrons_per_adu(&self) -> ascom_alpaca_rs::ASCOMResult<f64> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn exposure_max(&self) -> ascom_alpaca_rs::ASCOMResult<f64> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn exposure_min(&self) -> ascom_alpaca_rs::ASCOMResult<f64> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn exposure_resolution(&self) -> ascom_alpaca_rs::ASCOMResult<f64> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn fast_readout(&self) -> ascom_alpaca_rs::ASCOMResult<bool> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn set_fast_readout(&mut self, fast_readout: bool) -> ascom_alpaca_rs::ASCOMResult {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn full_well_capacity(&self) -> ascom_alpaca_rs::ASCOMResult<f64> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn gain(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn set_gain(&mut self, gain: i32) -> ascom_alpaca_rs::ASCOMResult {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn gain_max(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn gain_min(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn gains(&self) -> ascom_alpaca_rs::ASCOMResult<Vec<String>> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn has_shutter(&self) -> ascom_alpaca_rs::ASCOMResult<bool> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn heat_sink_temperature(&self) -> ascom_alpaca_rs::ASCOMResult<f64> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn image_array(
        &self,
    ) -> ascom_alpaca_rs::ASCOMResult<ascom_alpaca_rs::api::ImageArrayResponse> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn image_array_variant(
        &self,
    ) -> ascom_alpaca_rs::ASCOMResult<ascom_alpaca_rs::api::ImageArrayResponse> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn image_ready(&self) -> ascom_alpaca_rs::ASCOMResult<bool> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn is_pulse_guiding(&self) -> ascom_alpaca_rs::ASCOMResult<bool> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn last_exposure_duration(&self) -> ascom_alpaca_rs::ASCOMResult<f64> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn last_exposure_start_time(&self) -> ascom_alpaca_rs::ASCOMResult<String> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn max_adu(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn max_bin_x(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Ok(1)
    }

    fn max_bin_y(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Ok(1)
    }

    fn num_x(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn set_num_x(&mut self, num_x: i32) -> ascom_alpaca_rs::ASCOMResult {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn num_y(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn set_num_y(&mut self, num_y: i32) -> ascom_alpaca_rs::ASCOMResult {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn offset(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn set_offset(&mut self, offset: i32) -> ascom_alpaca_rs::ASCOMResult {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn offset_max(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn offset_min(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn offsets(&self) -> ascom_alpaca_rs::ASCOMResult<Vec<String>> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn percent_completed(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn pixel_size_x(&self) -> ascom_alpaca_rs::ASCOMResult<f64> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn pixel_size_y(&self) -> ascom_alpaca_rs::ASCOMResult<f64> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn readout_mode(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn set_readout_mode(&mut self, readout_mode: i32) -> ascom_alpaca_rs::ASCOMResult {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn readout_modes(&self) -> ascom_alpaca_rs::ASCOMResult<Vec<String>> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn sensor_name(&self) -> ascom_alpaca_rs::ASCOMResult<String> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn sensor_type(
        &self,
    ) -> ascom_alpaca_rs::ASCOMResult<ascom_alpaca_rs::api::SensorTypeResponse> {
        Ok(ascom_alpaca_rs::api::SensorTypeResponse::Color)
    }

    fn set_ccdtemperature(&self) -> ascom_alpaca_rs::ASCOMResult<f64> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn set_set_ccdtemperature(&mut self, set_ccdtemperature: f64) -> ascom_alpaca_rs::ASCOMResult {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn start_x(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn set_start_x(&mut self, start_x: i32) -> ascom_alpaca_rs::ASCOMResult {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn start_y(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn set_start_y(&mut self, start_y: i32) -> ascom_alpaca_rs::ASCOMResult {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn sub_exposure_duration(&self) -> ascom_alpaca_rs::ASCOMResult<f64> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn set_sub_exposure_duration(
        &mut self,
        sub_exposure_duration: f64,
    ) -> ascom_alpaca_rs::ASCOMResult {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn abort_exposure(&mut self) -> ascom_alpaca_rs::ASCOMResult {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn pulse_guide(
        &mut self,
        direction: ascom_alpaca_rs::api::PutPulseGuideDirection,
        duration: i32,
    ) -> ascom_alpaca_rs::ASCOMResult {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn start_exposure(&mut self, duration: f64, light: bool) -> ascom_alpaca_rs::ASCOMResult {
        if self.current_exposure.is_some() {
            return Err(ASCOMError::INVALID_OPERATION);
        }
        let inner = Rc::clone(&self.inner);
        self.current_exposure = Some(tokio::task::spawn_local(async move {
            let bulb_toggle = inner
                .config_key::<ToggleWidget>("bulb")
                .map_err(convert_err)?;
            bulb_toggle.set_toggled(true);
            inner.set_config(&bulb_toggle).map_err(convert_err)?;
            sleep(std::time::Duration::from_secs_f64(duration)).await;
            bulb_toggle.set_toggled(false);
            inner.set_config(&bulb_toggle).map_err(convert_err)?;
            let path = loop {
                match inner
                    .wait_event(std::time::Duration::from_secs(3))
                    .map_err(convert_err)?
                {
                    CameraEvent::NewFile(path) => break path,
                    CameraEvent::Timeout => {
                        return Err(convert_err("timeout while waiting for captured image"))
                    }
                    _ => {}
                }
            };
            Ok(path)
        }));
        Ok(())
    }

    fn stop_exposure(&mut self) -> ascom_alpaca_rs::ASCOMResult {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_span_events(FmtSpan::CLOSE)
        .finish()
        .init();

    gphoto2_test::set_env();

    let devices = DevicesBuilder::new()
        .with(MyCamera::new(
            gphoto2::Context::new()?.autodetect_camera()?,
        )?)
        .finish();

    // run our app with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    tracing::info!(%addr, "Starting server");

    LocalSet::new()
        .run_until(
            axum::Server::bind(&addr).serve(
                devices
                    .into_router()
                    .layer(TraceLayer::new_for_http())
                    .into_make_service(),
            ),
        )
        .await?;

    Ok(())
}

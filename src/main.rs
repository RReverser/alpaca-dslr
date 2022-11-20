use ascom_alpaca_rs::api::{Camera, Device};
use ascom_alpaca_rs::{
    ASCOMError, ASCOMErrorCode, ASCOMParams, ASCOMResult, DevicesBuilder, OpaqueResponse,
};
use atomic::Atomic;
use gphoto2::camera::CameraEvent;
use gphoto2::file::CameraFilePath;
use gphoto2::widget::ToggleWidget;
use net_literals::{addr, ipv6};
use send_wrapper::SendWrapper;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tokio::select;
use tokio::sync::oneshot;
use tokio::task::{JoinHandle, LocalSet};
use tokio::time::sleep;
use tower_http::trace::TraceLayer;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;

#[derive(Clone, Copy)]
enum ExposingState {
    Waiting,
    Exposing,
    Reading,
}

struct SuccessfulExposure {
    camera_file_path: CameraFilePath,
    start_utc: OffsetDateTime,
    duration: Duration,
}

enum State {
    Idle,
    InExposure(CurrentExposure),
    AfterExposure(ASCOMResult<SuccessfulExposure>),
}

struct CurrentExposure {
    join_handle: JoinHandle<()>,
    early_stop_sender: Option<oneshot::Sender<()>>,
    state: Arc<Atomic<ExposingState>>,
}

impl Drop for CurrentExposure {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}

async fn expose(
    camera: &gphoto2::Camera,
    duration: Duration,
    stop_exposure: oneshot::Receiver<()>,
    state: &Atomic<ExposingState>,
) -> gphoto2::Result<SuccessfulExposure> {
    let bulb_toggle = camera.config_key::<ToggleWidget>("bulb")?;
    bulb_toggle.set_toggled(true);
    camera.set_config(&bulb_toggle)?;
    state.store(ExposingState::Exposing, atomic::Ordering::Relaxed);
    let start_utc = OffsetDateTime::now_utc();
    let start_instant = Instant::now();
    select! {
        _ = sleep(duration) => {},
        _ = stop_exposure => {}
    };
    let elapsed = start_instant.elapsed();
    bulb_toggle.set_toggled(false);
    camera.set_config(&bulb_toggle)?;
    state.store(ExposingState::Reading, atomic::Ordering::Relaxed);
    let path = loop {
        match camera.wait_event(std::time::Duration::from_secs(3))? {
            CameraEvent::NewFile(path) => break path,
            CameraEvent::Timeout => {
                return Err(gphoto2::Error::new(
                    gphoto2::libgphoto2_sys::GP_ERROR_TIMEOUT,
                    Some("timeout while waiting for captured image".to_owned()),
                ))
            }
            _ => {}
        }
    };
    Ok(SuccessfulExposure {
        camera_file_path: path,
        start_utc,
        duration: elapsed,
    })
}

struct MyCamera {
    inner: SendWrapper<Rc<gphoto2::Camera>>,
    state: Arc<Mutex<State>>,
    dimensions: (u32, u32),
    subframe: image::math::Rect,
}

impl std::ops::Deref for MyCamera {
    type Target = gphoto2::Camera;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl MyCamera {
    pub fn new(inner: gphoto2::Camera) -> anyhow::Result<Self> {
        let (width, height) = {
            let span = tracing::trace_span!("Determine dimensions");
            let _enter = span.enter();

            tracing::trace!("Capturing test image");
            let camera_file = inner.capture_preview().or_else(|e| {
                if e.kind() != gphoto2::error::ErrorKind::NotSupported {
                    return Err(e);
                }
                tracing::warn!("Preview capture not supported, falling back to full image capture");
                let camera_file_path = inner.capture_image()?;
                let folder = camera_file_path.folder();
                let name = camera_file_path.name();
                let fs = inner.fs();
                tracing::trace!("Downloading test image from the camera");
                let camera_file = fs.download(&folder, &name)?;
                tracing::trace!("Deleting test image from the camera");
                fs.delete_file(&folder, &name)?;
                Ok(camera_file)
            })?;

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

        tracing::info!(width, height, "Detected camera dimensions");

        Ok(Self {
            inner: SendWrapper::new(Rc::new(inner)),
            state: Arc::new(Mutex::new(State::Idle)),
            dimensions: (width, height),
            subframe: image::math::Rect {
                x: 0,
                y: 0,
                width,
                height,
            },
        })
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap()
    }
}

#[derive(Default)]
struct MyCameraDevice {
    camera: Option<MyCamera>,
}

impl MyCameraDevice {
    fn camera(&self) -> ASCOMResult<&MyCamera> {
        self.camera.as_ref().ok_or(ASCOMError::NOT_CONNECTED)
    }

    fn camera_mut(&mut self) -> ASCOMResult<&mut MyCamera> {
        self.camera.as_mut().ok_or(ASCOMError::NOT_CONNECTED)
    }
}

fn convert_err(err: impl std::string::ToString) -> ASCOMError {
    // TODO: more granular error codes.
    ASCOMError::new(ASCOMErrorCode::UNSPECIFIED, err.to_string())
}

#[allow(unused_variables)]
impl Device for MyCameraDevice {
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
        Ok(self.camera.is_some())
    }

    fn set_connected(&mut self, connected: bool) -> ascom_alpaca_rs::ASCOMResult {
        if connected == self.camera.is_some() {
            return Ok(());
        }

        if connected {
            let span = tracing::trace_span!("Connecting to camera");
            let _enter = span.enter();

            self.camera = Some(
                MyCamera::new(
                    gphoto2::Context::new()
                        .and_then(|ctx| ctx.autodetect_camera())
                        .map_err(convert_err)?,
                )
                .map_err(convert_err)?,
            );
        } else {
            self.camera = None;
        }

        Ok(())
    }

    fn description(&self) -> ascom_alpaca_rs::ASCOMResult<String> {
        self.camera()?.about().map_err(convert_err)
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
        Ok(self.camera()?.abilities().model().into_owned())
    }

    fn supported_actions(&self) -> ascom_alpaca_rs::ASCOMResult<Vec<String>> {
        Ok(vec![])
    }
}

#[allow(unused_variables)]
impl Camera for MyCameraDevice {
    fn bayer_offset_x(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Ok(0)
    }

    fn bayer_offset_y(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Ok(0)
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
        // TODO: `Download` state
        Ok(match &*self.camera()?.state() {
            State::Idle => ascom_alpaca_rs::api::CameraStateResponse::Idle,
            State::InExposure(exposure) => match exposure.state.load(atomic::Ordering::Relaxed) {
                ExposingState::Waiting => ascom_alpaca_rs::api::CameraStateResponse::Waiting,
                ExposingState::Exposing => ascom_alpaca_rs::api::CameraStateResponse::Exposing,
                ExposingState::Reading => ascom_alpaca_rs::api::CameraStateResponse::Reading,
            },
            State::AfterExposure(result) => match result {
                Ok(_) => ascom_alpaca_rs::api::CameraStateResponse::Idle,
                Err(_) => ascom_alpaca_rs::api::CameraStateResponse::Error,
            },
        })
    }

    fn camera_xsize(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Ok(self.camera()?.dimensions.0 as i32)
    }

    fn camera_ysize(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Ok(self.camera()?.dimensions.1 as i32)
    }

    fn can_abort_exposure(&self) -> ascom_alpaca_rs::ASCOMResult<bool> {
        Ok(true)
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
        Ok(true)
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
        Ok(100. * 60. * 60.)
    }

    fn exposure_min(&self) -> ascom_alpaca_rs::ASCOMResult<f64> {
        Ok(0.1)
    }

    fn exposure_resolution(&self) -> ascom_alpaca_rs::ASCOMResult<f64> {
        // TODO: adjust this as we go.
        // Considering that we need to do some high-latency operations,
        // I'm not sure we can go very low in terms of precision here,
        // so for now setting to 0.1 seconds as a rough estimate.
        Ok(0.1)
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
        Ok(matches!(
            *self.camera()?.state(),
            State::AfterExposure { .. }
        ))
    }

    fn is_pulse_guiding(&self) -> ascom_alpaca_rs::ASCOMResult<bool> {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn last_exposure_duration(&self) -> ascom_alpaca_rs::ASCOMResult<f64> {
        match *self.camera()?.state() {
            State::AfterExposure(Ok(SuccessfulExposure { duration, .. })) => {
                Ok(duration.as_secs_f64())
            }
            _ => Err(ascom_alpaca_rs::ASCOMError::INVALID_OPERATION),
        }
    }

    fn last_exposure_start_time(&self) -> ascom_alpaca_rs::ASCOMResult<String> {
        match *self.camera()?.state() {
            State::AfterExposure(Ok(SuccessfulExposure { start_utc, .. })) => {
                // We need CCYY-MM-DDThh:mm:ss[.sss...]. This is close to RFC3339, but
                // we need to remove the Z timezone suffix.
                let mut result = start_utc.format(&Rfc3339).map_err(convert_err)?;
                let last_char = result.pop();
                debug_assert_eq!(last_char, Some('Z'));
                Ok(result)
            }
            _ => Err(ascom_alpaca_rs::ASCOMError::INVALID_OPERATION),
        }
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
        Ok(self.camera()?.subframe.width as i32)
    }

    fn set_num_x(&mut self, num_x: i32) -> ascom_alpaca_rs::ASCOMResult {
        self.camera_mut()?.subframe.width = num_x as _;
        Ok(())
    }

    fn num_y(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Ok(self.camera()?.subframe.height as _)
    }

    fn set_num_y(&mut self, num_y: i32) -> ascom_alpaca_rs::ASCOMResult {
        self.camera_mut()?.subframe.height = num_y as _;
        Ok(())
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
        Ok(self.camera()?.subframe.x as _)
    }

    fn set_start_x(&mut self, start_x: i32) -> ascom_alpaca_rs::ASCOMResult {
        self.camera_mut()?.subframe.x = start_x as _;
        Ok(())
    }

    fn start_y(&self) -> ascom_alpaca_rs::ASCOMResult<i32> {
        Ok(self.camera()?.subframe.y as _)
    }

    fn set_start_y(&mut self, start_y: i32) -> ascom_alpaca_rs::ASCOMResult {
        self.camera_mut()?.subframe.y = start_y as _;
        Ok(())
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
        match &mut *self.camera_mut()?.state() {
            camera_state @ State::InExposure(_) => {
                *camera_state = State::Idle;
                Ok(())
            }
            State::Idle | State::AfterExposure(_) => Ok(()),
        }
    }

    fn pulse_guide(
        &mut self,
        direction: ascom_alpaca_rs::api::PutPulseGuideDirection,
        duration: i32,
    ) -> ascom_alpaca_rs::ASCOMResult {
        Err(ascom_alpaca_rs::ASCOMError::NOT_IMPLEMENTED)
    }

    fn start_exposure(&mut self, duration: f64, light: bool) -> ascom_alpaca_rs::ASCOMResult {
        let camera = self.camera_mut()?;
        let inner = Rc::clone(&camera.inner);
        let (stop_exposure_sender, stop_exposure_receiver) = oneshot::channel();
        let state = Arc::clone(&camera.state);
        let exposing_state = Arc::new(Atomic::new(ExposingState::Waiting));
        *camera.state() = State::InExposure(CurrentExposure {
            state: Arc::clone(&exposing_state),
            join_handle: tokio::task::spawn_local(async move {
                let result = expose(
                    &inner,
                    Duration::from_secs_f64(duration),
                    stop_exposure_receiver,
                    &exposing_state,
                )
                .await
                .map_err(convert_err);

                *state.lock().unwrap() = State::AfterExposure(result);
            }),
            early_stop_sender: Some(stop_exposure_sender),
        });
        Ok(())
    }

    fn stop_exposure(&mut self) -> ascom_alpaca_rs::ASCOMResult {
        match &mut *self.camera_mut()?.state() {
            State::InExposure(current_exposure) => {
                current_exposure
                    .early_stop_sender
                    .take()
                    // `stop_exposure` was already called.
                    .ok_or(ASCOMError::INVALID_OPERATION)?
                    .send(())
                    // The exposure already finished or was aborted.
                    .map_err(|_| ASCOMError::INVALID_OPERATION)
            }
            // There is no exposure in progress.
            State::Idle | State::AfterExposure(_) => Ok(()),
        }
    }
}

async fn start_alpaca_discovery_server(alpaca_port: u16) -> anyhow::Result<()> {
    let discovery_msg = format!(r#"{{"AlpacaPort":{}}}"#, alpaca_port);
    let socket = tokio::net::UdpSocket::bind(addr!("[::]:32227")).await?;
    socket.join_multicast_v6(&ipv6!("ff12::a1:9aca"), 0)?;
    let mut buf = [0; 16];
    loop {
        let (len, src) = socket.recv_from(&mut buf).await?;
        let data = &buf[..len];
        if data == b"alpacadiscovery1" {
            tracing::debug!(%src, "Received Alpaca discovery request");
            socket.send_to(discovery_msg.as_bytes(), src).await?;
        } else {
            tracing::warn!(%src, "Received unknown multicast packet");
        }
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
        .with(MyCameraDevice::default())
        .finish();

    // run our app with hyper
    let addr = addr!("[::]:3000");

    tracing::info!(%addr, "Starting server");

    let server = axum::Server::try_bind(&addr)?;

    let local_set = LocalSet::new();

    tokio::try_join!(
        start_alpaca_discovery_server(addr.port()),
        local_set.run_until(async move {
            server
                .serve(
                    devices
                        .into_router()
                        .route(
                            "/management/apiversions",
                            axum::routing::get(|| async { r#"{"Value":[1]}"# }),
                        )
                        .route(
                            "/management/v1/description",
                            axum::routing::get(|| async {
                                r#"{
                                    "Value": {
                                        "ServerName": "alpaca-dslr",
                                        "Manufacturer": "RReverser",
                                        "ManufacturerVersion": "0.0.1",
                                        "Location": "Earth"
                                    }
                                }"#
                            }),
                        )
                        .route(
                            "/management/v1/configureddevices",
                            axum::routing::get(|| async {
                                r#"{
                                    "Value": [
                                        {
                                            "DeviceType": "Camera",
                                            "DeviceNumber": 0,
                                            "UniqueID": "MyCamera",
                                            "Description": "My Camera"
                                        }
                                    ]
                                }"#
                            }),
                        )
                        .layer(TraceLayer::new_for_http())
                        .into_make_service(),
                )
                .await?;

            Ok(())
        })
    )
    .map(|_| ())
}

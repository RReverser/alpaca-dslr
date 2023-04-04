use ascom_alpaca::api::{Camera, CargoServerInfo, Device};
use ascom_alpaca::{ASCOMError, ASCOMErrorCode, ASCOMResult, Server};
use async_trait::async_trait;
use atomic::Atomic;
use gphoto2::camera::CameraEvent;
use gphoto2::file::{CameraFile, CameraFilePath};
use gphoto2::widget::ToggleWidget;
use image::{DynamicImage, ImageBuffer, Pixel};
use std::borrow::Cow;
use std::sync::Arc;
use std::time::{Duration, Instant};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tokio::select;
use tokio::sync::{
    oneshot, Mutex, RwLock, RwLockMappedWriteGuard, RwLockReadGuard, RwLockWriteGuard,
};
use tokio::task::JoinHandle;
use tokio::time::sleep;

const ERR_UNSUPPORTED_IMAGE_FORMAT: ASCOMError = ASCOMError {
    code: ASCOMErrorCode::new_for_driver::<0>(),
    message: Cow::Borrowed("Unsupported image format"),
};

#[derive(Debug, Clone, Copy)]
enum ExposingState {
    Waiting,
    Exposing,
    Reading,
}

struct SuccessfulExposure {
    image: DynamicImage,
    start_utc: OffsetDateTime,
    duration: Duration,
}

enum State {
    Idle,
    InExposure(CurrentExposure),
    AfterExposure(ASCOMResult<SuccessfulExposure>),
}

impl State {
    fn as_successful_exposure(&self) -> ASCOMResult<&SuccessfulExposure> {
        match self {
            State::AfterExposure(Ok(exposure)) => Ok(exposure),
            State::AfterExposure(Err(err)) => Err(err.clone()),
            State::Idle => Err(ASCOMError::new(
                ASCOMErrorCode::INVALID_OPERATION,
                "Camera is idle",
            )),
            State::InExposure(CurrentExposure { state, .. }) => Err(ASCOMError::new(
                ASCOMErrorCode::INVALID_OPERATION,
                format!(
                    "Camera is currently making a picture (status: {:?})",
                    state.load(atomic::Ordering::Relaxed)
                ),
            )),
        }
    }
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

async fn download_and_delete(
    camera: &gphoto2::Camera,
    path: &CameraFilePath,
) -> gphoto2::Result<CameraFile> {
    let folder = path.folder();
    let folder = folder.as_ref();

    let filename = path.name();
    let filename = filename.as_ref();

    let span = tracing::trace_span!("Download & delete image from camera", folder, filename);
    let _enter = span.enter();

    let fs = camera.fs();

    tracing::trace!("Downloading image");
    let camera_file = fs.download(folder, filename).await?;

    tracing::trace!("Deleting image");
    fs.delete_file(folder, filename).await?;

    Ok(camera_file)
}

async fn camera_file_to_image(
    camera_file: CameraFile,
    context: &gphoto2::Context,
) -> anyhow::Result<image::io::Reader<impl std::io::BufRead + std::io::Seek>> {
    let mime_type = camera_file.mime_type();
    let img_format = image::ImageFormat::from_mime_type(&mime_type)
        .ok_or_else(|| anyhow::anyhow!("unsupported image format {mime_type}"))?;

    tracing::trace!(mime_type, ?img_format, "Determined image format");

    Ok(image::io::Reader::with_format(
        std::io::Cursor::new(camera_file.get_data(context).await?),
        img_format,
    ))
}

async fn expose(
    camera: &gphoto2::Camera,
    context: &gphoto2::Context,
    duration: Duration,
    stop_exposure: oneshot::Receiver<()>,
    state: &Atomic<ExposingState>,
    subframe: image::math::Rect,
) -> anyhow::Result<SuccessfulExposure> {
    let bulb_toggle = camera.config_key::<ToggleWidget>("bulb").await?;
    bulb_toggle.set_toggled(true);
    camera.set_config(&bulb_toggle).await?;
    state.store(ExposingState::Exposing, atomic::Ordering::Relaxed);
    let start_utc = OffsetDateTime::now_utc();
    let start_instant = Instant::now();
    select! {
        _ = sleep(duration) => {},
        _ = stop_exposure => {}
    };
    let elapsed = start_instant.elapsed();
    bulb_toggle.set_toggled(false);
    camera.set_config(&bulb_toggle).await?;
    state.store(ExposingState::Reading, atomic::Ordering::Relaxed);
    let path = loop {
        match camera.wait_event(std::time::Duration::from_secs(3)).await? {
            CameraEvent::NewFile(path) => {
                break path;
            }
            CameraEvent::Timeout => {
                anyhow::bail!("Timeout while waiting for camera to finish exposing");
            }
            _ => {}
        }
    };
    let camera_file = download_and_delete(camera, &path).await?;
    let image_reader = camera_file_to_image(camera_file, context).await?;
    Ok(SuccessfulExposure {
        image: image_reader.decode()?.crop_imm(
            subframe.x,
            subframe.y,
            subframe.width,
            subframe.height,
        ),
        start_utc,
        duration: elapsed,
    })
}

struct MyCamera {
    context: gphoto2::Context,
    inner: gphoto2::Camera,
    state: Arc<Mutex<State>>,
    dimensions: (u32, u32),
    subframe: image::math::Rect,
}

impl std::fmt::Debug for MyCamera {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MyCamera").finish_non_exhaustive()
    }
}

impl std::ops::Deref for MyCamera {
    type Target = gphoto2::Camera;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl MyCamera {
    pub async fn new(inner: gphoto2::Camera, context: gphoto2::Context) -> anyhow::Result<Self> {
        let (width, height) = {
            let span = tracing::trace_span!("Determine dimensions");
            let _enter = span.enter();

            tracing::trace!("Capturing test image");
            let capture_preview_task = inner.capture_preview();
            let camera_file = match capture_preview_task.await {
                Err(e) if e.kind() == gphoto2::error::ErrorKind::NotSupported => {
                    tracing::warn!(
                        "Preview capture not supported, falling back to full image capture"
                    );
                    let camera_file_path = inner.capture_image().await?;
                    let folder = camera_file_path.folder();
                    let name = camera_file_path.name();
                    let fs = inner.fs();
                    tracing::trace!("Downloading test image from the camera");
                    let camera_file = fs.download(&folder, &name).await?;
                    tracing::trace!("Deleting test image from the camera");
                    fs.delete_file(&folder, &name).await?;
                    camera_file
                }
                result => result?,
            };

            let mime_type = camera_file.mime_type();
            let img_format = image::ImageFormat::from_mime_type(&mime_type)
                .ok_or_else(|| anyhow::anyhow!("unknown image format {mime_type}"))?;

            tracing::trace!(mime_type, ?img_format, "Test image format");

            image::io::Reader::with_format(
                std::io::Cursor::new(camera_file.get_data(&context).await?),
                img_format,
            )
            .into_dimensions()
        }?;

        tracing::info!(width, height, "Detected camera dimensions");

        Ok(Self {
            context,
            inner,
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

    async fn state(&self) -> tokio::sync::MutexGuard<'_, State> {
        self.state.lock().await
    }
}

#[derive(Default, Debug)]
struct MyCameraDevice {
    camera: RwLock<Option<MyCamera>>,
}

impl MyCameraDevice {
    async fn camera(&self) -> ASCOMResult<RwLockReadGuard<'_, MyCamera>> {
        RwLockReadGuard::try_map(self.camera.read().await, |camera| camera.as_ref())
            .map_err(|_| ASCOMError::NOT_CONNECTED)
    }

    async fn camera_mut(&self) -> ASCOMResult<RwLockMappedWriteGuard<'_, MyCamera>> {
        RwLockWriteGuard::try_map(self.camera.write().await, |camera| camera.as_mut())
            .map_err(|_| ASCOMError::NOT_CONNECTED)
    }
}

fn convert_err(err: impl std::string::ToString) -> ASCOMError {
    // TODO: more granular error codes.
    ASCOMError::new(ASCOMErrorCode::UNSPECIFIED, err.to_string())
}

#[allow(unused_variables)]
#[async_trait]
impl Device for MyCameraDevice {
    fn unique_id(&self) -> &str {
        "ffe84935-e951-45b3-9835-d532b04ee932"
    }

    async fn action(&self, action: String, parameters: String) -> ASCOMResult<String> {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn command_blind(&self, command: String, raw: String) -> ASCOMResult {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn command_bool(&self, command: String, raw: String) -> ASCOMResult<bool> {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn command_string(&self, command: String, raw: String) -> ASCOMResult<String> {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn connected(&self) -> ASCOMResult<bool> {
        Ok(self.camera().await.is_ok())
    }

    async fn set_connected(&self, connected: bool) -> ASCOMResult {
        let mut camera = self.camera.write().await;

        if connected == camera.is_some() {
            return Ok(());
        }

        *camera = if connected {
            let span = tracing::trace_span!("Connecting to camera");
            let _enter = span.enter();

            let context = gphoto2::Context::new().map_err(convert_err)?;
            let camera = context.autodetect_camera().await.map_err(convert_err)?;

            Some(MyCamera::new(camera, context).await.map_err(convert_err)?)
        } else {
            None
        };

        Ok(())
    }

    async fn description(&self) -> ASCOMResult<String> {
        Ok(self.camera().await?.abilities().model().into_owned())
    }

    async fn driver_info(&self) -> ASCOMResult<String> {
        Ok(env!("CARGO_PKG_DESCRIPTION").to_owned())
    }

    async fn driver_version(&self) -> ASCOMResult<String> {
        Ok(env!("CARGO_PKG_VERSION").to_owned())
    }

    async fn interface_version(&self) -> ASCOMResult<i32> {
        Ok(3)
    }

    fn static_name(&self) -> &'static str {
        "My Camera"
    }

    async fn name(&self) -> ASCOMResult<String> {
        Ok("My Camera".to_owned())
    }

    async fn supported_actions(&self) -> ASCOMResult<Vec<String>> {
        Ok(vec![])
    }
}

#[allow(unused_variables)]
#[async_trait]
impl Camera for MyCameraDevice {
    async fn bayer_offset_x(&self) -> ASCOMResult<i32> {
        Ok(0)
    }

    async fn bayer_offset_y(&self) -> ASCOMResult<i32> {
        Ok(0)
    }

    async fn bin_x(&self) -> ASCOMResult<i32> {
        Ok(1)
    }

    async fn set_bin_x(&self, bin_x: i32) -> ASCOMResult {
        if bin_x != 1 {
            return Err(ASCOMError::new(
                ASCOMErrorCode::INVALID_VALUE,
                "binning not supported",
            ));
        }
        Ok(())
    }

    async fn bin_y(&self) -> ASCOMResult<i32> {
        Ok(1)
    }

    async fn set_bin_y(&self, bin_y: i32) -> ASCOMResult {
        if bin_y != 1 {
            return Err(ASCOMError::new(
                ASCOMErrorCode::INVALID_VALUE,
                "binning not supported",
            ));
        }
        Ok(())
    }

    async fn camera_state(&self) -> ASCOMResult<ascom_alpaca::api::CameraStateResponse> {
        // TODO: `Download` state
        Ok(match &*self.camera().await?.state().await {
            State::Idle => ascom_alpaca::api::CameraStateResponse::Idle,
            State::InExposure(exposure) => match exposure.state.load(atomic::Ordering::Relaxed) {
                ExposingState::Waiting => ascom_alpaca::api::CameraStateResponse::Waiting,
                ExposingState::Exposing => ascom_alpaca::api::CameraStateResponse::Exposing,
                ExposingState::Reading => ascom_alpaca::api::CameraStateResponse::Reading,
            },
            State::AfterExposure(result) => match result {
                Ok(_) => ascom_alpaca::api::CameraStateResponse::Idle,
                Err(_) => ascom_alpaca::api::CameraStateResponse::Error,
            },
        })
    }

    async fn camera_xsize(&self) -> ASCOMResult<i32> {
        Ok(self.camera().await?.dimensions.0 as i32)
    }

    async fn camera_ysize(&self) -> ASCOMResult<i32> {
        Ok(self.camera().await?.dimensions.1 as i32)
    }

    async fn can_abort_exposure(&self) -> ASCOMResult<bool> {
        Ok(true)
    }

    async fn can_asymmetric_bin(&self) -> ASCOMResult<bool> {
        Ok(false)
    }

    async fn can_fast_readout(&self) -> ASCOMResult<bool> {
        Ok(false)
    }

    async fn can_get_cooler_power(&self) -> ASCOMResult<bool> {
        Ok(false)
    }

    async fn can_pulse_guide(&self) -> ASCOMResult<bool> {
        Ok(false)
    }

    async fn can_set_ccdtemperature(&self) -> ASCOMResult<bool> {
        Ok(false)
    }

    async fn can_stop_exposure(&self) -> ASCOMResult<bool> {
        Ok(true)
    }

    async fn ccdtemperature(&self) -> ASCOMResult<f64> {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn cooler_on(&self) -> ASCOMResult<bool> {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn set_cooler_on(&self, cooler_on: bool) -> ASCOMResult {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn cooler_power(&self) -> ASCOMResult<f64> {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn electrons_per_adu(&self) -> ASCOMResult<f64> {
        // TODO: better default? Integrate camera info somehow?
        Ok(1.)
    }

    async fn exposure_max(&self) -> ASCOMResult<f64> {
        Ok(100. * 60. * 60.)
    }

    async fn exposure_min(&self) -> ASCOMResult<f64> {
        Ok(0.1)
    }

    async fn exposure_resolution(&self) -> ASCOMResult<f64> {
        // TODO: adjust this as we go.
        // Considering that we need to do some high-latency operations,
        // I'm not sure we can go very low in terms of precision here,
        // so for now setting to 0.1 seconds as a rough estimate.
        Ok(0.1)
    }

    async fn fast_readout(&self) -> ASCOMResult<bool> {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn set_fast_readout(&self, fast_readout: bool) -> ASCOMResult {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn full_well_capacity(&self) -> ASCOMResult<f64> {
        Ok(u16::MAX.into())
    }

    async fn gain(&self) -> ASCOMResult<i32> {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn set_gain(&self, gain: i32) -> ASCOMResult {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn gain_max(&self) -> ASCOMResult<i32> {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn gain_min(&self) -> ASCOMResult<i32> {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn gains(&self) -> ASCOMResult<Vec<String>> {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn has_shutter(&self) -> ASCOMResult<bool> {
        Ok(true)
    }

    async fn heat_sink_temperature(&self) -> ASCOMResult<f64> {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn image_array(&self) -> ASCOMResult<ascom_alpaca::api::ImageArrayResponse> {
        fn flat_samples<P: Pixel>(img: &ImageBuffer<P, Vec<P::Subpixel>>) -> (u8, Vec<i32>)
        where
            P::Subpixel: Into<i32>,
        {
            let flat_samples = img.as_flat_samples();
            let channels = flat_samples.layout.channels;
            let data = flat_samples.as_slice().iter().map(|&x| x.into()).collect();
            (channels, data)
        }

        let camera = self.camera().await?;
        let state = camera.state().await;
        let image = &state.as_successful_exposure()?.image;
        let (channels, data) = match image {
            DynamicImage::ImageLuma8(image) => flat_samples(image),
            DynamicImage::ImageLuma16(image) => flat_samples(image),
            DynamicImage::ImageRgb8(image) => flat_samples(image),
            DynamicImage::ImageRgb16(image) => flat_samples(image),
            _ => return Err(ERR_UNSUPPORTED_IMAGE_FORMAT),
        };

        Ok(ascom_alpaca::api::ImageArrayResponse {
            data: ndarray::Array::from_shape_vec(
                (
                    image.width() as usize,
                    image.height() as usize,
                    channels.into(),
                ),
                data,
            )
            .expect("shape mismatch when creating image array"),
        })
    }

    async fn image_ready(&self) -> ASCOMResult<bool> {
        Ok(matches!(
            *self.camera().await?.state().await,
            State::AfterExposure(Ok(_))
        ))
    }

    async fn is_pulse_guiding(&self) -> ASCOMResult<bool> {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn last_exposure_duration(&self) -> ASCOMResult<f64> {
        Ok(self
            .camera()
            .await?
            .state()
            .await
            .as_successful_exposure()?
            .duration
            .as_secs_f64())
    }

    async fn last_exposure_start_time(&self) -> ASCOMResult<String> {
        let start_utc = self
            .camera()
            .await?
            .state()
            .await
            .as_successful_exposure()?
            .start_utc;

        // We need CCYY-MM-DDThh:mm:ss[.sss...]. This is close to RFC3339, but
        // we need to remove the Z timezone suffix.
        let mut result = start_utc.format(&Rfc3339).map_err(convert_err)?;
        let last_char = result.pop();
        debug_assert_eq!(last_char, Some('Z'));
        Ok(result)
    }

    async fn max_adu(&self) -> ASCOMResult<i32> {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn max_bin_x(&self) -> ASCOMResult<i32> {
        Ok(1)
    }

    async fn max_bin_y(&self) -> ASCOMResult<i32> {
        Ok(1)
    }

    async fn num_x(&self) -> ASCOMResult<i32> {
        Ok(self.camera().await?.subframe.width as i32)
    }

    async fn set_num_x(&self, num_x: i32) -> ASCOMResult {
        self.camera_mut().await?.subframe.width = num_x as _;
        Ok(())
    }

    async fn num_y(&self) -> ASCOMResult<i32> {
        Ok(self.camera().await?.subframe.height as _)
    }

    async fn set_num_y(&self, num_y: i32) -> ASCOMResult {
        self.camera_mut().await?.subframe.height = num_y as _;
        Ok(())
    }

    async fn offset(&self) -> ASCOMResult<i32> {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn set_offset(&self, offset: i32) -> ASCOMResult {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn offset_max(&self) -> ASCOMResult<i32> {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn offset_min(&self) -> ASCOMResult<i32> {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn offsets(&self) -> ASCOMResult<Vec<String>> {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn percent_completed(&self) -> ASCOMResult<i32> {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn pixel_size_x(&self) -> ASCOMResult<f64> {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn pixel_size_y(&self) -> ASCOMResult<f64> {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn readout_mode(&self) -> ASCOMResult<i32> {
        Ok(0)
    }

    async fn set_readout_mode(&self, readout_mode: i32) -> ASCOMResult {
        if readout_mode == 0 {
            Ok(())
        } else {
            Err(ASCOMError::new(
                ASCOMErrorCode::INVALID_VALUE,
                "Invalid readout mode (only 0 is supported)",
            ))
        }
    }

    async fn readout_modes(&self) -> ASCOMResult<Vec<String>> {
        // TODO: allow to configure preview/JPEG/RAW here.
        Ok(vec!["As-is".to_owned()])
    }

    async fn sensor_name(&self) -> ASCOMResult<String> {
        Ok("Unknown".to_owned())
    }

    async fn sensor_type(&self) -> ASCOMResult<ascom_alpaca::api::SensorTypeResponse> {
        Ok(ascom_alpaca::api::SensorTypeResponse::Color)
    }

    async fn set_ccdtemperature(&self) -> ASCOMResult<f64> {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn set_set_ccdtemperature(&self, set_ccdtemperature: f64) -> ASCOMResult {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn start_x(&self) -> ASCOMResult<i32> {
        Ok(self.camera().await?.subframe.x as _)
    }

    async fn set_start_x(&self, start_x: i32) -> ASCOMResult {
        self.camera_mut().await?.subframe.x = start_x as _;
        Ok(())
    }

    async fn start_y(&self) -> ASCOMResult<i32> {
        Ok(self.camera().await?.subframe.y as _)
    }

    async fn set_start_y(&self, start_y: i32) -> ASCOMResult {
        self.camera_mut().await?.subframe.y = start_y as _;
        Ok(())
    }

    async fn sub_exposure_duration(&self) -> ASCOMResult<f64> {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn set_sub_exposure_duration(&self, sub_exposure_duration: f64) -> ASCOMResult {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn abort_exposure(&self) -> ASCOMResult {
        match &mut *self.camera_mut().await?.state().await {
            camera_state @ State::InExposure(_) => {
                *camera_state = State::Idle;
                Ok(())
            }
            State::Idle | State::AfterExposure(_) => Ok(()),
        }
    }

    async fn pulse_guide(
        &self,
        direction: ascom_alpaca::api::PutPulseGuideDirection,
        duration: i32,
    ) -> ASCOMResult {
        Err(ASCOMError::NOT_IMPLEMENTED)
    }

    async fn start_exposure(&self, duration: f64, light: bool) -> ASCOMResult {
        let duration = Duration::try_from_secs_f64(duration)
            .map_err(|err| ASCOMError::new(ASCOMErrorCode::INVALID_VALUE, err.to_string()))?;
        let camera = self.camera_mut().await?;
        let context = camera.context.clone();
        let inner = camera.inner.clone();
        let (stop_exposure_sender, stop_exposure_receiver) = oneshot::channel();
        let state = Arc::clone(&camera.state);
        let exposing_state = Arc::new(Atomic::new(ExposingState::Waiting));
        let subframe = camera.subframe;
        *camera.state().await = State::InExposure(CurrentExposure {
            state: Arc::clone(&exposing_state),
            join_handle: tokio::task::spawn(async move {
                let result = expose(
                    &inner,
                    &context,
                    duration,
                    stop_exposure_receiver,
                    &exposing_state,
                    subframe,
                )
                .await
                .map_err(convert_err);

                *state.lock().await = State::AfterExposure(result);
            }),
            early_stop_sender: Some(stop_exposure_sender),
        });
        Ok(())
    }

    async fn stop_exposure(&self) -> ASCOMResult {
        match &mut *self.camera_mut().await?.state().await {
            State::InExposure(current_exposure) => {
                current_exposure
                    .early_stop_sender
                    .take()
                    // `stop_exposure` was already called.
                    .ok_or_else(|| {
                        ASCOMError::new(
                            ASCOMErrorCode::INVALID_OPERATION,
                            "exposure was already stopped",
                        )
                    })?
                    .send(())
                    // The exposure already finished or was aborted.
                    .map_err(|_| {
                        ASCOMError::new(
                            ASCOMErrorCode::INVALID_OPERATION,
                            "exposure was already finished or aborted",
                        )
                    })
            }
            // There is no exposure in progress.
            State::Idle | State::AfterExposure(_) => Ok(()),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    gphoto2::libgphoto2_sys::test_utils::set_env();

    let mut server = Server {
        info: CargoServerInfo!(),
        ..Default::default()
    };
    server.devices.register(MyCameraDevice::default());

    tracing::debug!(?server.devices, "Registered Alpaca devices");

    server.start_server().await
}

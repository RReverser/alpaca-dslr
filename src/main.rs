use ascom_alpaca::api::{
    Camera, CameraState, CargoServerInfo, Device, ImageArray, PutPulseGuideDirection, SensorType,
};
use ascom_alpaca::{ASCOMError, ASCOMErrorCode, ASCOMResult, Server};
use async_trait::async_trait;
use atomic::{Atomic, Ordering};
use exif::{Exif, Tag};
use gphoto2::camera::CameraEvent;
use gphoto2::file::{CameraFile, CameraFilePath};
use gphoto2::list::CameraDescriptor;
use gphoto2::widget::{RadioWidget, ToggleWidget};
use image::{DynamicImage, ImageBuffer, Pixel};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::select;
use tokio::sync::{
    oneshot, watch, Mutex, RwLock, RwLockMappedWriteGuard, RwLockReadGuard, RwLockWriteGuard,
};
use tokio::time::sleep;
use tracing::Instrument;

struct Size {
    width: u32,
    height: u32,
}

struct StopExposure {
    want_image: bool,
}

struct CurrentExposure {
    rough_start: Instant,
    state: Arc<Atomic<CameraState>>,
    expected_duration: Duration,
    stop_tx: Option<oneshot::Sender<StopExposure>>,
    done_rx: watch::Receiver<bool>,
}

struct SuccessfulExposure {
    image: ImageArray,
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
            State::Idle => Err(ASCOMError::invalid_operation("Camera is idle")),
            State::InExposure(CurrentExposure { state, .. }) => {
                Err(ASCOMError::invalid_operation(format_args!(
                    "Camera is currently taking a picture (status: {:?})",
                    state.load(Ordering::Relaxed)
                )))
            }
        }
    }

    fn percent_completed(&self) -> i32 {
        match self {
            State::Idle => 0,
            State::InExposure(CurrentExposure {
                rough_start: start,
                expected_duration,
                ..
            }) => {
                let elapsed = start.elapsed().as_secs_f64();
                let max = expected_duration.as_secs_f64();
                (100.0 * (elapsed / max).min(1.0)).round() as i32
            }
            State::AfterExposure(_) => 100,
        }
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

    async {
        let fs = camera.fs();

        tracing::trace!("Downloading image");
        let camera_file = fs.download(folder, filename).await?;

        tracing::trace!("Deleting image");
        fs.delete_file(folder, filename).await?;

        Ok(camera_file)
    }
    .instrument(tracing::trace_span!(
        "Download & delete image from camera",
        folder,
        filename
    ))
    .await
}

async fn camera_file_to_image(
    camera_file: CameraFile,
    context: &gphoto2::Context,
) -> eyre::Result<(
    Option<Exif>,
    image::io::Reader<impl std::io::BufRead + std::io::Seek>,
)> {
    let mime_type = camera_file.mime_type();
    let img_format = image::ImageFormat::from_mime_type(&mime_type)
        .ok_or_else(|| eyre::eyre!("unsupported image format {mime_type}"))?;

    tracing::trace!(mime_type, ?img_format, "Determined image format");

    let mut data = std::io::Cursor::new(camera_file.get_data(context).await?);

    let exif = exif::Reader::new().read_from_container(&mut data).ok();
    data.set_position(0);

    Ok((exif, image::io::Reader::with_format(data, img_format)))
}

fn flat_samples<P: Pixel>(img: ImageBuffer<P, Vec<P::Subpixel>>) -> ImageArray
where
    ImageArray: From<ndarray::Array3<P::Subpixel>>,
{
    let flat_samples = img.into_flat_samples();
    ndarray::Array::from_shape_vec(
        (
            flat_samples.layout.width as usize,
            flat_samples.layout.height as usize,
            flat_samples.layout.channels.into(),
        ),
        flat_samples.samples,
    )
    .expect("shape mismatch when creating image array")
    .into()
}

struct MyCamera {
    inner: gphoto2::Camera,
    state: Arc<Mutex<State>>,
    dimensions: Size,
    iso: RadioWidget,
    bulb: ToggleWidget,
    last_exposure_start_time: Atomic<Option<SystemTime>>,
    last_exposure_duration: Arc<Atomic<Option<f64>>>,
    subframe: parking_lot::RwLock<image::math::Rect>,
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
    pub async fn new(camera: gphoto2::Camera, context: gphoto2::Context) -> eyre::Result<Self> {
        let (width, height) = async {
            tracing::trace!("Capturing test image");
            let capture_preview_task = camera.capture_preview();
            let camera_file = match capture_preview_task.await {
                Err(e) if e.kind() == gphoto2::error::ErrorKind::NotSupported => {
                    tracing::warn!(
                        "Preview capture not supported, falling back to full image capture"
                    );
                    let camera_file_path = camera.capture_image().await?;
                    let folder = camera_file_path.folder();
                    let name = camera_file_path.name();
                    let fs = camera.fs();
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
                .ok_or_else(|| eyre::eyre!("unknown image format {mime_type}"))?;

            tracing::trace!(mime_type, ?img_format, "Test image format");

            image::io::Reader::with_format(
                std::io::Cursor::new(camera_file.get_data(&context).await?),
                img_format,
            )
            .into_dimensions()
            .map_err(eyre::Error::from)
        }
        .instrument(tracing::trace_span!("Determine dimensions"))
        .await?;

        tracing::info!(width, height, "Detected camera dimensions");

        Ok(Self {
            iso: camera.config_key::<RadioWidget>("iso").await?,
            bulb: camera.config_key::<ToggleWidget>("bulb").await?,
            inner: camera,
            state: Arc::new(Mutex::new(State::Idle)),
            dimensions: Size { width, height },
            last_exposure_start_time: Default::default(),
            last_exposure_duration: Default::default(),
            subframe: parking_lot::RwLock::new(image::math::Rect {
                x: 0,
                y: 0,
                width,
                height,
            }),
        })
    }

    async fn state(&self) -> tokio::sync::MutexGuard<'_, State> {
        self.state.lock().await
    }
}

#[derive(custom_debug::Debug)]
struct MyCameraDevice {
    #[debug(skip)]
    context: gphoto2::Context,
    descriptor: CameraDescriptor,
    camera: RwLock<Option<MyCamera>>,
}

impl MyCameraDevice {
    fn new(context: &gphoto2::Context, descriptor: CameraDescriptor) -> Self {
        Self {
            context: context.clone(),
            descriptor,
            camera: Default::default(),
        }
    }

    async fn camera(&self) -> ASCOMResult<RwLockReadGuard<'_, MyCamera>> {
        RwLockReadGuard::try_map(self.camera.read().await, |camera| camera.as_ref())
            .map_err(|_| ASCOMError::NOT_CONNECTED)
    }

    async fn camera_mut(&self) -> ASCOMResult<RwLockMappedWriteGuard<'_, MyCamera>> {
        RwLockWriteGuard::try_map(self.camera.write().await, |camera| camera.as_mut())
            .map_err(|_| ASCOMError::NOT_CONNECTED)
    }

    async fn stop(&self, want_image: bool) -> ASCOMResult {
        // Make sure locks are not held when waiting for `done`.
        let mut done_rx = match &mut *self.camera().await?.state().await {
            State::InExposure(CurrentExposure {
                stop_tx, done_rx, ..
            }) => {
                if let Some(stop_tx) = stop_tx.take() {
                    let _ = stop_tx.send(StopExposure { want_image });
                }
                done_rx.clone()
            }
            _ => return Ok(()),
        };
        let done_res = done_rx.wait_for(|&done| done).await;
        match done_res {
            Ok(_) => Ok(()),
            Err(_) => Err(ASCOMError::unspecified("Exposure failed to stop correctly")),
        }
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

    async fn connected(&self) -> ASCOMResult<bool> {
        Ok(self.camera().await.is_ok())
    }

    async fn set_connected(&self, connected: bool) -> ASCOMResult {
        let mut camera = self.camera.write().await;

        if connected == camera.is_some() {
            return Ok(());
        }

        *camera = if connected {
            Some(
                MyCamera::new(
                    self.context
                        .get_camera(&self.descriptor)
                        .await
                        .map_err(convert_err)?,
                    self.context.clone(),
                )
                .await
                .map_err(convert_err)?,
            )
        } else {
            None
        };

        Ok(())
    }

    async fn description(&self) -> ASCOMResult<String> {
        // TODO: is there better description text? We already use model in the name.
        Ok(self.descriptor.model.clone())
    }

    async fn driver_info(&self) -> ASCOMResult<String> {
        Ok(env!("CARGO_PKG_DESCRIPTION").to_owned())
    }

    async fn driver_version(&self) -> ASCOMResult<String> {
        Ok(env!("CARGO_PKG_VERSION").to_owned())
    }

    fn static_name(&self) -> &str {
        &self.descriptor.model
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
            return Err(ASCOMError::invalid_value("binning not supported"));
        }
        Ok(())
    }

    async fn bin_y(&self) -> ASCOMResult<i32> {
        Ok(1)
    }

    async fn set_bin_y(&self, bin_y: i32) -> ASCOMResult {
        if bin_y != 1 {
            return Err(ASCOMError::invalid_value("binning not supported"));
        }
        Ok(())
    }

    async fn camera_state(&self) -> ASCOMResult<CameraState> {
        // TODO: `Download` state
        Ok(match &*self.camera().await?.state().await {
            State::Idle => CameraState::Idle,
            State::InExposure(exposure) => exposure.state.load(Ordering::Relaxed),
            State::AfterExposure(result) => match result {
                Ok(_) => CameraState::Idle,
                Err(_) => CameraState::Error,
            },
        })
    }

    async fn camera_xsize(&self) -> ASCOMResult<i32> {
        Ok(self.camera().await?.dimensions.width as _)
    }

    async fn camera_ysize(&self) -> ASCOMResult<i32> {
        Ok(self.camera().await?.dimensions.height as _)
    }

    async fn can_abort_exposure(&self) -> ASCOMResult<bool> {
        Ok(true)
    }

    async fn can_stop_exposure(&self) -> ASCOMResult<bool> {
        Ok(true)
    }

    // TODO: maybe read this from raw for Canon at least.
    async fn ccd_temperature(&self) -> ASCOMResult<f64> {
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

    async fn full_well_capacity(&self) -> ASCOMResult<f64> {
        Ok(u16::MAX.into())
    }

    async fn gain(&self) -> ASCOMResult<i32> {
        let iso = self.camera().await?.iso.clone();
        let choice_name = iso.choice();
        iso.choices_iter()
            .position(|choice| choice == choice_name)
            .map(|index| index as _)
            .ok_or_else(|| {
                ASCOMError::unspecified(format_args!(
                    "camera error: current ISO {choice_name} not found in the list of choices"
                ))
            })
    }

    async fn set_gain(&self, gain: i32) -> ASCOMResult {
        let camera = self.camera().await?;
        let iso = camera.iso.clone();
        let choice_name = iso.choices_iter().nth(gain as usize).ok_or_else(|| {
            ASCOMError::invalid_value(format_args!("gain index {gain} is out of range"))
        })?;
        iso.set_choice(&choice_name).map_err(convert_err)?;
        camera.set_config(&iso).await.map_err(convert_err)?;
        Ok(())
    }

    async fn gains(&self) -> ASCOMResult<Vec<String>> {
        Ok(self.camera().await?.iso.choices_iter().collect())
    }

    async fn has_shutter(&self) -> ASCOMResult<bool> {
        Ok(true)
    }

    async fn image_array(&self) -> ASCOMResult<ImageArray> {
        Ok(self
            .camera()
            .await?
            .state()
            .await
            .as_successful_exposure()?
            .image
            .clone())
    }

    async fn image_ready(&self) -> ASCOMResult<bool> {
        Ok(matches!(
            *self.camera().await?.state().await,
            State::AfterExposure(Ok(_))
        ))
    }

    async fn last_exposure_duration(&self) -> ASCOMResult<f64> {
        self.camera()
            .await?
            .last_exposure_duration
            .load(Ordering::Relaxed)
            .ok_or(ASCOMError::INVALID_OPERATION)
    }

    async fn last_exposure_start_time(&self) -> ASCOMResult<SystemTime> {
        self.camera()
            .await?
            .last_exposure_start_time
            .load(Ordering::Relaxed)
            .ok_or(ASCOMError::INVALID_OPERATION)
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

    async fn start_x(&self) -> ASCOMResult<i32> {
        Ok(self.camera().await?.subframe.read().x as _)
    }

    async fn set_start_x(&self, start_x: i32) -> ASCOMResult {
        self.camera().await?.subframe.write().x = start_x as _;
        Ok(())
    }

    async fn start_y(&self) -> ASCOMResult<i32> {
        Ok(self.camera().await?.subframe.read().y as _)
    }

    async fn set_start_y(&self, start_y: i32) -> ASCOMResult {
        self.camera().await?.subframe.write().y = start_y as _;
        Ok(())
    }

    async fn num_x(&self) -> ASCOMResult<i32> {
        Ok(self.camera().await?.subframe.read().x as _)
    }

    async fn set_num_x(&self, num_x: i32) -> ASCOMResult {
        self.camera().await?.subframe.write().x = num_x as _;
        Ok(())
    }

    async fn num_y(&self) -> ASCOMResult<i32> {
        Ok(self.camera().await?.subframe.read().height as _)
    }

    async fn set_num_y(&self, num_y: i32) -> ASCOMResult {
        self.camera().await?.subframe.write().height = num_y as _;
        Ok(())
    }

    async fn percent_completed(&self) -> ASCOMResult<i32> {
        Ok(self.camera().await?.state().await.percent_completed())
    }

    async fn readout_mode(&self) -> ASCOMResult<i32> {
        Ok(0)
    }

    async fn set_readout_mode(&self, readout_mode: i32) -> ASCOMResult {
        if readout_mode == 0 {
            Ok(())
        } else {
            Err(ASCOMError::invalid_value(
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

    async fn sensor_type(&self) -> ASCOMResult<SensorType> {
        // TODO: use format to switch between Bayer and color.
        Ok(SensorType::Color)
    }

    async fn start_exposure(&self, duration: f64, light: bool) -> ASCOMResult {
        let duration = Duration::try_from_secs_f64(duration).map_err(ASCOMError::invalid_value)?;
        let camera = self.camera_mut().await?;
        let state = Arc::clone(&camera.state);
        let state2 = Arc::clone(&state);
        let last_exposure_duration = Arc::clone(&camera.last_exposure_duration);
        let bulb_toggle = camera.bulb.clone();
        let subframe = *camera.subframe.read();
        let context = self.context.clone();
        let camera = camera.inner.clone();
        let (stop_tx, stop_rx) = oneshot::channel::<StopExposure>();
        let (done_tx, done_rx) = watch::channel(false);
        let exposing_state = Arc::new(Atomic::new(CameraState::Waiting));
        let exposting_state_2 = Arc::clone(&exposing_state);
        tokio::task::spawn(async move {
            let result = async {
                bulb_toggle.set_toggled(true);
                camera.set_config(&bulb_toggle).await?;
                exposing_state.store(CameraState::Exposing, Ordering::Relaxed);
                let start_utc = SystemTime::now();
                let start_instant = Instant::now();
                let want_image = select! {
                    _ = sleep(duration) => true,
                    Ok(stop) = stop_rx => stop.want_image,
                };
                let duration = start_instant.elapsed().as_secs_f64();
                bulb_toggle.set_toggled(false);
                camera.set_config(&bulb_toggle).await?;
                exposing_state.store(CameraState::Reading, Ordering::Relaxed);
                let path = tokio::time::timeout(Duration::from_secs(3), async {
                    loop {
                        if let CameraEvent::NewFile(path) =
                            camera.wait_event(std::time::Duration::from_secs(3)).await?
                        {
                            break Ok::<_, eyre::Error>(path);
                        }
                    }
                })
                .await
                .map_err(|_| {
                    eyre::eyre!("Timeout while waiting for camera to finish exposing")
                })??;
                exposing_state.store(CameraState::Download, Ordering::Relaxed);
                let camera_file = download_and_delete(&camera, &path).await?;
                let (exif, image_reader) = camera_file_to_image(camera_file, &context).await?;

                last_exposure_duration.store(
                    Some(
                        exif.as_ref()
                            .and_then(|exif| {
                                exif.get_field(exif::Tag::ExposureTime, exif::In::PRIMARY)
                            })
                            .and_then(|field| match &field.value {
                                exif::Value::Rational(rational) if rational.len() == 1 => {
                                    Some(rational[0].to_f64())
                                }
                                _ => None,
                            })
                            .unwrap_or(duration),
                    ),
                    Ordering::Relaxed,
                );

                let image = image_reader.decode()?.crop(
                    subframe.x,
                    subframe.y,
                    subframe.width,
                    subframe.height,
                );

                let image = match image {
                    DynamicImage::ImageLuma8(image) => flat_samples(image),
                    DynamicImage::ImageLuma16(image) => flat_samples(image),
                    DynamicImage::ImageRgb8(image) => flat_samples(image),
                    DynamicImage::ImageRgb16(image) => flat_samples(image),
                    _ => eyre::bail!("unsupported image format"),
                };

                Ok(SuccessfulExposure { image })
            }
            .await
            .map_err(convert_err);

            *state.lock().await = State::AfterExposure(result);
        });
        *state2.lock().await = State::InExposure(CurrentExposure {
            // this might slightly differ from actual start in the async task;
            // we use this only for progress reporting
            rough_start: Instant::now(),
            state: exposting_state_2,
            stop_tx: Some(stop_tx),
            done_rx,
            expected_duration: duration,
        });
        Ok(())
    }

    async fn stop_exposure(&self) -> ASCOMResult {
        self.stop(true).await
    }

    async fn abort_exposure(&self) -> ASCOMResult {
        self.stop(false).await
    }
}

#[tokio::main]
async fn main() -> eyre::Result<Infallible> {
    tracing_subscriber::fmt::init();

    gphoto2::libgphoto2_sys::test_utils::set_env();

    let context = gphoto2::Context::new()?;

    let mut server = Server {
        info: CargoServerInfo!(),
        ..Default::default()
    };

    for camera_descriptor in context.list_cameras().await? {
        server
            .devices
            .register(MyCameraDevice::new(&context, camera_descriptor));
    }

    server.listen_addr.set_port(3000);

    tracing::debug!(?server.devices, "Registered Alpaca devices");

    server.start().await
}

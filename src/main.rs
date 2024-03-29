mod bulb_control;
mod cached_radio_widget;
mod convert_image;
mod parse_image;

use ascom_alpaca::api::{Camera, CameraState, CargoServerInfo, Device, ImageArray, SensorType};
use ascom_alpaca::{ASCOMError, ASCOMResult, Server};
use async_trait::async_trait;
use atomic::{Atomic, Ordering};
use bulb_control::BulbControl;
use cached_radio_widget::CachedRadioWidget;
use convert_image::convert_dynamic_image;
use futures_util::TryFutureExt;
use gphoto2::camera::CameraEvent;
use gphoto2::file::CameraFilePath;
use gphoto2::list::CameraDescriptor;
use parse_image::ImgWithMetadata;
use std::convert::Infallible;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant, SystemTime};
use tokio::select;
use tokio::sync::{oneshot, watch, Mutex, RwLock, RwLockReadGuard};
use tokio::time::sleep;
use tracing::Instrument;

/// A singleton context for gphoto2 - we always need one throughout this app's lifetime,
/// so it's easier to store it in a static variable rather than keep passing it around.
fn gphoto2_context() -> &'static gphoto2::Context {
    static CONTEXT: OnceLock<gphoto2::Context> = OnceLock::new();
    CONTEXT.get_or_init(|| gphoto2::Context::new().unwrap())
}

macro_rules! crop_rect_side {
    ($parent:ident, $child:ident, $start:ident, $len:ident) => {
        match $child.$start.checked_add($child.$len) {
            Some(child_end) if child_end <= $parent.$len => { /* in bounds */ }
            _ => {
                return Err(ASCOMError::invalid_value(format_args!(
                    "Subframe {}+{} is out of image bounds",
                    stringify!($start),
                    stringify!($len)
                )))
            }
        }
        $child.$start += $parent.$start;
        $child.$len = $parent.$len;
    };
}

#[derive(Debug, Clone, Copy)]
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

async fn camera_file_to_image(
    camera: &gphoto2::Camera,
    path: &CameraFilePath,
) -> eyre::Result<ImgWithMetadata> {
    let folder = path.folder();
    let folder = folder.as_ref();

    let filename = path.name();
    let filename = filename.as_ref();

    async {
        let fs = camera.fs();

        let camera_file = fs.download(folder, filename).await?;

        fs.delete_file(folder, filename).await?;

        let data = camera_file.get_data(gphoto2_context()).await?;

        let img = ImgWithMetadata::from_data(data.into())?;

        Ok(img)
    }
    .instrument(tracing::error_span!(
        "camera_file_to_image",
        ?folder,
        ?filename
    ))
    .await
}

struct MyCamera {
    inner: gphoto2::Camera,
    state: Arc<Mutex<State>>,
    dimensions: Size,
    iso: CachedRadioWidget,
    bulb: BulbControl,
    image_format: CachedRadioWidget,
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

#[tracing::instrument(skip(camera), ret, err)]
async fn determine_dimensions(camera: &gphoto2::Camera) -> eyre::Result<Size> {
    let camera_file_path = camera.capture_image().await?;

    let rect = camera_file_to_image(camera, &camera_file_path)
        .await?
        .crop_area;

    Ok(Size {
        width: rect.width,
        height: rect.height,
    })
}

impl MyCamera {
    pub async fn new(camera: gphoto2::Camera) -> eyre::Result<Self> {
        let dimensions = determine_dimensions(&camera).await?;

        Ok(Self {
            iso: camera.config_key("iso").await?,
            bulb: BulbControl::new(&camera).await?,
            image_format: camera
                .config_key("imageformat")
                .or_else(|_| camera.config_key("imagequality"))
                .await?,
            dimensions,
            inner: camera,
            state: Arc::new(Mutex::new(State::Idle)),
            last_exposure_start_time: Default::default(),
            last_exposure_duration: Default::default(),
            subframe: parking_lot::RwLock::new(image::math::Rect {
                x: 0,
                y: 0,
                width: dimensions.width,
                height: dimensions.height,
            }),
        })
    }

    async fn state(&self) -> tokio::sync::MutexGuard<'_, State> {
        self.state.lock().await
    }
}

#[derive(Debug)]
struct MyCameraDevice {
    descriptor: CameraDescriptor,
    camera: RwLock<Option<MyCamera>>,
}

impl MyCameraDevice {
    fn new(descriptor: CameraDescriptor) -> Self {
        Self {
            descriptor,
            camera: Default::default(),
        }
    }

    async fn camera(&self) -> ASCOMResult<RwLockReadGuard<'_, MyCamera>> {
        RwLockReadGuard::try_map(self.camera.read().await, |camera| camera.as_ref())
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
        // if channel is already closed, this will return an error - ignore it as it still means that we're done
        let _ = done_rx.wait_for(|&done| done).await;
        Ok(())
    }
}

fn convert_err(err: impl std::fmt::Display) -> ASCOMError {
    // TODO: more granular error codes.
    ASCOMError::unspecified(format_args!("Camera error: {err:#}"))
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
                    gphoto2_context()
                        .get_camera(&self.descriptor)
                        .await
                        .map_err(convert_err)?,
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

    async fn max_adu(&self) -> ASCOMResult<i32> {
        Ok(u16::MAX.into())
    }

    async fn full_well_capacity(&self) -> ASCOMResult<f64> {
        Ok(u16::MAX.into())
    }

    async fn gain(&self) -> ASCOMResult<i32> {
        self.camera().await?.iso.choice_idx()
    }

    async fn set_gain(&self, gain: i32) -> ASCOMResult {
        self.camera().await?.iso.set_choice_idx(gain)
    }

    async fn gains(&self) -> ASCOMResult<Vec<String>> {
        Ok(self.camera().await?.iso.choices().to_vec())
    }

    async fn has_shutter(&self) -> ASCOMResult<bool> {
        Ok(true)
    }

    async fn image_array(&self) -> ASCOMResult<ImageArray> {
        match &*self.camera().await?.state().await {
            State::AfterExposure(Ok(exposure)) => Ok(exposure.image.clone()),
            _ => Err(ASCOMError::INVALID_OPERATION),
        }
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
        Ok(self.camera().await?.subframe.read().width as _)
    }

    async fn set_num_x(&self, num_x: i32) -> ASCOMResult {
        self.camera().await?.subframe.write().width = num_x as _;
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
        Ok(match &*self.camera().await?.state().await {
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
        })
    }

    async fn readout_mode(&self) -> ASCOMResult<i32> {
        self.camera().await?.image_format.choice_idx()
    }

    async fn set_readout_mode(&self, readout_mode: i32) -> ASCOMResult {
        self.camera()
            .await?
            .image_format
            .set_choice_idx(readout_mode)
    }

    async fn readout_modes(&self) -> ASCOMResult<Vec<String>> {
        Ok(self.camera().await?.image_format.choices().to_vec())
    }

    async fn sensor_name(&self) -> ASCOMResult<String> {
        Ok(String::default())
    }

    async fn sensor_type(&self) -> ASCOMResult<SensorType> {
        let image_format = self.camera().await?.image_format.choice();
        Ok(
            // Little crude but seems to match usual gphoto2 RAW names in settinngs.
            match image_format.contains("RAW") || image_format.contains("NEF") {
                true => SensorType::RGGB,
                false => SensorType::Color,
            },
        )
    }

    async fn start_exposure(&self, duration: f64, light: bool) -> ASCOMResult {
        if duration < 0. {
            return Err(ASCOMError::invalid_value("Duration must be non-negative"));
        }
        let duration = Duration::try_from_secs_f64(duration).map_err(ASCOMError::invalid_value)?;
        let camera = self.camera().await?;
        let state = Arc::clone(&camera.state);
        let mut state_lock = camera.state().await;
        if matches!(*state_lock, State::InExposure(_)) {
            return Err(ASCOMError::invalid_operation("Camera is already exposing"));
        }
        let last_exposure_duration = Arc::clone(&camera.last_exposure_duration);
        let bulb_toggle = camera.bulb.clone();
        let subframe = *camera.subframe.read();

        // Do this before the shot - otherwise we risk trying to update camera config
        // in the middle of a bulb exposure, which will result in a "camera busy" error.
        camera.set_config(&camera.iso).await.map_err(convert_err)?;
        camera
            .set_config(&camera.image_format)
            .await
            .map_err(convert_err)?;

        let camera = camera.inner.clone();
        let (stop_tx, stop_rx) = oneshot::channel::<StopExposure>();
        let (done_tx, done_rx) = watch::channel(false);
        let exposing_state = Arc::new(Atomic::new(CameraState::Waiting));

        *state_lock = State::InExposure(CurrentExposure {
            // this might slightly differ from actual start in the async task;
            // we use this only for progress reporting
            rough_start: Instant::now(),
            state: Arc::clone(&exposing_state),
            stop_tx: Some(stop_tx),
            done_rx,
            expected_duration: duration,
        });

        tokio::task::spawn(async move {
            let result = async {
                let bulb_exposure = bulb_toggle.start().await.map_err(convert_err)?;
                exposing_state.store(CameraState::Exposing, Ordering::Relaxed);
                let start_utc = SystemTime::now();
                let start_instant = Instant::now();
                let want_image = select! {
                    _ = sleep(duration) => true,
                    Ok(stop) = stop_rx => stop.want_image
                };
                let duration = start_instant.elapsed();
                bulb_exposure.stop().await.map_err(convert_err)?;

                if !want_image {
                    return Err(ASCOMError::invalid_operation("Exposure was aborted"));
                }

                exposing_state.store(CameraState::Reading, Ordering::Relaxed);

                let mut path = None;

                loop {
                    match camera.wait_event(std::time::Duration::from_secs(3)).await.map_err(convert_err)? {
                        CameraEvent::NewFile(new_file_path) => {
                            // Note: it's possible that we'll get multiple NewFile events for modes like RAW+JPG.
                            // User shouldn't set those modes, but might forget... for now we'll just take the last path
                            // but adjust behaviour here if it causes problems.
                            path = Some(new_file_path);
                        }
                        CameraEvent::Timeout => break,
                        CameraEvent::Unknown(_) => {},
                        e => tracing::trace!(event = ?e, "Ignoring event while waiting for exposure completion"),
                    }
                }

                let path = path.ok_or_else(|| ASCOMError::unspecified("Capture finished but didn't find file path"))?;

                exposing_state.store(CameraState::Download, Ordering::Relaxed);
                let img = camera_file_to_image(&camera, &path).await.map_err(convert_err)?;

                last_exposure_duration.store(
                    Some(img.exposure_time.unwrap_or(duration.as_secs_f64())),
                    Ordering::Relaxed,
                );

                let mut crop_area = img.crop_area;
                crop_rect_side!(subframe, crop_area, x, width);
                crop_rect_side!(subframe, crop_area, y, height);

                let image =
                    img.image
                        .crop_imm(crop_area.x, crop_area.y, crop_area.width, crop_area.height);

                let image = convert_dynamic_image(image).map_err(convert_err)?;

                Ok(SuccessfulExposure { image })
            }
            .await;

            *state.lock().await = State::AfterExposure(result);

            let _ = done_tx.send(true);
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
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let mut server = Server {
        info: CargoServerInfo!(),
        ..Default::default()
    };

    for camera_descriptor in gphoto2_context().list_cameras().await? {
        server
            .devices
            .register(MyCameraDevice::new(camera_descriptor));
    }

    server
        .listen_addr
        .set_ip(std::net::Ipv4Addr::LOCALHOST.into());
    server.listen_addr.set_port(3000);

    tracing::debug!(?server.devices, "Registered Alpaca devices");

    server.start().await
}

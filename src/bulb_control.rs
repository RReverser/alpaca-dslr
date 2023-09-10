use gphoto2::widget::{RadioWidget, ToggleWidget};
use gphoto2::Camera;

#[derive(Debug, Clone)]
enum BulbControlKind {
    Standard(ToggleWidget),
    EosRemoteRelease(RadioWidget),
}

#[derive(custom_debug::Debug, Clone)]
pub(crate) struct BulbControl {
    kind: BulbControlKind,
    #[debug(skip)]
    camera: Camera,
}

impl BulbControl {
    pub async fn new(camera: &Camera) -> eyre::Result<Self> {
        Ok(Self {
            kind: if let Ok(toggle) = camera.config_key("bulb").await {
                BulbControlKind::Standard(toggle)
            } else if let Ok(radio) = camera.config_key("eosremoterelease").await {
                BulbControlKind::EosRemoteRelease(radio)
            } else {
                eyre::bail!("Camera does not support bulb exposures")
            },
            camera: camera.clone(),
        })
    }

    async fn toggle(&self, on: bool) -> eyre::Result<()> {
        self.camera
            .set_config(match &self.kind {
                BulbControlKind::Standard(toggle) => {
                    toggle.set_toggled(on);
                    toggle
                }
                BulbControlKind::EosRemoteRelease(radio) => {
                    radio.set_choice(if on { "Immediate" } else { "Release Full" })?;
                    radio
                }
            })
            .await?;

        Ok(())
    }

    pub async fn start(self) -> eyre::Result<BulbExposure> {
        self.toggle(true).await?;
        Ok(BulbExposure { bulb: self })
    }
}

pub(crate) struct BulbExposure {
    bulb: BulbControl,
}

impl BulbExposure {
    // TODO: maybe also implement blocking Drop just in case?
    pub async fn stop(self) -> eyre::Result<()> {
        self.bulb.toggle(false).await
    }
}

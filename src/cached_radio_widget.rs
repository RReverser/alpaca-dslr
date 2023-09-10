use super::convert_err;
use ascom_alpaca::{ASCOMError, ASCOMResult};
use gphoto2::widget::{RadioWidget, Widget};
use std::ops::Deref;

/// A wrapper around RadioWidget that doesn't re-read the list of choices on each get/set.
#[derive(Debug)]
pub(crate) struct CachedRadioWidget {
    inner: RadioWidget,
    choices: Vec<String>,
}

impl From<RadioWidget> for CachedRadioWidget {
    fn from(inner: RadioWidget) -> Self {
        Self {
            choices: inner.choices_iter().collect(),
            inner,
        }
    }
}

impl TryFrom<Widget> for CachedRadioWidget {
    type Error = gphoto2::Error;

    fn try_from(widget: Widget) -> Result<Self, Self::Error> {
        RadioWidget::try_from(widget).map(Self::from)
    }
}

impl Deref for CachedRadioWidget {
    type Target = RadioWidget;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl CachedRadioWidget {
    pub fn choice_idx(&self) -> ASCOMResult<i32> {
        let choice_name = self.choice();

        self.choices
            .iter()
            .position(|name| *name == choice_name)
            .map(|index| index as _)
            .ok_or_else(|| {
                ASCOMError::unspecified(format_args!(
                    "camera error: current choice {choice_name} not found in the list of choices"
                ))
            })
    }

    pub fn set_choice_idx(&self, value: i32) -> ASCOMResult {
        let choice_name = self
            .choices
            .get(value as usize)
            .ok_or_else(|| ASCOMError::invalid_value("choice index out of range"))?;
        self.inner.set_choice(choice_name).map_err(convert_err)
    }

    pub fn choices(&self) -> &[String] {
        &self.choices
    }
}

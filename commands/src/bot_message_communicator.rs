use crate::bot_message::{TimesSettingRequest, TimesSettingResponce};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod recieved;
pub mod sender;
pub mod set;

#[derive(Debug, Error)]
pub enum TimesSettingControllerError {
    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),
    // 他のエラータイプもここに追加できます
}

pub type TimesSettingControllerResult<T> = Result<T, TimesSettingControllerError>;

pub trait TimesSettingRequestSender {
    async fn send(&self, req: TimesSettingRequest) -> TimesSettingControllerResult<()>;
}

pub trait TimesSettingRequestReciever {
    async fn recieve_and_response(
        &self,
        res: TimesSettingResponce,
    ) -> TimesSettingControllerResult<TimesSettingRequest>;
}

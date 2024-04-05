use super::request_receiver::PoiseWebhookReqReceiverError;
use super::request_sender::PoiseWebhookReqSenderError;
use super::response_receiver::PoiseWebhookResReceiverError;
use domain::thiserror;

#[derive(thiserror::Error, Debug)]
pub enum MessageCommunicatorError {
    #[error("RequestSenderError: {0}")]
    RequestSenderError(#[from] PoiseWebhookReqSenderError),
    #[error("RequestReceiverError: {0}")]
    RequestReceiverError(#[from] PoiseWebhookReqReceiverError),
    #[error("ResponseReceiverError: {0}")]
    ResponseReceiverError(#[from] PoiseWebhookResReceiverError),
}

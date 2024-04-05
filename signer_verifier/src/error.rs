use super::signer::SignError;
use super::verifier::VerifyError;
use domain::thiserror;

#[derive(thiserror::Error, Debug)]
pub enum SignerVerifierError {
    #[error("SignError: {0}")]
    SignError(#[from] SignError),
    #[error("VerifyError: {0}")]
    VerifyError(#[from] VerifyError),
}

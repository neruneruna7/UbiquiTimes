use domain::thiserror;

#[derive(thiserror::Error, Debug)]
pub enum CommandError {
    // repository
    #[error("Repository error: {0}")]
    Repository(#[from] sled_repository::error::RepositoryError),
    #[error("message communicator error: {0}")]
    MessageCommunicator(#[from] message_communicator::error::MessageCommunicatorError),
    #[error("signer verifier error: {0}")]
    SignerVerifier(#[from] signer_verifier::error::SignerVerifierError),
    #[error("ca driver error: {0}")]
    CaDriver(#[from] ca_driver::error::CaDriverError),
}

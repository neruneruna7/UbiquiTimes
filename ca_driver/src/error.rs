use super::my_ca_driver::MyCaDriverError;
use domain::thiserror;

#[derive(thiserror::Error, Debug)]
pub enum CaDriverError {
    #[error("CaDriverError: {0}")]
    MyCaDriverError(#[from] MyCaDriverError),
}

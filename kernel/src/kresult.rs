use core::alloc::AllocError;

#[derive(Debug, Copy, Clone)]
pub enum KError {
    AllocError,
    InvalidArgument,
    Unknown,
}

pub type KResult<T> = Result<T, KError>;

impl From<AllocError> for KError {
    fn from(_value: AllocError) -> Self {
        KError::AllocError
    }
}

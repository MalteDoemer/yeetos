
#[derive(Debug, Copy, Clone)]
pub enum KError {
    OutOfMemory,
    DeallocError,
    InvalidArgument,
    Unknown,
}

pub type KResult<T> = Result<T, KError>;

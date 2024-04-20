pub enum KError {
    OutOfMemory,
    InvalidArgument,
    Unknown,
}

pub type KResult<T> = Result<T, KError>;

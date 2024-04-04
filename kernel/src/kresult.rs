pub enum KError {
    Unknown,
}

pub type KResult<T> = Result<T, KError>;

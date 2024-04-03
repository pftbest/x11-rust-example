#[derive(Error, Debug)]
pub enum X11Error {
    #[error("operation failed: {0}")]
    OperationFailed(&'static str),
}

impl From<&'static str> for X11Error {
    fn from(s: &'static str) -> Self {
        X11Error::OperationFailed(s)
    }
}

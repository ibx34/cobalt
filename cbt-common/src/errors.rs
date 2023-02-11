#[derive(Debug)]
pub struct Error {
    pub inner: String,
}

impl Error {
    pub fn new(inner: String) -> Self {
        Self { inner }
    }
}

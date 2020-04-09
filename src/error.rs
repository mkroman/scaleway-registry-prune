use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "HTTP client error: {}", _0)]
    ReqwestError(#[fail(cause)] reqwest::Error),

    #[fail(display = "API error: {}", _0)]
    ApiError(String),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::ReqwestError(err)
    }
}

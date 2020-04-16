use failure::Fail;
use scaleway_sdk::Error as ScalewaySdkError;

#[derive(Fail, Debug)]
pub enum Error {
    /// Error that indicates there was a problem talking to the API through the SDK
    #[fail(display = "SDK error: {}", _0)]
    ApiError(#[fail(cause)] ScalewaySdkError),
    #[fail(display = "No image tags matches the given criteria")]
    NoMatchingImageTagsError,
    #[fail(display = "No such namespace")]
    NoSuchNamespace,
    #[fail(display = "No such image")]
    NoSuchImage,
    #[fail(display = "The image has no tags associated with it")]
    NoImageTagsError,
}

impl From<ScalewaySdkError> for Error {
    fn from(err: ScalewaySdkError) -> Error {
        Error::ApiError(err)
    }
}

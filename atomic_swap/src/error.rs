use casper_types::ApiError;

#[repr(u16)]
pub enum Error {
    InValidCaller = 0,
    InValidSecret = 1,
    NotEnoughBalance = 2,
}

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        ApiError::User(error as u16)
    }
}

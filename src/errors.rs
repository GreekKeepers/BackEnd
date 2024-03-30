use crate::communication::ChannelType;
use reqwest::Error as ReqwestError;
use thedex::errors::Error as TheDexError;
use thiserror::Error;
use warp::reject;

#[derive(Error, Debug)]
pub enum ManagerError {
    #[error("Channel `{0:?}` not present")]
    ChannelIsNotPresent(ChannelType),

    #[error("Feed for an address `{0:?}` not registered")]
    FeedDoesntExist(String),
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Db Error: {0}")]
    DbError(sqlx::Error),

    #[error("The game `{0}` for network `{1}` wasn't found")]
    GameDoesntExist(i64, String),

    #[error("The game with ID: `{0}` doesn't exist")]
    GameWithIDDoesntExist(i64),

    #[error("Bad signature provided address: `{0}` message: `{1}` signature: `{2}`")]
    BadSignature(String, String, String),

    #[error("{0}")]
    ArbitraryError(String),

    #[error("Bad range/step provided")]
    BadRange,

    #[error("Wrong login or password")]
    WrongLoginPassword,

    #[error("No auth header found")]
    NoAuthError,

    #[error("Invalid authentication header")]
    InvalidAuthHeaderError,

    #[error("Malformed token")]
    MalformedToken,

    #[error("Bad password")]
    BadPassword,

    #[error("User Doesn't exist")]
    UserDoesntExist,

    #[error("The endpoint is not yet implemented")]
    NotImplemented,

    #[error("Error generating qr code for data `{0}`")]
    QrGenerationError(String),

    #[error("Error with TheDex: {0}")]
    TheDexError(TheDexError),

    #[error("Bad Api Key")]
    TheDexBadApiKey,

    #[error("UknownCurrency `{0}`")]
    UnknownCurrency(String),

    #[error("Error with P2Way: {0}")]
    P2WayError(p2way::errors::Error),

    #[error("Error updating amounts")]
    UpdateAmountsError,

    #[error("Error with HCaptcha: {0}")]
    HCaptchaError(hcaptcha::errors::Error),

    #[error("Bad captcha")]
    BadCaptcha,

    #[error("Error with reqwest: {0}")]
    ReqwestError(ReqwestError),

    #[error("Error with google api: {0}")]
    GoogleApiError(String),
}

impl reject::Reject for ApiError {}

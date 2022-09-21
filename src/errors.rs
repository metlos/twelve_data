use std::{error::Error as StdError, result::Result as StdResult, fmt::Display, f32::consts::E};

use serde::{Deserialize, de::DeserializeOwned};

use crate::http_client::Response;

#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "reqwest-client")]
    HttpError(reqwest::Error),

    #[cfg(feature = "surf-client")]
    HttpError(surf::Error),

    QueryConstruction(serde_urlencoded::ser::Error),

    ResponseParsing(serde_json::Error),

    DataError(String),
}

pub type Result<T> = StdResult<T, Error>;

impl StdError for Error {
    fn cause(&self) -> Option<&dyn StdError> {
        match self {
            Error::HttpError(e) => Some(e),
            Error::QueryConstruction(e) => Some(e),
            Error::ResponseParsing(e) => Some(e),
            Error::DataError(_) => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::HttpError(e) => write!(f, "HTTP error (status {})", e.status().map_or_else(|| "unknown".to_owned(), |s| s.as_str().into())),
            Error::QueryConstruction(_) => write!(f, "query construction error"),
            Error::ResponseParsing(_) => write!(f, "failed to parse the output"),
            Error::DataError(reason) => write!(f, "failed to obtain data: {}", reason),
        }
    }
}

#[cfg(feature = "reqwest-client")]
impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::HttpError(e)
    }
}

#[cfg(feature = "surf-client")]
impl From<surf::Error> for Error {
    fn from(e: surf::Error) -> Self {
        Self::HttpError(e)
    }
}

impl From<serde_urlencoded::ser::Error> for Error {
    fn from(e: serde_urlencoded::ser::Error) -> Self {
        Self::QueryConstruction(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::ResponseParsing(e)
    }
}

impl<T: DeserializeOwned> From<Response> for Result<T> {
    fn from(res: Response) -> Self {
        if res.status == 200 {
            Ok(serde_json::from_str::<T>(&res.body)?)
        } else {
            Err(Error::DataError(format!("status {}", res.status)))   
        }
    }
} 
use crate::core::PriceRequest;
use crate::core::PriceResponse;
use crate::core::QuoteRequest;
use crate::core::QuoteResponse;
use crate::core::TimeSeriesRequest;
use crate::core::TimeSeriesResponse;
use fundamentals::LogoRequest;
use fundamentals::LogoResponse;
use serde::Deserialize;
use serde_with::skip_serializing_none;
use std::fmt::Display;

use errors::{Error, Result};
use http_client::HttpClient;
use serde_derive::Serialize;

use derive_builder::Builder;

pub mod core;
pub mod errors;
pub mod http_client;
pub mod fundamentals;

const API_URL: &str = "https://api.twelvedata.com";

pub struct TwelveData {
    api_key: String,
    client: Box<dyn HttpClient>,
}

impl TwelveData {
    pub fn new(api_key: &str, client: Box<dyn HttpClient>) -> Self {
        Self {
            api_key: api_key.to_owned(),
            client: client,
        }
    }

    pub async fn time_series(&self, req: TimeSeriesRequest) -> Result<TimeSeriesResponse> {
        self.send("time_series", &req).await
    }

    pub async fn quote(&self, req: QuoteRequest) -> Result<QuoteResponse> {
        self.send("quote", &req).await
    }

    pub async fn price(&self, req: PriceRequest) -> Result<PriceResponse> {
        self.send("price", &req).await
    }

    pub async fn logo(&self, req: LogoRequest) -> Result<LogoResponse> {
        self.send("logo", &req).await
    }

    async fn send<T: serde::ser::Serialize, U: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        req: &T,
    ) -> Result<U> {
        let params = serde_urlencoded::to_string(req)?;
        let url = format!("{}/{}?{}", API_URL, endpoint, params);

        let res = self.client.get(&url, &self.api_key).await?;

        if res.status == 200 {
            let val: serde_json::Value = serde_json::from_str(&res.body)?;
            if let Some(status) = val.get("status") {
                if !status.is_string() {
                    return Err(Error::DataError(
                        "status value in the response is not a string".into(),
                    ));
                }
                if status.as_str().unwrap() == "error" {
                    let reason = if let Some(error_message) = val.get("message") {
                        error_message.as_str().unwrap()
                    } else {
                        "<unknown reasuon>"
                    };

                    return Err(Error::DataError(reason.into()));
                }
            }

            Ok(serde_json::from_value::<U>(val)?)
        } else {
            Err(Error::DataError(format!("status {}", res.status)))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Interval {
    #[serde(rename = "1min")]
    Minute,

    #[serde(rename = "5min")]
    FiveMinutes,

    #[serde(rename = "15min")]
    FifteenMinutes,

    #[serde(rename = "30min")]
    ThirtyMinutes,

    #[serde(rename = "45min")]
    FortyFiveMinutes,

    #[serde(rename = "1h")]
    Hour,

    #[serde(rename = "2h")]
    TwoHours,

    #[serde(rename = "4h")]
    FourHours,

    #[serde(rename = "1day")]
    Day,

    #[serde(rename = "1week")]
    Week,

    #[serde(rename = "1month")]
    Month,
}

impl Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Interval::Minute => "1min",
                Interval::FiveMinutes => "5min",
                Interval::FifteenMinutes => "15min",
                Interval::ThirtyMinutes => "30min",
                Interval::FortyFiveMinutes => "45min",
                Interval::Hour => "1h",
                Interval::TwoHours => "2h",
                Interval::FourHours => "4h",
                Interval::Day => "1day",
                Interval::Week => "1week",
                Interval::Month => "1month",
            }
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum InstrumentType {
    Stock,
    Index,
    ETF,
    REIT,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OutputFormat {
    JSON,
    CSV,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::JSON
    }
}

#[derive(Debug, Serialize, Deserialize, Builder, Default)]
#[builder(pattern = "owned")]
#[skip_serializing_none]
pub struct  CommonQueryParameters {
    #[builder(default, setter(strip_option))]
    pub exchange: Option<String>,

    #[builder(default, setter(strip_option))]
    pub mic_code: Option<String>,

    #[builder(default, setter(strip_option))]
    pub country: Option<String>,

    #[serde(rename = "type")]
    #[builder(default, setter(strip_option))]
    pub instrument_type: Option<InstrumentType>,

    #[serde(default)]
    #[builder(default, setter(strip_option))]
    pub format: Option<OutputFormat>,

    #[builder(default, setter(strip_option))]
    pub delimiter: Option<String>,

    #[serde(rename = "dp")]
    #[builder(default, setter(strip_option))]
    pub decimal_places: Option<u8>,

    #[builder(default, setter(strip_option))]
    pub timezone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Order {
    ASC,
    DESC,
}

#[cfg(test)]
mod test {
    use std::env;

    use tokio_test::assert_ok;

    use crate::core::TimeSeriesRequest;

    use super::*;

    #[cfg(feature = "reqwest-client")]
    fn get_client() -> Box<impl HttpClient> {
        Box::new(reqwest::Client::new())
    }

    #[cfg(feature = "surf-client")]
    fn get_client() -> Box<impl HttpClient> {
        Box::new(surf::Client::new())
    }

    fn get_api_key() -> String {
        env::var("TWELVE_DATA_API_KEY").unwrap()
    }

    #[test]
    pub fn time_series() {
        let td = TwelveData::new(&get_api_key(), get_client());

        let res = tokio_test::block_on(
            td.time_series(
                TimeSeriesRequest::builder()
                    .symbol("TSLA".into())
                    .interval(Interval::Day)
                    .output_size(10)
                    .build()
                    .unwrap(),
            ),
        );

        assert_ok!(res);
    }
}

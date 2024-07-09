use derive_builder::Builder;
use serde_derive::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Debug, Serialize, Deserialize, Builder)]
#[builder(pattern = "owned")]
#[skip_serializing_none]
pub struct LogoRequest {
    pub symbol: String,
    pub exchange: String,
    pub mic_code: String,
    pub country: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogoResponse {
    url: String,
}

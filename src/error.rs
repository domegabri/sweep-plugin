use bitcoin::blockdata::transaction::ParseOutPointError;
use serde_json::Value;
use From;

pub enum PluginError {
    Message(String),
    Json(serde_json::Error),
    BitcoinSecpError(bitcoin::secp256k1::Error),
    BitcoinKeyError(bitcoin::util::key::Error),
    BitcoinOutpointError(ParseOutPointError),
    Http(reqwest::Error),
}

impl From<&'static str> for PluginError {
    fn from(e: &'static str) -> PluginError {
        PluginError::Message(e.to_string())
    }
}

impl From<serde_json::Error> for PluginError {
    fn from(e: serde_json::Error) -> Self {
        PluginError::Json(e)
    }
}

impl From<bitcoin::secp256k1::Error> for PluginError {
    fn from(e: bitcoin::secp256k1::Error) -> Self {
        PluginError::BitcoinSecpError(e)
    }
}

impl From<bitcoin::util::key::Error> for PluginError {
    fn from(e: bitcoin::util::key::Error) -> Self {
        PluginError::BitcoinKeyError(e)
    }
}

impl From<ParseOutPointError> for PluginError {
    fn from(e: ParseOutPointError) -> Self {
        PluginError::BitcoinOutpointError(e)
    }
}

impl From<reqwest::Error> for PluginError {
    fn from(e: reqwest::Error) -> Self {
        PluginError::Http(e)
    }
}

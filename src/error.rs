use crate::util::RpcResponse;
use bitcoin::blockdata::transaction::ParseOutPointError;
use bitcoin::util::address;
use serde_json::Value;
use From;

#[derive(Debug)]
pub enum PluginError {
    Message(String),
    Json(serde_json::Error),
    BitcoinSecpError(bitcoin::secp256k1::Error),
    BitcoinKeyError(bitcoin::util::key::Error),
    BitcoinOutpointError(ParseOutPointError),
    BitcoinAddressError(address::Error),
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

impl From<address::Error> for PluginError {
    fn from(e: address::Error) -> Self {
        PluginError::BitcoinAddressError(e)
    }
}

impl From<reqwest::Error> for PluginError {
    fn from(e: reqwest::Error) -> Self {
        PluginError::Http(e)
    }
}

impl PluginError {
    pub fn get_rpc_error(&self, id: u64) -> RpcResponse {
        match self {
            PluginError::Message(s) => RpcResponse::error(id, s.to_string()),
            PluginError::Json(_e) => RpcResponse::error(id, "json error".to_string()),
            PluginError::BitcoinSecpError(_e) => RpcResponse::error(id, "secp error".to_string()),
            PluginError::BitcoinKeyError(_e) => {
                RpcResponse::error(id, "bad private key".to_string())
            }
            PluginError::BitcoinOutpointError(_e) => {
                RpcResponse::error(id, "error parsing outpoint".to_string())
            }
            PluginError::BitcoinAddressError(_e) => {
                RpcResponse::error(id, "address not valid".to_string())
            }
            PluginError::Http(_e) => RpcResponse::error(id, "http request error".to_string()), //TODO: give more http error info
        }
    }
}

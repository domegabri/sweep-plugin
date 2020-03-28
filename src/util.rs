use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::Value;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub struct Req {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RpcRequestType {
    GetManifest(Req),
    Init(Req),
    Sweep(Req),
}

impl FromStr for RpcRequestType {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let req_string = s.clone();
        let req: Req = serde_json::from_str(&req_string)?;
        match req.method.as_str() {
            "getmanifest" => Ok(RpcRequestType::GetManifest(req)),
            "init" => Ok(RpcRequestType::Init(req)),
            "sweep" => Ok(RpcRequestType::Sweep(req)),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<Value>,
    pub error: Option<Value>,
}

impl RpcResponse {
    pub fn error(id: u64, error: String) -> RpcResponse {
        let v = serde_json::json!({ "error": error });
        RpcResponse {
            jsonrpc: "2.0".to_string(),
            id: id,
            result: None,
            error: Some(v),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcMethod {
    pub name: String,
    pub usage: String,
    pub description: String,
}

impl RpcMethod {
    pub fn sweep() -> RpcMethod {
        RpcMethod {
            name: "sweep".to_string(),
            usage: "lightning-cli sweep privatekey destinationaddress [feerate]".to_string(),
            description: "Command to sweep coins from a wif private key".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub options: Vec<Value>,
    pub rpcmethods: Vec<RpcMethod>,
    pub subscriptions: Vec<Value>,
    pub hooks: Vec<Value>,
    pub features: Value,
    pub dynamic: bool,
}

impl Manifest {
    pub fn new_for_method(method: RpcMethod) -> Manifest {
        Manifest {
            options: vec![],
            rpcmethods: vec![method],
            subscriptions: vec![],
            hooks: vec![],
            features: serde_json::json!("{}"),
            dynamic: false,
        }
    }
}

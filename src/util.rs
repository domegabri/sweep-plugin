use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::Value;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub struct req {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum rpcRequestType {
    GetManifest(req),
    Init(req),
    Sweep(req),
}

impl FromStr for rpcRequestType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let req_string = s.clone();
        let req: req = serde_json::from_str(&req_string).expect("parsing response panicked");
        match req.method.as_str() {
            "getmanifest" => Ok(rpcRequestType::GetManifest(req)),
            "init" => Ok(rpcRequestType::Init(req)),
            "sweep" => Ok(rpcRequestType::Sweep(req)),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct resp {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<Value>,
    pub error: Option<Value>,
}

impl resp {
    pub fn error(id: u64, error: String) -> resp {
        let v = serde_json::json!({ "error": error });
        resp {
            jsonrpc: "2.0".to_string(),
            id: id,
            result: None,
            error: Some(v),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct rpcMethod {
    pub name: String,
    pub usage: String,
    pub description: String,
}

impl rpcMethod {
    pub fn sweep() -> rpcMethod {
        rpcMethod {
            name: "sweep".to_string(),
            usage: "lightning-cli sweep privatekey destinationaddress [feerate]".to_string(),
            description: "Command to sweep coins from a wif private key".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub options: Vec<Value>,
    pub rpcmethods: Vec<rpcMethod>,
    pub subscriptions: Vec<Value>,
    pub hooks: Vec<Value>,
    pub features: Value,
    pub dynamic: bool,
}

impl Manifest {
    pub fn new_for_method(method: rpcMethod) -> Manifest {
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

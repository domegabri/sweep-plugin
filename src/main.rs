pub mod sweep;

use crate::rpcRequestType::{GetManifest, Init, Sweep};
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::to_value;
use serde_json::Value;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::stdin;
use std::io::stdout;
use std::io::Write;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub struct req {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Value,
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
            "getmanifest" => Ok(GetManifest(req)),
            "init" => Ok(Init(req)),
            "sweep" => Ok(Sweep(req)),
            _ => unreachable!()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct resp {
    jsonrpc: String,
    id: u64,
    result: Option<Value>,
    error: Option<Value>,
}

impl resp {
    pub fn error(id: u64, error: String) -> resp {
        let v = serde_json::json!({ "error": error });;
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
    name: String,
    usage: String,
    description: String,
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
    options: Vec<Value>,
    rpcmethods: Vec<rpcMethod>,
    subscriptions: Vec<Value>,
    hooks: Vec<Value>,
    features: Value,
    dynamic: bool,
}

impl Manifest {
    fn new(method: rpcMethod) -> Manifest {
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

fn main() {
    loop {
        let stdin = stdin();
        let mut buf = String::new();

        let read = stdin.read_line(&mut buf);
        let v: Vec<&str> = buf.split("\n").collect();
        let s = v[0].to_string();

        // log lightning requests to a file
        /*let mut plugin_log = OpenOptions::new()
            .append(true)
            .open("/tmp/pluginlog")
            .unwrap();
        let res = plugin_log
            .write(serde_json::to_string(&buf).unwrap().as_bytes())
            .unwrap();*/

        match read {
            Ok(w) => {
                // getting "\n" from c-lightning: ignore if less than 2 bytes
                if w < 2 {
                    continue;
                }

                let rType = rpcRequestType::from_str(&s).unwrap();
                match rType {
                    GetManifest(r) => {
                        let id = r.id as u64;
                        let manifest = Manifest::new(rpcMethod::sweep());

                        let response = serde_json::to_value(resp {
                            jsonrpc: "2.0".to_string(),
                            id,
                            error: None,
                            result: Some(
                                serde_json::to_value(manifest).expect("serializing manifest"),
                            ),
                        })
                        .expect("to_value for response");

                        println!(
                            "{}",
                            serde_json::to_string(&response)
                                .expect("serializing response to get manifest")
                        );
                        stdout().flush();
                    }

                    Init(r) => {
                        let config = r.params.get("configuration").unwrap();
                        let lightningDir = config.get("lightning-dir").unwrap().as_str().unwrap();
                        let lightningRpc = config.get("rpc-file").unwrap().as_str().unwrap();
                        // empty response for init
                        let response = resp {
                            jsonrpc: "2.0".to_string(),
                            id: r.id as u64,
                            error: None,
                            result: Some(serde_json::json!("{}")),
                        };

                        println!("{}", serde_json::to_string(&response).unwrap());
                        stdout().flush();
                    }
                    Sweep(r) => {

                        let params = r.params.as_array().unwrap();
                        // function that takes &Vec<Value> (request params) and returns a response, with results or errors.
                        // Later println! request to stdout
                        if (params.len() < 2) {
                            let response = resp::error(r.id, format!("missing parameters: expected 2, found {}", params.len()));
                            println!("{}", serde_json::to_string(&response).unwrap());
                            stdout().flush();
                            continue; // TODO: is this correct?
                        } else if (params.len() > 2) {
                            let response = resp::error(r.id, "too many parameters".to_string());
                            println!("{}", serde_json::to_string(&response).unwrap());
                            stdout().flush();
                            continue;
                        }

                        let response = resp {
                            jsonrpc: "2.0".to_string(),
                            id: r.id as u64,
                            error: None,
                            result: Some(r.params),
                        };
                        println!("{}", serde_json::to_string(&response).unwrap());
                        stdout().flush();
                    }
                }
            }

            Err(e) => panic!("failed to read plugin stdin"),
        }
    }
}

/* GET MANIFEST
 * {"jsonrpc":"2.0","id":3,"method":"getmanifest","params":{}}
 * test response:
 * {"error":null,"id":3,"jsonrpc":"2.0","result":{"rpcmethods":[{"description":"dummy method","name":"hello","usage":"lightnin-cli hello"}]}}
 *
 *
 *
 */

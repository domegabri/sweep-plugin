use crate::reqType::{GetManifest, Init, Other};
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
use std::{thread, time};

#[derive(Debug, Serialize, Deserialize)]
pub struct req {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum reqType {
    GetManifest(req),
    Init(req),
    Other(req),
}

impl FromStr for reqType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let req_string = s.clone();
        let req: req = serde_json::from_str(&req_string).unwrap();
        match req.method.as_str() {
            "getmanifest" => Ok(GetManifest(req)),
            "init" => Ok(Init(req)),
            _ => Ok(Other(req)),
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

#[derive(Debug, Serialize, Deserialize)]
pub struct rpcMethod {
    name: String,
    usage: String,
    description: String,
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
    let stdin = stdin();
    let mut buf = String::new();
    // log stuff to file (that already exists)
    /*let mut plugin_log = OpenOptions::new().write(true)
        .open("/tmp/pluginlog")
        .unwrap();
    */

    loop {
        let millisec = time::Duration::from_millis(100);
        thread::sleep(millisec);

        match stdin.read_line(&mut buf) {
            //Ok(0) => continue,
            Ok(w) => {
                if w == 0 {
                    continue;
                }
                //let get_manifest: req = serde_json::from_str(&buf).unwrap();
                let rType = reqType::from_str(&buf).unwrap();
                match rType {
                    GetManifest(r) => {
                        let id = r.id as u64;

                        /*let res = plugin_log.write(serde_json::to_string(&r)
                        .unwrap()
                        .as_bytes())
                        .unwrap();*/

                        let hello = rpcMethod {
                            name: "hello".to_string(),
                            usage: "lightning-cli hello".to_string(),
                            description: "dummy method".to_string(),
                        };
                        let manifest = Manifest::new(hello);

                        let response = serde_json::to_value(resp {
                            jsonrpc: "2.0".to_string(),
                            id,
                            error: None,
                            result: Some(serde_json::to_value(manifest).unwrap()),
                        })
                        .unwrap();

                        println!("{}", serde_json::to_string(&response).unwrap());
                        stdout().flush();
                    }

                    Init(r) => {
                        /*let res = plugin_log.write(serde_json::to_string(&r)
                        .unwrap()
                        .as_bytes())
                        .unwrap();*/

                        let response = resp {
                            jsonrpc: "2.0".to_string(),
                            id: r.id as u64,
                            error: None,
                            result: Some(serde_json::json!("{}")),
                        }; // empty init response

                        println!("{}", serde_json::to_string(&response).unwrap());
                        stdout().flush();
                    }
                    Other(r) => {}
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

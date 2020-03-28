mod error;
pub mod sweep;
pub mod util;

use crate::util::RpcRequestType;
use crate::util::RpcRequestType::{GetManifest, Init, Sweep};
use crate::util::{Manifest, RpcMethod, RpcResponse};
use serde_json;
use std::io::stdin;
use std::io::stdout;
use std::io::Write;
use std::str::FromStr;

fn main() {
    //TODO refactor main return
    loop {
        let stdin = stdin();
        let mut buf = String::new();

        let read = stdin.read_line(&mut buf);
        let v: Vec<&str> = buf.split("\n").collect();
        let s = v[0].to_string();

        match read {
            Ok(w) => {
                // getting "\n" from c-lightning: ignore if less than 2 bytes
                if w < 2 {
                    continue;
                }

                let req_type = RpcRequestType::from_str(&s).unwrap();
                match req_type {
                    GetManifest(r) => {
                        let id = r.id as u64;
                        let manifest = Manifest::new_for_method(RpcMethod::sweep());

                        let response = serde_json::to_value(RpcResponse {
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
                        stdout().flush().unwrap();
                    }

                    Init(r) => {
                        let config = r.params.get("configuration").unwrap();
                        let _lightning_dir = config.get("lightning-dir").unwrap().as_str().unwrap();
                        let _lightning_rpc = config.get("rpc-file").unwrap().as_str().unwrap();
                        // empty response for init
                        let response = RpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: r.id as u64,
                            error: None,
                            result: Some(serde_json::json!("{}")),
                        };

                        println!("{}", serde_json::to_string(&response).unwrap());
                        stdout().flush().unwrap();
                    }
                    Sweep(r) => {
                        let params = r.params.as_array().unwrap();
                        // function that takes &Vec<Value> (request params) and returns a response, with results or errors.
                        // Later println! request to stdout
                        if params.len() < 2 {
                            let response = RpcResponse::error(
                                r.id,
                                format!("missing parameters: expected 2, found {}", params.len()),
                            );
                            println!("{}", serde_json::to_string(&response).unwrap());
                            stdout().flush().unwrap();
                            continue; // TODO: is this correct?
                        } else if params.len() > 2 {
                            let response =
                                RpcResponse::error(r.id, "too many parameters".to_string());
                            println!("{}", serde_json::to_string(&response).unwrap());
                            stdout().flush().unwrap();
                            continue;
                        }

                        let response = RpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: r.id as u64,
                            error: None,
                            result: Some(r.params),
                        };
                        println!("{}", serde_json::to_string(&response).unwrap());
                        stdout().flush().unwrap();
                    }
                }
            }

            Err(_e) => panic!("failed to read plugin stdin"),
        }
    }
}

mod error;
pub mod sweep;
pub mod util;

use crate::error::PluginError;
use crate::sweep::SweepData;
use crate::util::RpcRequestType;
use crate::util::RpcRequestType::{GetManifest, Init, Sweep};
use crate::util::{Manifest, RpcMethod, RpcResponse};
use bitcoin::hashes::hex::ToHex;
use bitcoin::Txid;
use bitcoin::{Address, Transaction};
use serde_json;
use std::io::stdin;
use std::io::stdout;
use std::io::Write;
use std::str::FromStr;

fn main() {
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

                match process_request(&req_type) {
                    Ok(resp) => {
                        println!("{}", serde_json::to_string(&resp).unwrap());
                        stdout().flush().unwrap();
                        continue;
                    }
                    Err(e) => {
                        let resp = e.get_rpc_error(req_type.id());
                        println!("{}", serde_json::to_string(&resp).unwrap());
                        stdout().flush().unwrap();
                        continue;
                    }
                }
            }

            Err(_e) => panic!("failed to read plugin stdin"),
        }
    }
}

pub fn process_request(request_type: &RpcRequestType) -> Result<RpcResponse, PluginError> {
    match request_type {
        GetManifest(r) => {
            let id = r.id as u64;
            let manifest = Manifest::new_for_method(RpcMethod::sweep());

            let response = RpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                error: None,
                result: Some(serde_json::to_value(manifest).expect("serializing manifest")),
            };

            Ok(response)
        }

        Init(r) => {
            // empty response for init
            Ok(RpcResponse {
                jsonrpc: "2.0".to_string(),
                id: r.id as u64,
                error: None,
                result: Some(serde_json::json!("{}")),
            })
        }
        Sweep(r) => {
            let params = r.params.as_array().ok_or("json error")?;

            if params.len() < 2 {
                return Err(PluginError::Message(format!(
                    "missing parameters: expected 2, found {}",
                    params.len()
                )));
            } else if params.len() > 2 {
                return Err(PluginError::Message("too many parameters".to_string()));
            }

            let priv_key = params[0].as_str().ok_or("error parsing privkey param")?;
            let sweep_data = SweepData::from_wif(priv_key)?;

            let addr = r.params[1].as_str().ok_or("error parsing address param")?;

            let dest_address = Address::from_str(addr)?;

            let tx = sweep_data.sweep(dest_address)?;
            let hex = bitcoin::consensus::serialize(&tx).to_hex();
            let id = tx.txid().to_hex();

            Ok(RpcResponse {
                jsonrpc: "2.0".to_string(),
                id: r.id as u64,
                error: None,
                result: Some(serde_json::json!({"txid": id, "hex": hex})),
            })
        }
    }
}

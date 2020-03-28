use bitcoin::blockdata::{opcodes, script};
use bitcoin::hashes::Hash; // trait
use bitcoin::secp256k1::{self, Message, Secp256k1, SignOnly};
use bitcoin::PrivateKey;
use bitcoin::SigHashType;
use bitcoin::Transaction;
use bitcoin::TxIn;
use bitcoin::TxOut;
use bitcoin::{Address, OutPoint};
use bitcoin::{Network, Script};
use reqwest::blocking::{get, Request, Response};
use reqwest::blocking::{Client, ClientBuilder};
use reqwest::{Method, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
//use std::option::NoneError;
use crate::error::PluginError;
use bitcoin::hashes::hex::ToHex;
use std::str::FromStr;

#[derive(Debug)]
pub struct SweepData {
    private_key: PrivateKey,
    utxo: TxOut,
    outpoint: OutPoint,
}

impl SweepData {
    pub fn new(private_key: PrivateKey, utxo: TxOut, outpoint: OutPoint) -> SweepData {
        SweepData {
            private_key,
            utxo,
            outpoint,
        }
    }

    pub fn from_wif(wif_key: &str) -> Result<SweepData, PluginError> {
        let secp = Secp256k1::new();

        let private_key = PrivateKey::from_wif(&wif_key)?;
        let pubkey = private_key.public_key(&secp);
        let address = Address::p2pkh(&pubkey, Network::Regtest);
        let script = address.script_pubkey();

        let url_str = format!(
            "https://blockstream.info/testnet/api/address/{}/utxo",
            address.to_string()
        );

        let url = Url::parse(&url_str).ok().ok_or("Error parsing url")?;
        let response = get(url)?; // blocking
        let r = response.text()?;
        let mut j = Value::from_str(&r)?;
        // TODO: remove unwraps
        // use std::option::NoneError?
        let obj: &Value = &j
            .as_array_mut()
            .ok_or("None found while unwrapping option")?[0]; // Vec<Value>[0]

        let txid = obj
            .get("txid")
            .ok_or("Error getting key txid from object")?
            .as_str()
            .unwrap();
        let vout = obj
            .get("vout")
            .ok_or("Error getting key vout from object")?
            .as_u64()
            .unwrap() as u32; // TODO: check for confirmation
        let value = obj
            .get("value")
            .ok_or("Error getting key value from object")?
            .as_u64()
            .unwrap();
        let outpoint = bitcoin::OutPoint::from_str(format!("{}:{}", txid, vout).as_str())?;

        let utxo = TxOut {
            script_pubkey: script,
            value: value,
        };

        Ok(SweepData {
            private_key,
            utxo,
            outpoint,
        })
    }

    pub fn sweep(&self, dest: Address) -> Result<Transaction, PluginError> {
        let secp = Secp256k1::new();
        let dest_script = dest.script_pubkey();
        let dest_value = self.utxo.value;
        let dest_out = TxOut {
            script_pubkey: dest_script.clone(),
            value: dest_value - 500 as u64, // TODO: allow to pass fee_rate
        };

        let mut tx_in = TxIn {
            previous_output: self.outpoint,
            script_sig: Script::new(),
            sequence: u32::max_value(),
            witness: Vec::new(),
        };

        let mut tx = Transaction {
            version: 2 as u32,
            lock_time: 0 as u32,
            input: vec![tx_in.clone()],
            output: vec![dest_out.clone()],
        };

        let hash = tx.signature_hash(0, &self.utxo.script_pubkey, SigHashType::All as u32);
        let msg = &bitcoin::secp256k1::Message::from_slice(&hash.into_inner()[..])?; // to check
        let k = &self.private_key.key;
        let signature = secp.sign(msg, k);
        let mut signature = signature.serialize_der().to_vec();
        signature.push(SigHashType::All as u8);

        let pubkey = self.private_key.public_key(&secp);
        let script_sig = script::Builder::new()
            .push_slice(&signature)
            .push_key(&pubkey)
            .into_script();

        tx.input[0].script_sig = script_sig.clone();

        Ok(tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::consensus::Encodable;
    use bitcoin::hashes::hex::ToHex;

    #[test]
    pub fn test_sweep() {
        let sweep_data =
            SweepData::from_wif("cUXdF9YgyfNCAXTwyuBknt6PVU1ASMJyqPFxeTLcQTfQLxfFGZrx").unwrap();
        println!("{:?}", sweep_data.utxo);
    }

    #[test]
    pub fn test_tx() {
        let secp = Secp256k1::new();
        let private =
            PrivateKey::from_wif("cUXdF9YgyfNCAXTwyuBknt6PVU1ASMJyqPFxeTLcQTfQLxfFGZrx").unwrap();
        let addr = Address::p2pkh(&private.public_key(&secp), Network::Regtest); // mqY1swhTNr2ctjqDkbDErMxubj3yFWfbEX
        let script = addr.script_pubkey();

        let outpoint = OutPoint::from_str(
            "6c4516ba377381c6be2085ad740b9ae2866c2667af9b797b98031c3c7e18c668:1",
        )
        .unwrap();

        let utxo = TxOut {
            script_pubkey: script,
            value: 100000000 as u64,
        };

        let sweep_data = SweepData::new(private, utxo, outpoint);

        println!("SWEEP DATA: {:?}", &sweep_data);

        let dest_address = Address::from_str("2MuoLNztiorA56ea1pUGrqVudJb7hLG4kq4").unwrap();

        let tx = sweep_data.sweep(dest_address).unwrap();
        let hex = bitcoin::consensus::serialize(&tx).to_hex();

        println!("{:?}", hex);
    }
}

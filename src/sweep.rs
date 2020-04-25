use bitcoin::blockdata::script;
use bitcoin::hashes::Hash; // trait
use bitcoin::secp256k1::Secp256k1;
use bitcoin::PrivateKey;
use bitcoin::SigHashType;
use bitcoin::Transaction;
use bitcoin::TxIn;
use bitcoin::TxOut;
use bitcoin::{Address, OutPoint};
use bitcoin::{Network, Script};
use reqwest::blocking::get;
use reqwest::Url;
use serde_json::Value;
//use std::option::NoneError;
use crate::error::PluginError;
use std::str::FromStr;
use std::u64;

#[derive(Debug)]
pub struct SweepData {
    private_key: PrivateKey,
    utxo: Vec<TxOut>,
    outpoint: Vec<OutPoint>,
}

impl SweepData {
    pub fn new(private_key: PrivateKey, utxo: TxOut, outpoint: OutPoint) -> SweepData {
        SweepData {
            private_key,
            utxo: vec![utxo],
            outpoint: vec![outpoint],
        }
    }

    pub fn from_wif(wif_key: &str) -> Result<SweepData, PluginError> {
        let secp = Secp256k1::new();

        let private_key = PrivateKey::from_wif(&wif_key)?;
        let pubkey = private_key.public_key(&secp);
        let address = Address::p2pkh(&pubkey, Network::Regtest);
        let script = address.script_pubkey();

        let net_string = match private_key.network {
            bitcoin::Network::Bitcoin => "",
            bitcoin::Network::Testnet => "testnet/",
            bitcoin::Network::Regtest => "regtest/",
        };

        let url_str = format!(
            "https://blockstream.info/{}api/address/{}/utxo",
            net_string.to_string(),
            address.to_string()
        );

        let url = Url::parse(&url_str).ok().ok_or("Error parsing url")?;
        let response = get(url)?; // blocking
        let r = response.text()?;
        let mut j = Value::from_str(&r)?;
        let obj = j
            .as_array_mut()
            .ok_or("None found while unwrapping option")?; // Vec<Value>

        let sweep_data = match obj.len() {
            0usize => Err(PluginError::Message("no utxos found".to_string())),
            x => {
                let mut utxos: Vec<TxOut> = Vec::with_capacity(x);
                let mut outpoints: Vec<OutPoint> = Vec::with_capacity(x);

                for o in obj {
                    let txid = o
                        .get("txid")
                        .ok_or("Error getting key txid from object")?
                        .as_str()
                        .unwrap();
                    let vout = o
                        .get("vout")
                        .ok_or("Error getting key vout from object")?
                        .as_u64()
                        .unwrap() as u32; // TODO: check for confirmation
                    let value = o
                        .get("value")
                        .ok_or("Error getting key value from object")?
                        .as_u64()
                        .unwrap();
                    let outpoint =
                        bitcoin::OutPoint::from_str(format!("{}:{}", txid, vout).as_str())?;
                    outpoints.push(outpoint);

                    let utxo = TxOut {
                        script_pubkey: script.clone(),
                        value: value,
                    };
                    utxos.push(utxo);
                }

                Ok(SweepData {
                    private_key,
                    utxo: utxos,
                    outpoint: outpoints,
                })
            }
        };

        sweep_data
    }

    pub fn sweep(&self, dest: Address, sat_byte: u64) -> Result<Transaction, PluginError> {
        let secp = Secp256k1::new();
        let dest_script = dest.script_pubkey();
        let mut dest_value: u64 = 0;

        let mut tx = Transaction {
            version: 2 as u32,
            lock_time: 0 as u32,
            input: vec![],
            output: vec![],
        };

        for i in 0..self.utxo.len() {
            let tx_in = TxIn {
                previous_output: self.outpoint[i],
                script_sig: Script::new(),
                sequence: u32::max_value(),
                witness: Vec::new(),
            };
            tx.input.push(tx_in);
            dest_value = dest_value + self.utxo[i].value;
        }

        let mut dest_out = TxOut {
            script_pubkey: dest_script.clone(),
            value: 0u64,
        };

        let fee = sat_byte * (10 + 147 * (tx.input.len() as u64) + 32);
        let value: u64 = dest_value
            .checked_sub(fee)
            .ok_or("Overflow error, fee_rate too high")?;
        //TODO: check for dust

        dest_out.value = value;
        tx.output.push(dest_out);

        //sign inputs
        for i in 0..tx.input.len() {
            let hash = tx.signature_hash(i, &self.utxo[i].script_pubkey, SigHashType::All as u32);
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

            tx.input[i].script_sig = script_sig.clone();
        }

        Ok(tx)
    }

    pub fn network(&self, dest: &Address) -> Result<(), PluginError> {
        let key_network = self.private_key.network;
        let address_network = dest.network;
        if key_network != address_network {
            return Err(PluginError::Message(
                "key network and address network don't match".to_string(),
            ));
        } else {
            return Ok(());
        }
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
    pub fn test_no_utxos() {
        let sweep_data_empty =
            SweepData::from_wif("cQSmqDe1a2YRYbw69133GCy9QdMd1RjqRQG8SLfHeN747m63FxsU");
        match sweep_data_empty {
            Ok(_) => println!("no error"),
            Err(e) => println!("{}", serde_json::to_string(&e.get_rpc_error(1u64)).unwrap()),
        }
    }

    #[test]
    pub fn test_fee_rate() {
        let mut x = 10u64;
        let mut result = 20u64;
        let mut json = serde_json::json!(["value", 5]);
        let mut value = json.as_array_mut().ok_or("error").unwrap();
        let value_u64 = value[1].as_u64().unwrap();
        let result = x.checked_sub(value_u64).ok_or("error").unwrap();
        println!("{:?}", result);
    }

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
            value: 100000000u64,
        };

        let sweep_data = SweepData::new(private, utxo, outpoint);

        println!("SWEEP DATA: {:?}", &sweep_data);

        let dest_address = Address::from_str("2MuoLNztiorA56ea1pUGrqVudJb7hLG4kq4").unwrap();

        let tx = sweep_data.sweep(dest_address, 2u64).unwrap();
        let hex = bitcoin::consensus::serialize(&tx).to_hex();

        println!("{:?}", hex);
    }
}

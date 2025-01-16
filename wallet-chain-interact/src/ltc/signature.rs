use super::{provider::Provider, utxos::Usedutxo};
use crate::ltc::script::BtcScript;
use litecoin::{
    ecdsa,
    key::{Keypair, Secp256k1, TapTweak, TweakedKeypair},
    script::{self, PushBytes},
    secp256k1::{self, All, Message},
    sighash::{Prevouts, SighashCache},
    Amount, CompressedPublicKey, EcdsaSighashType, PrivateKey, ScriptBuf, TapSighashType,
    Transaction, TxOut, Witness,
};
use wallet_types::chain::address::r#type::LtcAddressType;

pub struct LtcSignature {
    secp: Secp256k1<All>,
    used_utxo: Usedutxo,
    private_key: PrivateKey,
}

impl LtcSignature {
    pub fn new(key_str: &str, used_utxo: Usedutxo) -> crate::Result<Self> {
        let secp = Secp256k1::new();

        let private_key = litecoin::PrivateKey::from_wif(key_str)
            .map_err(|e| crate::Error::SignError(e.to_string()))?;

        Ok(Self {
            secp,
            used_utxo,
            private_key,
        })
    }

    pub async fn sign(
        self,
        address_type: LtcAddressType,
        provider: &Provider,
        transaction: &mut Transaction,
    ) -> crate::Result<()> {
        match address_type {
            LtcAddressType::P2pkh => self.p2pkh(transaction)?,
            LtcAddressType::P2wpkh => self.p2wpkh(transaction)?,
            LtcAddressType::P2shWpkh => self.p2sh_wpkh(transaction)?,
            LtcAddressType::P2tr => self.p2tr(transaction, provider).await?,
            _ => {
                return Err(crate::Error::SignError(format!(
                    "address type not support {address_type:?}",
                )))
            }
        }
        Ok(())
    }

    pub fn p2pkh(&self, tx: &mut Transaction) -> crate::Result<()> {
        let sk = self.private_key.inner;
        let sighash_type = EcdsaSighashType::All as u32;

        let pk = self.private_key.public_key(&self.secp);
        let script = ScriptBuf::new_p2pkh(&pk.pubkey_hash());

        for i in 0..tx.input.len() {
            let cache = SighashCache::new(&mut *tx);
            let sighash = cache
                .legacy_signature_hash(i, &script, sighash_type)
                .map_err(|e| crate::Error::SignError(format!("p2pkh build sign hash err{e:}")))?;

            let msg = secp256k1::Message::from(sighash);
            let signature = litecoin::ecdsa::Signature {
                signature: self.secp.sign_ecdsa(&msg, &sk),
                sighash_type: EcdsaSighashType::All,
            };

            tx.input[i].script_sig = BtcScript::sign_script_sig(signature, pk);
        }
        Ok(())
    }

    pub fn p2wpkh(&self, tx: &mut Transaction) -> crate::Result<()> {
        let sk = self.private_key.inner;
        let pk = sk.public_key(&self.secp);

        let sighash_type = EcdsaSighashType::All;

        let compressed_pk = CompressedPublicKey::from_private_key(&self.secp, &self.private_key)
            .map_err(|e| crate::Error::SignError(format!("failed to get compressed_pk {e:}")))?;

        let script = ScriptBuf::new_p2wpkh(&compressed_pk.wpubkey_hash());

        for i in 0..tx.input.len() {
            let previous = &tx.input[i].previous_output;
            let amount = self.get_amount(previous.txid, previous.vout)?;

            let mut cache = SighashCache::new(&mut *tx);
            let sighash = cache
                .p2wpkh_signature_hash(i, &script, amount, sighash_type)
                .map_err(|e| {
                    crate::Error::SignError(format!("failed to compute sighash for p2wpkh{e:}"))
                })?;

            let msg = secp256k1::Message::from(sighash);
            let signature = litecoin::ecdsa::Signature {
                signature: self.secp.sign_ecdsa(&msg, &sk),
                sighash_type,
            };
            *cache.witness_mut(i).unwrap() = Witness::p2wpkh(&signature, &pk);
        }
        Ok(())
    }

    pub fn p2sh_wpkh(&self, tx: &mut Transaction) -> crate::Result<()> {
        let sk = self.private_key.inner;
        let pk = sk.public_key(&self.secp);

        let sighash_type = EcdsaSighashType::All;

        let compressed_pk = CompressedPublicKey::from_private_key(&self.secp, &self.private_key)
            .map_err(|e| crate::Error::SignError(format!("failed to get compressed_pk {e:}")))?;

        let builder = script::Builder::new()
            .push_int(0)
            .push_slice(compressed_pk.pubkey_hash())
            .into_script();
        let mut script_sig = ScriptBuf::new();
        let bb: &PushBytes = builder.as_bytes().try_into().unwrap();
        script_sig.push_slice(bb);

        for i in 0..tx.input.len() {
            let previous = &tx.input[i].previous_output;
            let amount = self.get_amount(previous.txid, previous.vout)?;

            let mut cache = SighashCache::new(&mut *tx);
            let sighash = cache
                .p2wpkh_signature_hash(i, &builder, amount, sighash_type)
                .map_err(|e| {
                    crate::Error::SignError(format!("failed to compute sighash for p2sh_wpkh{e:}"))
                })?;

            let msg = Message::from(sighash);
            let signature = ecdsa::Signature {
                signature: self.secp.sign_ecdsa(&msg, &sk),
                sighash_type,
            };
            tx.input[i].script_sig = script_sig.clone();

            let mut witness = Witness::new();
            witness.push(signature.serialize());
            witness.push(pk.serialize());

            tx.input[i].witness = witness;
        }

        Ok(())
    }

    // pub async fn multisig_sign_v1(
    //     &self,
    //     address_type: BtcAddressType,
    //     script: ScriptBuf,
    //     tx: Transaction,
    //     provider: &Provider,
    // ) -> crate::Result<Vec<Vec<u8>>> {
    //     match address_type {
    //         BtcAddressType::P2sh => self.p2sh(&tx, script),
    //         BtcAddressType::P2wsh | BtcAddressType::P2shWsh => self.p2wsh(&tx, script),
    //         BtcAddressType::P2trSh => self.p2tr_sh(&tx, script, provider).await,
    //         _ => panic!("sign not support multisig address"),
    //     }
    // }

    // p2sh multisig sign
    // pub fn p2sh(&self, tx: &Transaction, script: ScriptBuf) -> crate::Result<Vec<Vec<u8>>> {
    //     let sk = self.private_key.inner;
    //     let sighash_type = EcdsaSighashType::All;

    //     let len = tx.input.len();
    //     let mut sig = vec![];
    //     for i in 0..len {
    //         let sighash = SighashCache::new(tx)
    //             .legacy_signature_hash(i, &script, sighash_type as u32)
    //             .map_err(|e| {
    //                 crate::Error::SignError(format!("p2sh failed to compute sighash{e:}"))
    //             })?;

    //         let msg = Message::from(sighash);
    //         let signature = litecoin::ecdsa::Signature {
    //             signature: self.secp.sign_ecdsa(&msg, &sk),
    //             sighash_type,
    //         };

    //         sig.push(signature.to_vec());
    //     }
    //     Ok(sig)
    // }

    // // p2wsh multisig sign
    // pub fn p2wsh(&self, tx: &Transaction, script: ScriptBuf) -> crate::Result<Vec<Vec<u8>>> {
    //     let sk = self.private_key.inner;
    //     let sighash_type = EcdsaSighashType::All;

    //     let mut sig = vec![];
    //     for i in 0..tx.input.len() {
    //         let previous = &tx.input[i].previous_output;
    //         let amount = self.get_amount(previous.txid, previous.vout).unwrap();

    //         let sighash = SighashCache::new(tx)
    //             .p2wsh_signature_hash(i, &script, amount, sighash_type)
    //             .map_err(|e| {
    //                 crate::Error::SignError(format!("p2sh failed to compute sighash{e:}"))
    //             })?;

    //         let msg = secp256k1::Message::from(sighash);
    //         let signature = self.secp.sign_ecdsa(&msg, &sk);
    //         let signature = litecoin::ecdsa::Signature {
    //             signature,
    //             sighash_type,
    //         };
    //         sig.push(signature.to_vec());
    //     }
    //     Ok(sig)
    // }

    pub fn get_amount(&self, txid: litecoin::Txid, vout: u32) -> crate::Result<Amount> {
        let key = format!("{}-{}", txid, vout);

        let utxo = self.used_utxo.get(&key).ok_or(crate::Error::Other(
            "sign get_amount(),not found!".to_string(),
        ))?;

        Ok(Amount::from_sat(utxo.value))
    }

    pub async fn p2tr(&self, tx: &mut Transaction, provider: &Provider) -> crate::Result<()> {
        let keypair = Keypair::from_secret_key(&self.secp, &self.private_key.inner);

        let mut prevouts = vec![];

        let len = tx.input.len();
        for i in 0..len {
            // TODO： 是否有更好的方式获取签名的script_pubkey,又去rpc node 查询了一次 增加了网络io
            let tx_id = tx.input[i].previous_output.txid;
            let index = tx.input[i].previous_output.vout;
            let out = provider.utxo_out(&tx_id.to_string(), index).await?;
            let tx_out = TxOut::try_from(out).unwrap();
            prevouts.push(tx_out);
        }
        let prevouts = Prevouts::All(&prevouts);

        let mut sighasher = SighashCache::new(&mut *tx);

        let sighash_type = TapSighashType::Default;
        for i in 0..len {
            let sighash = sighasher
                .taproot_key_spend_signature_hash(i, &prevouts, sighash_type)
                .map_err(|e| {
                    crate::Error::SignError(format!("p2tr failed to compute sighash{e:}"))
                })?;

            let tweaked: TweakedKeypair = keypair.tap_tweak(&self.secp, None);
            let msg = Message::from(sighash);
            let signature = self.secp.sign_schnorr(&msg, &tweaked.to_inner());
            let signature = litecoin::taproot::Signature {
                signature,
                sighash_type,
            };

            *sighasher.witness_mut(i).unwrap() = Witness::p2tr_key_spend(&signature);
        }
        Ok(())
    }

    // pub async fn p2tr_sh(
    //     &self,
    //     tx: &Transaction,
    //     script: ScriptBuf,
    //     provider: &Provider,
    // ) -> crate::Result<Vec<Vec<u8>>> {
    //     let keypair = Keypair::from_secret_key(&self.secp, &self.private_key.inner);

    //     let mut prevouts = vec![];
    //     let len = tx.input.len();
    //     for i in 0..len {
    //         // TODO： 是否有更好的方式获取签名的script_pubkey,又去rpc node 查询了一次 增加了网络io
    //         let tx_id = tx.input[i].previous_output.txid;
    //         let index = tx.input[i].previous_output.vout;
    //         let out = provider.utxo_out(&tx_id.to_string(), index).await?;
    //         let tx_out = TxOut::try_from(out).unwrap();
    //         prevouts.push(tx_out);
    //     }
    //     let prevouts = Prevouts::All(&prevouts);

    //     let mut sig = vec![];
    //     let sighash_type = TapSighashType::Default;
    //     let script_path = ScriptPath::with_defaults(&script);

    //     let mut sighasher = SighashCache::new(tx);
    //     for i in 0..len {
    //         let sighash = sighasher
    //             .taproot_script_spend_signature_hash(
    //                 i,
    //                 &prevouts,
    //                 script_path.clone(),
    //                 sighash_type,
    //             )
    //             .map_err(|e| {
    //                 crate::Error::SignError(format!("p2tr-sh failed to compute sighash{e:}"))
    //             })?;

    //         let msg = Message::from(sighash);
    //         let signature = litecoin::taproot::Signature {
    //             signature: self.secp.sign_schnorr(&msg, &keypair),
    //             sighash_type,
    //         };
    //         sig.push(signature.to_vec());
    //     }
    //     Ok(sig)
    // }
}

// pub struct SignatureCombiner {
//     pub signatures: Vec<String>,
//     pub redeem_script: ScriptBuf,
// }
// impl SignatureCombiner {
//     pub fn new(signatures: Vec<String>, redeem_script: ScriptBuf) -> Self {
//         Self {
//             signatures,
//             redeem_script,
//         }
//     }
// }
// impl SignatureCombiner {
//     pub fn p2sh(&self, transaction: &mut litecoin::Transaction) -> crate::Result<()> {
//         let len = transaction.input.len();

//         for i in 0..len {
//             let mut buf = ScriptBuf::new();
//             buf.push_opcode(OP_0);
//             for sign in self.signatures.iter() {
//                 let res = hex_func::bincode_decode::<Vec<Vec<u8>>>(sign)?;

//                 let sign_bytes = res[i].as_slice();
//                 let push_bytes: &PushBytes = sign_bytes.try_into().map_err(|e| {
//                     Error::SignError(format!("p2sh sign bytes to push_bytes err: {e}"))
//                 })?;
//                 buf.push_slice(push_bytes);
//             }

//             let a: &PushBytes =
//                 self.redeem_script.as_bytes().try_into().map_err(|e| {
//                     Error::SignError(format!("p2sh sign bytes to push_bytes err: {e}"))
//                 })?;
//             buf.push_slice(a);
//             transaction.input[i].script_sig = buf;
//         }
//         Ok(())
//     }

//     pub fn p2sh_wsh(&self, transaction: &mut litecoin::Transaction) -> crate::Result<()> {
//         let len = transaction.input.len();

//         for i in 0..len {
//             let builder = script::Builder::new()
//                 .push_int(0)
//                 .push_slice(self.redeem_script.wscript_hash())
//                 .into_script();
//             let mut script_sig = ScriptBuf::new();
//             let push_bytes: &PushBytes = builder
//                 .as_bytes()
//                 .try_into()
//                 .map_err(|e| Error::SignError(format!("p2sh sign bytes to push_bytes err: {e}")))?;
//             script_sig.push_slice(push_bytes);

//             let mut witness = Witness::new();
//             witness.push(Vec::new());

//             for sign in self.signatures.iter() {
//                 let res = hex_func::bincode_decode::<Vec<Vec<u8>>>(sign)?;
//                 witness.push(&res[i]);
//             }

//             witness.push(self.redeem_script.as_bytes());
//             transaction.input[i].witness = witness;
//             transaction.input[i].script_sig = script_sig;
//         }

//         Ok(())
//     }

//     pub fn p2wsh(&self, transaction: &mut litecoin::Transaction) -> crate::Result<()> {
//         let len = transaction.input.len();

//         for i in 0..len {
//             let mut witness = Witness::new();
//             witness.push(Vec::new());
//             for sign in self.signatures.iter() {
//                 let res = hex_func::bincode_decode::<Vec<Vec<u8>>>(sign)?;
//                 witness.push(&res[i]);
//             }
//             witness.push(self.redeem_script.as_bytes());
//             transaction.input[i].witness = witness;
//         }

//         Ok(())
//     }

//     pub fn p2tr_sh(
//         &self,
//         transaction: &mut litecoin::Transaction,
//         inner_key: &str,
//     ) -> crate::Result<()> {
//         let len = transaction.input.len();

//         for i in 0..len {
//             let secp = Secp256k1::new();
//             let internal_key = litecoin::XOnlyPublicKey::from_str(inner_key).unwrap();

//             let taproot_builder =
//                 TaprootBuilder::with_huffman_tree(vec![(1, self.redeem_script.clone())]).unwrap();
//             let taproot_data = taproot_builder.finalize(&secp, internal_key).unwrap();
//             let control_block = taproot_data
//                 .control_block(&(self.redeem_script.clone(), LeafVersion::TapScript))
//                 .unwrap();

//             let mut witness = Witness::new();
//             for sign in self.signatures.iter() {
//                 if sign.is_empty() {
//                     witness.push(Vec::new());
//                 } else {
//                     let res = hex_func::bincode_decode::<Vec<Vec<u8>>>(sign)?;
//                     witness.push(&res[i]);
//                 }
//             }
//             witness.push(self.redeem_script.as_bytes());
//             witness.push(control_block.serialize());
//             transaction.input[i].witness = witness;
//         }

//         Ok(())
//     }
// }

/// This method is used to estimate the size of a transaction.
/// The signature data and witness data used in the calculation are dummy data,
/// and do not represent actual transaction content.
/// It is mainly intended for estimating the transaction size and does not involve
/// actual transaction validation or signing.
pub fn predict_transaction_size(
    mut tx: litecoin::Transaction,
    change_address: litecoin::Address,
    address_type: LtcAddressType,
) -> crate::Result<usize> {
    match address_type {
        LtcAddressType::P2pkh => {
            let bytes = [
                72, 48, 69, 2, 33, 0, 199, 18, 48, 98, 71, 105, 115, 75, 245, 25, 245, 245, 235,
                127, 226, 94, 203, 186, 149, 42, 87, 185, 68, 252, 65, 245, 220, 187, 178, 212, 30,
                122, 2, 32, 55, 187, 187, 179, 154, 112, 87, 248, 204, 12, 230, 75, 34, 115, 214,
                124, 255, 7, 175, 152, 231, 35, 89, 201, 191, 229, 104, 155, 124, 20, 167, 68, 1,
                33, 2, 43, 28, 139, 236, 245, 140, 224, 167, 219, 46, 175, 86, 102, 242, 149, 199,
                200, 52, 48, 119, 224, 154, 11, 38, 102, 235, 81, 241, 203, 192, 132, 70,
            ]
            .to_vec();
            let script = ScriptBuf::from_bytes(bytes);
            for input in tx.input.iter_mut() {
                input.script_sig = script.clone();
            }
        }
        LtcAddressType::P2sh => {
            let bytes = [
                0, 72, 48, 69, 2, 33, 0, 155, 190, 251, 131, 126, 168, 191, 101, 74, 172, 149, 35,
                33, 153, 90, 216, 41, 90, 202, 144, 50, 102, 52, 235, 44, 185, 172, 153, 175, 112,
                11, 95, 2, 32, 49, 38, 46, 144, 235, 44, 27, 167, 246, 100, 39, 15, 73, 42, 140,
                76, 57, 255, 139, 229, 67, 238, 250, 34, 46, 1, 17, 164, 181, 114, 123, 224, 1, 71,
                48, 68, 2, 32, 45, 253, 86, 132, 58, 252, 222, 100, 66, 168, 251, 130, 126, 242,
                253, 136, 147, 57, 18, 248, 141, 151, 43, 117, 26, 205, 38, 134, 167, 246, 246, 23,
                2, 32, 44, 15, 45, 85, 181, 15, 103, 141, 200, 206, 100, 66, 92, 140, 207, 99, 251,
                69, 172, 233, 1, 254, 240, 35, 114, 189, 166, 60, 14, 104, 169, 250, 1, 71, 48, 68,
                2, 32, 24, 70, 18, 72, 31, 227, 209, 214, 110, 219, 50, 151, 143, 53, 250, 100,
                237, 133, 47, 201, 186, 150, 186, 24, 203, 105, 224, 188, 21, 76, 33, 12, 2, 32,
                92, 152, 210, 216, 75, 157, 51, 181, 81, 98, 118, 226, 91, 250, 71, 17, 99, 73, 19,
                232, 115, 213, 11, 195, 17, 30, 91, 206, 3, 83, 65, 85, 1, 76, 104, 32, 43, 28,
                139, 236, 245, 140, 224, 167, 219, 46, 175, 86, 102, 242, 149, 199, 200, 52, 48,
                119, 224, 154, 11, 38, 102, 235, 81, 241, 203, 192, 132, 70, 172, 32, 146, 58, 233,
                117, 115, 144, 210, 78, 57, 67, 157, 123, 211, 55, 241, 203, 253, 206, 56, 4, 142,
                224, 4, 175, 216, 142, 28, 234, 9, 151, 25, 191, 186, 32, 74, 156, 38, 217, 195,
                149, 18, 156, 140, 9, 122, 123, 37, 85, 104, 65, 14, 169, 212, 192, 147, 178, 41,
                184, 201, 106, 37, 243, 67, 91, 220, 20, 186, 82, 162,
            ]
            .to_vec();
            let script = ScriptBuf::from_bytes(bytes);
            for input in tx.input.iter_mut() {
                input.script_sig = script.clone();
            }
        }
        LtcAddressType::P2wpkh => {
            let witness_bytes = [
                &[
                    0x30, 0x45, 0x02, 0x21, 0x00, 0xc4, 0xfa, 0x6a, 0x60, 0x86, 0x92, 0xa7, 0x25,
                    0x76, 0xe2, 0xa3, 0xcd, 0x6f, 0x16, 0x45, 0x2b, 0x9a, 0x92, 0x38, 0x28, 0x1b,
                    0x3a, 0x4d, 0x77, 0x99, 0x37, 0xcc, 0xcf, 0x54, 0x23, 0x9b, 0x88, 0x02, 0x20,
                    0x48, 0x15, 0x86, 0x40, 0x6d, 0xcf, 0xe7, 0xf9, 0xbb, 0x5f, 0x19, 0x59, 0x37,
                    0x25, 0x9d, 0x74, 0x69, 0x5e, 0x2e, 0xce, 0x66, 0x82, 0x84, 0xd8, 0x6b, 0x3b,
                    0xcf, 0xf4, 0x58, 0xd4, 0xbd, 0xa5, 0x01,
                ][..],
                &[
                    0x02, 0x2b, 0x1c, 0x8b, 0xec, 0xf5, 0x8c, 0xe0, 0xa7, 0xdb, 0x2e, 0xaf, 0x56,
                    0x66, 0xf2, 0x95, 0xc7, 0xc8, 0x34, 0x30, 0x77, 0xe0, 0x9a, 0x0b, 0x26, 0x66,
                    0xeb, 0x51, 0xf1, 0xcb, 0xc0, 0x84, 0x46,
                ][..],
            ];
            let witness = Witness::from_slice(witness_bytes.as_slice());
            for input in tx.input.iter_mut() {
                input.witness = witness.clone();
            }
        }
        LtcAddressType::P2wsh => {
            let witness_bytes = [
                &[][..],
                &[
                    0x30, 0x44, 0x02, 0x20, 0x38, 0xcc, 0x50, 0xd5, 0xd3, 0xb6, 0xf3, 0xf2, 0x6a,
                    0xca, 0x37, 0xc2, 0x6a, 0xe4, 0x20, 0xcd, 0x45, 0x3f, 0xff, 0xac, 0x49, 0xca,
                    0x0b, 0xc5, 0xa7, 0xdc, 0x58, 0xa9, 0x1d, 0x01, 0x2f, 0xbc, 0x02, 0x20, 0x19,
                    0xea, 0x5a, 0x45, 0x12, 0xef, 0xd7, 0x1e, 0x84, 0x11, 0xd2, 0x0c, 0x4a, 0xf1,
                    0x87, 0x52, 0xab, 0xea, 0x58, 0xfe, 0xd6, 0xd9, 0x59, 0x9e, 0x98, 0x4b, 0x78,
                    0x0f, 0x50, 0x82, 0x57, 0x2f, 0x01,
                ][..],
                &[
                    0x30, 0x44, 0x02, 0x20, 0x2c, 0x08, 0xac, 0xed, 0x48, 0x14, 0xf5, 0x85, 0xac,
                    0x09, 0x5e, 0x6b, 0xa7, 0xa8, 0xc2, 0xee, 0xcf, 0x8c, 0x60, 0xba, 0x17, 0xea,
                    0x44, 0x47, 0xfa, 0x79, 0xce, 0xec, 0x5a, 0xa9, 0xe0, 0x7c, 0x02, 0x20, 0x5b,
                    0xe9, 0x51, 0xc2, 0x16, 0xb8, 0xad, 0xd1, 0xad, 0x4c, 0x85, 0xbb, 0xe4, 0x07,
                    0xf4, 0x72, 0x81, 0xc6, 0xe4, 0xab, 0xc5, 0x27, 0x1b, 0xbb, 0x92, 0xe3, 0x61,
                    0xab, 0x02, 0x11, 0x16, 0xfb, 0x01,
                ][..],
                &[
                    0x30, 0x45, 0x02, 0x21, 0x00, 0xa9, 0xc5, 0x1c, 0xee, 0x01, 0x95, 0xec, 0xba,
                    0x4f, 0x38, 0x41, 0x44, 0xf4, 0x98, 0xe9, 0xc3, 0x6c, 0x75, 0x35, 0x42, 0x92,
                    0xde, 0x6d, 0x0d, 0x45, 0xb8, 0x5c, 0xbd, 0xc5, 0x66, 0xb5, 0x1f, 0x02, 0x20,
                    0x3e, 0x8a, 0xd2, 0x07, 0x57, 0xbb, 0xe2, 0x37, 0xcd, 0x2a, 0x51, 0x0e, 0xfc,
                    0x32, 0xa8, 0xc4, 0xf9, 0xa7, 0x03, 0x73, 0x78, 0x3e, 0xa7, 0x9b, 0xf9, 0xfb,
                    0xa5, 0x9d, 0x78, 0xb1, 0x67, 0xdc, 0x01,
                ][..],
                &[
                    0x20, 0x2b, 0x1c, 0x8b, 0xec, 0xf5, 0x8c, 0xe0, 0xa7, 0xdb, 0x2e, 0xaf, 0x56,
                    0x66, 0xf2, 0x95, 0xc7, 0xc8, 0x34, 0x30, 0x77, 0xe0, 0x9a, 0x0b, 0x26, 0x66,
                    0xeb, 0x51, 0xf1, 0xcb, 0xc0, 0x84, 0x46, 0xac, 0x20, 0x92, 0x3a, 0xe9, 0x75,
                    0x73, 0x90, 0xd2, 0x4e, 0x39, 0x43, 0x9d, 0x7b, 0xd3, 0x37, 0xf1, 0xcb, 0xfd,
                    0xce, 0x38, 0x04, 0x8e, 0xe0, 0x04, 0xaf, 0xd8, 0x8e, 0x1c, 0xea, 0x09, 0x97,
                    0x19, 0xbf, 0xba, 0x20, 0x4a, 0x9c, 0x26, 0xd9, 0xc3, 0x95, 0x12, 0x9c, 0x8c,
                    0x09, 0x7a, 0x7b, 0x25, 0x55, 0x68, 0x41, 0x0e, 0xa9, 0xd4, 0xc0, 0x93, 0xb2,
                    0x29, 0xb8, 0xc9, 0x6a, 0x25, 0xf3, 0x43, 0x5b, 0xdc, 0x14, 0xba, 0x52, 0xa2,
                ][..],
            ];
            let witness = Witness::from_slice(witness_bytes.as_slice());
            for input in tx.input.iter_mut() {
                input.witness = witness.clone();
            }
        }
        LtcAddressType::P2tr => {
            let witness_bytes = [[
                0x0e, 0x30, 0xa4, 0x02, 0xce, 0x97, 0x5a, 0x9e, 0x97, 0xb7, 0x82, 0x2e, 0x0a, 0xff,
                0xcf, 0x0e, 0x1a, 0xde, 0xef, 0x2c, 0x10, 0x78, 0x9b, 0xa7, 0xa7, 0x5d, 0xd7, 0xd0,
                0x32, 0x3d, 0x21, 0x21, 0x91, 0x00, 0xc0, 0x32, 0x85, 0x41, 0xdb, 0x64, 0x52, 0xe8,
                0xbe, 0xf9, 0x70, 0xf3, 0x02, 0x24, 0x7f, 0x67, 0x33, 0x58, 0x15, 0xa2, 0x15, 0xbe,
                0x14, 0xf1, 0x26, 0x1f, 0x54, 0x56, 0x8c, 0x8e,
            ]];
            let witness = Witness::from_slice(witness_bytes.as_slice());
            for input in tx.input.iter_mut() {
                input.witness = witness.clone();
            }
        }
        LtcAddressType::P2shWpkh => {
            let bytes = [
                22, 0, 20, 235, 55, 162, 228, 166, 224, 55, 151, 185, 230, 245, 21, 15, 171, 242,
                160, 164, 229, 103, 81,
            ]
            .to_vec();
            let script = ScriptBuf::from_bytes(bytes);

            let witness_bytes = [
                &[
                    0x30, 0x45, 0x02, 0x21, 0x00, 0xf5, 0x5b, 0x7c, 0x11, 0xc9, 0x06, 0x87, 0x2c,
                    0xe6, 0x7b, 0xcc, 0xff, 0xc9, 0xac, 0xe1, 0xc2, 0x19, 0xcf, 0xc6, 0x53, 0xbc,
                    0x6f, 0x86, 0xee, 0x72, 0x17, 0x5d, 0x31, 0x56, 0x81, 0x51, 0x38, 0x02, 0x20,
                    0x10, 0xc4, 0x81, 0xdb, 0x3c, 0xbf, 0x56, 0x21, 0x78, 0x0d, 0x39, 0x57, 0xf2,
                    0xba, 0xb5, 0x69, 0xc6, 0x97, 0x5d, 0x76, 0xe3, 0x51, 0x7e, 0xb0, 0x9c, 0xc7,
                    0x71, 0xac, 0xfe, 0x2a, 0x1d, 0x81, 0x01,
                ][..],
                &[
                    0x02, 0x2b, 0x1c, 0x8b, 0xec, 0xf5, 0x8c, 0xe0, 0xa7, 0xdb, 0x2e, 0xaf, 0x56,
                    0x66, 0xf2, 0x95, 0xc7, 0xc8, 0x34, 0x30, 0x77, 0xe0, 0x9a, 0x0b, 0x26, 0x66,
                    0xeb, 0x51, 0xf1, 0xcb, 0xc0, 0x84, 0x46,
                ][..],
            ];
            let witness = Witness::from_slice(witness_bytes.as_slice());
            for input in tx.input.iter_mut() {
                input.script_sig = script.clone();
                input.witness = witness.clone();
            }
        }
        LtcAddressType::P2shWsh => {
            let bytes = [
                34, 0, 32, 188, 73, 172, 222, 235, 145, 178, 120, 49, 0, 34, 27, 236, 30, 156, 38,
                170, 5, 232, 236, 138, 21, 245, 60, 112, 129, 84, 3, 142, 141, 238, 164,
            ]
            .to_vec();
            let script = ScriptBuf::from_bytes(bytes);

            let witness_bytes = [
                &[][..],
                &[
                    0x30, 0x44, 0x02, 0x20, 0x77, 0x52, 0xe7, 0x28, 0x99, 0x29, 0x8d, 0xa1, 0xfb,
                    0xb1, 0x01, 0x76, 0xf1, 0xa7, 0x0c, 0x6f, 0x8d, 0xa8, 0x44, 0x21, 0xc2, 0x2c,
                    0xd8, 0xf2, 0x90, 0x9d, 0xeb, 0xc7, 0xaf, 0xcb, 0xda, 0x84, 0x02, 0x20, 0x6e,
                    0xb7, 0xbc, 0x09, 0xea, 0x80, 0x2d, 0x99, 0xe4, 0x05, 0x80, 0xff, 0xb6, 0x35,
                    0x3e, 0xdf, 0x75, 0x1b, 0xdc, 0x18, 0x37, 0x47, 0xbe, 0xdb, 0x9f, 0x1f, 0x61,
                    0x35, 0x43, 0xb9, 0x0e, 0x9d, 0x01,
                ][..],
                &[
                    0x30, 0x44, 0x02, 0x20, 0x66, 0x9e, 0x8a, 0x64, 0xb0, 0x4b, 0x6f, 0x70, 0xf6,
                    0x97, 0xdb, 0xba, 0xf5, 0x1c, 0xe1, 0x1a, 0xa8, 0x82, 0x92, 0xd3, 0x3d, 0xe4,
                    0xe8, 0xed, 0x28, 0x4e, 0xd8, 0x67, 0x3c, 0x2d, 0xf6, 0x83, 0x02, 0x20, 0x15,
                    0x34, 0x5b, 0xde, 0x2d, 0x84, 0x15, 0xa0, 0x99, 0xcb, 0x83, 0x9b, 0xc5, 0xad,
                    0xf0, 0x6b, 0xe7, 0x5e, 0x8c, 0x8f, 0x01, 0xe4, 0x48, 0x28, 0xa1, 0x33, 0xaa,
                    0x78, 0xa2, 0xf7, 0xcc, 0x57, 0x01,
                ][..],
                &[
                    0x30, 0x45, 0x02, 0x21, 0x00, 0xb6, 0xda, 0xa2, 0xea, 0xaf, 0x34, 0xc1, 0x66,
                    0x0c, 0xa8, 0x31, 0x53, 0x08, 0x64, 0xd3, 0x4b, 0x5c, 0xf1, 0x45, 0x88, 0x44,
                    0x46, 0x06, 0x0d, 0x4b, 0xb4, 0xa7, 0xe7, 0x21, 0x58, 0x89, 0x3f, 0x02, 0x20,
                    0x64, 0x1f, 0x40, 0x3e, 0x89, 0x78, 0x6c, 0x50, 0x1c, 0xc4, 0xd5, 0xf9, 0x17,
                    0xd2, 0x49, 0x1c, 0x8e, 0x7d, 0x0e, 0xbd, 0xbb, 0x5a, 0x3d, 0x27, 0xfc, 0x63,
                    0x18, 0x3c, 0xc3, 0x2b, 0xf0, 0xdd, 0x01,
                ][..],
                &[
                    0x20, 0x2b, 0x1c, 0x8b, 0xec, 0xf5, 0x8c, 0xe0, 0xa7, 0xdb, 0x2e, 0xaf, 0x56,
                    0x66, 0xf2, 0x95, 0xc7, 0xc8, 0x34, 0x30, 0x77, 0xe0, 0x9a, 0x0b, 0x26, 0x66,
                    0xeb, 0x51, 0xf1, 0xcb, 0xc0, 0x84, 0x46, 0xac, 0x20, 0x92, 0x3a, 0xe9, 0x75,
                    0x73, 0x90, 0xd2, 0x4e, 0x39, 0x43, 0x9d, 0x7b, 0xd3, 0x37, 0xf1, 0xcb, 0xfd,
                    0xce, 0x38, 0x04, 0x8e, 0xe0, 0x04, 0xaf, 0xd8, 0x8e, 0x1c, 0xea, 0x09, 0x97,
                    0x19, 0xbf, 0xba, 0x20, 0x4a, 0x9c, 0x26, 0xd9, 0xc3, 0x95, 0x12, 0x9c, 0x8c,
                    0x09, 0x7a, 0x7b, 0x25, 0x55, 0x68, 0x41, 0x0e, 0xa9, 0xd4, 0xc0, 0x93, 0xb2,
                    0x29, 0xb8, 0xc9, 0x6a, 0x25, 0xf3, 0x43, 0x5b, 0xdc, 0x14, 0xba, 0x52, 0xa2,
                ][..],
            ];
            let witness = Witness::from_slice(witness_bytes.as_slice());
            for input in tx.input.iter_mut() {
                input.script_sig = script.clone();
                input.witness = witness.clone();
            }
        }
        LtcAddressType::P2trSh => {
            let witness = [
                &[
                    0x14, 0x19, 0x46, 0xf6, 0x10, 0x59, 0xdf, 0x6b, 0x0d, 0xe0, 0x18, 0x45, 0x5d,
                    0xb9, 0xb9, 0x0b, 0x07, 0x10, 0x59, 0x3e, 0x47, 0x14, 0x7e, 0xc7, 0x23, 0x8c,
                    0xb8, 0x8f, 0x60, 0x9e, 0x71, 0x28, 0xf2, 0x42, 0xe7, 0xf1, 0x9b, 0x8a, 0x81,
                    0x83, 0x15, 0x7c, 0x99, 0x0c, 0x05, 0x2f, 0xa4, 0x3a, 0xf9, 0xdf, 0x73, 0xb7,
                    0x91, 0xd5, 0x08, 0xb9, 0x96, 0xcd, 0x0e, 0x5f, 0x57, 0xb0, 0x9a, 0x97,
                ][..],
                &[
                    0x18, 0x77, 0xff, 0xa8, 0xcd, 0xba, 0xef, 0x60, 0xff, 0xce, 0x6b, 0xbf, 0xd0,
                    0x1e, 0x27, 0x2d, 0x63, 0x3c, 0xb6, 0x50, 0x4e, 0x1f, 0x56, 0x52, 0x12, 0x44,
                    0x64, 0x01, 0x77, 0x8f, 0xfb, 0x73, 0x20, 0x3a, 0xc5, 0x82, 0xf9, 0x12, 0xd9,
                    0xea, 0x4e, 0xb5, 0x86, 0x99, 0x95, 0xe7, 0x13, 0x76, 0x6d, 0xcf, 0x4e, 0x10,
                    0xfd, 0xa7, 0x5f, 0xfd, 0x10, 0x89, 0x03, 0xec, 0x2d, 0x4b, 0x08, 0x54,
                ][..],
                &[
                    0xaf, 0x6c, 0xdd, 0xc2, 0x79, 0x15, 0x82, 0x42, 0xca, 0x15, 0x62, 0xd7, 0x7c,
                    0x15, 0xd2, 0xfa, 0xf4, 0xf8, 0xfa, 0x6b, 0xb8, 0x16, 0x30, 0xc5, 0xcd, 0x76,
                    0x46, 0x0e, 0xce, 0x9b, 0x74, 0x28, 0x6e, 0x85, 0xce, 0x02, 0xb1, 0x21, 0xcc,
                    0x4e, 0xe2, 0x87, 0x62, 0x14, 0xcc, 0x78, 0x2e, 0x73, 0x17, 0x40, 0x31, 0x40,
                    0x1d, 0x92, 0x5b, 0x72, 0x4e, 0x9b, 0x92, 0x11, 0x83, 0xd6, 0xbe, 0x3c,
                ][..],
                &[
                    0x20, 0x2b, 0x1c, 0x8b, 0xec, 0xf5, 0x8c, 0xe0, 0xa7, 0xdb, 0x2e, 0xaf, 0x56,
                    0x66, 0xf2, 0x95, 0xc7, 0xc8, 0x34, 0x30, 0x77, 0xe0, 0x9a, 0x0b, 0x26, 0x66,
                    0xeb, 0x51, 0xf1, 0xcb, 0xc0, 0x84, 0x46, 0xac, 0x20, 0x92, 0x3a, 0xe9, 0x75,
                    0x73, 0x90, 0xd2, 0x4e, 0x39, 0x43, 0x9d, 0x7b, 0xd3, 0x37, 0xf1, 0xcb, 0xfd,
                    0xce, 0x38, 0x04, 0x8e, 0xe0, 0x04, 0xaf, 0xd8, 0x8e, 0x1c, 0xea, 0x09, 0x97,
                    0x19, 0xbf, 0xba, 0x20, 0x4a, 0x9c, 0x26, 0xd9, 0xc3, 0x95, 0x12, 0x9c, 0x8c,
                    0x09, 0x7a, 0x7b, 0x25, 0x55, 0x68, 0x41, 0x0e, 0xa9, 0xd4, 0xc0, 0x93, 0xb2,
                    0x29, 0xb8, 0xc9, 0x6a, 0x25, 0xf3, 0x43, 0x5b, 0xdc, 0x14, 0xba, 0x52, 0xa2,
                ][..],
                &[
                    0xc0, 0x46, 0xc8, 0x58, 0xae, 0xc8, 0x7a, 0x3e, 0x36, 0xcf, 0x6b, 0x5d, 0x4c,
                    0xcd, 0xf1, 0x37, 0x12, 0x23, 0xce, 0x8f, 0x8c, 0x08, 0x38, 0xe7, 0xc9, 0xfb,
                    0xe4, 0x6b, 0x4d, 0x7d, 0xf9, 0x02, 0x8e,
                ][..],
            ];
            let witness = Witness::from_slice(witness.as_slice());
            for input in tx.input.iter_mut() {
                input.witness = witness.clone();
            }
        }
    }

    let mut size = tx.vsize();
    // 默认给到一个找零的地址输出大小
    let out = TxOut {
        value: Amount::from_sat(1000),
        script_pubkey: change_address.script_pubkey(),
    };

    size += out.size();

    Ok(size)
}

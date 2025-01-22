use super::{provider::Provider, utxos::Usedutxo};
use crate::ltc::script::LtcScript;
use litecoin::{
    ecdsa,
    key::{Keypair, Secp256k1, TapTweak, TweakedKeypair},
    opcodes::OP_0,
    script::{self, PushBytes},
    secp256k1::{self, All, Message},
    sighash::{Prevouts, SighashCache},
    taproot::{LeafVersion, TaprootBuilder},
    Amount, CompressedPublicKey, EcdsaSighashType, PrivateKey, ScriptBuf, TapSighashType,
    Transaction, TxOut, Witness,
};
use std::str::FromStr as _;
use wallet_types::chain::address::r#type::LtcAddressType;
use wallet_utils::hex_func;

// 用于模拟多签交易签名的参数
#[derive(Debug)]
pub struct MultisigSignParams {
    pub threlod: i8,
    pub memember: i8,
    // 脚本的16进制字符串
    pub redeem_script: String,
    pub inner_key: String,
}

impl MultisigSignParams {
    pub fn new(threlod: i8, memember: i8, redeem_script: String) -> Self {
        Self {
            threlod,
            memember,
            redeem_script,
            inner_key: String::new(),
        }
    }

    pub fn with_inner_key(mut self, inner_key: String) -> Self {
        self.inner_key = inner_key;
        self
    }

    pub fn to_script(&self) -> crate::Result<ScriptBuf> {
        let redeem_script = ScriptBuf::from_hex(&self.redeem_script)
            .map_err(|e| crate::Error::BtcScript(e.to_string()))?;

        Ok(redeem_script)
    }
}

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

            tx.input[i].script_sig = LtcScript::sign_script_sig(signature, pk);
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
    mutlsig_sign_params: &Option<MultisigSignParams>,
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
            let multisig_sign = mutlsig_sign_params
                .as_ref()
                .ok_or_else(|| crate::Error::Other("Multisig parameters missing".to_string()))?;

            let buf = estimate_p2sh(multisig_sign)?;

            for input in tx.input.iter_mut() {
                input.script_sig = buf.clone();
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
            let multisig_sign = mutlsig_sign_params
                .as_ref()
                .ok_or_else(|| crate::Error::Other("Multisig parameters missing".to_string()))?;

            let witness = estimate_p2wsh(multisig_sign)?;
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
            let multisig_sign = mutlsig_sign_params
                .as_ref()
                .ok_or_else(|| crate::Error::Other("Multisig parameters missing".to_string()))?;

            let (witness, script) = estimate_p2sh_wsh(multisig_sign)?;

            for input in tx.input.iter_mut() {
                input.script_sig = script.clone();
                input.witness = witness.clone();
            }
        }
        LtcAddressType::P2trSh => {
            let multisig_sign = mutlsig_sign_params
                .as_ref()
                .ok_or_else(|| crate::Error::Other("Multisig parameters missing".to_string()))?;
            let witness = estimate_p2tr_sh(multisig_sign)?;

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

// 默认使用签名的格式
pub const ESTIMATE_SIGN_BYTES: [u8; 72] = [
    48, 69, 2, 33, 0, 167, 108, 196, 128, 152, 212, 181, 18, 178, 16, 251, 53, 222, 24, 65, 210,
    208, 40, 134, 255, 255, 173, 19, 251, 164, 204, 5, 157, 222, 158, 173, 128, 2, 32, 90, 86, 141,
    240, 85, 230, 170, 228, 205, 228, 119, 96, 229, 55, 203, 195, 196, 151, 21, 77, 111, 8, 226,
    165, 56, 220, 35, 48, 148, 63, 88, 160, 1,
];

// 模拟p2sh 交易
fn estimate_p2sh(multisig_sign: &MultisigSignParams) -> crate::Result<ScriptBuf> {
    let mut buf = ScriptBuf::new();
    buf.push_opcode(OP_0);

    let push_bytes: &PushBytes = (&ESTIMATE_SIGN_BYTES).try_into().unwrap();
    for _i in 0..multisig_sign.threlod {
        buf.push_slice(push_bytes);
    }
    // 添加一个赎回脚本
    let rs = hex_func::hex_decode(&multisig_sign.redeem_script)?;
    let sign_bytes = rs.as_slice();

    let push_bytes: &PushBytes = sign_bytes.try_into().unwrap();
    buf.push_slice(push_bytes);

    Ok(buf)
}

// 模拟p2wsh 交易大小
fn estimate_p2wsh(multisig_sign: &MultisigSignParams) -> crate::Result<Witness> {
    let mut witness = Witness::new();
    witness.push(Vec::new());

    for _i in 0..multisig_sign.threlod {
        witness.push(ESTIMATE_SIGN_BYTES);
    }

    let script_bytes = multisig_sign.to_script()?;
    witness.push(script_bytes.as_bytes());

    Ok(witness)
}

fn estimate_p2sh_wsh(multisig_sign: &MultisigSignParams) -> crate::Result<(Witness, ScriptBuf)> {
    let bytes = [
        34, 0, 32, 188, 73, 172, 222, 235, 145, 178, 120, 49, 0, 34, 27, 236, 30, 156, 38, 170, 5,
        232, 236, 138, 21, 245, 60, 112, 129, 84, 3, 142, 141, 238, 164,
    ]
    .to_vec();
    let script = ScriptBuf::from_bytes(bytes);

    // wintess data
    let mut witness = Witness::new();
    witness.push(Vec::new());

    for _i in 0..multisig_sign.threlod {
        witness.push(ESTIMATE_SIGN_BYTES);
    }

    let reedm_script = multisig_sign.to_script()?;
    witness.push(reedm_script.as_bytes());

    Ok((witness, script))
}

fn estimate_p2tr_sh(multisig_sign: &MultisigSignParams) -> crate::Result<Witness> {
    let secp = Secp256k1::new();

    let redeem_script = multisig_sign.to_script()?;

    // 控制块
    let internal_key = litecoin::XOnlyPublicKey::from_str(&multisig_sign.inner_key).unwrap();
    let taproot_builder =
        TaprootBuilder::with_huffman_tree(vec![(1, redeem_script.clone())]).unwrap();
    let taproot_data = taproot_builder.finalize(&secp, internal_key).unwrap();
    let control_block = taproot_data
        .control_block(&(redeem_script.clone(), LeafVersion::TapScript))
        .unwrap();

    let sign_bytpes = [
        0x14, 0x19, 0x46, 0xf6, 0x10, 0x59, 0xdf, 0x6b, 0x0d, 0xe0, 0x18, 0x45, 0x5d, 0xb9, 0xb9,
        0x0b, 0x07, 0x10, 0x59, 0x3e, 0x47, 0x14, 0x7e, 0xc7, 0x23, 0x8c, 0xb8, 0x8f, 0x60, 0x9e,
        0x71, 0x28, 0xf2, 0x42, 0xe7, 0xf1, 0x9b, 0x8a, 0x81, 0x83, 0x15, 0x7c, 0x99, 0x0c, 0x05,
        0x2f, 0xa4, 0x3a, 0xf9, 0xdf, 0x73, 0xb7, 0x91, 0xd5, 0x08, 0xb9, 0x96, 0xcd, 0x0e, 0x5f,
        0x57, 0xb0, 0x9a, 0x97,
    ];

    let mut witness = Witness::new();
    for i in 0..multisig_sign.memember {
        if i < multisig_sign.threlod {
            witness.push(sign_bytpes);
        } else {
            witness.push(Vec::new());
        }
    }

    witness.push(redeem_script.as_bytes());
    witness.push(control_block.serialize());

    Ok(witness)
}

use dogcoin::{
    ScriptBuf, ecdsa,
    opcodes::all::{OP_CHECKMULTISIG, OP_CHECKSIG, OP_CHECKSIGADD, OP_GREATERTHANOREQUAL},
    script::{Builder, PushBytes},
};
use std::str::FromStr;
use wallet_types::valueobject::AddressPubkey;

pub struct DogScript;

impl DogScript {
    // 时间锁脚本
    pub fn time_lock_script(height: i64, pk: dogcoin::PublicKey) -> ScriptBuf {
        Builder::new()
            .push_int(height)
            .push_opcode(dogcoin::blockdata::opcodes::all::OP_CHECKMULTISIGVERIFY)
            .push_opcode(dogcoin::blockdata::opcodes::all::OP_DROP)
            .push_key(&pk)
            .push_opcode(dogcoin::blockdata::opcodes::all::OP_CHECKSIG)
            .into_script()
    }

    pub fn multisig_script(
        threshold: u8,
        member_lists: &[AddressPubkey],
    ) -> crate::Result<ScriptBuf> {
        let mut script = Builder::new().push_int(threshold as i64);

        for item in member_lists.iter() {
            let pk = dogcoin::PublicKey::from_str(&item.pubkey).unwrap();
            script = script.push_key(&pk);
        }

        let script = script
            .push_int(member_lists.len() as i64)
            .push_opcode(OP_CHECKMULTISIG)
            .into_script();
        Ok(script)
    }

    pub fn multisig_p2tr_script(
        threshold: u8,
        member_lists: &[AddressPubkey],
    ) -> crate::Result<ScriptBuf> {
        let mut builder = Builder::new();

        for (i, item) in member_lists.iter().enumerate() {
            let pk = dogcoin::PublicKey::from_str(&item.pubkey).unwrap();
            builder = builder.push_x_only_key(&pk.into());
            if i == 0 {
                builder = builder.push_opcode(OP_CHECKSIG);
            } else {
                builder = builder.push_opcode(OP_CHECKSIGADD);
            }
        }

        let script = builder
            .push_int(threshold as i64)
            .push_opcode(OP_GREATERTHANOREQUAL)
            .into_script();
        Ok(script)
    }

    pub fn sign_script_sig(signature: ecdsa::Signature, pk: dogcoin::PublicKey) -> ScriptBuf {
        let signature_bytes = signature.to_vec();

        let bytes: &PushBytes = signature_bytes.as_slice().try_into().unwrap();
        Builder::new().push_slice(bytes).push_key(&pk).into_script()
    }
}

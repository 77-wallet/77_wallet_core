use super::{
    errors::TonError,
    get_keypair,
    operations::BuildInternalMsg,
    params::EstimateFeeParams,
    protocol::{jettons::JettonWalletAddress, transaction::EstimateFeeResp},
    provider::Provider,
};
use crate::{ton::protocol::account::AddressInformation, types::ChainPrivateKey};
use alloy::primitives::U256;
use num_bigint::BigUint;
use std::sync::Arc;
use tonlib_core::{
    cell::{BagOfCells, Cell, EitherCellLayout},
    message::{CommonMsgInfo, InternalMessage, JettonTransferMessage, TonMessage, TransferMessage},
    wallet::{mnemonic::KeyPair, ton_wallet::TonWallet, wallet_version::WalletVersion},
    TonAddress,
};

pub struct TonChain {
    pub provider: Provider,
}
impl TonChain {
    pub fn new(provider: Provider) -> crate::Result<Self> {
        Ok(Self { provider })
    }
}

impl TonChain {
    pub async fn balance(&self, addr: &str, token: Option<String>) -> crate::Result<U256> {
        if let Some(t) = token {
            let jetton_address =
                JettonWalletAddress::wallet_address(&t, addr, &self.provider).await?;

            let result = self
                .provider
                .token_data(&jetton_address.to_base64_url())
                .await?;

            Ok(U256::from(result.balance))
        } else {
            self.provider.balance(addr).await
        }
    }

    pub async fn estimate_fee<T: BuildInternalMsg>(
        &self,
        args: T,
    ) -> crate::Result<EstimateFeeResp> {
        let now_time = wallet_utils::time::now().timestamp() as u32;
        let internal = args.build(now_time, true)?;

        let address = args.get_src().to_base64_url();
        let msg = TransferMessage::new(CommonMsgInfo::InternalMessage(internal))
            .build()
            .unwrap();

        let boc = BagOfCells::from_root(msg);
        let tx = boc.serialize(true).unwrap();

        let body = wallet_utils::bytes_to_base64(&tx);

        let params = EstimateFeeParams::new(&address, body);

        let result = self.provider.estimate_fee(params).await?;

        Ok(result)
    }

    pub async fn exec<T: BuildInternalMsg>(
        &self,
        args: T,
        key: ChainPrivateKey,
    ) -> crate::Result<String> {
        let key_pair = get_keypair(key)?;

        let wallet = TonWallet::new(WalletVersion::V4R2, key_pair).map_err(TonError::CellBuild)?;

        let address = args.get_src();
        let seqno = AddressInformation::seqno(address, &self.provider).await?;

        let now_time = wallet_utils::time::now().timestamp() as u32;

        let internal = args.build(now_time, false)?;
        let boc_str = self.build_boc(internal, now_time, wallet, seqno)?;

        self.provider.send_boc_return(boc_str).await
    }

    fn build_boc(
        &self,
        internal: InternalMessage,
        now_time: u32,
        wallet: TonWallet,
        seqno: u32,
    ) -> Result<String, TonError> {
        let expire_at = now_time + 60;

        let msg = TransferMessage::new(CommonMsgInfo::InternalMessage(internal)).build()?;
        let internal_msgs = vec![Arc::new(msg)];

        let body = wallet.create_external_body(expire_at, seqno, internal_msgs)?;
        let signed = wallet.sign_external_body(&body)?;
        let wrapped = wallet.wrap_signed_body(signed, false)?;

        let boc = BagOfCells::from_root(wrapped);
        let tx = boc.serialize(true)?;

        Ok(wallet_utils::bytes_to_base64(&tx))
    }

    pub async fn token_transfer(&self, key: ChainPrivateKey) -> crate::Result<String> {
        let src = TonAddress::from_base64_url("UQAPwCDD910mi8FO1cd5qYdfTHWwEyqMB-RsGkRv-PI2w05u")
            .unwrap();

        let seqno = AddressInformation::seqno(src.clone(), &self.provider)
            .await
            .unwrap();

        // usdt
        let jetton_master = "EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs";

        let src_jetton_address = JettonWalletAddress::wallet_address(
            jetton_master,
            "UQAPwCDD910mi8FO1cd5qYdfTHWwEyqMB-RsGkRv-PI2w05u",
            &self.provider,
        )
        .await
        .unwrap();
        tracing::warn!("src_jetton {}", src_jetton_address.to_base64_url());

        let dest = TonAddress::from_base64_url("UQBud2VI5S1IhaPm3OJ7wYUewhBSK7VhfPbnp_0tvvBpx7ze")
            .unwrap();

        let keypair = self.get_keypair(key).unwrap();
        let wallet = TonWallet::new(WalletVersion::V4R2, keypair).unwrap();

        // jetton transfer
        let query_id = wallet_utils::time::now().timestamp() as u64;
        let jetton_amount = BigUint::from(100_000u64);
        let jetton_transfer = JettonTransferMessage {
            query_id,
            amount: jetton_amount,
            destination: dest.clone(),
            response_destination: src.clone(),
            custom_payload: None,
            forward_ton_amount: BigUint::from(1u32),
            forward_payload: Arc::new(Cell::default()),
            forward_payload_layout: EitherCellLayout::Native,
        }
        .build()
        .unwrap();

        // 基础消息
        let ton_amount = BigUint::from(10000000u64);

        let now = wallet_utils::time::now().timestamp() as u32;
        let internal = InternalMessage {
            ihr_disabled: true,
            bounce: false,
            bounced: false,
            src: src.clone(),
            dest: src_jetton_address,
            value: ton_amount,
            ihr_fee: 0u32.into(),
            fwd_fee: 0u32.into(),
            created_lt: 0,
            created_at: now,
        };

        let common_msg_info = CommonMsgInfo::InternalMessage(internal);
        let transfer = TransferMessage::new(common_msg_info)
            .with_data(jetton_transfer.into())
            .build()
            .unwrap();

        let now = wallet_utils::time::now().timestamp() as u32;
        let body = wallet
            .create_external_body(now + 60, seqno, vec![Arc::new(transfer)])
            .unwrap();
        let signed = wallet.sign_external_body(&body).unwrap();
        let wrapped = wallet.wrap_signed_body(signed, false).unwrap();
        let boc = BagOfCells::from_root(wrapped);
        let tx = boc.serialize(true).unwrap();

        let base64_str = wallet_utils::bytes_to_base64(&tx);
        self.provider.send_boc_return(base64_str).await
    }

    fn get_keypair(&self, key: ChainPrivateKey) -> crate::Result<KeyPair> {
        let sk = ed25519_dalek_bip32::SecretKey::from_bytes(&key.to_bytes().unwrap()).unwrap();
        let pk = ed25519_dalek_bip32::PublicKey::from(&sk);
        let mut sk_bytes = sk.as_bytes().to_vec();
        let pk_bytes = pk.as_bytes().to_vec();
        sk_bytes.extend(&pk_bytes);
        let key_pair = KeyPair {
            secret_key: sk_bytes,
            public_key: pk_bytes,
        };

        Ok(key_pair)
    }
}

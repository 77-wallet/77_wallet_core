use super::{
    errors::TonError,
    get_keypair,
    operations::BuildInternalMsg,
    params::EstimateFeeParams,
    protocol::{
        jettons::{JettonMasterResp, JettonWalletAddress, JettonWalletResp},
        transaction::EstimateFeeResp,
    },
    provider::Provider,
};
use crate::{ton::protocol::account::AddressInformation, types::ChainPrivateKey};
use alloy::primitives::U256;
use std::sync::Arc;
use tonlib_core::{
    cell::{BagOfCells, CellBuilder},
    message::{TonMessage, TransferMessage},
    wallet::{
        ton_wallet::TonWallet, version_helper::VersionHelper, versioned::DEFAULT_WALLET_ID,
        wallet_version::WalletVersion,
    },
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
                .token_data::<JettonWalletResp>(&jetton_address.to_base64_url())
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
        let address = args.get_src();
        let seqno = AddressInformation::seqno(address.clone(), &self.provider).await?;

        let address = args.get_src().to_base64_url_flags(false, false);

        let now_time = wallet_utils::time::now().timestamp() as u32;
        let trans = args
            .build(now_time, false, &self.provider)
            .await?
            .build()
            .map_err(TonError::TonMsg)?;
        let msgs_refs = vec![Arc::new(trans)];

        // 构建body
        let ext_msg = VersionHelper::build_ext_msg(
            WalletVersion::V4R2,
            now_time + 60,
            seqno,
            DEFAULT_WALLET_ID,
            msgs_refs,
        )
        .map_err(TonError::CellBuild)?;

        let bytes = vec![
            63, 75, 245, 118, 99, 227, 251, 194, 161, 58, 74, 88, 143, 140, 10, 193, 197, 73, 16,
            230, 118, 214, 181, 42, 174, 11, 55, 135, 195, 140, 0, 221, 3, 156, 4, 115, 152, 226,
            75, 132, 53, 66, 19, 148, 98, 213, 124, 218, 182, 215, 97, 52, 71, 190, 141, 251, 157,
            245, 109, 38, 197, 132, 11, 89,
        ];
        let mut builder = CellBuilder::new();
        builder.store_slice(&bytes).map_err(TonError::CellBuild)?;
        builder.store_cell(&ext_msg).map_err(TonError::CellBuild)?;
        let signed = builder.build().map_err(TonError::CellBuild)?;

        let boc = BagOfCells::from_root(signed)
            .serialize(false)
            .map_err(TonError::CellBuild)?;
        let body = wallet_utils::bytes_to_base64(&boc);

        let params = EstimateFeeParams::new(&address, body, true);

        self.provider.estimate_fee(params).await
    }

    // exec transaction
    pub async fn exec<T: BuildInternalMsg>(
        &self,
        args: T,
        key: ChainPrivateKey,
    ) -> crate::Result<String> {
        let key_pair = get_keypair(key)?;

        let wallet = TonWallet::new(WalletVersion::V4R2, key_pair).map_err(TonError::CellBuild)?;

        let address = args.get_src();
        let seqno = AddressInformation::seqno(address.clone(), &self.provider).await?;

        let now_time = wallet_utils::time::now().timestamp() as u32;
        let internal = args.build(now_time, false, &self.provider).await?;

        let boc_str = self.build_boc(internal, now_time, wallet, seqno)?;

        self.provider.send_boc_return(boc_str).await
    }

    fn build_boc(
        &self,
        internal: TransferMessage,
        now_time: u32,
        wallet: TonWallet,
        seqno: u32,
    ) -> Result<String, TonError> {
        let expire_at = now_time + 500;

        let internal_msgs = vec![Arc::new(internal.build()?)];

        let body = wallet.create_external_body(expire_at, seqno, internal_msgs)?;

        let signed = wallet.sign_external_body(&body)?;
        let wrapped = wallet.wrap_signed_body(signed, false)?;

        let boc = BagOfCells::from_root(wrapped);
        let tx = boc.serialize(true)?;

        Ok(wallet_utils::bytes_to_base64(&tx))
    }

    pub async fn decimals(&self, address: &str) -> crate::Result<u8> {
        let result = self
            .provider
            .token_data::<JettonMasterResp>(address)
            .await?;

        result.decimal()
    }
}

use super::{
    errors::TonError,
    get_keypair,
    params::EstimateFeeParams,
    protocol::{
        jettons::{JettonMasterResp, JettonWalletAddress, JettonWalletResp},
        transaction::EstimateFeeResp,
    },
    provider::Provider,
};
use crate::{ton::protocol::jettons::JettonMeta, types::ChainPrivateKey, QueryTransactionResult};
use alloy::primitives::U256;
use tonlib_core::{
    cell::{BagOfCells, Cell, CellBuilder},
    wallet::{ton_wallet::TonWallet, wallet_version::WalletVersion},
};
use wallet_types::chain::address::r#type::TonAddressType;

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

    pub async fn estimate_fee(
        &self,
        msg_cell: Cell,
        address: &str,
        address_type: TonAddressType,
    ) -> crate::Result<EstimateFeeResp> {
        // use default sign to estimate fee
        let sign = vec![
            63, 75, 245, 118, 99, 227, 251, 194, 161, 58, 74, 88, 143, 140, 10, 193, 197, 73, 16,
            230, 118, 214, 181, 42, 174, 11, 55, 135, 195, 140, 0, 221, 3, 156, 4, 115, 152, 226,
            75, 132, 53, 66, 19, 148, 98, 213, 124, 218, 182, 215, 97, 52, 71, 190, 141, 251, 157,
            245, 109, 38, 197, 132, 11, 89,
        ];
        let version = match address_type {
            TonAddressType::V4R2 => WalletVersion::V4R2,
            TonAddressType::V5R1 => WalletVersion::V5R1,
        };
        let signed_cell = match version {
            // different order
            WalletVersion::V5R1 => {
                let mut builder = CellBuilder::new();
                builder.store_cell(&msg_cell).map_err(TonError::CellBuild)?;
                builder.store_slice(&sign).map_err(TonError::CellBuild)?;
                builder.build().map_err(TonError::CellBuild)?
            }
            _ => {
                let mut builder = CellBuilder::new();
                builder.store_slice(&sign).map_err(TonError::CellBuild)?;
                builder.store_cell(&msg_cell).map_err(TonError::CellBuild)?;
                builder.build().map_err(TonError::CellBuild)?
            }
        };

        let boc = BagOfCells::from_root(signed_cell)
            .serialize(false)
            .map_err(TonError::CellBuild)?;
        let body = wallet_utils::bytes_to_base64(&boc);

        let params = EstimateFeeParams::new(&address, body, true);

        self.provider.estimate_fee(params).await
    }

    // exec transaction
    pub async fn exec(
        &self,
        msg_cell: Cell,
        key: ChainPrivateKey,
        address_type: TonAddressType,
    ) -> crate::Result<String> {
        let key_pair = get_keypair(key)?;

        let version = match address_type {
            TonAddressType::V4R2 => WalletVersion::V4R2,
            TonAddressType::V5R1 => WalletVersion::V5R1,
        };
        let wallet = TonWallet::new(version, key_pair).map_err(TonError::CellBuild)?;

        let signed = wallet
            .sign_external_body(&msg_cell)
            .map_err(TonError::CellBuild)?;
        let wrapped = wallet
            .wrap_signed_body(signed, false)
            .map_err(TonError::TonMsg)?;

        let boc = BagOfCells::from_root(wrapped);
        let tx = boc.serialize(true).map_err(TonError::CellBuild)?;

        let boc_str = wallet_utils::bytes_to_base64(&tx);
        self.provider.send_boc_return(boc_str).await
    }

    pub async fn decimals(&self, address: &str) -> crate::Result<u8> {
        let result = self
            .provider
            .token_data::<JettonMasterResp>(address)
            .await?;

        result.decimal()
    }

    pub async fn query_tx_res(&self, _hash: &str) -> crate::Result<Option<QueryTransactionResult>> {
        // 这个需要 新的api 支持
        Ok(None)
    }

    pub async fn token_symbol(&self, token: &str) -> crate::Result<String> {
        let token_data = self.provider.token_data::<JettonMasterResp>(token).await?;

        // 从meta uri 获取 symbol
        let uri = token_data.jetton_content.data.uri;
        let mete = self
            .provider
            .client
            .client
            .get(uri)
            .send()
            .await
            .map_err(|e| wallet_utils::Error::Http(e.into()))?;
        let content = mete
            .text()
            .await
            .map_err(|e| wallet_utils::Error::Http(e.into()))?;

        let meta = wallet_utils::serde_func::serde_from_str::<JettonMeta>(&content)?;

        Ok(meta.symbol)
    }

    pub async fn token_name(&self, token: &str) -> crate::Result<String> {
        let token_data = self.provider.token_data::<JettonMasterResp>(token).await?;

        // 从meta uri 获取 symbol
        let uri = token_data.jetton_content.data.uri;
        let mete = self
            .provider
            .client
            .client
            .get(uri)
            .send()
            .await
            .map_err(|e| wallet_utils::Error::Http(e.into()))?;
        let content = mete
            .text()
            .await
            .map_err(|e| wallet_utils::Error::Http(e.into()))?;

        let meta = wallet_utils::serde_func::serde_from_str::<JettonMeta>(&content)?;

        Ok(meta.name)
    }

    pub async fn block_num(&self) -> crate::Result<u64> {
        let block = self.provider.consensus_block().await?;

        Ok(block.consensus_block)
    }
}

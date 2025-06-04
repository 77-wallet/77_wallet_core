use super::{
    consts::DEFAULT_SIGN_KEY,
    errors::TonError,
    get_keypair,
    params::EstimateFeeParams,
    protocol::{
        jettons::{JettonMasterResp, JettonWalletAddress, JettonWalletResp},
        transaction::EstimateFeeResp,
    },
    provider::Provider,
};
use crate::{QueryTransactionResult, types::ChainPrivateKey};
use alloy::primitives::U256;
use tonlib_core::{
    cell::{BagOfCells, Cell},
    tlb_types::tlb::TLB as _,
    wallet::{
        ton_wallet::TonWallet, version_helper::VersionHelper, versioned::DEFAULT_WALLET_ID,
        wallet_version::WalletVersion,
    },
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

            Ok(wallet_utils::unit::u256_from_str(
                &result.balance.to_string(),
            )?)
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
        let version = address_type.to_version();
        let key_pair = get_keypair(DEFAULT_SIGN_KEY.into())?;

        let wallet = TonWallet::new(version, key_pair.clone()).map_err(TonError::CellBuild)?;

        let signed_body = wallet
            .sign_external_body(&msg_cell)
            .map_err(TonError::CellBuild)?;

        // 知道钱包的状态,决定是否部署钱包
        let account_info = self.provider.address_information(&address).await?;

        let boc = BagOfCells::from_root(signed_body)
            .serialize(false)
            .map_err(TonError::CellBuild)?;
        let body = wallet_utils::bytes_to_base64(&boc);

        let mut params = EstimateFeeParams::new(&address, body, true);
        if !account_info.is_init() {
            let code = VersionHelper::get_code(version)
                .map_err(TonError::CellBuild)?
                .clone();
            let data = VersionHelper::get_data(version, &key_pair, DEFAULT_WALLET_ID)
                .map_err(TonError::CellBuild)?;

            params.init_code = Some(code.to_boc_b64(true).map_err(TonError::CellBuild)?);
            params.init_data = Some(data.to_boc_b64(true).map_err(TonError::CellBuild)?);
        }

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

        // 知道钱包的状态,决定是否部署钱包
        let address = wallet.address.to_base64_url();
        let account_info = self.provider.address_information(&address).await?;

        let signed_body = wallet
            .sign_external_body(&msg_cell)
            .map_err(TonError::CellBuild)?;
        let wrapped = wallet
            .wrap_signed_body(signed_body, !account_info.is_init())
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

        if let Some(symbol) = token_data.jetton_content.data.symbol {
            Ok(symbol)
        } else {
            if let Some(uri) = token_data.jetton_content.data.uri {
                let meta = self.provider.get_token_meta(&uri).await?;

                Ok(meta.symbol)
            } else {
                Ok("".to_string())
            }
        }
    }

    pub async fn token_name(&self, token: &str) -> crate::Result<String> {
        let token_data = self.provider.token_data::<JettonMasterResp>(token).await?;

        if let Some(name) = token_data.jetton_content.data.name {
            Ok(name)
        } else {
            if let Some(uri) = token_data.jetton_content.data.uri {
                let meta = self.provider.get_token_meta(&uri).await?;

                Ok(meta.name)
            } else {
                Ok("".to_string())
            }
        }
    }

    pub async fn block_num(&self) -> crate::Result<u64> {
        let block = self.provider.consensus_block().await?;

        Ok(block.consensus_block)
    }
}

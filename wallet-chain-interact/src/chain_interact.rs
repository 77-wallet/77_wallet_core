use crate::types::{
    ExecTransactionParams, Executor, FetchMultisigAddressResp, MultiSigAccount,
    MultiSigAccountResp, MultisigSignResp, MultisigTxResp, SignTransactionArgs,
};
use crate::{FeeResponse as Fee, QueryTransactionResult as tx_result, TransactionParams};
use alloy::primitives::U256;
use async_trait::async_trait;

/// The `ChainInteract` trait defines the common interface for interacting with different blockchain networks.
#[async_trait]
#[deprecated]
pub trait ChainInteract {
    /// get the balance of a token or main coin
    async fn balance(&self, addr: &str, token: Option<String>) -> crate::Result<U256>;

    /// estimate the fee for a transaction
    async fn estimate_fee(&self, params: TransactionParams) -> crate::Result<Fee>;

    /// transfer tokens or main coins from one address to another
    async fn transfer(&self, params: TransactionParams) -> crate::Result<String>;

    /// get token decimals
    async fn decimals(&self, token_addr: &str) -> crate::Result<u8>;

    /// query a transaction info by hash
    async fn query_tx_res(&self, hash: &str) -> crate::Result<Option<tx_result>>;

    async fn deploy_multisig_account(
        &self,
        params: &MultiSigAccount,
    ) -> crate::Result<MultiSigAccountResp>;

    async fn deploy_multisig_fee(&self, params: &MultiSigAccount) -> crate::Result<U256>;

    // fetch multisig contract address
    async fn multisig_address(
        &self,
        params: &MultiSigAccount,
    ) -> crate::Result<FetchMultisigAddressResp>;

    async fn build_multisig_tx(
        &self,
        params: TransactionParams,
        executor: Option<Executor>,
    ) -> crate::Result<MultisigTxResp>;

    async fn sign_multisig_tx(
        &self,
        params: SignTransactionArgs,
    ) -> crate::Result<MultisigSignResp>;

    async fn exec_multisig_tx(&self, params: ExecTransactionParams) -> crate::Result<String>;
}

use async_trait::async_trait;
use bytes::Bytes;
use espresso_types::NamespaceId;

use crate::{
    config::VerificationMode,
    error::Result,
    types::{TransactionHash, TransactionRef, Verification},
};

#[async_trait]
pub trait Backend: Clone + Send + Sync + 'static {
    async fn submit(&self, namespace: NamespaceId, payload: Bytes) -> Result<TransactionHash>;

    async fn confirm(
        &self,
        namespace: NamespaceId,
        payload: Bytes,
        hash: TransactionHash,
    ) -> Result<Option<TransactionRef>>;

    async fn get(&self, tx: TransactionRef) -> Result<Bytes>;

    async fn verify(&self, tx: TransactionRef, mode: VerificationMode) -> Result<Verification>;
}

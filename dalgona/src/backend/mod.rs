use async_trait::async_trait;
use bytes::Bytes;
use espresso_types::NamespaceId;

use crate::{
    config::VerificationMode,
    error::Result,
    types::{TransactionHash, TransactionRef, Verification},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubmittedTransaction {
    pub hash: TransactionHash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SequencedTransaction {
    pub tx: TransactionRef,
}

#[async_trait]
pub trait Backend: Clone + Send + Sync + 'static {
    async fn submit(&self, namespace: NamespaceId, payload: Bytes) -> Result<SubmittedTransaction>;

    async fn confirm(
        &self,
        namespace: NamespaceId,
        payload: Bytes,
        submission: &SubmittedTransaction,
    ) -> Result<Option<SequencedTransaction>>;

    async fn get(&self, tx: TransactionRef) -> Result<Bytes>;

    async fn verify(&self, tx: TransactionRef, mode: VerificationMode) -> Result<Verification>;
}

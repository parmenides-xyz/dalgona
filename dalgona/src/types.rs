use std::fmt;

use bytes::{Buf, BufMut, Bytes};
use committable::{Commitment, Committable};
use commonware_codec::{FixedSize, Read, ReadExt, Write};
use commonware_utils::sequence::Span;
use espresso_types::{Header, NamespaceId, Transaction};

use crate::{Error, Result};

pub type TransactionHash = Commitment<Transaction>;
pub type BlockHash = Commitment<Header>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubmissionId(pub [u8; 32]);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TransactionRef {
    pub hash: TransactionHash,
    pub index: u64,
    pub block_hash: BlockHash,
    pub block_height: u64,
    pub namespace: NamespaceId,
    pub pos_in_namespace: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TransactionKey(pub [u8; 92]);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishRequest {
    pub id: SubmissionId,
    pub payload: Bytes,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishReceipt {
    pub id: SubmissionId,
    pub tx: TransactionRef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verification {
    NoProof,
    ProofVerified,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedTransaction {
    pub tx: TransactionRef,
    pub payload: Bytes,
    pub verification: Verification,
}

impl TransactionRef {
    pub fn key(&self) -> TransactionKey {
        self.into()
    }
}

impl From<&TransactionRef> for TransactionKey {
    fn from(value: &TransactionRef) -> Self {
        let mut bytes = [0u8; 92];

        let hash: [u8; 32] = value.hash.into();
        let block_hash: [u8; 32] = value.block_hash.into();
        let namespace: u64 = value.namespace.into();

        bytes[..32].copy_from_slice(&hash);
        bytes[32..40].copy_from_slice(&value.index.to_be_bytes());
        bytes[40..72].copy_from_slice(&block_hash);
        bytes[72..80].copy_from_slice(&value.block_height.to_be_bytes());
        bytes[80..88].copy_from_slice(&namespace.to_be_bytes());
        bytes[88..92].copy_from_slice(&value.pos_in_namespace.to_be_bytes());

        Self(bytes)
    }
}

impl From<TransactionKey> for TransactionRef {
    fn from(value: TransactionKey) -> Self {
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&value.0[0..32]);

        let mut index = [0u8; 8];
        index.copy_from_slice(&value.0[32..40]);

        let mut block_hash = [0u8; 32];
        block_hash.copy_from_slice(&value.0[40..72]);

        let mut block_height = [0u8; 8];
        block_height.copy_from_slice(&value.0[72..80]);

        let mut namespace = [0u8; 8];
        namespace.copy_from_slice(&value.0[80..88]);

        let mut pos_in_namespace = [0u8; 4];
        pos_in_namespace.copy_from_slice(&value.0[88..92]);

        Self {
            hash: TransactionHash::from_raw(hash),
            index: u64::from_be_bytes(index),
            block_hash: BlockHash::from_raw(block_hash),
            block_height: u64::from_be_bytes(block_height),
            namespace: NamespaceId::from(u64::from_be_bytes(namespace)),
            pos_in_namespace: u32::from_be_bytes(pos_in_namespace),
        }
    }
}

pub fn compute_commitment(namespace: NamespaceId, payload: &[u8]) -> TransactionHash {
    Transaction::new(namespace, payload.to_vec()).commit()
}

pub fn validate_commitment(tx: &TransactionRef, payload: &[u8]) -> Result<()> {
    let commitment = compute_commitment(tx.namespace, payload);
    if commitment != tx.hash {
        return Err(Error::CommitmentMismatch);
    }
    Ok(())
}

impl fmt::Display for TransactionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl Span for TransactionKey {}

macro_rules! impl_fixed_codec {
    ($ty:ty, $size:expr, $field:tt) => {
        impl Write for $ty {
            fn write(&self, buf: &mut impl BufMut) {
                self.$field.write(buf);
            }
        }

        impl FixedSize for $ty {
            const SIZE: usize = $size;
        }

        impl Read for $ty {
            type Cfg = ();

            fn read_cfg(
                buf: &mut impl Buf,
                _: &(),
            ) -> std::result::Result<Self, commonware_codec::Error> {
                Ok(Self(<[u8; $size]>::read(buf)?))
            }
        }
    };
}

impl_fixed_codec!(SubmissionId, 32, 0);
impl_fixed_codec!(TransactionKey, 92, 0);

impl Write for TransactionRef {
    fn write(&self, buf: &mut impl BufMut) {
        self.key().write(buf);
    }
}

impl FixedSize for TransactionRef {
    const SIZE: usize = TransactionKey::SIZE;
}

impl Read for TransactionRef {
    type Cfg = ();

    fn read_cfg(buf: &mut impl Buf, _: &()) -> std::result::Result<Self, commonware_codec::Error> {
        Ok(TransactionKey::read(buf)?.into())
    }
}

impl Write for PublishRequest {
    fn write(&self, buf: &mut impl BufMut) {
        self.id.write(buf);
        self.payload.write(buf);
    }
}

impl commonware_codec::EncodeSize for PublishRequest {
    fn encode_size(&self) -> usize {
        self.id.encode_size() + self.payload.encode_size()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PayloadCfg {
    pub max_payload_bytes: usize,
}

impl Read for PublishRequest {
    type Cfg = PayloadCfg;

    fn read_cfg(
        buf: &mut impl Buf,
        cfg: &Self::Cfg,
    ) -> std::result::Result<Self, commonware_codec::Error> {
        Ok(Self {
            id: SubmissionId::read(buf)?,
            payload: Bytes::read_cfg(buf, &(0..=cfg.max_payload_bytes).into())?,
        })
    }
}

impl Write for PublishReceipt {
    fn write(&self, buf: &mut impl BufMut) {
        self.id.write(buf);
        self.tx.write(buf);
    }
}

impl FixedSize for PublishReceipt {
    const SIZE: usize = SubmissionId::SIZE + TransactionRef::SIZE;
}

impl Read for PublishReceipt {
    type Cfg = ();

    fn read_cfg(buf: &mut impl Buf, _: &()) -> std::result::Result<Self, commonware_codec::Error> {
        Ok(Self {
            id: SubmissionId::read(buf)?,
            tx: TransactionRef::read(buf)?,
        })
    }
}

impl Write for Verification {
    fn write(&self, buf: &mut impl BufMut) {
        let tag = match self {
            Self::NoProof => 0u8,
            Self::ProofVerified => 1u8,
        };
        tag.write(buf);
    }
}

impl FixedSize for Verification {
    const SIZE: usize = u8::SIZE;
}

impl Read for Verification {
    type Cfg = ();

    fn read_cfg(buf: &mut impl Buf, _: &()) -> std::result::Result<Self, commonware_codec::Error> {
        match u8::read(buf)? {
            0 => Ok(Self::NoProof),
            1 => Ok(Self::ProofVerified),
            other => Err(commonware_codec::Error::InvalidEnum(other)),
        }
    }
}

impl Write for VerifiedTransaction {
    fn write(&self, buf: &mut impl BufMut) {
        self.tx.write(buf);
        self.payload.write(buf);
        self.verification.write(buf);
    }
}

impl commonware_codec::EncodeSize for VerifiedTransaction {
    fn encode_size(&self) -> usize {
        self.tx.encode_size() + self.payload.encode_size() + self.verification.encode_size()
    }
}

impl Read for VerifiedTransaction {
    type Cfg = PayloadCfg;

    fn read_cfg(
        buf: &mut impl Buf,
        cfg: &Self::Cfg,
    ) -> std::result::Result<Self, commonware_codec::Error> {
        Ok(Self {
            tx: TransactionRef::read(buf)?,
            payload: Bytes::read_cfg(buf, &(0..=cfg.max_payload_bytes).into())?,
            verification: Verification::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction_ref_key_roundtrip() {
        let transaction_ref = TransactionRef {
            hash: TransactionHash::from_raw([1; 32]),
            index: 2,
            block_hash: BlockHash::from_raw([3; 32]),
            block_height: 4,
            namespace: NamespaceId::from(5u64),
            pos_in_namespace: 6,
        };

        let key = transaction_ref.key();

        assert_eq!(TransactionRef::from(key), transaction_ref);
    }

    #[test]
    fn commitment_includes_namespace() {
        let payload = b"payload";

        assert_ne!(
            compute_commitment(NamespaceId::from(1u64), payload),
            compute_commitment(NamespaceId::from(2u64), payload),
        );
    }
}

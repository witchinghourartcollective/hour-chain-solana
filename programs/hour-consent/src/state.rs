//! On-chain consent record layout (fixed size, manual encoding).

use crate::consent::P256_COMPRESSED_PUBKEY_LEN;

pub const CONSENT_SEED: &[u8] = b"consent";
pub const RECORD_DISCRIMINATOR: [u8; 8] = *b"HBCCONS1";
pub const RECORD_VERSION: u8 = 1;
/// 8 disc + 1 version + 1 bump + 32*3 ids + 33 pubkey + 32 digest + 8 nonce + 8 expiry
/// + 8 slot + 8 unix_ts + 32 payer
pub const CONSENT_RECORD_LEN: usize =
    8 + 1 + 1 + 96 + P256_COMPRESSED_PUBKEY_LEN + 32 + 8 + 8 + 8 + 8 + 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsentRecord {
    pub bump: u8,
    pub release_id: [u8; 32],
    pub split_version_hash: [u8; 32],
    pub contributor_id: [u8; 32],
    pub passkey_pubkey: [u8; P256_COMPRESSED_PUBKEY_LEN],
    pub digest: [u8; 32],
    pub nonce: u64,
    pub expiry: i64,
    pub recorded_slot: u64,
    pub recorded_unix_timestamp: i64,
    pub payer: [u8; 32],
}

impl ConsentRecord {
    pub fn pack_into(&self, dst: &mut [u8]) -> Option<()> {
        if dst.len() < CONSENT_RECORD_LEN {
            return None;
        }
        let mut w = Writer { buf: dst, pos: 0 };
        w.put(&RECORD_DISCRIMINATOR);
        w.put(&[RECORD_VERSION, self.bump]);
        w.put(&self.release_id);
        w.put(&self.split_version_hash);
        w.put(&self.contributor_id);
        w.put(&self.passkey_pubkey);
        w.put(&self.digest);
        w.put(&self.nonce.to_le_bytes());
        w.put(&self.expiry.to_le_bytes());
        w.put(&self.recorded_slot.to_le_bytes());
        w.put(&self.recorded_unix_timestamp.to_le_bytes());
        w.put(&self.payer);
        Some(())
    }

    pub fn unpack(src: &[u8]) -> Option<Self> {
        if src.len() < CONSENT_RECORD_LEN
            || src[..8] != RECORD_DISCRIMINATOR
            || src[8] != RECORD_VERSION
        {
            return None;
        }
        let mut r = Reader { buf: src, pos: 9 };
        Some(Self {
            bump: r.take::<1>()[0],
            release_id: r.take(),
            split_version_hash: r.take(),
            contributor_id: r.take(),
            passkey_pubkey: r.take(),
            digest: r.take(),
            nonce: u64::from_le_bytes(r.take()),
            expiry: i64::from_le_bytes(r.take()),
            recorded_slot: u64::from_le_bytes(r.take()),
            recorded_unix_timestamp: i64::from_le_bytes(r.take()),
            payer: r.take(),
        })
    }
}

struct Writer<'a> {
    buf: &'a mut [u8],
    pos: usize,
}
impl Writer<'_> {
    fn put(&mut self, b: &[u8]) {
        self.buf[self.pos..self.pos + b.len()].copy_from_slice(b);
        self.pos += b.len();
    }
}
struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}
impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> [u8; N] {
        let out: [u8; N] = self.buf[self.pos..self.pos + N].try_into().unwrap();
        self.pos += N;
        out
    }
}

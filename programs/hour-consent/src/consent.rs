//! Canonical consent message and contributor key binding.
//!
//! These byte layouts are normative and mirrored by the TypeScript SDK
//! (`sdk/ts/src/consent.ts`) and the shared test vectors in `spec/test-vectors/`.

use solana_sha256_hasher::hashv;

/// Domain separator for consent digests (versioned; never reuse across layouts).
pub const CONSENT_DOMAIN: &[u8] = b"HBC-SOLANA-CONSENT-v1";
/// Domain separator for deriving a Milestone-1 contributor key ID from a passkey public key.
pub const CONTRIBUTOR_KEY_DOMAIN: &[u8] = b"HBC-CONTRIBUTOR-KEY-v1";
/// Compressed SEC1 P-256 public key length.
pub const P256_COMPRESSED_PUBKEY_LEN: usize = 33;

/// Fields a contributor approves. All hashes/ids are opaque 32-byte values produced
/// off-chain (see docs/spec.md §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsentFields {
    /// SHA-256 of the hOUR Chain release identifier.
    pub release_id: [u8; 32],
    /// SHA-256 of the canonical (RFC 8785) split-version document. The hOUR event carrying
    /// this document may additionally be ML-DSA-65 signed off-chain.
    pub split_version_hash: [u8; 32],
    /// Contributor key ID (Milestone 1: derived from the passkey public key, see
    /// [`contributor_key_id`]).
    pub contributor_id: [u8; 32],
    /// Client-chosen nonce (unique per consent request).
    pub nonce: u64,
    /// Unix timestamp (seconds) after which the signed approval may not be recorded.
    pub expiry: i64,
}

impl ConsentFields {
    /// `SHA-256(CONSENT_DOMAIN || release_id || split_version_hash || contributor_id ||
    /// nonce_le_u64 || expiry_le_i64)` — the exact 32-byte message the passkey signs in M1.
    pub fn digest(&self) -> [u8; 32] {
        hashv(&[
            CONSENT_DOMAIN,
            &self.release_id,
            &self.split_version_hash,
            &self.contributor_id,
            &self.nonce.to_le_bytes(),
            &self.expiry.to_le_bytes(),
        ])
        .to_bytes()
    }
}

/// Milestone-1 contributor key ID: `SHA-256(CONTRIBUTOR_KEY_DOMAIN || compressed_p256_pubkey)`.
///
/// This binds a consent to the passkey that signed it without an on-chain registry.
/// It intentionally does NOT support key rotation; a rotatable contributor registry
/// is Milestone 2 work (docs/spec.md §6).
pub fn contributor_key_id(pubkey: &[u8; P256_COMPRESSED_PUBKEY_LEN]) -> [u8; 32] {
    hashv(&[CONTRIBUTOR_KEY_DOMAIN, pubkey]).to_bytes()
}

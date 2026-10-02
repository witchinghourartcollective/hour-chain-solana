//! Parsing of `Secp256r1SigVerify` precompile instruction data (SIMD-0075).
//!
//! The precompile itself performs the signature verification when the transaction
//! executes; if verification fails the whole transaction fails. This module only
//! extracts *what* was verified, so the program can check it signed the expected
//! consent digest with the expected key.

use crate::consent::P256_COMPRESSED_PUBKEY_LEN;
use crate::error::ConsentError;

/// `Secp256r1SigVerify1111111111111111111111111`
pub const SECP256R1_PROGRAM_ID: solana_pubkey::Pubkey =
    solana_pubkey::pubkey!("Secp256r1SigVerify1111111111111111111111111");

pub const SIGNATURE_OFFSETS_START: usize = 2;
pub const SIGNATURE_OFFSETS_SERIALIZED_SIZE: usize = 14;
pub const SIGNATURE_SERIALIZED_SIZE: usize = 64;
/// Instruction index meaning "data lives in this same precompile instruction".
pub const SAME_INSTRUCTION: u16 = u16::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Secp256r1SignatureOffsets {
    pub signature_offset: u16,
    pub signature_instruction_index: u16,
    pub public_key_offset: u16,
    pub public_key_instruction_index: u16,
    pub message_data_offset: u16,
    pub message_data_size: u16,
    pub message_instruction_index: u16,
}

impl Secp256r1SignatureOffsets {
    fn parse(b: &[u8]) -> Self {
        let u = |i: usize| u16::from_le_bytes([b[i], b[i + 1]]);
        Self {
            signature_offset: u(0),
            signature_instruction_index: u(2),
            public_key_offset: u(4),
            public_key_instruction_index: u(6),
            message_data_offset: u(8),
            message_data_size: u(10),
            message_instruction_index: u(12),
        }
    }
}

/// The verified (public key, message) pair from a single-signature precompile instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedSignature<'a> {
    pub public_key: [u8; P256_COMPRESSED_PUBKEY_LEN],
    pub message: &'a [u8],
}

fn slice(data: &[u8], offset: u16, len: usize) -> Result<&[u8], ConsentError> {
    let start = offset as usize;
    let end = start
        .checked_add(len)
        .ok_or(ConsentError::MalformedPrecompileData)?;
    data.get(start..end)
        .ok_or(ConsentError::MalformedPrecompileData)
}

/// Extract the single verified signature from secp256r1 precompile instruction data.
///
/// Fails closed unless: exactly one signature; signature, key and message all live in
/// the precompile instruction itself (no cross-instruction references, which prevents
/// a "verify one thing, point the program at another" confusion); all ranges in bounds.
pub fn extract_single_signature(data: &[u8]) -> Result<VerifiedSignature<'_>, ConsentError> {
    if data.len() < SIGNATURE_OFFSETS_START {
        return Err(ConsentError::MalformedPrecompileData);
    }
    if data[0] != 1 {
        return Err(ConsentError::UnsupportedSignatureCount);
    }
    let offsets_bytes = slice(
        data,
        SIGNATURE_OFFSETS_START as u16,
        SIGNATURE_OFFSETS_SERIALIZED_SIZE,
    )?;
    let o = Secp256r1SignatureOffsets::parse(offsets_bytes);
    if o.signature_instruction_index != SAME_INSTRUCTION
        || o.public_key_instruction_index != SAME_INSTRUCTION
        || o.message_instruction_index != SAME_INSTRUCTION
    {
        return Err(ConsentError::CrossInstructionReference);
    }
    // Signature presence is checked so malformed data fails here, not only in the precompile.
    slice(data, o.signature_offset, SIGNATURE_SERIALIZED_SIZE)?;
    let pk = slice(data, o.public_key_offset, P256_COMPRESSED_PUBKEY_LEN)?;
    let message = slice(data, o.message_data_offset, o.message_data_size as usize)?;
    let mut public_key = [0u8; P256_COMPRESSED_PUBKEY_LEN];
    public_key.copy_from_slice(pk);
    Ok(VerifiedSignature {
        public_key,
        message,
    })
}

/// Build single-signature precompile instruction data (used by tests and mirrored by the SDK).
/// Layout: `[1, 0, offsets(14), pubkey(33), signature(64), message(..)]`.
pub fn build_single_signature_data(
    public_key: &[u8; P256_COMPRESSED_PUBKEY_LEN],
    signature: &[u8; SIGNATURE_SERIALIZED_SIZE],
    message: &[u8],
) -> Vec<u8> {
    let pk_off = SIGNATURE_OFFSETS_START + SIGNATURE_OFFSETS_SERIALIZED_SIZE;
    let sig_off = pk_off + P256_COMPRESSED_PUBKEY_LEN;
    let msg_off = sig_off + SIGNATURE_SERIALIZED_SIZE;
    let mut d = Vec::with_capacity(msg_off + message.len());
    d.push(1);
    d.push(0);
    for v in [
        sig_off as u16,
        SAME_INSTRUCTION,
        pk_off as u16,
        SAME_INSTRUCTION,
        msg_off as u16,
        message.len() as u16,
        SAME_INSTRUCTION,
    ] {
        d.extend_from_slice(&v.to_le_bytes());
    }
    d.extend_from_slice(public_key);
    d.extend_from_slice(signature);
    d.extend_from_slice(message);
    d
}

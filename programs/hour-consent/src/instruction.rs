//! Instruction encoding. Manual fixed layout (no Borsh) so the TS SDK can mirror it exactly.

use crate::consent::ConsentFields;
use crate::error::ConsentError;

pub const RECORD_CONSENT_TAG: u8 = 0;
/// tag(1) + release_id(32) + split_version_hash(32) + contributor_id(32) + nonce(8) + expiry(8)
pub const RECORD_CONSENT_LEN: usize = 1 + 32 * 3 + 8 + 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsentInstruction {
    /// Record a passkey-verified consent.
    ///
    /// Accounts:
    /// 0. `[signer, writable]` payer (fee payer / rent payer; NOT the consenting party)
    /// 1. `[writable]` consent record PDA: seeds `["consent", release_id, split_version_hash, contributor_id]`
    /// 2. `[]` instructions sysvar
    /// 3. `[]` system program
    ///
    /// The instruction immediately before this one MUST be a single-signature
    /// `Secp256r1SigVerify` instruction whose message is `fields.digest()`.
    RecordConsent(ConsentFields),
}

impl ConsentInstruction {
    pub fn pack(&self) -> Vec<u8> {
        match self {
            ConsentInstruction::RecordConsent(f) => {
                let mut d = Vec::with_capacity(RECORD_CONSENT_LEN);
                d.push(RECORD_CONSENT_TAG);
                d.extend_from_slice(&f.release_id);
                d.extend_from_slice(&f.split_version_hash);
                d.extend_from_slice(&f.contributor_id);
                d.extend_from_slice(&f.nonce.to_le_bytes());
                d.extend_from_slice(&f.expiry.to_le_bytes());
                d
            }
        }
    }

    pub fn unpack(data: &[u8]) -> Result<Self, ConsentError> {
        match data.first() {
            Some(&RECORD_CONSENT_TAG) if data.len() == RECORD_CONSENT_LEN => {
                let a32 = |o: usize| -> [u8; 32] { data[o..o + 32].try_into().unwrap() };
                let a8 = |o: usize| -> [u8; 8] { data[o..o + 8].try_into().unwrap() };
                Ok(ConsentInstruction::RecordConsent(ConsentFields {
                    release_id: a32(1),
                    split_version_hash: a32(33),
                    contributor_id: a32(65),
                    nonce: u64::from_le_bytes(a8(97)),
                    expiry: i64::from_le_bytes(a8(105)),
                }))
            }
            _ => Err(ConsentError::InvalidInstructionData),
        }
    }
}

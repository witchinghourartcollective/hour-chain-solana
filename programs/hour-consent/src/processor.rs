use solana_account_info::{next_account_info, AccountInfo};
use solana_clock::Clock;
use solana_cpi::invoke_signed;
use solana_instructions_sysvar::{load_current_index_checked, load_instruction_at_checked};
use solana_msg::msg;
use solana_program_error::{ProgramError, ProgramResult};
use solana_pubkey::Pubkey;
use solana_rent::Rent;
use solana_sysvar::Sysvar;

use crate::consent::{contributor_key_id, ConsentFields};
use crate::error::ConsentError;
use crate::instruction::ConsentInstruction;
use crate::precompile::{extract_single_signature, SECP256R1_PROGRAM_ID};
use crate::state::{ConsentRecord, CONSENT_RECORD_LEN, CONSENT_SEED};

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    match ConsentInstruction::unpack(data)? {
        ConsentInstruction::RecordConsent(fields) => record_consent(program_id, accounts, &fields),
    }
}

/// Pure check shared by the processor and unit tests: given the verified precompile
/// (pubkey, message), confirm it is a valid consent for `fields` at time `now`.
pub fn check_consent(
    fields: &ConsentFields,
    verified_pubkey: &[u8; 33],
    verified_message: &[u8],
    now_unix: i64,
) -> Result<(), ConsentError> {
    if verified_message != fields.digest() {
        return Err(ConsentError::MessageDigestMismatch);
    }
    if contributor_key_id(verified_pubkey) != fields.contributor_id {
        return Err(ConsentError::ContributorKeyMismatch);
    }
    if now_unix > fields.expiry {
        return Err(ConsentError::ConsentExpired);
    }
    Ok(())
}

pub fn consent_record_address(program_id: &Pubkey, f: &ConsentFields) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            CONSENT_SEED,
            &f.release_id,
            &f.split_version_hash,
            &f.contributor_id,
        ],
        program_id,
    )
}

fn record_consent(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    f: &ConsentFields,
) -> ProgramResult {
    let it = &mut accounts.iter();
    let payer = next_account_info(it)?;
    let record = next_account_info(it)?;
    let ix_sysvar = next_account_info(it)?;
    let system_program = next_account_info(it)?;

    if !payer.is_signer {
        return Err(ConsentError::MissingRequiredSignature.into());
    }
    if *ix_sysvar.key != solana_sdk_ids::sysvar::instructions::ID {
        return Err(ConsentError::InvalidSysvar.into());
    }
    if *system_program.key != solana_sdk_ids::system_program::ID {
        return Err(ConsentError::InvalidSystemProgram.into());
    }

    // The precompile instruction must be immediately before this one.
    let current = load_current_index_checked(ix_sysvar)?;
    if current == 0 {
        return Err(ConsentError::MissingPrecompileInstruction.into());
    }
    let prev = load_instruction_at_checked((current - 1) as usize, ix_sysvar)?;
    if prev.program_id != SECP256R1_PROGRAM_ID {
        return Err(ConsentError::WrongPrecompileProgram.into());
    }
    let verified = extract_single_signature(&prev.data)?;

    let clock = Clock::get()?;
    check_consent(
        f,
        &verified.public_key,
        verified.message,
        clock.unix_timestamp,
    )?;

    let (expected, bump) = consent_record_address(program_id, f);
    if *record.key != expected {
        return Err(ConsentError::InvalidRecordAddress.into());
    }
    // Replay protection: one record per (release, split version, contributor key).
    if record.lamports() > 0 || !record.data_is_empty() {
        return Err(ConsentError::ConsentAlreadyRecorded.into());
    }

    let rent = Rent::get()?;
    let ix = solana_system_interface::instruction::create_account(
        payer.key,
        record.key,
        rent.minimum_balance(CONSENT_RECORD_LEN),
        CONSENT_RECORD_LEN as u64,
        program_id,
    );
    invoke_signed(
        &ix,
        &[payer.clone(), record.clone(), system_program.clone()],
        &[&[
            CONSENT_SEED,
            &f.release_id,
            &f.split_version_hash,
            &f.contributor_id,
            &[bump],
        ]],
    )?;

    let rec = ConsentRecord {
        bump,
        release_id: f.release_id,
        split_version_hash: f.split_version_hash,
        contributor_id: f.contributor_id,
        passkey_pubkey: verified.public_key,
        digest: f.digest(),
        nonce: f.nonce,
        expiry: f.expiry,
        recorded_slot: clock.slot,
        recorded_unix_timestamp: clock.unix_timestamp,
        payer: payer.key.to_bytes(),
    };
    rec.pack_into(&mut record.try_borrow_mut_data()?)
        .ok_or(ProgramError::AccountDataTooSmall)?;
    msg!("hour-consent: consent recorded");
    Ok(())
}

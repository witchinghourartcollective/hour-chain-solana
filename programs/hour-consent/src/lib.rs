//! hOUR Solana Consent Kit — `hour-consent` program (pre-alpha, Milestone 1 scaffold).
//!
//! Records that a contributor approved a specific creator-rights split version,
//! where the approval is a P-256 (secp256r1 / passkey) signature verified by
//! Solana's native `Secp256r1SigVerify1111111111111111111111111` precompile in
//! the instruction immediately preceding `RecordConsent`.
//!
//! NOT AUDITED. NOT DEPLOYED. See README.md "Status" for what is and is not implemented.

#![deny(unsafe_code)]

pub mod consent;
pub mod error;
pub mod instruction;
pub mod precompile;
pub mod processor;
pub mod state;

solana_pubkey::declare_id!("HbcCons111111111111111111111111111111111111");

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint {
    use solana_account_info::AccountInfo;
    use solana_program_error::ProgramResult;
    use solana_pubkey::Pubkey;

    solana_program_entrypoint::entrypoint!(process_instruction);

    fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        crate::processor::process(program_id, accounts, instruction_data)
    }

    solana_security_txt::security_txt! {
        name: "hOUR Solana Consent Kit (hour-consent)",
        project_url: "https://github.com/witchinghourartcollective/hour-chain-solana",
        contacts: "email:witchinghour@witchinghourmac.com,link:https://github.com/witchinghourartcollective/hour-chain-solana/security/advisories/new",
        policy: "https://github.com/witchinghourartcollective/hour-chain-solana/blob/main/SECURITY.md",
        preferred_languages: "en",
        source_code: "https://github.com/witchinghourartcollective/hour-chain-solana"
    }
}

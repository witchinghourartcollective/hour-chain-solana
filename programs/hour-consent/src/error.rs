use solana_program_error::ProgramError;

/// Program-specific errors, surfaced as `ProgramError::Custom(code)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ConsentError {
    InvalidInstructionData = 0,
    MissingPrecompileInstruction = 1,
    WrongPrecompileProgram = 2,
    MalformedPrecompileData = 3,
    UnsupportedSignatureCount = 4,
    CrossInstructionReference = 5,
    MessageDigestMismatch = 6,
    ContributorKeyMismatch = 7,
    ConsentExpired = 8,
    ConsentAlreadyRecorded = 9,
    InvalidRecordAddress = 10,
    MissingRequiredSignature = 11,
    InvalidSysvar = 12,
    InvalidSystemProgram = 13,
}

impl From<ConsentError> for ProgramError {
    fn from(e: ConsentError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

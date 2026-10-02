import { AccountRole, getProgramDerivedAddress, type Address, type Instruction } from "@solana/kit";
import { concatBytes, i64le, u64le } from "./bytes.js";
import { consentDigest, validateConsentFields, type ConsentFields } from "./consent.js";
import { buildSecp256r1InstructionData, SECP256R1_PROGRAM_ADDRESS } from "./precompile.js";

export const RECORD_CONSENT_TAG = 0;
export const CONSENT_SEED = new TextEncoder().encode("consent");
export const INSTRUCTIONS_SYSVAR_ADDRESS = "Sysvar1nstructions1111111111111111111111111" as Address;
export const SYSTEM_PROGRAM_ADDRESS = "11111111111111111111111111111111" as Address;
/** Placeholder program ID (not deployed). Replace with the real devnet ID at Milestone 2. */
export const HOUR_CONSENT_PROGRAM_ADDRESS = "HbcCons111111111111111111111111111111111111" as Address;

export function encodeRecordConsent(f: ConsentFields): Uint8Array {
  validateConsentFields(f);
  return concatBytes(
    new Uint8Array([RECORD_CONSENT_TAG]),
    f.releaseId,
    f.splitVersionHash,
    f.contributorId,
    u64le(f.nonce),
    i64le(f.expiry),
  );
}

export async function deriveConsentRecordAddress(
  f: ConsentFields,
  programAddress: Address = HOUR_CONSENT_PROGRAM_ADDRESS,
): Promise<readonly [Address, number]> {
  return getProgramDerivedAddress({
    programAddress,
    seeds: [CONSENT_SEED, f.releaseId, f.splitVersionHash, f.contributorId],
  });
}

/**
 * Build the two instructions that must appear back-to-back in one transaction:
 * [0] Secp256r1SigVerify over consentDigest(f), [1] hour-consent RecordConsent.
 */
export async function buildRecordConsentInstructions(args: {
  fields: ConsentFields;
  passkeyPublicKey: Uint8Array;
  signature: Uint8Array;
  payer: Address;
  programAddress?: Address;
}): Promise<[Instruction, Instruction]> {
  const programAddress = args.programAddress ?? HOUR_CONSENT_PROGRAM_ADDRESS;
  const [record] = await deriveConsentRecordAddress(args.fields, programAddress);
  const verify: Instruction = {
    programAddress: SECP256R1_PROGRAM_ADDRESS,
    accounts: [],
    data: buildSecp256r1InstructionData(args.passkeyPublicKey, args.signature, consentDigest(args.fields)),
  };
  const recordIx: Instruction = {
    programAddress,
    accounts: [
      { address: args.payer, role: AccountRole.WRITABLE_SIGNER },
      { address: record, role: AccountRole.WRITABLE },
      { address: INSTRUCTIONS_SYSVAR_ADDRESS, role: AccountRole.READONLY },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
    ],
    data: encodeRecordConsent(args.fields),
  };
  return [verify, recordIx];
}

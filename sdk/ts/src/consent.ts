import { sha256 } from "@noble/hashes/sha2.js";
import { assertLen, concatBytes, i64le, u64le } from "./bytes.js";

/** Must match programs/hour-consent/src/consent.rs */
export const CONSENT_DOMAIN = new TextEncoder().encode("HBC-SOLANA-CONSENT-v1");
export const CONTRIBUTOR_KEY_DOMAIN = new TextEncoder().encode("HBC-CONTRIBUTOR-KEY-v1");
export const P256_COMPRESSED_PUBKEY_LEN = 33;

export interface ConsentFields {
  /** SHA-256 of the hOUR Chain release identifier (32 bytes). */
  releaseId: Uint8Array;
  /** SHA-256 of the RFC 8785-canonical split-version document (32 bytes). */
  splitVersionHash: Uint8Array;
  /** Contributor key ID (32 bytes). Milestone 1: contributorKeyId(passkeyPublicKey). */
  contributorId: Uint8Array;
  /** Unique nonce (u64). */
  nonce: bigint;
  /** Unix seconds after which the approval cannot be recorded (i64). */
  expiry: bigint;
}

export function validateConsentFields(f: ConsentFields): void {
  assertLen("releaseId", f.releaseId, 32);
  assertLen("splitVersionHash", f.splitVersionHash, 32);
  assertLen("contributorId", f.contributorId, 32);
}

/** The exact 32-byte message the passkey signs (Milestone 1 raw-digest format). */
export function consentDigest(f: ConsentFields): Uint8Array {
  validateConsentFields(f);
  return sha256(
    concatBytes(CONSENT_DOMAIN, f.releaseId, f.splitVersionHash, f.contributorId, u64le(f.nonce), i64le(f.expiry)),
  );
}

/** SHA-256(CONTRIBUTOR_KEY_DOMAIN || compressed P-256 public key). No rotation in M1. */
export function contributorKeyId(compressedP256PublicKey: Uint8Array): Uint8Array {
  assertLen("compressedP256PublicKey", compressedP256PublicKey, P256_COMPRESSED_PUBKEY_LEN);
  return sha256(concatBytes(CONTRIBUTOR_KEY_DOMAIN, compressedP256PublicKey));
}

/** Convenience: SHA-256 of a UTF-8 string (e.g. a release identifier). */
export function hashId(id: string): Uint8Array {
  return sha256(new TextEncoder().encode(id));
}

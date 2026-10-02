import { p256 } from "@noble/curves/nist.js";
import { fromHex, toHex } from "../src/bytes.js";
import { consentDigest, contributorKeyId, hashId, type ConsentFields } from "../src/consent.js";
import { buildSecp256r1InstructionData } from "../src/precompile.js";
import { encodeRecordConsent } from "../src/instruction.js";

/** TEST-ONLY key. Never use for anything real. */
export const TEST_SECRET_KEY = fromHex("c9afa9d845ba75166b5c215767b1d6934e50c3db36e89b127b8a622b120f6721");

export function vectorFields(): { fields: ConsentFields; publicKey: Uint8Array } {
  const publicKey = p256.getPublicKey(TEST_SECRET_KEY, true);
  return {
    publicKey,
    fields: {
      releaseId: hashId("hour:release:test-vector-001"),
      splitVersionHash: hashId("hour:split:test-vector-001:v1"),
      contributorId: contributorKeyId(publicKey),
      nonce: 42n,
      expiry: 1_893_456_000n, // 2030-01-01T00:00:00Z
    },
  };
}

export function buildVector() {
  const { fields, publicKey } = vectorFields();
  const digest = consentDigest(fields);
  const signature = p256.sign(digest, TEST_SECRET_KEY); // sha256(digest) is what the precompile verifies; low-S
  return {
    description: "hOUR Solana Consent Kit consent v1 test vector (TEST KEY ONLY)",
    release_id: toHex(fields.releaseId),
    split_version_hash: toHex(fields.splitVersionHash),
    contributor_id: toHex(fields.contributorId),
    nonce: fields.nonce.toString(),
    expiry: fields.expiry.toString(),
    passkey_pubkey_compressed: toHex(publicKey),
    consent_digest: toHex(digest),
    signature_compact_low_s: toHex(signature),
    record_consent_ix_data: toHex(encodeRecordConsent(fields)),
    secp256r1_ix_data: toHex(buildSecp256r1InstructionData(publicKey, signature, digest)),
  };
}

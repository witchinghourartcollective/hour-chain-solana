import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { p256 } from "@noble/curves/nist.js";
import { address } from "@solana/kit";
import {
  allocate,
  buildRecordConsentInstructions,
  buildSettlementInstruction,
  consentDigest,
  contributorKeyId,
  deriveConsentRecordAddress,
  derToCompactLowS,
  encodeRecordConsent,
  fromHex,
  normalizeLowS,
  SECP256R1_PROGRAM_ADDRESS,
  toHex,
  validateSplit,
} from "../src/index.js";
import { buildVector, TEST_SECRET_KEY, vectorFields } from "./vector.js";

const vectorPath = new URL("../../../../spec/test-vectors/consent-v1.json", import.meta.url);

test("committed test vector matches what the SDK generates (shared with Rust tests)", () => {
  const committed = JSON.parse(readFileSync(vectorPath, "utf8"));
  assert.deepEqual(buildVector(), committed);
});

test("vector signature verifies as P-256 ECDSA over SHA-256(digest), like the precompile", () => {
  const v = buildVector();
  assert.ok(p256.verify(fromHex(v.signature_compact_low_s), fromHex(v.consent_digest), fromHex(v.passkey_pubkey_compressed)));
});

test("digest changes when any field changes", () => {
  const { fields } = vectorFields();
  const base = toHex(consentDigest(fields));
  const variants = [
    { ...fields, nonce: 43n },
    { ...fields, expiry: fields.expiry + 1n },
    { ...fields, releaseId: new Uint8Array(32) },
    { ...fields, splitVersionHash: new Uint8Array(32) },
    { ...fields, contributorId: new Uint8Array(32) },
  ];
  for (const f of variants) assert.notEqual(toHex(consentDigest(f)), base);
});

test("field length validation", () => {
  const { fields } = vectorFields();
  assert.throws(() => consentDigest({ ...fields, releaseId: new Uint8Array(31) }), /32 bytes/);
  assert.throws(() => contributorKeyId(new Uint8Array(65)), /33 bytes/);
});

test("record-consent encoding is 113 bytes with tag 0", () => {
  const data = encodeRecordConsent(vectorFields().fields);
  assert.equal(data.length, 113);
  assert.equal(data[0], 0);
});

test("normalizeLowS flips high-S and keeps low-S", () => {
  const v = buildVector();
  const low = fromHex(v.signature_compact_low_s);
  assert.deepEqual(normalizeLowS(low), low);
  const n = p256.Point.CURVE().n;
  const s = BigInt("0x" + toHex(low.subarray(32)));
  const high = new Uint8Array([...low.subarray(0, 32), ...fromHex((n - s).toString(16).padStart(64, "0"))]);
  assert.deepEqual(normalizeLowS(high), low);
});

test("DER (WebAuthn-style) signature converts to the same compact low-S signature", () => {
  const v = buildVector();
  const der = p256.sign(fromHex(v.consent_digest), TEST_SECRET_KEY, { format: "der" });
  assert.equal(toHex(derToCompactLowS(der)), v.signature_compact_low_s);
});

test("buildRecordConsentInstructions orders precompile first, then program", async () => {
  const { fields, publicKey } = vectorFields();
  const v = buildVector();
  const payer = address("11111111111111111111111111111112");
  const [verify, record] = await buildRecordConsentInstructions({
    fields,
    passkeyPublicKey: publicKey,
    signature: fromHex(v.signature_compact_low_s),
    payer,
  });
  assert.equal(verify.programAddress, SECP256R1_PROGRAM_ADDRESS);
  assert.equal(toHex(verify.data as Uint8Array), v.secp256r1_ix_data);
  assert.equal(record.accounts?.length, 4);
  const [pda] = await deriveConsentRecordAddress(fields);
  assert.equal(record.accounts?.[1].address, pda);
});

test("split validation and deterministic largest-remainder allocation", () => {
  assert.throws(() => validateSplit([{ payee: "a", bps: 5000 }]), /10000/);
  assert.throws(() => validateSplit([{ payee: "a", bps: 5000 }, { payee: "a", bps: 5000 }]), /duplicate/);
  const lines = allocate(100n, [
    { payee: "a", bps: 3333 },
    { payee: "b", bps: 3333 },
    { payee: "c", bps: 3334 },
  ]);
  assert.equal(lines.reduce((n, l) => n + l.amount, 0n), 100n);
  assert.deepEqual(lines.map((l) => l.amount), [33n, 33n, 34n]);
  const big = allocate(1_000_000_001n, [
    { payee: "a", bps: 1 },
    { payee: "b", bps: 9999 },
  ]);
  assert.equal(big.reduce((n, l) => n + l.amount, 0n), 1_000_000_001n);
});

test("settlement instruction targets Base USDC, requires approval, and hashes stably", () => {
  const { fields } = vectorFields();
  const args = {
    releaseId: fields.releaseId,
    splitVersionHash: fields.splitVersionHash,
    totalAtomic: 25_000_000n,
    shares: [
      { payee: "0x000000000000000000000000000000000000dEaD", bps: 6000 },
      { payee: "claim:contributor-2", bps: 4000 },
    ],
  };
  const a = buildSettlementInstruction(args);
  const b = buildSettlementInstruction(args);
  assert.equal(a.instruction.network, "base");
  assert.equal(a.instruction.asset, "USDC");
  assert.equal(a.instruction.requiresApproval, true);
  assert.equal(a.instructionHash, b.instructionHash);
  assert.deepEqual(a.instruction.lines.map((l) => l.amount), ["15000000", "10000000"]);
});

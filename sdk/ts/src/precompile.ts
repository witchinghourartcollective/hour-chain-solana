import { p256 } from "@noble/curves/nist.js";
import { address, type Address } from "@solana/kit";
import { assertLen, concatBytes, u16le } from "./bytes.js";
import { P256_COMPRESSED_PUBKEY_LEN } from "./consent.js";

export const SECP256R1_PROGRAM_ADDRESS: Address = address("Secp256r1SigVerify1111111111111111111111111");
export const SIGNATURE_OFFSETS_START = 2;
export const SIGNATURE_OFFSETS_SERIALIZED_SIZE = 14;
export const SIGNATURE_SERIALIZED_SIZE = 64;
export const SAME_INSTRUCTION = 0xffff;

/** P-256 group order n and n/2 (the precompile rejects high-S signatures). */
const N = p256.Point.CURVE().n;
const HALF_N = N >> 1n;

/** Normalize a 64-byte compact (r||s) signature to low-S form. */
export function normalizeLowS(compactSig: Uint8Array): Uint8Array {
  assertLen("signature", compactSig, SIGNATURE_SERIALIZED_SIZE);
  const s = BigInt("0x" + Buffer.from(compactSig.subarray(32)).toString("hex"));
  if (s <= HALF_N) return compactSig;
  const lowS = (N - s).toString(16).padStart(64, "0");
  return concatBytes(compactSig.subarray(0, 32), new Uint8Array(Buffer.from(lowS, "hex")));
}

/** WebAuthn authenticators return DER-encoded ECDSA signatures; convert to low-S compact. */
export function derToCompactLowS(der: Uint8Array): Uint8Array {
  const sig = p256.Signature.fromBytes(der, "der");
  return normalizeLowS(sig.toBytes("compact"));
}

/** Single-signature Secp256r1SigVerify instruction data; mirrors precompile.rs. */
export function buildSecp256r1InstructionData(
  compressedPublicKey: Uint8Array,
  compactSignature: Uint8Array,
  message: Uint8Array,
): Uint8Array {
  assertLen("compressedPublicKey", compressedPublicKey, P256_COMPRESSED_PUBKEY_LEN);
  assertLen("compactSignature", compactSignature, SIGNATURE_SERIALIZED_SIZE);
  const pkOff = SIGNATURE_OFFSETS_START + SIGNATURE_OFFSETS_SERIALIZED_SIZE;
  const sigOff = pkOff + P256_COMPRESSED_PUBKEY_LEN;
  const msgOff = sigOff + SIGNATURE_SERIALIZED_SIZE;
  return concatBytes(
    new Uint8Array([1, 0]),
    u16le(sigOff),
    u16le(SAME_INSTRUCTION),
    u16le(pkOff),
    u16le(SAME_INSTRUCTION),
    u16le(msgOff),
    u16le(message.length),
    u16le(SAME_INSTRUCTION),
    compressedPublicKey,
    compactSignature,
    message,
  );
}

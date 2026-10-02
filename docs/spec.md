# hOUR Solana Consent Kit: Consent & Attestation Specification

**Status:** Draft 0.1 (Milestone 1). Normative byte layouts in §3–§4 are implemented and tested. §5 (SAS schemas) and §7 (settlement receipts) are drafts with no on-chain deployment yet.
**License:** MIT OR Apache-2.0. Anyone may implement this spec.

The key words MUST, SHOULD, and MAY are used as in RFC 2119.

## 1. Purpose

Record, verifiably and publicly, that a specific contributor approved a specific version of a creator-rights split (credits and percentages) for a specific release, using a **passkey** (WebAuthn / FIDO2, P-256) instead of a crypto wallet.

**Chain roles (hOUR Chain ADR-0004):** Solana is the primary chain for approvals/consent and attestations. Base is the payments/settlement chain. Payment receipts on Base are linked back to Solana consent records (§7).

## 2. Actors and identifiers

| Name | Size | Definition |
| --- | --- | --- |
| `release_id` | 32 bytes | SHA-256 of the hOUR Chain release identifier string (e.g. `hour:release:<id>`). |
| `split_version_hash` | 32 bytes | SHA-256 of the RFC 8785 (JCS) canonical JSON split-version document (contributors, roles, rights category, basis points totaling 10000 per category, version number). |
| `passkey_pubkey` | 33 bytes | Compressed SEC1 P-256 public key of the contributor's passkey. |
| `contributor_id` | 32 bytes | **M1:** `SHA-256("HBC-CONTRIBUTOR-KEY-v1" ‖ passkey_pubkey)`. Binds consent to the signing passkey without an on-chain registry. **No key rotation in M1.** M2 replaces this with a rotatable contributor registry (§6). |
| `nonce` | u64 LE | Unique per consent request. |
| `expiry` | i64 LE | Unix seconds. The approval MUST NOT be recorded after this time. |

The payer (fee/rent payer) is any Solana account and is **not** the consenting party. Contributors need no SOL and no wallet.

## 3. Consent digest (normative)

```
consent_digest = SHA-256(
  "HBC-SOLANA-CONSENT-v1"   // 21 ASCII bytes, domain separator
  ‖ release_id              // 32
  ‖ split_version_hash      // 32
  ‖ contributor_id          // 32
  ‖ nonce   (u64 little-endian)
  ‖ expiry  (i64 little-endian)
)
```

### 3.1 Signed message format

- **M1 (implemented):** the P-256 key signs `consent_digest` directly. The precompile verifies ECDSA-P256 over `SHA-256(message)`, where `message = consent_digest`. Signatures MUST be low-S (the precompile rejects high-S). This format is produced by software P-256 keys (tests, server-side signers). **Browser passkeys cannot produce it**, because WebAuthn authenticators sign `authenticatorData ‖ SHA-256(clientDataJSON)`.
- **M2 (planned):** WebAuthn assertion format. The precompile message is `authenticatorData ‖ SHA-256(clientDataJSON)`. The program MUST check that `clientDataJSON.type == "webauthn.get"`, that `challenge == base64url(consent_digest)`, that `origin` is an allow-listed origin, and that `authenticatorData.rpIdHash` matches the registered RP ID with the UP (and SHOULD require UV) flag set.

## 4. On-chain program: `hour-consent` (normative for M1)

### 4.1 Transaction shape

A consent transaction MUST contain, back-to-back:

1. `Secp256r1SigVerify1111111111111111111111111` instruction with **exactly one** signature. The signature, public key, and message MUST all be inside that same instruction (all three instruction-index fields = `0xFFFF`). Cross-instruction references are rejected.
2. `hour-consent` `RecordConsent` instruction.

The precompile verifies the signature (the whole transaction fails if it's invalid). The program reads instruction `current_index − 1` from the instructions sysvar and checks the program ID, single-signature layout, `message == consent_digest(fields)`, `contributor_id == SHA-256("HBC-CONTRIBUTOR-KEY-v1" ‖ pubkey)`, and `clock.unix_timestamp ≤ expiry`.

### 4.2 Precompile instruction data (single signature)

```
[0]      num_signatures = 1
[1]      padding = 0
[2..16]  offsets: signature_offset u16, signature_ix_index u16 (=0xFFFF),
                  public_key_offset u16, public_key_ix_index u16 (=0xFFFF),
                  message_offset u16, message_size u16, message_ix_index u16 (=0xFFFF)
[16..49] public key (33, compressed)
[49..113] signature (64, r‖s, low-S)
[113..]  message (32 = consent_digest in M1)
```

### 4.3 `RecordConsent` instruction

Data (113 bytes): `tag=0 (u8) ‖ release_id ‖ split_version_hash ‖ contributor_id ‖ nonce u64 LE ‖ expiry i64 LE`.

| # | Account | Flags |
| --- | --- | --- |
| 0 | payer | signer, writable |
| 1 | consent record PDA, seeds `["consent", release_id, split_version_hash, contributor_id]` | writable |
| 2 | instructions sysvar `Sysvar1nstructions1111111111111111111111111` | readonly |
| 3 | system program | readonly |

**Replay protection:** one record per (release, split version, contributor). A second `RecordConsent` for the same triple fails with `ConsentAlreadyRecorded`. A *new* split version gets a new hash, so it needs a new consent.

### 4.4 Consent record account (235 bytes)

| Offset | Size | Field |
| ---: | ---: | --- |
| 0 | 8 | discriminator `"HBCCONS1"` |
| 8 | 1 | version = 1 |
| 9 | 1 | PDA bump |
| 10 | 32 | release_id |
| 42 | 32 | split_version_hash |
| 74 | 32 | contributor_id |
| 106 | 33 | passkey_pubkey |
| 139 | 32 | consent_digest |
| 171 | 8 | nonce (u64 LE) |
| 179 | 8 | expiry (i64 LE) |
| 187 | 8 | recorded_slot (u64 LE) |
| 195 | 8 | recorded_unix_timestamp (i64 LE) |
| 203 | 32 | payer |

### 4.5 Error codes (`ProgramError::Custom(n)`)

0 InvalidInstructionData · 1 MissingPrecompileInstruction · 2 WrongPrecompileProgram · 3 MalformedPrecompileData · 4 UnsupportedSignatureCount · 5 CrossInstructionReference · 6 MessageDigestMismatch · 7 ContributorKeyMismatch · 8 ConsentExpired · 9 ConsentAlreadyRecorded · 10 InvalidRecordAddress · 11 MissingRequiredSignature · 12 InvalidSysvar · 13 InvalidSystemProgram

(Note: when the precompile itself rejects a signature, the error is reported against instruction 0 with the precompile's own error code, not ours.)

## 5. Solana Attestation Service schemas (draft; not yet created on any cluster)

Records are mirrored as [SAS](https://solana.com/docs/tools/attestations) attestations so wallets, marketplaces, and label tools can verify them with standard tooling. Credential: `hOUR Chain` (issuer authority TBD in M2).

| Schema | Fields |
| --- | --- |
| `hour.contributor-credit` v1 | release_id (bytes32), contributor_id (bytes32), role (string), hour_event_hash (bytes32) |
| `hour.split-version` v1 | release_id, split_version_hash, rights_category (string), version (u64), hour_event_hash |
| `hour.consent-receipt` v1 | release_id, split_version_hash, contributor_id, consent_digest, hour_event_hash |
| `hour.settlement-receipt` v1 | release_id, split_version_hash, settlement_network (string, e.g. `base`), settlement_tx (string), instruction_hash (bytes32) |

Machine-readable drafts: `sdk/ts/src/sas.ts`.

## 6. Contributor registry (planned, M2)

A rotatable registry mapping a stable contributor ID to one or more registered passkeys (with RP ID / origin), with rotation and revocation events. This satisfies hOUR Chain's requirement that identity is never permanently coupled to one key.

## 7. Settlement link (draft)

A consented split version produces a deterministic **settlement instruction** (`sdk/ts/src/splits.ts`): network `base`, asset `USDC` (6 decimals), largest-remainder allocation over basis points, `requiresApproval: true`, `simulateFirst: true`. Its SHA-256 `instruction_hash` and the resulting Base transaction hash are attested on Solana as `hour.settlement-receipt`. This repository **does not execute payments**.

## 8. Post-quantum boundary

`hour_event_hash` can reference an hOUR Chain event signed off-chain with ML-DSA-65 (FIPS 204) per hOUR Chain ADR-0002. Verifiers MUST report the layers separately: (a) the Solana record/attestation is valid, and (b) the ML-DSA signature on the referenced hOUR event is valid. Neither secp256r1, Ed25519 (Solana transactions), nor Base settlement is quantum-safe. **Nothing in this kit claims end-to-end post-quantum security.**

## 9. Test vectors

`spec/test-vectors/consent-v1.json` (TEST KEY ONLY: the RFC 6979 A.2.5 example private key) has fields, digest, contributor ID, signature, and both instruction encodings. The TypeScript SDK generates it, and both the Rust and TypeScript test suites check it.

## 10. Versioning

Domain strings (`HBC-SOLANA-CONSENT-v1`, `HBC-CONTRIBUTOR-KEY-v1`), the record discriminator (`HBCCONS1`), and the record version byte MUST change on any incompatible layout change.

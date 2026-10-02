# hOUR Solana Consent Kit (`hour-chain-solana`)

**Open-source passkey consent and creator-rights attestations on Solana.** Part of [hOUR Chain](https://github.com/witchinghourartcollective/hOUR-Chain), the creator-rights, provenance, and settlement protocol from Witching Hour Music & Art Collective (Savannah, GA).

> **Status: pre-alpha (Milestone 1 scaffold).** Not audited. Not deployed to devnet or mainnet. The program ID in the code is a placeholder. Don't use it for real rights decisions or funds.

## Why

When a song comes out, who agreed to which credits and splits usually lives in text threads. Most collaborators don't have crypto wallets. This kit lets a collaborator approve a specific split version with a **passkey** (Face ID / Touch ID / security key) and records that approval **verifiably on Solana**:

- **Passkey consent:** P-256 signatures verified by Solana's native `Secp256r1SigVerify` precompile ([SIMD-0075](https://github.com/solana-foundation/solana-improvement-documents/blob/main/proposals/0075-precompile-for-secp256r1-sigverify.md)). No seed phrase, wallet, or custodial key.
- **Solana Attestation Service records:** open schemas (contributor credit, split version, consent receipt, settlement receipt) so any app can verify them.
- **Links to hOUR post-quantum-signed records:** each record can commit to an hOUR Chain event signed off-chain with ML-DSA-65 (FIPS 204). The boundary is explicit: Solana, secp256r1, and Base are not quantum-safe.
- **USDC split reference flow:** deterministic payout math producing settlement instructions for **Base** (hOUR Chain's payments chain), whose receipts are attested back on Solana.

**Chain roles:** Solana = primary chain for approvals/consent and attestations. Base = payments/settlement. (hOUR Chain ADR-0004.)

## What's implemented today (honest status)

| Component | Status |
| --- | --- |
| `programs/hour-consent` (native Rust, `solana-program` component crates; no Anchor) | ✅ `RecordConsent`: reads the preceding secp256r1 precompile instruction via the instructions sysvar; enforces single signature, no cross-instruction references, digest match, passkey-bound contributor ID, expiry, PDA address, and replay protection; creates a 235-byte consent record PDA. Embeds `security.txt`. Builds to SBF with `cargo build-sbf`. |
| End-to-end tests (LiteSVM with Agave precompiles) | ✅ 6 tests send real transactions: valid consent recorded; replay rejected; invalid signature rejected by the precompile; signature over different fields rejected; missing precompile rejected; expired consent rejected. |
| Rust unit tests | ✅ 9 tests incl. cross-language test vectors. |
| `sdk/ts` TypeScript SDK (`@solana/kit`, `@noble/*`) | ✅ consent digest, contributor key ID, secp256r1 instruction builder, DER→compact low-S conversion (for WebAuthn signatures), RecordConsent encoding, PDA derivation, instruction pair builder, USDC split allocation and Base settlement-instruction builder. 10 tests. Not published to npm. |
| Shared test vectors | ✅ `spec/test-vectors/consent-v1.json` (test key only), checked by both Rust and TS. |
| Spec | ✅ `docs/spec.md` Draft 0.1 |
| **WebAuthn assertion format** (browser passkeys) | ❌ **Not yet.** M1 signs the raw consent digest (software P-256 keys). Real browser passkeys sign `authenticatorData ‖ SHA-256(clientDataJSON)`. On-chain clientData/challenge/origin/rpId checks are M2. |
| Rotatable contributor registry | ❌ M2 (M1 binds contributor ID to one passkey; no rotation). |
| SAS credential/schemas on-chain, attestation issuing | ❌ M2 (drafts in `docs/spec.md` §5, `sdk/ts/src/sas.ts`). |
| Devnet/mainnet deployment, audit | ❌ M2 / M3. |
| Executing payments | ❌ Out of scope here; the split flow only computes instructions. |

## Roadmap (milestones from the Solana Foundation grant proposal, pending)

1. **M1: Open spec + repo + passkey spike.** Spec, schemas, threat model, ADR-0004, scaffold program/SDK/CI *(this commit covers the scaffold, spec, and tests; still to do: a real browser-passkey devnet spike and a written threat model)*.
2. **M2: Devnet program + SDKs.** WebAuthn assertion verification, contributor registry, SAS attestations, TS SDK + CLI, React/WebAuthn helpers, Rust client crate, ≥90% SDK coverage.
3. **M3: Audit + mainnet + PQC commitments + first pilot.** Independent audit, mainnet with multisig upgrade authority, ML-DSA-65 commitment verification, Base settlement-receipt loop, first real release with ≥3 collaborators approving by passkey (≥2 without a crypto wallet).
4. **M4: Adoption, reference app, docs.** Free reference app for independent artists, three tutorials, external integrations.
5. **M5: Six months of maintenance.**

## Layout

```
programs/hour-consent/   native Solana program (Rust)
tests/litesvm/           end-to-end tests against the compiled .so in LiteSVM
sdk/ts/                  TypeScript SDK + tests
spec/test-vectors/       shared cross-language vectors
docs/spec.md             consent & attestation specification
.well-known/security.txt security contacts (also embedded in the program)
ci/                      GitHub Actions workflow (to be moved to .github/workflows)
```

## Build & test

Requirements: Rust stable, [Agave CLI](https://docs.anza.xyz/cli/install) (`cargo build-sbf`), Node ≥ 20. LiteSVM's precompile support builds OpenSSL, so `make`, `perl`, and `pkg-config` are needed.

```sh
cargo build-sbf --manifest-path programs/hour-consent/Cargo.toml   # builds target/deploy/hour_consent.so
cargo test --workspace                                             # unit + LiteSVM e2e (e2e skips if .so missing)
cd sdk/ts && npm ci && npm test
```

## CI

The GitHub Actions workflow is at [`ci/github-actions-ci.yml`](ci/github-actions-ci.yml). It hasn't been activated yet because the token used to publish this repo lacks the `workflow` scope. To enable it, move the file to `.github/workflows/ci.yml`. It runs fmt, clippy, `cargo build-sbf`, Rust unit + LiteSVM tests, and the TS SDK tests on the free GitHub-hosted Ubuntu runners. Until then, results come from local runs only; this README doesn't claim CI-tested status.

## Related

- hOUR Chain protocol: https://github.com/witchinghourartcollective/hOUR-Chain
- hOUR Chain Filecoin evidence module: https://github.com/witchinghourartcollective/hour-chain-filecoin

## License

Licensed under either of [Apache License 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option. `SPDX-License-Identifier: MIT OR Apache-2.0`

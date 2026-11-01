# Solana consent launch gates

Solana is an active major consent/attestation workstream under hOUR-Chain ADR-0008.
The development pause is lifted. No deployed program or production browser consent
is claimed by this change.

## Existing implementation

RecordConsent verifies a preceding secp256r1 precompile instruction over a raw
consent digest, contributor-key binding, expiry, PDA and replay protection. The
SDK has shared vectors and instruction construction. This is useful prototype
code, but browser WebAuthn signs a different message structure. DER-to-compact
conversion alone does not implement WebAuthn verification.

## Required work

1. Implement real WebAuthn signed-message verification binding authenticatorData,
   clientDataJSON type/challenge/origin, RP ID hash, UP/UV policy and credential
   identity to the consent digest. Test wrong origin/RP/challenge, stale approval,
   missing UV, replay and reordered/cross-instruction signature references. Define
   counter handling for backed-up/synchronized credentials explicitly.
2. Add contributor enrollment, identity/key rotation, recovery and revocation policy.
   A fee payer is not the contributor and must not be accepted as one.
3. Keep Rust fmt/clippy, unit tests, SBF build, LiteSVM execution and SDK tests
   passing. Latest observed main CI run 39819078982 succeeded; rerun these checks
   on each candidate program. Passing CI is not deployment evidence.
4. Provision a managed deployment signer and actual program keypair/address; update
   the placeholder program ID consistently. Never commit program keypairs. Record
   cluster genesis hash, artifact digest, program ID and deployment transaction.
5. Establish upgrade-authority custody and independent review before mainnet.
6. Define Cosmos ingestion that verifies finalized consent account ownership,
   program version/layout, release/split identity, evidence hash, nonce/expiry and
   revocation. Keep ingestion idempotent and prove rejection paths.
7. Execute the creator pilot with real browser passkeys, persist exact evidence on
   Filecoin, and link accepted consent and settlement receipts on hOUR Chain.

## IBC boundary

A DO-hosted watcher copying Solana records is an adapter, not an IBC light client.
Native Cosmos IBC wiring does not prove a Solana route. Before claiming Solana
IBC, select and deploy compatible clients/programs/contracts and relayer(s), define
finality/trust assumptions and show packet acknowledgement/timeout evidence. Until
then report adapter verification and IBC verification separately.

Read hOUR-Chain/docs/LIVE-READINESS.md for the full weekly plan and owner decisions.
Start on devnet, not with real rights decisions or funds.

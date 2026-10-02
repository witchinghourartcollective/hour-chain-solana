# Security policy

**Status: pre-alpha. Not audited. Not deployed to any cluster.** Do not use this code to custody funds, record real rights decisions, or as evidence in a dispute.

## Reporting a vulnerability

Please report privately. Do not open a public issue for anything that could expose private creator data, credentials, keys, or let someone forge or replay a consent.

- GitHub private advisory: https://github.com/witchinghourartcollective/hour-chain-solana/security/advisories/new
- Email: witchinghour@witchinghourmac.com with the subject `hour-chain-solana security report`

We aim to acknowledge reports within 5 business days. There's no bug bounty yet.

## On-chain security.txt

The `hour-consent` program embeds a [`security.txt`](https://github.com/neodyme-labs/solana-security-txt) section with the same contacts (see `programs/hour-consent/src/lib.rs`). A plain-text copy lives at `.well-known/security.txt`.

## Scope notes

- The secp256r1 signature itself is verified by Solana's native `Secp256r1SigVerify` precompile. This program checks *what* was verified (key, message), its position in the transaction, and replay and expiry rules.
- Post-quantum claims are limited. hOUR events may carry ML-DSA-65 signatures off-chain, but Solana transactions, the secp256r1 precompile, and Base settlement are **not** quantum-safe. See `docs/spec.md` §8.
- A full independent audit is planned before any mainnet deployment (Milestone 3).

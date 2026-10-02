# Contributing

Issues and pull requests are welcome.

Before opening a PR:

1. Keep creator data, credentials, private keys, program keypairs (`*-keypair.json`), unreleased media, and confidential agreements out of the repository.
2. Add or update tests for any behavior change. If you touch a byte layout, update `docs/spec.md` **and** regenerate the shared vectors (`cd sdk/ts && npm run vectors`). The Rust and TS tests both check them.
3. Run the full local check:
   ```sh
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo build-sbf --manifest-path programs/hour-consent/Cargo.toml
   cargo test --workspace
   (cd sdk/ts && npm ci && npm test)
   ```
4. Say which milestone (see README roadmap) the change supports.

By contributing, you agree your contribution is licensed under `MIT OR Apache-2.0`.

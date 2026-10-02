//! End-to-end tests: load the compiled SBF program into LiteSVM (with Agave precompiles
//! enabled) and send real transactions [Secp256r1SigVerify, RecordConsent].
//!
//! Requires `cargo build-sbf` first (CI does this). Skips with a message if the .so is missing.

use std::path::PathBuf;

use hour_consent::consent::ConsentFields;
use hour_consent::instruction::ConsentInstruction;
use hour_consent::precompile::build_single_signature_data;
use hour_consent::state::ConsentRecord;
use litesvm::LiteSVM;
use solana_address::Address;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_signer::Signer;
use solana_transaction::Transaction;

const SECP256R1: Address = Address::from_str_const("Secp256r1SigVerify1111111111111111111111111");
const IX_SYSVAR: Address = Address::from_str_const("Sysvar1nstructions1111111111111111111111111");
const SYSTEM: Address = Address::from_str_const("11111111111111111111111111111111");

fn program_id() -> Address {
    Address::new_from_array(hour_consent::ID.to_bytes())
}

fn so_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy/hour_consent.so")
}

fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

struct V {
    fields: ConsentFields,
    pubkey: [u8; 33],
    digest: [u8; 32],
    sig: [u8; 64],
}

fn vector() -> V {
    let j: serde_json::Value =
        serde_json::from_str(include_str!("../../../spec/test-vectors/consent-v1.json")).unwrap();
    let s = |k: &str| hex(j[k].as_str().unwrap());
    let n = |k: &str| j[k].as_str().unwrap().to_string();
    V {
        fields: ConsentFields {
            release_id: s("release_id").try_into().unwrap(),
            split_version_hash: s("split_version_hash").try_into().unwrap(),
            contributor_id: s("contributor_id").try_into().unwrap(),
            nonce: n("nonce").parse().unwrap(),
            expiry: n("expiry").parse().unwrap(),
        },
        pubkey: s("passkey_pubkey_compressed").try_into().unwrap(),
        digest: s("consent_digest").try_into().unwrap(),
        sig: s("signature_compact_low_s").try_into().unwrap(),
    }
}

fn setup() -> Option<(LiteSVM, Keypair)> {
    if !so_path().exists() {
        eprintln!(
            "SKIP: {} not found; run `cargo build-sbf` first",
            so_path().display()
        );
        return None;
    }
    let mut svm = LiteSVM::new();
    svm.add_program_from_file(program_id(), so_path()).unwrap();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();
    Some((svm, payer))
}

fn record_address(f: &ConsentFields) -> Address {
    Address::find_program_address(
        &[
            b"consent",
            &f.release_id,
            &f.split_version_hash,
            &f.contributor_id,
        ],
        &program_id(),
    )
    .0
}

fn ixs(payer: &Address, f: &ConsentFields, precompile_data: Vec<u8>) -> Vec<Instruction> {
    vec![
        Instruction {
            program_id: SECP256R1,
            accounts: vec![],
            data: precompile_data,
        },
        Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(*payer, true),
                AccountMeta::new(record_address(f), false),
                AccountMeta::new_readonly(IX_SYSVAR, false),
                AccountMeta::new_readonly(SYSTEM, false),
            ],
            data: ConsentInstruction::RecordConsent(*f).pack(),
        },
    ]
}

fn send(svm: &mut LiteSVM, payer: &Keypair, ixs: &[Instruction]) -> Result<(), String> {
    let msg = Message::new(ixs, Some(&payer.pubkey()));
    let tx = Transaction::new(&[payer], msg, svm.latest_blockhash());
    svm.send_transaction(tx)
        .map(|_| ())
        .map_err(|e| format!("{:?}", e.err))
}

#[test]
fn records_valid_passkey_consent() {
    let Some((mut svm, payer)) = setup() else {
        return;
    };
    let v = vector();
    let data = build_single_signature_data(&v.pubkey, &v.sig, &v.digest);
    send(&mut svm, &payer, &ixs(&payer.pubkey(), &v.fields, data))
        .expect("consent should be recorded");

    let acct = svm
        .get_account(&record_address(&v.fields))
        .expect("record exists");
    assert_eq!(acct.owner, program_id());
    let rec = ConsentRecord::unpack(&acct.data).expect("record decodes");
    assert_eq!(rec.passkey_pubkey, v.pubkey);
    assert_eq!(rec.digest, v.digest);
    assert_eq!(rec.contributor_id, v.fields.contributor_id);
    assert_eq!(rec.payer, payer.pubkey().to_bytes());
}

#[test]
fn rejects_replay_of_same_consent() {
    let Some((mut svm, payer)) = setup() else {
        return;
    };
    let v = vector();
    let data = build_single_signature_data(&v.pubkey, &v.sig, &v.digest);
    send(
        &mut svm,
        &payer,
        &ixs(&payer.pubkey(), &v.fields, data.clone()),
    )
    .unwrap();
    svm.expire_blockhash();
    let err = send(&mut svm, &payer, &ixs(&payer.pubkey(), &v.fields, data)).unwrap_err();
    assert!(
        err.contains("Custom(9)"),
        "expected ConsentAlreadyRecorded, got {err}"
    );
}

#[test]
fn invalid_signature_fails_in_precompile() {
    let Some((mut svm, payer)) = setup() else {
        return;
    };
    let v = vector();
    let mut bad = v.sig;
    bad[10] ^= 0xff;
    let data = build_single_signature_data(&v.pubkey, &bad, &v.digest);
    let err = send(&mut svm, &payer, &ixs(&payer.pubkey(), &v.fields, data)).unwrap_err();
    // Failure must come from instruction 0 (the secp256r1 precompile), not from our program.
    assert!(
        err.starts_with("InstructionError(0,"),
        "expected precompile failure, got {err}"
    );
    assert!(svm.get_account(&record_address(&v.fields)).is_none());
}

#[test]
fn rejects_signature_over_different_fields() {
    let Some((mut svm, payer)) = setup() else {
        return;
    };
    let v = vector();
    // Valid signature over the vector digest, but the program is asked to record a different nonce.
    let data = build_single_signature_data(&v.pubkey, &v.sig, &v.digest);
    let mut other = v.fields;
    other.nonce += 1;
    let err = send(&mut svm, &payer, &ixs(&payer.pubkey(), &other, data)).unwrap_err();
    assert!(
        err.contains("Custom(6)"),
        "expected MessageDigestMismatch, got {err}"
    );
}

#[test]
fn rejects_missing_precompile_instruction() {
    let Some((mut svm, payer)) = setup() else {
        return;
    };
    let v = vector();
    let data = build_single_signature_data(&v.pubkey, &v.sig, &v.digest);
    let only_record = vec![ixs(&payer.pubkey(), &v.fields, data).remove(1)];
    let err = send(&mut svm, &payer, &only_record).unwrap_err();
    assert!(
        err.contains("Custom(1)"),
        "expected MissingPrecompileInstruction, got {err}"
    );
}

#[test]
fn rejects_expired_consent() {
    let Some((mut svm, payer)) = setup() else {
        return;
    };
    let v = vector();
    let mut clock: solana_clock::Clock = svm.get_sysvar();
    clock.unix_timestamp = v.fields.expiry + 1;
    svm.set_sysvar(&clock);
    let data = build_single_signature_data(&v.pubkey, &v.sig, &v.digest);
    let err = send(&mut svm, &payer, &ixs(&payer.pubkey(), &v.fields, data)).unwrap_err();
    assert!(
        err.contains("Custom(8)"),
        "expected ConsentExpired, got {err}"
    );
}

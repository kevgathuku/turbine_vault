use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    turbine_vault::{error::ErrorCode, state::Counter},
};

fn setup() -> (LiteSVM, Keypair, Pubkey) {
    let program_id = turbine_vault::id();
    let payer = Keypair::new();
    let (counter, _) =
        Pubkey::find_program_address(&[turbine_vault::constants::COUNTER_SEED], &program_id);
    let mut svm = LiteSVM::new();
    let program_bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/turbine_vault.so"
    ));
    svm.add_program(program_id, program_bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();
    (svm, payer, counter)
}

fn initialize_tx(payer: &Pubkey, counter: &Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        turbine_vault::id(),
        &turbine_vault::instruction::Initialize {}.data(),
        turbine_vault::accounts::Initialize {
            payer: *payer,
            counter: *counter,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

fn increment_tx(counter: &Pubkey, authority: &Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        turbine_vault::id(),
        &turbine_vault::instruction::Increment {}.data(),
        turbine_vault::accounts::Increment {
            counter: *counter,
            authority: *authority,
        }
        .to_account_metas(None),
    )
}

fn send(
    svm: &mut LiteSVM,
    signers: &[&Keypair],
    ix: Instruction,
) -> litesvm::types::TransactionResult {
    svm.expire_blockhash();
    let payer = signers[0].pubkey();
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx)
}

fn counter_state(svm: &LiteSVM, counter: &Pubkey) -> Counter {
    let account = svm.get_account(counter).unwrap();
    Counter::try_deserialize(&mut &account.data[..]).unwrap()
}

#[test]
fn test_initialize_and_increment() {
    let (mut svm, payer, counter) = setup();

    send(
        &mut svm,
        &[&payer],
        initialize_tx(&payer.pubkey(), &counter),
    )
    .expect("initialize should succeed");

    let state = counter_state(&svm, &counter);
    assert_eq!(state.count, 0);
    assert_eq!(state.authority, payer.pubkey());

    send(&mut svm, &[&payer], increment_tx(&counter, &payer.pubkey()))
        .expect("increment should succeed");

    let state = counter_state(&svm, &counter);
    assert_eq!(state.count, 1);
    assert_eq!(state.authority, payer.pubkey());
}

#[test]
fn test_double_initialize_fails() {
    let (mut svm, payer, counter) = setup();

    send(
        &mut svm,
        &[&payer],
        initialize_tx(&payer.pubkey(), &counter),
    )
    .expect("first initialize should succeed");

    let err = send(
        &mut svm,
        &[&payer],
        initialize_tx(&payer.pubkey(), &counter),
    )
    .unwrap_err();
    let err_str = format!("{err:?}");
    assert!(
        err_str.to_lowercase().contains("already in use")
            || err_str.contains("AccountInUse")
            || err_str.contains("0x0"),
        "expected already-in-use error, got: {err_str}"
    );
}

#[test]
fn test_unauthorized_increment_fails() {
    let (mut svm, payer, counter) = setup();
    let attacker = Keypair::new();
    svm.airdrop(&attacker.pubkey(), 1_000_000_000).unwrap();

    send(
        &mut svm,
        &[&payer],
        initialize_tx(&payer.pubkey(), &counter),
    )
    .expect("initialize should succeed");

    let err = send(
        &mut svm,
        &[&attacker],
        increment_tx(&counter, &attacker.pubkey()),
    )
    .unwrap_err();
    let expected_msg = ErrorCode::Unauthorized.to_string();
    let err_str = format!("{err:?}");
    assert!(
        err_str.contains(&expected_msg),
        "expected unauthorized error, got: {err_str}"
    );

    // state unchanged
    assert_eq!(counter_state(&svm, &counter).count, 0);
}

#[test]
fn test_increment_overflow_fails() {
    let (mut svm, payer, counter) = setup();

    send(
        &mut svm,
        &[&payer],
        initialize_tx(&payer.pubkey(), &counter),
    )
    .expect("initialize should succeed");

    for _ in 0..turbine_vault::constants::MAX_COUNT {
        send(&mut svm, &[&payer], increment_tx(&counter, &payer.pubkey()))
            .expect("increment should succeed");
    }
    assert_eq!(
        counter_state(&svm, &counter).count,
        turbine_vault::constants::MAX_COUNT
    );

    let err = send(&mut svm, &[&payer], increment_tx(&counter, &payer.pubkey())).unwrap_err();

    let expected_msg = ErrorCode::CounterOverflow.to_string();
    let err_str = format!("{err:?}");
    assert!(
        err_str.contains(&expected_msg),
        "expected overflow error, got: {err_str}"
    );
}

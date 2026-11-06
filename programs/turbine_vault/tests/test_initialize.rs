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
    turbine_vault::constants::{STATE, VAULT_SEED},
    turbine_vault::VaultState,
};

struct SetupFixture {
    svm: LiteSVM,
    user: Keypair,
    vault_state: Pubkey,
    state_bump: u8,
    vault_bump: u8,
    vault: Pubkey,
}

fn setup() -> SetupFixture {
    let program_id = turbine_vault::id();
    let user = Keypair::new();

    let (vault, vault_bump) =
        Pubkey::find_program_address(&[VAULT_SEED, user.pubkey().as_ref()], &program_id);
    let (vault_state, state_bump) =
        Pubkey::find_program_address(&[STATE, user.pubkey().as_ref()], &program_id);

    let mut svm = LiteSVM::new();
    let program_bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/turbine_vault.so"
    ));

    svm.add_program(program_id, program_bytes).unwrap();
    svm.airdrop(&user.pubkey(), 1_000_000_000).unwrap();

    SetupFixture {
        svm,
        user,
        vault,
        state_bump,
        vault_bump,
        vault_state,
    }
}

fn initialize_tx(user: &Pubkey, vault: &Pubkey, vault_state: &Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        turbine_vault::id(),
        &turbine_vault::instruction::Initialize {}.data(),
        turbine_vault::accounts::Initialize {
            user: *user,
            vault: *vault,
            vault_state: *vault_state,
            system_program: system_program::ID,
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

#[test]
fn test_initialize() {
    let SetupFixture {
        mut svm,
        user,
        vault_state,
        vault,
        state_bump,
        vault_bump,
    } = setup();

    let program_id = turbine_vault::id();

    let instruction = initialize_tx(&user.pubkey(), &vault, &vault_state);

    // tx succeeded
    send(&mut svm, &[&user], instruction).expect("initialize should succeed");

    let vault_state_account = svm
        .get_account(&vault_state)
        .expect("vault_state should exist after initialize");
    let new_state = VaultState::try_deserialize(&mut &vault_state_account.data[..]).unwrap();

    // vault State checks
    assert_eq!(
        vault_state_account.owner, program_id,
        "vault_state owner should be program id"
    );
    assert_eq!(new_state.state_bump, state_bump);
    assert_eq!(new_state.vault_bump, vault_bump);

    // vault PDA was created with exactly rent-exempt funding
    let rent_exempt = svm.minimum_balance_for_rent_exemption(0);
    assert_eq!(
        svm.get_balance(&vault).unwrap(),
        rent_exempt,
        "vault should be rent-exempt after initialize"
    );

    // vault is still a plain system account
    let vault_acct = svm.get_account(&vault).unwrap();
    assert_eq!(vault_acct.owner, system_program::ID);
    assert!(vault_acct.data.is_empty());
}

#[test]
fn test_double_initialize_fails() {
    let SetupFixture {
        mut svm,
        user,
        vault_state,
        vault,
        state_bump: _,
        vault_bump: _,
    } = setup();

    let instruction = initialize_tx(&user.pubkey(), &vault, &vault_state);

    send(&mut svm, &[&user], instruction).expect("first initialize should succeed");

    let err = send(
        &mut svm,
        &[&user],
        initialize_tx(&user.pubkey(), &vault, &vault_state),
    )
    .unwrap_err();

    let err_str = format!("{err:?}");
    assert!(
        err_str.to_lowercase().contains("already in use"),
        "expected already-in-use error, got: {err_str}"
    );
}

use {
    anchor_lang::{solana_program::system_program, AccountDeserialize},
    solana_keypair::Keypair,
    solana_signer::Signer,
    turbine_vault::VaultState,
};

mod common;
use common::*;

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
        ..
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

#[test]
fn test_initialize_rejects_non_signer_user() {
    let SetupFixture {
        mut svm,
        user,
        vault_state,
        vault,
        ..
    } = setup();

    let stranger = Keypair::new();
    svm.airdrop(&stranger.pubkey(), 1_000_000_000).unwrap();

    let mut init_txn = initialize_tx(&user.pubkey(), &vault, &vault_state);
    init_txn.accounts[0].is_signer = false;

    // stranger pays + signs
    let err = send(&mut svm, &[&stranger], init_txn);

    let err_str = format!("{err:?}");
    assert!(
        err_str.contains("AccountNotSigner"),
        "expected account not signer error, got: {err_str}"
    );
}

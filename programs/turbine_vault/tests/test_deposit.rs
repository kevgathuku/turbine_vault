use {
    anchor_lang::{
        error::ErrorCode::AccountNotInitialized, error::ErrorCode::ConstraintSeeds, prelude::Pubkey,
    },
    solana_keypair::Keypair,
    solana_signer::Signer,
    turbine_vault::constants::{VAULT_SEED, VAULT_STATE_SEED},
    turbine_vault::error::ErrorCode,
};

mod common;
use common::*;

const DEPOSIT_AMOUNT: u64 = 500 * LAMPORTS_PER_SOL;

#[test]
fn test_deposit() {
    let SetupFixture {
        mut svm,
        user,
        vault_state,
        vault,
        state_bump: _,
        vault_bump: _,
    } = setup();

    let rent_exempt = svm.minimum_balance_for_rent_exemption(0);

    let initialize_txn = initialize_tx(&user.pubkey(), &vault, &vault_state);
    let deposit_txn = deposit_tx(&user.pubkey(), &vault, &vault_state, DEPOSIT_AMOUNT);

    send(&mut svm, &[&user], initialize_txn).expect("initialize should succeed");
    send(&mut svm, &[&user], deposit_txn).expect("deposit should succeed");

    assert_eq!(
        svm.get_balance(&vault).unwrap(),
        rent_exempt + DEPOSIT_AMOUNT,
        "vault should increase by the deposit"
    );
}

#[test]
fn test_deposit_before_initialize_fails() {
    let SetupFixture {
        mut svm,
        user,
        vault_state,
        vault,
        state_bump: _,
        vault_bump: _,
    } = setup();

    let deposit_tx = deposit_tx(&user.pubkey(), &vault, &vault_state, DEPOSIT_AMOUNT);
    let err = send(&mut svm, &[&user], deposit_tx).unwrap_err();

    let expected_msg = AccountNotInitialized.to_string();

    let err_str = format!("{err:?}");
    assert!(
        err_str.contains(&expected_msg),
        "expected account not initialized error, got: {err_str}"
    );
}

#[test]
fn test_deposit_zero_amount_fails() {
    let SetupFixture {
        mut svm,
        user,
        vault_state,
        vault,
        state_bump: _,
        vault_bump: _,
    } = setup();

    let init_txn = initialize_tx(&user.pubkey(), &vault, &vault_state);
    let deposit_txn = deposit_tx(&user.pubkey(), &vault, &vault_state, 0);

    send(&mut svm, &[&user], init_txn).expect("initialize should succeed");

    let err = send(&mut svm, &[&user], deposit_txn).unwrap_err();

    let expected_msg = ErrorCode::InvalidAmount.to_string();
    let err_str = format!("{err:?}");

    assert!(
        err_str.contains(&expected_msg),
        "expected invalid amount error, got: {err_str}"
    );
}

#[test]
fn test_deposit_rejects_non_signer_user() {
    let SetupFixture {
        mut svm,
        user,
        vault_state,
        vault,
        state_bump: _,
        vault_bump: _,
    } = setup();

    let stranger = Keypair::new();
    svm.airdrop(&stranger.pubkey(), DEPOSIT_AMOUNT).unwrap();

    let init_txn = initialize_tx(&user.pubkey(), &vault, &vault_state);
    send(&mut svm, &[&user], init_txn).expect("initialize should succeed");

    let mut deposit_txn = deposit_tx(&user.pubkey(), &vault, &vault_state, DEPOSIT_AMOUNT);
    deposit_txn.accounts[0].is_signer = false;

    // stranger pays + signs
    let err = send(&mut svm, &[&stranger], deposit_txn).unwrap_err();

    let err_str = format!("{err:?}");
    assert!(
        err_str.contains("AccountNotSigner"),
        "expected account not signer error, got: {err_str}"
    );
}

#[test]
fn test_deposit_rejects_amount_exceeding_user_balance() {
    let SetupFixture {
        mut svm,
        user,
        vault_state,
        vault,
        state_bump: _,
        vault_bump: _,
    } = setup();

    let init_txn = initialize_tx(&user.pubkey(), &vault, &vault_state);
    send(&mut svm, &[&user], init_txn).expect("initialize should succeed");

    let balance = svm.get_balance(&user.pubkey()).unwrap();
    let deposit_txn = deposit_tx(&user.pubkey(), &vault, &vault_state, balance + 1);

    let err = send(&mut svm, &[&user], deposit_txn).unwrap_err();
    let expected_msg = ErrorCode::InsufficientFunds.to_string();
    let err_str = format!("{err:?}");
    assert!(
        err_str.contains(&expected_msg),
        "expected insufficient funds error, got: {err_str}"
    );
}

#[test]
fn test_deposit_wrong_vault_state_fails() {
    let SetupFixture {
        mut svm,
        user,
        vault,
        vault_state,
        ..
    } = setup();

    // Initialize a different user so its vault_state exists
    let stranger = Keypair::new();
    svm.airdrop(&stranger.pubkey(), DEPOSIT_AMOUNT).unwrap();
    let (stranger_vault, _) = Pubkey::find_program_address(
        &[VAULT_SEED, stranger.pubkey().as_ref()],
        &turbine_vault::id(),
    );
    let (stranger_state, _) = Pubkey::find_program_address(
        &[VAULT_STATE_SEED, stranger.pubkey().as_ref()],
        &turbine_vault::id(),
    );

    let stranger_init_txn = initialize_tx(&stranger.pubkey(), &stranger_vault, &stranger_state);
    send(&mut svm, &[&stranger], stranger_init_txn).expect("stranger initialize should succeed");

    let init_txn = initialize_tx(&user.pubkey(), &vault, &vault_state);
    send(&mut svm, &[&user], init_txn).expect("initialize should succeed");

    // Pass vault state derived with different user's key
    let bad_ix = deposit_tx(&user.pubkey(), &vault, &stranger_state, DEPOSIT_AMOUNT);
    let err = send(&mut svm, &[&user], bad_ix).unwrap_err();

    let expected_msg = ConstraintSeeds.to_string();
    let err_str = format!("{err:?}");
    assert!(
        err_str.contains(&expected_msg),
        "expected seeds error, got: {err_str}"
    );
}

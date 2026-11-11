use {
    anchor_lang::error::ErrorCode::AccountNotInitialized, solana_keypair::Keypair,
    solana_signer::Signer, turbine_vault::error::ErrorCode,
};

mod common;
use common::*;

const DEPOSIT_AMOUNT: u64 = 500 * LAMPORTS_PER_SOL;
const WITHDRAW_AMOUNT: u64 = 200 * LAMPORTS_PER_SOL;

#[test]
fn test_withdraw() {
    let SetupFixture {
        mut svm,
        user,
        vault_state,
        vault,
        ..
    } = setup();

    let rent_exempt = svm.minimum_balance_for_rent_exemption(0);

    let initialize_txn = initialize_tx(&user.pubkey(), &vault, &vault_state);
    let deposit_txn = deposit_tx(&user.pubkey(), &vault, &vault_state, DEPOSIT_AMOUNT);
    let withdraw_txn = withdraw_tx(&user.pubkey(), &vault, &vault_state, WITHDRAW_AMOUNT);

    send(&mut svm, &[&user], initialize_txn).expect("initialize should succeed");
    send(&mut svm, &[&user], deposit_txn).expect("deposit should succeed");
    send(&mut svm, &[&user], withdraw_txn).expect("withdraw should succeed");

    assert_eq!(
        svm.get_balance(&vault).unwrap(),
        rent_exempt + DEPOSIT_AMOUNT - WITHDRAW_AMOUNT,
        "vault should decrease by the withdraw amount"
    );
}

#[test]
fn test_withdraw_before_initialize_fails() {
    let SetupFixture {
        mut svm,
        user,
        vault_state,
        vault,
        ..
    } = setup();

    let withdraw_txn = withdraw_tx(&user.pubkey(), &vault, &vault_state, WITHDRAW_AMOUNT);
    let err = send(&mut svm, &[&user], withdraw_txn).unwrap_err();

    let expected_msg = AccountNotInitialized.to_string();

    let err_str = format!("{err:?}");
    assert!(
        err_str.contains(&expected_msg),
        "expected account not initialized error, got: {err_str}"
    );
}

#[test]
fn test_withdraw_zero_amount_fails() {
    let SetupFixture {
        mut svm,
        user,
        vault_state,
        vault,
        ..
    } = setup();

    let initialize_txn = initialize_tx(&user.pubkey(), &vault, &vault_state);
    let withdraw_txn = withdraw_tx(&user.pubkey(), &vault, &vault_state, 0);

    send(&mut svm, &[&user], initialize_txn).expect("initialize should succeed");

    let err = send(&mut svm, &[&user], withdraw_txn).unwrap_err();

    let expected_msg = ErrorCode::InvalidAmount.to_string();
    let err_str = format!("{err:?}");

    assert!(
        err_str.contains(&expected_msg),
        "expected invalid amount error, got: {err_str}"
    );
}

#[test]
fn test_withdraw_rejects_non_signer_user() {
    let SetupFixture {
        mut svm,
        user,
        vault_state,
        vault,
        ..
    } = setup();

    let stranger = Keypair::new();
    svm.airdrop(&stranger.pubkey(), 1_000_000_000).unwrap();

    let init_txn = initialize_tx(&user.pubkey(), &vault, &vault_state);
    send(&mut svm, &[&user], init_txn).expect("initialize should succeed");

    let deposit_txn = deposit_tx(&user.pubkey(), &vault, &vault_state, DEPOSIT_AMOUNT);
    send(&mut svm, &[&user], deposit_txn).expect("deposit should succeed");

    let mut withdraw_txn = withdraw_tx(&user.pubkey(), &vault, &vault_state, WITHDRAW_AMOUNT);
    withdraw_txn.accounts[0].is_signer = false;

    // stranger pays + signs
    let err = send(&mut svm, &[&stranger], withdraw_txn).unwrap_err();

    let err_str = format!("{err:?}");
    assert!(
        err_str.contains("AccountNotSigner"),
        "expected account not signer error, got: {err_str}"
    );
}

#[test]
fn test_withdraw_rejects_amount_exceeding_vault_balance() {
    let SetupFixture {
        mut svm,
        user,
        vault_state,
        vault,
        ..
    } = setup();

    let init_txn = initialize_tx(&user.pubkey(), &vault, &vault_state);
    send(&mut svm, &[&user], init_txn).expect("initialize should succeed");

    let vault_balance = svm.get_balance(&vault).unwrap();
    let withdraw_txn = withdraw_tx(&user.pubkey(), &vault, &vault_state, vault_balance + 1);

    let err = send(&mut svm, &[&user], withdraw_txn).unwrap_err();
    let expected_msg = ErrorCode::InsufficientVaultFunds.to_string();
    let err_str = format!("{err:?}");
    assert!(
        err_str.contains(&expected_msg),
        "expected insufficient funds error, got: {err_str}"
    );
}

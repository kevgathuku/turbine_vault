use {
    anchor_lang::error::ErrorCode::AccountNotInitialized, solana_keypair::Keypair,
    solana_signer::Signer, turbine_vault::error::ErrorCode,
};

mod common;
use common::*;

const BASE_UNITS_PER_TOKEN: u64 = 1_000_000;
const DEPOSIT_AMOUNT: u64 = 500 * BASE_UNITS_PER_TOKEN;

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

    let initialize_tx = initialize_tx(&user.pubkey(), &vault, &vault_state);
    let deposit_tx = deposit_tx(&user.pubkey(), &vault, &vault_state, DEPOSIT_AMOUNT);

    send(&mut svm, &[&user], initialize_tx).expect("initialize should succeed");
    send(&mut svm, &[&user], deposit_tx).expect("deposit should succeed");

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
    svm.airdrop(&stranger.pubkey(), 1_000_000_000).unwrap();

    let init_txn = initialize_tx(&user.pubkey(), &vault, &vault_state);
    send(&mut svm, &[&user], init_txn).expect("initialize should succeed");

    let mut deposit_txn = deposit_tx(&user.pubkey(), &vault, &vault_state, DEPOSIT_AMOUNT);
    deposit_txn.accounts[0].is_signer = false;

    // stranger pays + signs
    let err = send(&mut svm, &[&stranger], deposit_txn);

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

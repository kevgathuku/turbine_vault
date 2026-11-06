use {anchor_lang::error::ErrorCode::AccountNotInitialized, solana_signer::Signer};

mod common;
use common::*;

const DEPOSIT_AMOUNT: u64 = LAMPORTS_PER_SOL * 2;

#[test]
fn test_close() {
    let SetupFixture {
        mut svm,
        user,
        vault_state,
        vault,
        state_bump: _,
        vault_bump: _,
    } = setup();

    let initialize_txn = initialize_tx(&user.pubkey(), &vault, &vault_state);
    let deposit_txn = deposit_tx(&user.pubkey(), &vault, &vault_state, DEPOSIT_AMOUNT);
    let close_txn = close_tx(&user.pubkey(), &vault, &vault_state);

    send(&mut svm, &[&user], initialize_txn).expect("initialize should succeed");
    send(&mut svm, &[&user], deposit_txn).expect("deposit should succeed");

    let user_before_close = svm.get_balance(&user.pubkey()).unwrap();
    let vault_before_close = svm.get_balance(&vault).unwrap();
    let vault_state_rent = svm.get_account(&vault_state).unwrap().lamports;

    let close_meta = send(&mut svm, &[&user], close_txn).expect("close should succeed");

    assert!(
        svm.get_account(&vault_state).is_none(),
        "vault_state should be closed"
    );
    assert_eq!(
        svm.get_balance(&vault).unwrap_or(0),
        0,
        "vault should be empty"
    );
    assert_eq!(
        svm.get_balance(&user.pubkey()).unwrap(),
        user_before_close + vault_before_close + vault_state_rent - close_meta.fee,
        "user should recover vault balance + state rent minus tx fee"
    );
}

#[test]
fn test_close_before_initialize_fails() {
    let SetupFixture {
        mut svm,
        user,
        vault_state,
        vault,
        state_bump: _,
        vault_bump: _,
    } = setup();

    let close_txn = close_tx(&user.pubkey(), &vault, &vault_state);
    let err = send(&mut svm, &[&user], close_txn).unwrap_err();

    let expected_msg = AccountNotInitialized.to_string();

    let err_str = format!("{err:?}");
    assert!(
        err_str.contains(&expected_msg),
        "expected account not initialized error, got: {err_str}"
    );
}

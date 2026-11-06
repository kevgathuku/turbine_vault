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
    turbine_vault::{VaultState, STATE, VAULT_SEED},
};

#[test]
fn test_initialize() {
    let program_id = turbine_vault::id();
    let user = Keypair::new();
    let (vault_state, state_bump) =
        Pubkey::find_program_address(&[STATE, user.pubkey().as_ref()], &program_id);
    let (vault, vault_bump) =
        Pubkey::find_program_address(&[VAULT_SEED, user.pubkey().as_ref()], &program_id);

    let mut svm = LiteSVM::new();
    let program_bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/turbine_vault.so"
    ));
    svm.add_program(program_id, program_bytes).unwrap();
    svm.airdrop(&user.pubkey(), 1_000_000_000).unwrap();

    let instruction = Instruction::new_with_bytes(
        program_id,
        &turbine_vault::instruction::Initialize {}.data(),
        turbine_vault::accounts::Initialize {
            user: user.pubkey(),
            vault_state,
            vault,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&user.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&user]).unwrap();

    let res = svm.send_transaction(tx);
    assert!(res.is_ok());

    let rent_exempt = svm.minimum_balance_for_rent_exemption(0);
    let vault_state_account = svm
        .get_account(&vault_state)
        .expect("vault_state should exist after initialize");
    assert_eq!(vault_state_account.owner, program_id);
    let vault_state_data =
        VaultState::try_deserialize(&mut &vault_state_account.data[..]).unwrap();
    assert_eq!(vault_state_data.state_bump, state_bump);
    assert_eq!(vault_state_data.vault_bump, vault_bump);
    assert_eq!(
        svm.get_balance(&vault).unwrap(),
        rent_exempt,
        "vault should be rent-exempt after initialize"
    );
}

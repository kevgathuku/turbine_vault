#![allow(dead_code)]

use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    turbine_vault::accounts,
    turbine_vault::constants::{VAULT_SEED, VAULT_STATE_SEED},
    turbine_vault::instruction,
};

pub const LAMPORTS_PER_SOL: u64 = 1_000_000;

pub struct SetupFixture {
    pub svm: LiteSVM,
    pub user: Keypair,
    pub vault_state: Pubkey,
    pub state_bump: u8,
    pub vault_bump: u8,
    pub vault: Pubkey,
}

pub fn setup() -> SetupFixture {
    let program_id = turbine_vault::id();
    let user = Keypair::new();

    let (vault, vault_bump) =
        Pubkey::find_program_address(&[VAULT_SEED, user.pubkey().as_ref()], &program_id);
    let (vault_state, state_bump) =
        Pubkey::find_program_address(&[VAULT_STATE_SEED, user.pubkey().as_ref()], &program_id);

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

pub fn initialize_tx(user: &Pubkey, vault: &Pubkey, vault_state: &Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        turbine_vault::id(),
        &instruction::Initialize {}.data(),
        accounts::Initialize {
            user: *user,
            vault: *vault,
            vault_state: *vault_state,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

pub fn deposit_tx(user: &Pubkey, vault: &Pubkey, vault_state: &Pubkey, amount: u64) -> Instruction {
    Instruction::new_with_bytes(
        turbine_vault::id(),
        &instruction::Deposit { amount }.data(),
        accounts::Deposit {
            user: *user,
            vault: *vault,
            vault_state: *vault_state,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

pub fn withdraw_tx(
    user: &Pubkey,
    vault: &Pubkey,
    vault_state: &Pubkey,
    amount: u64,
) -> Instruction {
    Instruction::new_with_bytes(
        turbine_vault::id(),
        &instruction::Withdraw { amount }.data(),
        accounts::Withdraw {
            user: *user,
            vault: *vault,
            vault_state: *vault_state,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

pub fn close_tx(user: &Pubkey, vault: &Pubkey, vault_state: &Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        turbine_vault::id(),
        &instruction::Close {}.data(),
        accounts::Close {
            user: *user,
            vault: *vault,
            vault_state: *vault_state,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

pub fn send(
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

use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Deposit amount must be greater than zero")]
    InvalidAmount,
    #[msg("Insufficient funds for deposit")]
    InsufficientFunds,
    #[msg("Insufficient funds for withdrawal")]
    InsufficientVaultFunds,
}

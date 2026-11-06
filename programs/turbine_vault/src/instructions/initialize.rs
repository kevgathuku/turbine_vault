use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{constants::*, state::Counter};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + Counter::INIT_SPACE,
        seeds = [COUNTER_SEED],
        bump
    )]
    pub counter: Account<'info, Counter>,
    pub system_program: Program<'info, System>,
}

pub fn handle_initialize(ctx: Context<Initialize>) -> Result<()> {
    ctx.accounts.counter.count = 0;
    ctx.accounts.counter.authority = ctx.accounts.payer.key();

    let cpi_accounts = Transfer {
        from: ctx.accounts.payer.to_account_info(),
        to: ctx.accounts.counter.to_account_info(),
    };
    // read the system program from context
    let cpi_program = ctx.accounts.system_program.key();

    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    transfer(cpi_ctx, HELLO_WORLD_LAMPORTS)?;

    msg!("Hello, world! Counter initialized");
    Ok(())
}

use anchor_lang::prelude::*;

declare_id!("Fg6PaFFmS5q6S5rk6pJ2A6q7Q5q6S5rk6pJ2A6q7");

/// PDA smart account. Stub — M9 implements execute (CPI via invoke_signed),
/// add_session_key, recover.
#[program]
pub mod smart_account {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

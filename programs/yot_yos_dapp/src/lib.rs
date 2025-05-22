use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};

declare_id!("ReplaceMeWithRealProgramId11111111111111111111111111");

#[program]
pub mod yot_yos_dapp {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        global_state.pool_authority = ctx.accounts.pool_authority.key();
        global_state.accumulated_yot = 0;
        global_state.cashback_bps = 500;
        global_state.liquidity_threshold = 100_000_000;
        global_state.weekly_apr_bps = 7000;
        Ok(())
    }

    pub fn swap_now(ctx: Context<SwapAndDistribute>, yot_amount: u64) -> Result<()> {
        msg!("Start swap_now");

        let global_state_ref = &ctx.accounts.global_state;
        let cashback_amount = yot_amount * global_state_ref.cashback_bps / 10_000;
        let vault_amount = yot_amount * 20 / 100;
        let user_receive = yot_amount - cashback_amount - vault_amount;

        token::transfer(ctx.accounts.transfer_to_user_context(), user_receive)?;
        token::transfer(ctx.accounts.transfer_to_vault_context(), vault_amount)?;
        token::mint_to(ctx.accounts.mint_yos_to_user_context(), cashback_amount)?;

        ctx.accounts.global_state.accumulated_yot += vault_amount;

        msg!("End swap_now");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = payer, space = 8 + 32 + 8 + 8 + 8 + 8)]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: storing pubkey
    pub pool_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SwapAndDistribute<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub yot_source: Account<'info, TokenAccount>,
    #[account(mut)]
    pub yot_user_dest: Account<'info, TokenAccount>,
    #[account(mut)]
    pub yot_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub yos_mint: Account<'info, Mint>,
    #[account(mut)]
    pub yos_user_dest: Account<'info, TokenAccount>,
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,
    pub token_program: Program<'info, Token>,
}

impl<'info> SwapAndDistribute<'info> {
    fn transfer_to_user_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.yot_source.to_account_info(),
                to: self.yot_user_dest.to_account_info(),
                authority: self.user.to_account_info(),
            },
        )
    }

    fn transfer_to_vault_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.yot_source.to_account_info(),
                to: self.yot_vault.to_account_info(),
                authority: self.user.to_account_info(),
            },
        )
    }

    fn mint_yos_to_user_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.yos_mint.to_account_info(),
                to: self.yos_user_dest.to_account_info(),
                authority: self.global_state.to_account_info(),
            },
        )
    }
}

#[account]
pub struct GlobalState {
    pub pool_authority: Pubkey,
    pub accumulated_yot: u64,
    pub cashback_bps: u64,
    pub liquidity_threshold: u64,
    pub weekly_apr_bps: u64,
}
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};

declare_id!("EymbuAC1fMRMieLcmAf8edFA39292ckFqjEunxjrwgu8");

#[program]
pub mod yot_yos_dapp {
pub fn debug_log_sighash(_ctx: Context<Ping>) -> Result<()> {
        use anchor_lang::solana_program::hash::hash;
        let sighash = hash(b"global::swap_and_distribute");
        msg!("🧩 Expected sighash for swap_and_distribute: {:?}", &sighash.to_bytes()[..8]);
        Ok(())
    }
    use super::*;

    pub fn ping(_ctx: Context<Ping>) -> Result<()> {
        msg!("Ping successful");
        Ok(())
    }

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        global_state.pool_authority = ctx.accounts.pool_authority.key();
        global_state.accumulated_yot = 0;
        global_state.cashback_bps = 500; // Default 5%
        global_state.liquidity_threshold = 100_000_000; // 0.1 YOT assuming 9 decimals
        global_state.weekly_apr_bps = 7000; // 365% APR / 52 weeks ~= 7%
        Ok(())
    }

    pub fn set_params(
        ctx: Context<SetParams>,
        cashback_bps: u64,
        liquidity_threshold: u64,
        weekly_apr_bps: u64,
    ) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        global_state.cashback_bps = cashback_bps;
        global_state.liquidity_threshold = liquidity_threshold;
        global_state.weekly_apr_bps = weekly_apr_bps;
        Ok(())
    }

    pub fn swap_now(ctx: Context<SwapAndDistribute>, yot_amount: u64) -> Result<()> {
        msg!("Start swap_now");

        let cashback_amount;
        let vault_amount;
        let user_receive;

        {
            let global_state_ref = &ctx.accounts.global_state;
            cashback_amount = yot_amount * global_state_ref.cashback_bps / 10_000;
            vault_amount = yot_amount * 20 / 100;
            user_receive = yot_amount - cashback_amount - vault_amount;
        }

        let transfer_user_ctx = ctx.accounts.transfer_to_user_context();
        token::transfer(transfer_user_ctx, user_receive)?;

        let transfer_vault_ctx = ctx.accounts.transfer_to_vault_context();
        token::transfer(transfer_vault_ctx, vault_amount)?;

        let mint_ctx = ctx.accounts.mint_yos_to_user_context();
        token::mint_to(mint_ctx, cashback_amount)?;

        ctx.accounts.global_state.accumulated_yot += vault_amount;

        msg!("End swap_now");
        Ok(())
    }

    pub fn add_liquidity_if_threshold(ctx: Context<AddLiquidityIfThreshold>) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;

        if global_state.accumulated_yot >= global_state.liquidity_threshold {
            msg!("Threshold met. Proceeding to add liquidity.");
            global_state.accumulated_yot = 0;
        } else {
            msg!("Threshold not met. Skipping liquidity addition.");
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Ping {}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = payer, space = 8 + 32 + 8 + 8 + 8 + 8)]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: Just storing as pubkey
    pub pool_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetParams<'info> {
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,
    pub authority: Signer<'info>,
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
        let cpi_accounts = Transfer {
            from: self.yot_source.to_account_info(),
            to: self.yot_user_dest.to_account_info(),
            authority: self.user.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn transfer_to_vault_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.yot_source.to_account_info(),
            to: self.yot_vault.to_account_info(),
            authority: self.user.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn mint_yos_to_user_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        let cpi_accounts = MintTo {
            mint: self.yos_mint.to_account_info(),
            to: self.yos_user_dest.to_account_info(),
            authority: self.global_state.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct AddLiquidityIfThreshold<'info> {
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,
}

#[account]
pub struct GlobalState {
    pub pool_authority: Pubkey,
    pub accumulated_yot: u64,
    pub cashback_bps: u64,
    pub liquidity_threshold: u64,
    pub weekly_apr_bps: u64,
}

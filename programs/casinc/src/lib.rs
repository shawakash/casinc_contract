use anchor_lang::prelude::*;

declare_id!("5V4zAh34TPQdrbXqBNbEbW7o9zknXceM8fVK5VK5HnM7");

#[program]
pub mod casinc {
    use super::*;

    // Initialize game parameters (admin only)
    pub fn initialize(
        ctx: Context<Initialize>,
        multiplier: u64,
        withdrawal_delay: i64,
        admins: Vec<Pubkey>,
        threshold: u8,
    ) -> Result<()> {
        ctx.accounts.game_params.multiplier = multiplier;
        ctx.accounts.game_params.withdrawal_delay = withdrawal_delay;
        ctx.accounts.game_params.admins = admins;
        ctx.accounts.game_params.threshold = threshold;
        ctx.accounts.game_params.bump = *ctx.bumps.get("game_params").unwrap();
        Ok(())
    }

    // Initialize user state
    pub fn initialize_user(ctx: Context<InitializeUser>) -> Result<()> {
        ctx.accounts.user_state.user = ctx.accounts.user.key();
        ctx.accounts.user_state.deposit = 0;
        ctx.accounts.user_state.winnings = 0;
        ctx.accounts.user_state.unlock_time = 0;
        ctx.accounts.user_state.bump = *ctx.bumps.get("user_state").unwrap();
        Ok(())
    }

    // Deposit funds into user's account
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.user_state.deposit += amount;
        Ok(())
    }

    // Place a bet and calculate winnings
    pub fn place_bet(ctx: Context<PlaceBet>, bet_amount: u64) -> Result<()> {
        require!(
            ctx.accounts.user_state.deposit >= bet_amount,
            CasincError::InsufficientFunds
        );
        ctx.accounts.user_state.deposit -= bet_amount;

        let winnings = bet_amount
            .checked_mul(ctx.accounts.game_params.multiplier)
            .unwrap();
        ctx.accounts.user_state.winnings += winnings;

        let clock = Clock::get()?;
        ctx.accounts.user_state.unlock_time =
            clock.unix_timestamp + ctx.accounts.game_params.withdrawal_delay;

        Ok(())
    }

    // Request withdrawal of winnings
    pub fn request_withdrawal(ctx: Context<RequestWithdrawal>, amount: u64) -> Result<()> {
        let clock = Clock::get()?;
        require!(
            clock.unix_timestamp >= ctx.accounts.user_state.unlock_time,
            CasincError::WithdrawalLocked
        );
        require!(
            ctx.accounts.user_state.winnings >= amount,
            CasincError::InsufficientWinnings
        );

        ctx.accounts.user_state.winnings -= amount;
        ctx.accounts.withdrawal_request.amount = amount;
        ctx.accounts.withdrawal_request.user = ctx.accounts.user.key();
        ctx.accounts.withdrawal_request.approved = false;
        ctx.accounts.withdrawal_request.bump = *ctx.bumps.get("withdrawal_request").unwrap();

        Ok(())
    }

    // Approve withdrawal request (admin multisig)
    pub fn approve_withdrawal(ctx: Context<ApproveWithdrawal>) -> Result<()> {
        let mut num_signers = 0;
        for admin in &ctx.accounts.game_params.admins {
            for acc in ctx.remaining_accounts.iter() {
                if acc.key() == *admin && acc.is_signer {
                    num_signers += 1;
                }
            }
        }
        require!(
            num_signers >= ctx.accounts.game_params.threshold,
            CasincError::NotEnoughSigners
        );

        ctx.accounts.withdrawal_request.approved = true;
        Ok(())
    }

    // Execute approved withdrawal
    pub fn execute_withdrawal(ctx: Context<ExecuteWithdrawal>) -> Result<()> {
        require!(
            ctx.accounts.withdrawal_request.approved,
            CasincError::WithdrawalNotApproved
        );

        let amount = ctx.accounts.withdrawal_request.amount;
        **ctx
            .accounts
            .user
            .to_account_info()
            .try_borrow_mut_lamports()? += amount;
        **ctx
            .accounts
            .withdrawal_request
            .to_account_info()
            .try_borrow_mut_lamports()? -= amount;

        Ok(())
    }

    pub fn advance_clock(_ctx: Context<AdvanceClock>, _seconds: u64) -> Result<()> {
        // This is a mock function for testing environments
        Ok(())
    }
}

// Data Structures
#[account]
pub struct GameParameters {
    pub multiplier: u64,
    pub withdrawal_delay: i64,
    pub admins: Vec<Pubkey>,
    pub threshold: u8,
    pub bump: u8,
}

#[account]
pub struct UserState {
    user: Pubkey,
    pub deposit: u64,
    pub winnings: u64,
    pub unlock_time: i64,
    pub bump: u8,
}

#[account]
pub struct WithdrawalRequest {
    pub user: Pubkey,
    pub amount: u64,
    pub approved: bool,
    pub bump: u8,
}

// Contexts
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = admin, space = 8 + 8 + 8 + 4 + 32*10 + 1 + 1, seeds = [b"game_params"], bump)]
    pub game_params: Account<'info, GameParameters>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeUser<'info> {
    #[account(init, payer = user, space = 8 + 8 + 8 + 8 + 1, seeds = [b"user_state", user.key().as_ref()], bump)]
    pub user_state: Account<'info, UserState>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut, has_one = user)]
    pub user_state: Account<'info, UserState>,
    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct PlaceBet<'info> {
    #[account(mut)]
    pub user_state: Account<'info, UserState>,
    pub game_params: Account<'info, GameParameters>,
}

#[derive(Accounts)]
pub struct RequestWithdrawal<'info> {
    #[account(mut)]
    pub user_state: Account<'info, UserState>,
    #[account(init, payer = user, space = 8 + 32 + 8 + 1 + 1, seeds = [b"withdrawal_request", user.key().as_ref()], bump)]
    pub withdrawal_request: Account<'info, WithdrawalRequest>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ApproveWithdrawal<'info> {
    #[account(mut)]
    pub withdrawal_request: Account<'info, WithdrawalRequest>,
    pub game_params: Account<'info, GameParameters>,
}

#[derive(Accounts)]
pub struct ExecuteWithdrawal<'info> {
    #[account(mut, close = user)]
    pub withdrawal_request: Account<'info, WithdrawalRequest>,
    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct AdvanceClock<'info> {
    pub clock: Sysvar<'info, Clock>,
}

// Error Handling
#[error_code]
pub enum CasincError {
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Withdrawal is still locked")]
    WithdrawalLocked,
    #[msg("Not enough admin signers")]
    NotEnoughSigners,
    #[msg("Withdrawal not approved")]
    WithdrawalNotApproved,
    #[msg("Insufficient winnings")]
    InsufficientWinnings,
}

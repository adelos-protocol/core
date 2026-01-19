use anchor_lang::prelude::*;

declare_id!("7T1UxHJ6psKiQheKZXxANu6mhgsmgaX55eNKZZL5u4Rp");

const DISCRIMINATOR_SIZE: usize = 8;
const REGISTRY_ACCOUNT_SIZE: usize = DISCRIMINATOR_SIZE + 32 + 32 + 1;

#[program]
pub mod adelos_registry {
    use super::*;

    pub fn register_identity(ctx: Context<RegisterIdentity>, meta_pubkey: [u8; 32]) -> Result<()> {
        require!(meta_pubkey != [0u8; 32], AdelosError::InvalidMetaPubkey);

        let registry = &mut ctx.accounts.registry;
        registry.owner = ctx.accounts.owner.key();
        registry.meta_pubkey = meta_pubkey;
        registry.bump = ctx.bumps.registry;

        msg!("Identity registered for {}", registry.owner);
        Ok(())
    }

    pub fn update_identity(ctx: Context<UpdateIdentity>, new_meta_pubkey: [u8; 32]) -> Result<()> {
        require!(new_meta_pubkey != [0u8; 32], AdelosError::InvalidMetaPubkey);
        let registry = &mut ctx.accounts.registry;
        registry.meta_pubkey = new_meta_pubkey;
        Ok(())
    }

    pub fn close_registry(_ctx: Context<CloseRegistry>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct RegisterIdentity<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init,
        payer = owner,
        space = REGISTRY_ACCOUNT_SIZE,
        seeds = [b"registry", owner.key().as_ref()],
        bump
    )]
    pub registry: Account<'info, RegistryAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateIdentity<'info> {
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [b"registry", owner.key().as_ref()],
        bump = registry.bump,
        has_one = owner @ AdelosError::Unauthorized
    )]
    pub registry: Account<'info, RegistryAccount>,
}

#[derive(Accounts)]
pub struct CloseRegistry<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [b"registry", owner.key().as_ref()],
        bump = registry.bump,
        has_one = owner @ AdelosError::Unauthorized,
        close = owner
    )]
    pub registry: Account<'info, RegistryAccount>,
}

#[account]
pub struct RegistryAccount {
    pub owner: Pubkey,
    pub meta_pubkey: [u8; 32],
    pub bump: u8,
}

#[error_code]
pub enum AdelosError {
    #[msg("Invalid meta_pubkey")]
    InvalidMetaPubkey,
    #[msg("Unauthorized")]
    Unauthorized,
}

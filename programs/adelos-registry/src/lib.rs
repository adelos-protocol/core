//! Adelos Registry Program
//! 
//! A privacy-first registry for storing stealth address metadata on Solana.
//! This program enables Unlinkability through single-key stealth addresses
//! by storing meta_pubkey values that recipients publish for senders to derive
//! unique stealth addresses.
//!
//! ## Architecture
//! 
//! - **PDA Seeds**: `[b"registry", owner_pubkey]`
//! - **Account Size**: 65 bytes (optimized for rent efficiency)
//!   - owner: 32 bytes (Pubkey)
//!   - meta_pubkey: 32 bytes ([u8; 32])
//!   - bump: 1 byte (u8)
//!
//! ## Instructions
//! 
//! - `register_identity`: Create a new registry entry with meta_pubkey
//! - `update_identity`: Update an existing meta_pubkey
//! - `close_registry`: Close account and reclaim rent
//!
//! ## Privacy Flow
//! 
//! 1. Recipient registers their `meta_pubkey` (derived from their private meta_sk)
//! 2. Sender fetches recipient's `meta_pubkey` from this registry
//! 3. Sender generates ephemeral keypair and derives stealth address
//! 4. Sender sends funds to stealth address with ephemeral_pubkey in memo
//! 5. Recipient scans memos, recovers shared secret, and derives stealth_sk

use anchor_lang::prelude::*;

declare_id!("AdeL5RWqQJLDd33WPEggNJxuFmYa5oLXQHYu6Jv3Svae");

/// Discriminator size for Anchor accounts (8 bytes)
const DISCRIMINATOR_SIZE: usize = 8;

/// Registry account size calculation:
/// - discriminator: 8 bytes
/// - owner: 32 bytes (Pubkey)
/// - meta_pubkey: 32 bytes ([u8; 32])
/// - bump: 1 byte (u8)
/// Total: 73 bytes
const REGISTRY_ACCOUNT_SIZE: usize = DISCRIMINATOR_SIZE + 32 + 32 + 1;

#[program]
pub mod adelos_registry {
    use super::*;

    /// Register a new identity with a meta_pubkey for stealth address derivation.
    ///
    /// This instruction creates a PDA that stores the caller's meta_pubkey,
    /// which other users can fetch to derive stealth addresses for private payments.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context containing all accounts required for registration
    /// * `meta_pubkey` - The 32-byte public key point used for ECDH shared secret derivation
    ///
    /// # Errors
    ///
    /// * `InvalidMetaPubkey` - If the meta_pubkey is all zeros (invalid point)
    pub fn register_identity(ctx: Context<RegisterIdentity>, meta_pubkey: [u8; 32]) -> Result<()> {
        // Validate meta_pubkey is not a zero point (invalid for curve operations)
        require!(
            meta_pubkey != [0u8; 32],
            AdelosError::InvalidMetaPubkey
        );

        let registry = &mut ctx.accounts.registry;
        registry.owner = ctx.accounts.owner.key();
        registry.meta_pubkey = meta_pubkey;
        registry.bump = ctx.bumps.registry;

        emit!(IdentityRegistered {
            owner: registry.owner,
            meta_pubkey: registry.meta_pubkey,
            timestamp: Clock::get()?.unix_timestamp,
        });

        msg!(
            "Adelos: Identity registered for {} with meta_pubkey {:?}",
            registry.owner,
            &registry.meta_pubkey[..8]
        );

        Ok(())
    }

    /// Update an existing identity's meta_pubkey.
    ///
    /// This allows users to rotate their meta_pubkey for enhanced privacy
    /// or if their previous key was compromised.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context containing the registry account to update
    /// * `new_meta_pubkey` - The new 32-byte public key point
    ///
    /// # Errors
    ///
    /// * `InvalidMetaPubkey` - If the new meta_pubkey is all zeros
    /// * `Unauthorized` - If the signer is not the registry owner
    pub fn update_identity(ctx: Context<UpdateIdentity>, new_meta_pubkey: [u8; 32]) -> Result<()> {
        // Validate new meta_pubkey is not a zero point
        require!(
            new_meta_pubkey != [0u8; 32],
            AdelosError::InvalidMetaPubkey
        );

        let registry = &mut ctx.accounts.registry;
        let old_meta_pubkey = registry.meta_pubkey;
        registry.meta_pubkey = new_meta_pubkey;

        emit!(IdentityUpdated {
            owner: registry.owner,
            old_meta_pubkey,
            new_meta_pubkey: registry.meta_pubkey,
            timestamp: Clock::get()?.unix_timestamp,
        });

        msg!(
            "Adelos: Identity updated for {} - old: {:?}, new: {:?}",
            registry.owner,
            &old_meta_pubkey[..8],
            &registry.meta_pubkey[..8]
        );

        Ok(())
    }

    /// Close a registry account and reclaim rent.
    ///
    /// This allows users to deregister their identity and recover the
    /// SOL deposited for rent exemption.
    pub fn close_registry(ctx: Context<CloseRegistry>) -> Result<()> {
        emit!(IdentityClosed {
            owner: ctx.accounts.registry.owner,
            timestamp: Clock::get()?.unix_timestamp,
        });

        msg!(
            "Adelos: Registry closed for {}",
            ctx.accounts.registry.owner
        );

        Ok(())
    }
}

// ============================================================================
// Account Contexts
// ============================================================================

/// Accounts required for registering a new identity.
#[derive(Accounts)]
pub struct RegisterIdentity<'info> {
    /// The wallet registering their identity. This becomes the owner of the registry.
    #[account(mut)]
    pub owner: Signer<'info>,

    /// The registry PDA account to be created.
    /// Seeds: ["registry", owner_pubkey]
    #[account(
        init,
        payer = owner,
        space = REGISTRY_ACCOUNT_SIZE,
        seeds = [b"registry", owner.key().as_ref()],
        bump
    )]
    pub registry: Account<'info, RegistryAccount>,

    /// System program for account creation.
    pub system_program: Program<'info, System>,
}

/// Accounts required for updating an existing identity.
#[derive(Accounts)]
pub struct UpdateIdentity<'info> {
    /// The wallet that owns this registry. Must be the signer.
    pub owner: Signer<'info>,

    /// The registry PDA account to update.
    #[account(
        mut,
        seeds = [b"registry", owner.key().as_ref()],
        bump = registry.bump,
        has_one = owner @ AdelosError::Unauthorized
    )]
    pub registry: Account<'info, RegistryAccount>,
}

/// Accounts required for closing a registry account.
#[derive(Accounts)]
pub struct CloseRegistry<'info> {
    /// The wallet that owns this registry.
    #[account(mut)]
    pub owner: Signer<'info>,

    /// The registry PDA account to close.
    #[account(
        mut,
        seeds = [b"registry", owner.key().as_ref()],
        bump = registry.bump,
        has_one = owner @ AdelosError::Unauthorized,
        close = owner
    )]
    pub registry: Account<'info, RegistryAccount>,
}

// ============================================================================
// Account Definitions
// ============================================================================

/// Registry account storing stealth address metadata for a user.
///
/// ## Fields
///
/// - `owner`: The wallet that created and controls this registry
/// - `meta_pubkey`: The Ed25519 public point used in ECDH derivation
/// - `bump`: The PDA bump seed for address derivation
///
/// ## Size: 65 bytes (excluding 8-byte discriminator)
#[account]
#[derive(Default, Debug)]
pub struct RegistryAccount {
    /// The wallet that owns this registry entry. Immutable after creation.
    pub owner: Pubkey,

    /// The meta_pubkey used for deriving stealth addresses.
    /// This is a 32-byte Ed25519 public key point.
    /// Stealth formula: stealth_pubkey = meta_pubkey + hash(shared_secret) * G
    pub meta_pubkey: [u8; 32],

    /// The PDA bump seed for this account.
    pub bump: u8,
}

// ============================================================================
// Events
// ============================================================================

/// Emitted when a new identity is registered.
#[event]
pub struct IdentityRegistered {
    pub owner: Pubkey,
    pub meta_pubkey: [u8; 32],
    pub timestamp: i64,
}

/// Emitted when an identity's meta_pubkey is updated.
#[event]
pub struct IdentityUpdated {
    pub owner: Pubkey,
    pub old_meta_pubkey: [u8; 32],
    pub new_meta_pubkey: [u8; 32],
    pub timestamp: i64,
}

/// Emitted when a registry account is closed.
#[event]
pub struct IdentityClosed {
    pub owner: Pubkey,
    pub timestamp: i64,
}

// ============================================================================
// Errors
// ============================================================================

/// Custom errors for the Adelos Registry program.
#[error_code]
pub enum AdelosError {
    /// The provided meta_pubkey is invalid (e.g., all zeros).
    #[msg("Invalid meta_pubkey: cannot be zero bytes")]
    InvalidMetaPubkey,

    /// The signer is not authorized to perform this action.
    #[msg("Unauthorized: only the registry owner can perform this action")]
    Unauthorized,

    /// The registry account does not exist.
    #[msg("Registry not found for the given owner")]
    RegistryNotFound,
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Derives the PDA address for a registry account.
pub fn derive_registry_pda(owner: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"registry", owner.as_ref()], program_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_account_size() {
        assert_eq!(REGISTRY_ACCOUNT_SIZE, 73);
    }

    #[test]
    fn test_derive_registry_pda() {
        let owner = Pubkey::new_unique();
        let (pda, bump) = derive_registry_pda(&owner, &crate::ID);
        
        assert!(pda != Pubkey::default());
        assert!(bump <= 255);
        
        let (pda2, bump2) = derive_registry_pda(&owner, &crate::ID);
        assert_eq!(pda, pda2);
        assert_eq!(bump, bump2);
    }
}

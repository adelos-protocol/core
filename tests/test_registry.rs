use litesvm::LiteSVM;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};
use borsh::BorshDeserialize;
use sha2::{Sha256, Digest};
use std::path::PathBuf;

// Program ID from lib.rs
const PROGRAM_ID: Pubkey = solana_sdk::pubkey!("AdeL5RWqQJLDd33WPEggNJxuFmYa5oLXQHYu6Jv3Svae");

// The Registry account structure matching lib.rs
#[derive(Debug, BorshDeserialize)]
struct RegistryAccount {
    pub owner: Pubkey,
    pub meta_pubkey: [u8; 32],
    pub bump: u8,
}

// Helper function to calculate instruction discriminator
fn get_discriminator(instruction_name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(format!("global:{}", instruction_name));
    let result = hasher.finalize();
    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&result[..8]);
    discriminator
}

// Helper to derive the Registry PDA
fn get_registry_pda(owner: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"registry", owner.as_ref()],
        program_id,
    )
}

// Helper to load program binary
fn load_program_bytes() -> Vec<u8> {
    let program_path = std::env::var("CARGO_TARGET_DIR")
        .map(|dir| PathBuf::from(dir).join("deploy/adelos_registry.so"))
        .unwrap_or_else(|_| PathBuf::from("target/deploy/adelos_registry.so"));
    
    std::fs::read(&program_path)
        .expect(&format!("Failed to read program at {:?}. Run `anchor build` first.", program_path))
}

// Helper to build register_identity instruction
fn build_register_instruction(
    program_id: &Pubkey,
    owner: &Pubkey,
    meta_pubkey: [u8; 32],
) -> Instruction {
    let (registry_pda, _bump) = get_registry_pda(owner, program_id);

    let discriminator = get_discriminator("register_identity");
    let mut instruction_data = Vec::new();
    instruction_data.extend_from_slice(&discriminator);
    instruction_data.extend_from_slice(&meta_pubkey);

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*owner, true),
            AccountMeta::new(registry_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: instruction_data,
    }
}

// Helper to build update_identity instruction
fn build_update_instruction(
    program_id: &Pubkey,
    owner: &Pubkey,
    new_meta_pubkey: [u8; 32],
) -> Instruction {
    let (registry_pda, _bump) = get_registry_pda(owner, program_id);

    let discriminator = get_discriminator("update_identity");
    let mut instruction_data = Vec::new();
    instruction_data.extend_from_slice(&discriminator);
    instruction_data.extend_from_slice(&new_meta_pubkey);

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new_readonly(*owner, true),
            AccountMeta::new(registry_pda, false),
        ],
        data: instruction_data,
    }
}

// Helper to build close_registry instruction
fn build_close_instruction(
    program_id: &Pubkey,
    owner: &Pubkey,
) -> Instruction {
    let (registry_pda, _bump) = get_registry_pda(owner, program_id);

    let discriminator = get_discriminator("close_registry");

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*owner, true),
            AccountMeta::new(registry_pda, false),
        ],
        data: discriminator.to_vec(),
    }
}

#[test]
fn test_register_identity() {
    // Initialize LiteSVM
    let mut svm = LiteSVM::new();

    // Load program
    let program_bytes = load_program_bytes();
    svm.add_program(PROGRAM_ID, &program_bytes);

    // Create and fund owner
    let owner = Keypair::new();
    svm.airdrop(&owner.pubkey(), 10_000_000_000).unwrap();

    // Test data
    let meta_pubkey = [1u8; 32];
    let (registry_pda, _bump) = get_registry_pda(&owner.pubkey(), &PROGRAM_ID);

    // Build and send transaction
    let register_ix = build_register_instruction(&PROGRAM_ID, &owner.pubkey(), meta_pubkey);
    let tx = Transaction::new_signed_with_payer(
        &[register_ix],
        Some(&owner.pubkey()),
        &[&owner],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);
    assert!(result.is_ok(), "Register failed: {:?}", result.err());

    // Verify account
    let account = svm.get_account(&registry_pda)
        .expect("Registry should exist");

    // Skip 8-byte discriminator
    let registry = RegistryAccount::deserialize(&mut &account.data[8..])
        .expect("Failed to deserialize");

    assert_eq!(registry.owner, owner.pubkey());
    assert_eq!(registry.meta_pubkey, meta_pubkey);

    println!("✅ Register test passed!");
}

#[test]
fn test_update_identity() {
    let mut svm = LiteSVM::new();
    let program_bytes = load_program_bytes();
    svm.add_program(PROGRAM_ID, &program_bytes);

    let owner = Keypair::new();
    svm.airdrop(&owner.pubkey(), 10_000_000_000).unwrap();

    let original_meta = [1u8; 32];
    let updated_meta = [2u8; 32];
    let (registry_pda, _) = get_registry_pda(&owner.pubkey(), &PROGRAM_ID);

    // First register
    let register_ix = build_register_instruction(&PROGRAM_ID, &owner.pubkey(), original_meta);
    let tx = Transaction::new_signed_with_payer(
        &[register_ix],
        Some(&owner.pubkey()),
        &[&owner],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    // Now update
    let update_ix = build_update_instruction(&PROGRAM_ID, &owner.pubkey(), updated_meta);
    let tx = Transaction::new_signed_with_payer(
        &[update_ix],
        Some(&owner.pubkey()),
        &[&owner],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);
    assert!(result.is_ok(), "Update failed: {:?}", result.err());

    // Verify update
    let account = svm.get_account(&registry_pda).unwrap();
    let registry = RegistryAccount::deserialize(&mut &account.data[8..]).unwrap();
    assert_eq!(registry.meta_pubkey, updated_meta);

    println!("✅ Update test passed!");
}

#[test]
fn test_close_registry() {
    let mut svm = LiteSVM::new();
    let program_bytes = load_program_bytes();
    svm.add_program(PROGRAM_ID, &program_bytes);

    let owner = Keypair::new();
    svm.airdrop(&owner.pubkey(), 10_000_000_000).unwrap();

    let meta_pubkey = [1u8; 32];
    let (registry_pda, _) = get_registry_pda(&owner.pubkey(), &PROGRAM_ID);

    // Register first
    let register_ix = build_register_instruction(&PROGRAM_ID, &owner.pubkey(), meta_pubkey);
    let tx = Transaction::new_signed_with_payer(
        &[register_ix],
        Some(&owner.pubkey()),
        &[&owner],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    // Get balance before close
    let balance_before = svm.get_balance(&owner.pubkey()).unwrap();

    // Close registry
    let close_ix = build_close_instruction(&PROGRAM_ID, &owner.pubkey());
    let tx = Transaction::new_signed_with_payer(
        &[close_ix],
        Some(&owner.pubkey()),
        &[&owner],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);
    assert!(result.is_ok(), "Close failed: {:?}", result.err());

    // Verify closed
    let account = svm.get_account(&registry_pda);
    match account {
        Some(acc) => assert_eq!(acc.lamports, 0, "Should be closed"),
        None => {}
    }

    // Verify rent returned
    let balance_after = svm.get_balance(&owner.pubkey()).unwrap();
    assert!(balance_after > balance_before, "Should receive rent back");

    println!("✅ Close test passed!");
}

#[test]
fn test_full_lifecycle() {
    let mut svm = LiteSVM::new();
    let program_bytes = load_program_bytes();
    svm.add_program(PROGRAM_ID, &program_bytes);

    let owner = Keypair::new();
    svm.airdrop(&owner.pubkey(), 10_000_000_000).unwrap();

    let (registry_pda, _) = get_registry_pda(&owner.pubkey(), &PROGRAM_ID);

    // 1. REGISTER
    println!("Step 1: Register identity...");
    let meta_v1 = [1u8; 32];
    let ix = build_register_instruction(&PROGRAM_ID, &owner.pubkey(), meta_v1);
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&owner.pubkey()), &[&owner], svm.latest_blockhash());
    svm.send_transaction(tx).expect("Register failed");

    // 2. VERIFY CREATION
    println!("Step 2: Verify creation...");
    let acc = svm.get_account(&registry_pda).expect("Should exist");
    let reg = RegistryAccount::deserialize(&mut &acc.data[8..]).unwrap();
    assert_eq!(reg.meta_pubkey, meta_v1);

    // 3. UPDATE
    println!("Step 3: Update identity...");
    let meta_v2 = [2u8; 32];
    let ix = build_update_instruction(&PROGRAM_ID, &owner.pubkey(), meta_v2);
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&owner.pubkey()), &[&owner], svm.latest_blockhash());
    svm.send_transaction(tx).expect("Update failed");

    // 4. VERIFY UPDATE
    println!("Step 4: Verify update...");
    let acc = svm.get_account(&registry_pda).unwrap();
    let reg = RegistryAccount::deserialize(&mut &acc.data[8..]).unwrap();
    assert_eq!(reg.meta_pubkey, meta_v2);

    // 5. CLOSE
    println!("Step 5: Close registry...");
    let ix = build_close_instruction(&PROGRAM_ID, &owner.pubkey());
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&owner.pubkey()), &[&owner], svm.latest_blockhash());
    svm.send_transaction(tx).expect("Close failed");

    // 6. VERIFY CLOSED
    println!("Step 6: Verify closed...");
    let acc = svm.get_account(&registry_pda);
    assert!(acc.is_none() || acc.unwrap().lamports == 0);

    println!("✅ Full lifecycle test passed!");
}

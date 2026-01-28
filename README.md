# Adelos Registry - Anchor Smart Contract

Solana smart contract for managing identity registry in the Adelos Protocol.

## 📋 Overview

The Adelos Registry stores user identities with their meta public keys. This enables senders to derive unique stealth addresses for private transfers.

## 🏗️ Program Structure

```rust
#[account]
pub struct RegistryAccount {
    pub owner: Pubkey,          // Owner's wallet address
    pub meta_pubkey: [u8; 32],  // Ed25519 meta public key
    pub bump: u8,               // PDA bump seed
}
```

**Space**: 8 (discriminator) + 32 (owner) + 32 (meta_pubkey) + 1 (bump) = **73 bytes**

## 📝 Instructions

| Instruction | Description | Signer |
|-------------|-------------|--------|
| `register_identity` | Create new registry with meta_pubkey | Owner |
| `update_identity` | Update existing meta_pubkey | Owner |
| `close_registry` | Close registry and reclaim rent | Owner |

### Register Identity

```typescript
// Create and send register transaction
const tx = await sdk.createRegisterTransaction(ownerPubkey, metaPubkey);
const signedTx = await wallet.signTransaction(tx);
await sdk.sendAndConfirm(signedTx);
```

### Update Identity

```typescript
// Update meta pubkey (e.g., after wallet recovery)
const tx = await sdk.createUpdateTransaction(ownerPubkey, newMetaPubkey);
const signedTx = await wallet.signTransaction(tx);
await sdk.sendAndConfirm(signedTx);
```

### Close Registry

```typescript
// Close registry and reclaim rent (~0.002 SOL)
const tx = await sdk.createCloseTransaction(ownerPubkey);
const signedTx = await wallet.signTransaction(tx);
await sdk.sendAndConfirm(signedTx);
```

## 🔑 Program ID

| Network | Program ID |
|---------|------------|
| Devnet | `7T1UxHJ6psKiQheKZXxANu6mhgsmgaX55eNKZZL5u4Rp` |

## 🛠️ Development

### Prerequisites

- Rust 1.70+
- Solana CLI 1.18+
- Anchor 0.32+

### Build

```bash
anchor build
```

### Test

```bash
anchor test
```

### Deploy to Devnet

```bash
solana config set --url devnet
anchor deploy --provider.cluster devnet
```

### Verify Deployment

```bash
solana program show 7T1UxHJ6psKiQheKZXxANu6mhgsmgaX55eNKZZL5u4Rp
```

## 📁 Files

```
anchor/
├── programs/adelos-registry/
│   └── src/lib.rs            # Main program logic
├── target/
│   ├── idl/                  # Generated IDL
│   └── types/                # Generated TypeScript types
├── client/
│   └── scripts/              # Test scripts
├── Anchor.toml               # Anchor configuration
└── README.md
```

## 🔒 Security

- PDA seeds: `["registry", owner.as_ref()]`
- Only owner can modify or close their registry
- Security.txt embedded in binary
- Contact: albary6700@gmail.com

## 📄 License

MIT

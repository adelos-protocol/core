import * as anchor from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import * as fs from "fs";
import * as os from "os";

// Program ID (deployed on devnet)
const PROGRAM_ID = new PublicKey("7T1UxHJ6psKiQheKZXxANu6mhgsmgaX55eNKZZL5u4Rp");

// Load wallet from default Solana config
const HOME = os.homedir();
const keypairPath = `${HOME}/.config/solana/id.json`;
const secretKey = JSON.parse(fs.readFileSync(keypairPath, "utf-8"));
const wallet = Keypair.fromSecretKey(Uint8Array.from(secretKey));

// Connection to devnet
const connection = new Connection("https://api.devnet.solana.com", "confirmed");

// IDL with address for Anchor 0.32
const IDL = {
  address: "7T1UxHJ6psKiQheKZXxANu6mhgsmgaX55eNKZZL5u4Rp",
  metadata: { name: "adelos_registry", version: "0.1.0", spec: "0.1.0" },
  instructions: [
    {
      name: "register_identity",
      discriminator: [164, 118, 227, 177, 47, 176, 187, 248],
      accounts: [
        { name: "owner", writable: true, signer: true },
        { name: "registry", writable: true },
        { name: "system_program", address: "11111111111111111111111111111111" },
      ],
      args: [{ name: "meta_pubkey", type: { array: ["u8", 32] } }],
    },
    {
      name: "update_identity",
      discriminator: [130, 54, 88, 104, 222, 124, 238, 252],
      accounts: [
        { name: "owner", signer: true },
        { name: "registry", writable: true },
      ],
      args: [{ name: "new_meta_pubkey", type: { array: ["u8", 32] } }],
    },
    {
      name: "close_registry",
      discriminator: [76, 32, 154, 180, 51, 159, 218, 102],
      accounts: [
        { name: "owner", writable: true, signer: true },
        { name: "registry", writable: true },
      ],
      args: [],
    },
  ],
  accounts: [
    {
      name: "RegistryAccount",
      discriminator: [113, 93, 106, 201, 100, 166, 146, 98],
    },
  ],
  types: [
    {
      name: "RegistryAccount",
      type: {
        kind: "struct",
        fields: [
          { name: "owner", type: "pubkey" },
          { name: "meta_pubkey", type: { array: ["u8", 32] } },
          { name: "bump", type: "u8" },
        ],
      },
    },
  ],
};

async function main() {
  console.log("🔑 Wallet:", wallet.publicKey.toBase58());

  // Check balance
  const balance = await connection.getBalance(wallet.publicKey);
  console.log("💰 Balance:", balance / 1e9, "SOL");

  if (balance < 0.01 * 1e9) {
    console.log("⚠️ Low balance! Run: solana airdrop 1 --url devnet");
    return;
  }

  // Derive PDA
  const [registryPda, bump] = PublicKey.findProgramAddressSync(
    [Buffer.from("registry"), wallet.publicKey.toBuffer()],
    PROGRAM_ID
  );
  console.log("📍 Registry PDA:", registryPda.toBase58());

  // Create provider
  const provider = new anchor.AnchorProvider(
    connection,
    new anchor.Wallet(wallet),
    { commitment: "confirmed" }
  );

  // Create program instance (Anchor 0.32 API)
  const program = new anchor.Program(IDL as any, provider);

  // Generate random meta_pubkey (32 bytes)
  const metaPubkey = Keypair.generate().publicKey.toBytes();
  console.log(
    "🔐 Meta Pubkey (first 8 bytes):",
    Buffer.from(metaPubkey.slice(0, 8)).toString("hex")
  );

  try {
    // Call register_identity
    const tx = await program.methods
      .registerIdentity(Array.from(metaPubkey))
      .accounts({
        owner: wallet.publicKey,
        registry: registryPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([wallet])
      .rpc();

    console.log("✅ Transaction successful!");
    console.log("📝 Signature:", tx);
    console.log("🔗 Explorer: https://solscan.io/tx/" + tx + "?cluster=devnet");
  } catch (error: any) {
    if (error.message?.includes("already in use")) {
      console.log("⚠️ Registry already exists for this wallet");
    } else {
      console.error("❌ Error:", error.message || error);
    }
  }
}

main().catch(console.error);

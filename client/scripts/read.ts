import { Connection, PublicKey } from "@solana/web3.js";
import fs from "fs";

// Program ID
const PROGRAM_ID = new PublicKey("7T1UxHJ6psKiQheKZXxANu6mhgsmgaX55eNKZZL5u4Rp");

// Load wallet
const HOME = process.env.HOME || process.env.USERPROFILE;
const keypairPath = `${HOME}/.config/solana/id.json`;
const secretKey = JSON.parse(fs.readFileSync(keypairPath, "utf-8"));
const ownerPubkey = PublicKey.decode(Buffer.from(secretKey.slice(32)));

// Connection
const connection = new Connection("https://api.devnet.solana.com", "confirmed");

async function main() {
  // Get owner from args or default wallet
  const ownerArg = process.argv[2];
  const owner = ownerArg ? new PublicKey(ownerArg) : new PublicKey(secretKey.slice(32, 64));

  console.log("👤 Owner:", owner.toBase58());

  // Derive PDA
  const [registryPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("registry"), owner.toBuffer()],
    PROGRAM_ID
  );
  console.log("📍 Registry PDA:", registryPda.toBase58());

  try {
    // Fetch account
    const accountInfo = await connection.getAccountInfo(registryPda);

    if (!accountInfo) {
      console.log("❌ No registry found for this owner");
      return;
    }

    console.log("\n📦 Account Data:");
    console.log("   Lamports:", accountInfo.lamports);
    console.log("   Data length:", accountInfo.data.length, "bytes");
    console.log("   Owner program:", accountInfo.owner.toBase58());

    // Parse account data (skip 8-byte discriminator)
    const data = accountInfo.data.slice(8);

    // Owner (32 bytes)
    const storedOwner = new PublicKey(data.slice(0, 32));
    console.log("\n🔑 Stored Owner:", storedOwner.toBase58());

    // Meta Pubkey (32 bytes)
    const metaPubkey = data.slice(32, 64);
    console.log("🔐 Meta Pubkey:", Buffer.from(metaPubkey).toString("hex"));

    // Bump (1 byte)
    const bump = data[64];
    console.log("📊 Bump:", bump);

  } catch (error: any) {
    console.error("❌ Error:", error.message || error);
  }
}

main().catch(console.error);

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";

describe("adelos-registry", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Program ID from Anchor.toml
  const programId = new PublicKey("AdeL5RWqQJLDd33WPEggNJxuFmYa5oLXQHYu6Jv3Svae");

  // Test accounts
  const owner = provider.wallet;
  let registryPda: PublicKey;
  let registryBump: number;

  // Test meta_pubkey (32 random bytes)
  const metaPubkey = Keypair.generate().publicKey.toBytes();
  const newMetaPubkey = Keypair.generate().publicKey.toBytes();

  before(async () => {
    // Derive registry PDA
    [registryPda, registryBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("registry"), owner.publicKey.toBuffer()],
      programId
    );
    console.log("Registry PDA:", registryPda.toBase58());
    console.log("Registry Bump:", registryBump);
  });

  describe("register_identity", () => {
    it("should register a new identity", async () => {
      // Build register instruction
      // Discriminator: sha256("global:register_identity")[0..8]
      const discriminator = Buffer.from([175, 134, 166, 216, 201, 46, 235, 126]);
      const data = Buffer.concat([discriminator, Buffer.from(metaPubkey)]);

      const ix = new anchor.web3.TransactionInstruction({
        keys: [
          { pubkey: owner.publicKey, isSigner: true, isWritable: true },
          { pubkey: registryPda, isSigner: false, isWritable: true },
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        programId,
        data,
      });

      const tx = new anchor.web3.Transaction().add(ix);
      const sig = await provider.sendAndConfirm(tx);
      console.log("Register tx:", sig);

      // Fetch and verify account
      const accountInfo = await provider.connection.getAccountInfo(registryPda);
      expect(accountInfo).to.not.be.null;
      expect(accountInfo!.data.length).to.equal(73); // 8 + 32 + 32 + 1

      // Parse account data
      const accountData = accountInfo!.data;
      const ownerBytes = accountData.slice(8, 40);
      const metaPubkeyBytes = accountData.slice(40, 72);
      const bump = accountData[72];

      expect(new PublicKey(ownerBytes).toBase58()).to.equal(owner.publicKey.toBase58());
      expect(Buffer.from(metaPubkeyBytes)).to.deep.equal(Buffer.from(metaPubkey));
      expect(bump).to.equal(registryBump);
    });

    it("should fail when registering with same owner twice", async () => {
      const discriminator = Buffer.from([175, 134, 166, 216, 201, 46, 235, 126]);
      const data = Buffer.concat([discriminator, Buffer.from(metaPubkey)]);

      const ix = new anchor.web3.TransactionInstruction({
        keys: [
          { pubkey: owner.publicKey, isSigner: true, isWritable: true },
          { pubkey: registryPda, isSigner: false, isWritable: true },
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        programId,
        data,
      });

      const tx = new anchor.web3.Transaction().add(ix);

      try {
        await provider.sendAndConfirm(tx);
        expect.fail("Should have thrown an error");
      } catch (err: any) {
        expect(err.toString()).to.include("already in use");
      }
    });
  });

  describe("update_identity", () => {
    it("should update meta_pubkey", async () => {
      // Discriminator for 'update_identity'
      const discriminator = Buffer.from([208, 184, 166, 170, 104, 75, 31, 110]);
      const data = Buffer.concat([discriminator, Buffer.from(newMetaPubkey)]);

      const ix = new anchor.web3.TransactionInstruction({
        keys: [
          { pubkey: owner.publicKey, isSigner: true, isWritable: false },
          { pubkey: registryPda, isSigner: false, isWritable: true },
        ],
        programId,
        data,
      });

      const tx = new anchor.web3.Transaction().add(ix);
      const sig = await provider.sendAndConfirm(tx);
      console.log("Update tx:", sig);

      // Verify update
      const accountInfo = await provider.connection.getAccountInfo(registryPda);
      const metaPubkeyBytes = accountInfo!.data.slice(40, 72);
      expect(Buffer.from(metaPubkeyBytes)).to.deep.equal(Buffer.from(newMetaPubkey));
    });
  });

  describe("close_registry", () => {
    it("should close registry and return rent", async () => {
      const balanceBefore = await provider.connection.getBalance(owner.publicKey);

      // Discriminator for 'close_registry'
      const discriminator = Buffer.from([174, 156, 240, 194, 101, 126, 33, 46]);

      const ix = new anchor.web3.TransactionInstruction({
        keys: [
          { pubkey: owner.publicKey, isSigner: true, isWritable: true },
          { pubkey: registryPda, isSigner: false, isWritable: true },
        ],
        programId,
        data: discriminator,
      });

      const tx = new anchor.web3.Transaction().add(ix);
      const sig = await provider.sendAndConfirm(tx);
      console.log("Close tx:", sig);

      // Verify account is closed
      const accountInfo = await provider.connection.getAccountInfo(registryPda);
      expect(accountInfo).to.be.null;

      // Verify rent was returned
      const balanceAfter = await provider.connection.getBalance(owner.publicKey);
      expect(balanceAfter).to.be.greaterThan(balanceBefore - 10000);
    });
  });
});

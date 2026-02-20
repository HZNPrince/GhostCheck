import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { GhostCheck } from "../target/types/ghost_check";
import {
  PublicKey,
  Keypair,
  Transaction,
  sendAndConfirmTransaction,
  Ed25519Program,
  SYSVAR_INSTRUCTIONS_PUBKEY,
} from "@solana/web3.js";
import { MPL_CORE_PROGRAM_ID } from "@metaplex-foundation/mpl-core";
import crypto from "crypto";
import nacl from "tweetnacl";
import { expect } from "chai";

// ── Helpers ──

function sha256(data: Buffer | Uint8Array): Buffer {
  return crypto.createHash("sha256").update(data).digest();
}

function u32BE(n: number): Buffer {
  const buf = Buffer.alloc(4);
  buf.writeUInt32BE(n);
  return buf;
}

// sign dev metrics
function signDevMetrics(
  secretKey: Uint8Array,
  username: string,
  repoCount: number,
  totalCommits: number,
  ownedRepoCount: number,
  totalStars: number,
  prsMerged: number,
  issuesClosed: number,
  followers: number,
  accountAgeDays: number,
  reputationLevel: number
) {
  const hashedUsername = sha256(Buffer.from(username));
  const message = Buffer.concat([
    hashedUsername,
    u32BE(repoCount),
    u32BE(totalCommits),
    u32BE(ownedRepoCount),
    u32BE(totalStars),
    u32BE(prsMerged),
    u32BE(issuesClosed),
    u32BE(followers),
    u32BE(accountAgeDays),
    Buffer.from([reputationLevel]),
  ]);
  const hashedMessage = sha256(message);
  const signature = nacl.sign.detached(hashedMessage, secretKey);
  return { hashedUsername, hashedMessage, signature };
}

// Sign repo metrics
function signRepoMetrics(
  secretKey: Uint8Array,
  username: string,
  repoName: string,
  lang1: Buffer,
  lang2: Buffer,
  stars: number,
  commits: number,
  forks: number,
  openIssues: number,
  isFork: number
) {
  const hashedUsername = sha256(Buffer.from(username));
  const message = Buffer.concat([
    hashedUsername,
    Buffer.from(repoName),
    lang1,
    lang2,
    u32BE(stars),
    u32BE(commits),
    u32BE(forks),
    u32BE(openIssues),
    Buffer.from([isFork]),
  ]);
  const hashedMessage = sha256(message);
  const signature = nacl.sign.detached(hashedMessage, secretKey);
  return { hashedUsername, hashedMessage, signature };
}

// ── Tests ──

describe("ghost_check", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.GhostCheck as Program<GhostCheck>;
  const payer = provider.wallet.payer;
  const connection = provider.connection;

  // Backend signer keypair (generated fresh for tests)
  const backendKeypair = Keypair.generate();
  const backendPubkeyArray = [
    46, 64, 199, 231, 85, 86, 213, 48, 219, 165, 147, 79, 95, 123, 208, 118,
    104, 1, 16, 172, 144, 233, 143, 174, 80, 168, 163, 27, 202, 6, 19, 114,
  ];

  // Second user for vouch tests
  const user2 = Keypair.generate();

  const BPF_LOADER = new PublicKey(
    "BPFLoaderUpgradeab1e11111111111111111111111"
  );
  const repoNamePadded = Buffer.from("Raydium-Indexer".padEnd(32, "\0"));

  let ghostConfigPda: PublicKey;
  let devStatePda: PublicKey;
  let devBadgePda: PublicKey;
  let programDataPda: PublicKey;
  let repoStatePda: PublicKey;
  let repoBadgePda: PublicKey;
  let user2DevStatePda: PublicKey;
  let user2DevBadgePda: PublicKey;

  before(async () => {
    [ghostConfigPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("ghost_config")],
      program.programId
    );
    [devStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("dev_state"), payer.publicKey.toBuffer()],
      program.programId
    );
    [devBadgePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("dev_badge"), payer.publicKey.toBuffer()],
      program.programId
    );
    [programDataPda] = PublicKey.findProgramAddressSync(
      [program.programId.toBuffer()],
      BPF_LOADER
    );
    [repoStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("repo_state"), devBadgePda.toBuffer(), repoNamePadded],
      program.programId
    );
    [repoBadgePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("repo_badge"), devBadgePda.toBuffer(), repoNamePadded],
      program.programId
    );
    [user2DevStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("dev_state"), user2.publicKey.toBuffer()],
      program.programId
    );
    [user2DevBadgePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("dev_badge"), user2.publicKey.toBuffer()],
      program.programId
    );

    const sig = await connection.requestAirdrop(user2.publicKey, 2e9);
    await connection.confirmTransaction(sig);
  });

  // ════════════════════════════════════════
  // 1. INIT CONFIG
  // ════════════════════════════════════════

  it("Initialize Config", async () => {
    const tx = await program.methods
      .initConfig(backendPubkeyArray)
      .accounts({
        admin: payer.publicKey,
        programData: programDataPda,
      })
      .rpc();
    console.log("Config initialized:", tx);
  });

  // it("Should fail — unauthorized init_config", async () => {
  //   const fake = Keypair.generate();
  //   const sig = await connection.requestAirdrop(fake.publicKey, 1e9);
  //   await connection.confirmTransaction(sig);

  //   try {
  //     await program.methods
  //       .initConfig(backendPubkeyArray)
  //       .accounts({ admin: fake.publicKey, programData: programDataPda })
  //       .signers([fake])
  //       .rpc();
  //     throw new Error("Should have failed");
  //   } catch (e) {
  //     expect(e.message).to.not.equal("Should have failed");
  //   }
  // });

  // // ════════════════════════════════════════
  // // 2. MINT DEV BADGE
  // // ════════════════════════════════════════

  // it("Mint Dev Badge", async () => {
  //   const { hashedUsername, hashedMessage, signature } = signDevMetrics(
  //     backendKeypair.secretKey,
  //     "HZNPrince",
  //     18,
  //     107,
  //     10,
  //     50,
  //     5,
  //     3,
  //     20,
  //     365,
  //     2
  //   );

  //   const ed25519Ix = Ed25519Program.createInstructionWithPublicKey({
  //     publicKey: backendKeypair.publicKey.toBytes(),
  //     message: hashedMessage,
  //     signature: signature,
  //   });

  //   const mintIx = await program.methods
  //     .mintDevBadge(
  //       Array.from(hashedUsername),
  //       18,
  //       10,
  //       50,
  //       107,
  //       5,
  //       3,
  //       20,
  //       365,
  //       2
  //     )
  //     .accounts({
  //       dev: payer.publicKey,
  //       instructionSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
  //       coreProgram: MPL_CORE_PROGRAM_ID,
  //     })
  //     .instruction();

  //   const tx = new Transaction().add(ed25519Ix).add(mintIx);
  //   const txSig = await sendAndConfirmTransaction(connection, tx, [payer]);
  //   console.log("Dev Badge minted:", txSig);
  // });

  // it("Verify DevState on-chain data", async () => {
  //   const state = await program.account.devState.fetch(devStatePda);
  //   expect(state.repoCount).to.equal(18);
  //   expect(state.totalCommits).to.equal(107);
  //   expect(state.ownedRepoCount).to.equal(10);
  //   expect(state.totalStars).to.equal(50);
  //   expect(state.prsMerged).to.equal(5);
  //   expect(state.issuesClosed).to.equal(3);
  //   expect(state.followers).to.equal(20);
  //   expect(state.accountAgeDays).to.equal(365);
  //   expect(state.reputationLevel).to.equal(2);
  //   expect(state.vouchCount.toNumber()).to.equal(0);
  //   expect(state.verifiedRepos.toNumber()).to.equal(0);
  //   console.log("DevState verified ✓");
  // });

  // it("Should fail — duplicate mint dev badge", async () => {
  //   const { hashedUsername, hashedMessage, signature } = signDevMetrics(
  //     backendKeypair.secretKey,
  //     "HZNPrince",
  //     18,
  //     107,
  //     10,
  //     50,
  //     5,
  //     3,
  //     20,
  //     365,
  //     2
  //   );

  //   const ed25519Ix = Ed25519Program.createInstructionWithPublicKey({
  //     publicKey: backendKeypair.publicKey.toBytes(),
  //     message: hashedMessage,
  //     signature: signature,
  //   });

  //   const mintIx = await program.methods
  //     .mintDevBadge(
  //       Array.from(hashedUsername),
  //       18,
  //       10,
  //       50,
  //       107,
  //       5,
  //       3,
  //       20,
  //       365,
  //       2
  //     )
  //     .accounts({
  //       dev: payer.publicKey,
  //       instructionSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
  //       coreProgram: MPL_CORE_PROGRAM_ID,
  //     })
  //     .instruction();

  //   try {
  //     const tx = new Transaction().add(ed25519Ix).add(mintIx);
  //     await sendAndConfirmTransaction(connection, tx, [payer]);
  //     throw new Error("Should have failed");
  //   } catch (e) {
  //     expect(e.message).to.not.equal("Should have failed");
  //   }
  // });

  // // ════════════════════════════════════════
  // // 3. MINT REPO BADGE
  // // ════════════════════════════════════════

  // it("Mint Repo Badge", async () => {
  //   const lang1 = Buffer.from([82, 117, 115, 116]); // "Rust"
  //   const lang2 = Buffer.from([]);

  //   const { hashedUsername, hashedMessage, signature } = signRepoMetrics(
  //     backendKeypair.secretKey,
  //     "HZNPrince",
  //     "Raydium-Indexer",
  //     lang1,
  //     lang2,
  //     1,
  //     11,
  //     0,
  //     2,
  //     0
  //   );

  //   const ed25519Ix = Ed25519Program.createInstructionWithPublicKey({
  //     publicKey: backendKeypair.publicKey.toBytes(),
  //     message: hashedMessage,
  //     signature: signature,
  //   });

  //   const mintIx = await program.methods
  //     .mintRepoBadge(
  //       Array.from(repoNamePadded),
  //       Array.from(hashedUsername),
  //       1,
  //       11,
  //       0,
  //       2,
  //       0,
  //       lang1,
  //       lang2
  //     )
  //     .accounts({
  //       dev: payer.publicKey,
  //       instructionSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
  //       coreProgram: MPL_CORE_PROGRAM_ID,
  //     })
  //     .instruction();

  //   const tx = new Transaction().add(ed25519Ix).add(mintIx);
  //   const txSig = await sendAndConfirmTransaction(connection, tx, [payer]);
  //   console.log("Repo Badge minted:", txSig);
  // });

  // it("Verify RepoState on-chain data", async () => {
  //   const state = await program.account.repoState.fetch(repoStatePda);
  //   expect(state.stars).to.equal(1);
  //   expect(state.commits).to.equal(11);
  //   expect(state.forks).to.equal(0);
  //   expect(state.openIssues).to.equal(2);
  //   expect(state.isFork).to.equal(0);

  //   const devState = await program.account.devState.fetch(devStatePda);
  //   expect(devState.verifiedRepos.toNumber()).to.equal(1);
  //   console.log("RepoState verified ✓");
  // });

  // it("Should fail — forked repo badge mint", async () => {
  //   const forkRepoName = Buffer.from("SomeForkedRepo".padEnd(32, "\0"));
  //   const lang1 = Buffer.from([82, 117, 115, 116]);
  //   const lang2 = Buffer.from([]);

  //   const { hashedUsername, hashedMessage, signature } = signRepoMetrics(
  //     backendKeypair.secretKey,
  //     "HZNPrince",
  //     "SomeForkedRepo",
  //     lang1,
  //     lang2,
  //     5,
  //     3,
  //     1,
  //     0,
  //     1 // is_fork = 1
  //   );

  //   const ed25519Ix = Ed25519Program.createInstructionWithPublicKey({
  //     publicKey: backendKeypair.publicKey.toBytes(),
  //     message: hashedMessage,
  //     signature: signature,
  //   });

  //   const mintIx = await program.methods
  //     .mintRepoBadge(
  //       Array.from(forkRepoName),
  //       Array.from(hashedUsername),
  //       5,
  //       3,
  //       1,
  //       0,
  //       1, // is_fork = 1
  //       lang1,
  //       lang2
  //     )
  //     .accounts({
  //       dev: payer.publicKey,
  //       instructionSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
  //       coreProgram: MPL_CORE_PROGRAM_ID,
  //     })
  //     .instruction();

  //   try {
  //     const tx = new Transaction().add(ed25519Ix).add(mintIx);
  //     await sendAndConfirmTransaction(connection, tx, [payer]);
  //     throw new Error("Should have failed");
  //   } catch (e) {
  //     expect(e.message).to.not.equal("Should have failed");
  //   }
  // });

  // // ════════════════════════════════════════
  // // 4. UPDATE DEV BADGE
  // // ════════════════════════════════════════

  // it("Update Dev Badge with new stats", async () => {
  //   const { hashedUsername, hashedMessage, signature } = signDevMetrics(
  //     backendKeypair.secretKey,
  //     "HZNPrince",
  //     25,
  //     200,
  //     15,
  //     80,
  //     10,
  //     8,
  //     30,
  //     400,
  //     3
  //   );

  //   const ed25519Ix = Ed25519Program.createInstructionWithPublicKey({
  //     publicKey: backendKeypair.publicKey.toBytes(),
  //     message: hashedMessage,
  //     signature: signature,
  //   });

  //   const updateIx = await program.methods
  //     .updateDevBadge(
  //       Array.from(hashedUsername),
  //       25,
  //       15,
  //       80,
  //       200,
  //       10,
  //       8,
  //       30,
  //       400,
  //       3
  //     )
  //     .accounts({
  //       dev: payer.publicKey,
  //       instructionSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
  //     })
  //     .instruction();

  //   const tx = new Transaction().add(ed25519Ix).add(updateIx);
  //   const txSig = await sendAndConfirmTransaction(connection, tx, [payer]);
  //   console.log("Dev Badge updated:", txSig);

  //   const state = await program.account.devState.fetch(devStatePda);
  //   expect(state.repoCount).to.equal(25);
  //   expect(state.totalCommits).to.equal(200);
  //   expect(state.reputationLevel).to.equal(3);
  //   expect(state.vouchCount.toNumber()).to.equal(0); // preserved
  //   expect(state.verifiedRepos.toNumber()).to.equal(1); // preserved
  //   console.log("Updated DevState verified ✓");
  // });

  // // ════════════════════════════════════════
  // // 5. UPDATE REPO BADGE
  // // ════════════════════════════════════════

  // it("Update Repo Badge with new stats", async () => {
  //   const lang1 = Buffer.from([82, 117, 115, 116]);
  //   const lang2 = Buffer.from([]);

  //   const { hashedUsername, hashedMessage, signature } = signRepoMetrics(
  //     backendKeypair.secretKey,
  //     "HZNPrince",
  //     "Raydium-Indexer",
  //     lang1,
  //     lang2,
  //     10,
  //     50,
  //     3,
  //     5,
  //     0
  //   );

  //   const ed25519Ix = Ed25519Program.createInstructionWithPublicKey({
  //     publicKey: backendKeypair.publicKey.toBytes(),
  //     message: hashedMessage,
  //     signature: signature,
  //   });

  //   const updateIx = await program.methods
  //     .updateRepoBadge(
  //       Array.from(repoNamePadded),
  //       Array.from(hashedUsername),
  //       10,
  //       50,
  //       3,
  //       5,
  //       lang1,
  //       lang2
  //     )
  //     .accounts({
  //       dev: payer.publicKey,
  //       instructionSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
  //       coreProgram: MPL_CORE_PROGRAM_ID,
  //     })
  //     .instruction();

  //   const tx = new Transaction().add(ed25519Ix).add(updateIx);
  //   const txSig = await sendAndConfirmTransaction(connection, tx, [payer]);
  //   console.log("Repo Badge updated:", txSig);

  //   const state = await program.account.repoState.fetch(repoStatePda);
  //   expect(state.stars).to.equal(10);
  //   expect(state.commits).to.equal(50);
  //   expect(state.forks).to.equal(3);
  //   expect(state.openIssues).to.equal(5);
  //   console.log("Updated RepoState verified ✓");
  // });

  // // ════════════════════════════════════════
  // // 6. VERIFY DEV
  // // ════════════════════════════════════════

  // it("Verify Dev — should pass (level 3, min_lvl 2)", async () => {
  //   await program.methods
  //     .verifyDev(Array.from(payer.publicKey.toBytes()), 2)
  //     .accounts({})
  //     .rpc();
  //   console.log("Verify dev passed ✓");
  // });

  // it("Verify Dev — should pass (level 3, min_lvl 3)", async () => {
  //   await program.methods
  //     .verifyDev(Array.from(payer.publicKey.toBytes()), 3)
  //     .accounts({})
  //     .rpc();
  //   console.log("Verify dev level 3 passed ✓");
  // });

  // it("Should fail — verify dev insufficient level (min_lvl 5)", async () => {
  //   try {
  //     await program.methods
  //       .verifyDev(Array.from(payer.publicKey.toBytes()), 5)
  //       .accounts({})
  //       .rpc();
  //     throw new Error("Should have failed");
  //   } catch (e) {
  //     expect(e.message).to.not.equal("Should have failed");
  //   }
  // });

  // it("Should fail — verify dev invalid min_lvl (0)", async () => {
  //   try {
  //     await program.methods
  //       .verifyDev(Array.from(payer.publicKey.toBytes()), 0)
  //       .accounts({})
  //       .rpc();
  //     throw new Error("Should have failed");
  //   } catch (e) {
  //     expect(e.message).to.not.equal("Should have failed");
  //   }
  // });

  // it("Should fail — verify dev invalid min_lvl (6)", async () => {
  //   try {
  //     await program.methods
  //       .verifyDev(Array.from(payer.publicKey.toBytes()), 6)
  //       .accounts({})
  //       .rpc();
  //     throw new Error("Should have failed");
  //   } catch (e) {
  //     expect(e.message).to.not.equal("Should have failed");
  //   }
  // });

  // // ════════════════════════════════════════
  // // 7. VOUCH FOR DEV
  // // ════════════════════════════════════════

  // it("Mint Dev Badge for user2 (level 1)", async () => {
  //   const { hashedUsername, hashedMessage, signature } = signDevMetrics(
  //     backendKeypair.secretKey,
  //     "User2Dev",
  //     5,
  //     50,
  //     3,
  //     10,
  //     2,
  //     1,
  //     5,
  //     180,
  //     1
  //   );

  //   const ed25519Ix = Ed25519Program.createInstructionWithPublicKey({
  //     publicKey: backendKeypair.publicKey.toBytes(),
  //     message: hashedMessage,
  //     signature: signature,
  //   });

  //   const mintIx = await program.methods
  //     .mintDevBadge(Array.from(hashedUsername), 5, 3, 10, 50, 2, 1, 5, 180, 1)
  //     .accounts({
  //       dev: user2.publicKey,
  //       instructionSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
  //       coreProgram: MPL_CORE_PROGRAM_ID,
  //     })
  //     .instruction();

  //   const tx = new Transaction().add(ed25519Ix).add(mintIx);
  //   await sendAndConfirmTransaction(connection, tx, [user2]);
  //   console.log("User2 Dev Badge minted ✓");
  // });

  // it("Payer vouches for user2", async () => {
  //   await program.methods
  //     .vouchForDev(Array.from(user2.publicKey.toBytes()))
  //     .accounts({ voucher: payer.publicKey })
  //     .rpc();

  //   const targetState = await program.account.devState.fetch(user2DevStatePda);
  //   expect(targetState.vouchCount.toNumber()).to.equal(1);

  //   const config = await program.account.ghostConfig.fetch(ghostConfigPda);
  //   expect(config.vouchesCount).to.equal(1);
  //   console.log("Vouch succeeded ✓");
  // });

  // it("Should fail — duplicate vouch", async () => {
  //   try {
  //     await program.methods
  //       .vouchForDev(Array.from(user2.publicKey.toBytes()))
  //       .accounts({ voucher: payer.publicKey })
  //       .rpc();
  //     throw new Error("Should have failed");
  //   } catch (e) {
  //     expect(e.message).to.not.equal("Should have failed");
  //   }
  // });

  // it("Should fail — self-vouch", async () => {
  //   try {
  //     await program.methods
  //       .vouchForDev(Array.from(payer.publicKey.toBytes()))
  //       .accounts({ voucher: payer.publicKey })
  //       .rpc();
  //     throw new Error("Should have failed");
  //   } catch (e) {
  //     expect(e.message).to.not.equal("Should have failed");
  //   }
  // });

  // it("Should fail — user2 (level 1) tries to vouch for payer", async () => {
  //   try {
  //     await program.methods
  //       .vouchForDev(Array.from(payer.publicKey.toBytes()))
  //       .accounts({ voucher: user2.publicKey })
  //       .signers([user2])
  //       .rpc();
  //     throw new Error("Should have failed");
  //   } catch (e) {
  //     expect(e.message).to.not.equal("Should have failed");
  //   }
  // });
});

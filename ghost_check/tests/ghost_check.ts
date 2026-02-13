import * as anchor from "@coral-xyz/anchor"
import { Program } from "@coral-xyz/anchor"
import { GhostCheck } from "../target/types/ghost_check"
import {
  PublicKey,
  Keypair,
  Transaction,
  sendAndConfirmTransaction,
  Ed25519Program,
  SYSVAR_INSTRUCTIONS_PUBKEY,
} from "@solana/web3.js"
import { MPL_CORE_PROGRAM_ID } from "@metaplex-foundation/mpl-core"
import crypto from "crypto"
import fs from "fs"
import path from "path"

describe("ghost_check", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)

  const program = anchor.workspace.GhostCheck as Program<GhostCheck>
  const payer = provider.wallet.payer
  const connection = provider.connection

  const fake_payer = Keypair.generate()
  const backendPubkey = [
    46, 64, 199, 231, 85, 86, 213, 48, 219, 165, 147, 79, 95, 123, 208, 118, 104, 1, 16, 172, 144, 233, 143, 174, 80,
    168, 163, 27, 202, 6, 19, 114,
  ]
  // For Dev_Badge
  const Ed25519Signature = [
    92, 167, 49, 69, 200, 211, 252, 170, 164, 250, 78, 218, 173, 56, 242, 138, 12, 50, 54, 231, 131, 87, 149, 97, 91,
    152, 80, 248, 203, 86, 209, 72, 187, 230, 90, 225, 203, 228, 115, 99, 80, 152, 252, 142, 57, 110, 74, 195, 227, 20,
    247, 18, 154, 137, 198, 106, 146, 249, 122, 80, 7, 144, 93, 2,
  ]

  const messageHash = Buffer.from([
    200, 114, 63, 63, 43, 42, 55, 100, 20, 181, 162, 91, 107, 174, 30, 175, 70, 149, 43, 141, 68, 111, 173, 203, 16,
    169, 89, 109, 53, 172, 154, 108,
  ])

  // For Repo_Badge
  const Ed25519SignatureRepo = [
    246, 217, 229, 21, 127, 70, 69, 129, 61, 179, 31, 15, 245, 214, 138, 139, 21, 159, 217, 226, 252, 3, 65, 19, 152,
    219, 200, 142, 134, 228, 91, 3, 195, 115, 202, 7, 18, 188, 124, 163, 69, 145, 146, 46, 161, 194, 53, 60, 26, 188,
    171, 79, 130, 248, 24, 131, 67, 164, 101, 152, 101, 234, 20, 12,
  ]
  const hashedMessageRepo = [
    252, 152, 237, 160, 153, 61, 109, 104, 203, 166, 128, 160, 55, 173, 242, 124, 239, 43, 202, 116, 183, 111, 137, 186,
    12, 64, 229, 126, 101, 186, 13, 230,
  ]

  let ghostConfigPda: PublicKey
  let devStatePda: PublicKey
  let devBadgePda: PublicKey
  let programDataPda: PublicKey
  let repoBadgePda: PublicKey

  const BPF_LOADER_UPGRADEABLE_PROGRAM_ID = new PublicKey("BPFLoaderUpgradeab1e11111111111111111111111")
  const repoNamePadded = Buffer.from("Raydium-Indexer".padEnd(32, "\0"))

  before("Initialize wallets and setup accounts", async () => {
    ;[ghostConfigPda] = PublicKey.findProgramAddressSync([Buffer.from("ghost_config")], program.programId)
    ;[devStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("dev_state"), payer.publicKey.toBuffer()],
      program.programId,
    )
    ;[devBadgePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("dev_badge_collection"), payer.publicKey.toBuffer()],
      program.programId,
    )
    ;[programDataPda] = PublicKey.findProgramAddressSync(
      [program.programId.toBuffer()],
      BPF_LOADER_UPGRADEABLE_PROGRAM_ID,
    )
    ;[repoBadgePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("repo_badge_asset"), devBadgePda.toBuffer(), repoNamePadded],
      program.programId,
    )
    console.log("RepoBadgePda: ", repoBadgePda)
  })

  it("Initialize Config", async () => {
    const tx = await program.methods
      .initConfig(backendPubkey)
      .accounts({
        admin: payer.publicKey,
        programData: programDataPda,
      })
      .rpc()
    console.log("Ghost Config Initialized: ", tx)
  })

  it("Should fail when unauthorized address tries to call", async () => {
    try {
      await program.methods
        .initConfig(backendPubkey)
        .accounts({
          admin: fake_payer.publicKey,
          programData: programDataPda,
        })
        .signers([fake_payer])
        .rpc()
      throw new Error("Should have failed")
    } catch (e) {
      // Expected failure
    }
  })

  it("Getting DevBadge for dev stats", async () => {
    const username = Buffer.from("HZNPrince".padEnd(32, "\0"))
    const repo_count = 18
    const total_commits = 107

    console.log("Hashed message:", Array.from(messageHash))

    const ix = anchor.web3.Ed25519Program.createInstructionWithPublicKey({
      publicKey: new Uint8Array(backendPubkey),
      message: new Uint8Array(messageHash),
      signature: new Uint8Array(Ed25519Signature),
    })

    const ix2 = await program.methods
      .mintDevBadge(Array.from(username), repo_count, total_commits)
      .accounts({
        dev: payer.publicKey,
        ghostConfig: ghostConfigPda,
        instructionSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
        coreProgram: MPL_CORE_PROGRAM_ID, // MPL Core ID
        asset: devBadgePda,
      })
      .instruction()

    const transaction = new Transaction().add(ix).add(ix2)
    let signature = await sendAndConfirmTransaction(connection, transaction, [payer])

    console.log("Dev Badge Minted: ", signature)
  })

  it("Getting Repo_Badge for repo_metrics", async () => {
    const usernamePadded = Buffer.from("HZNPrince".padEnd(32, "\0"))

    const ix = Ed25519Program.createInstructionWithPublicKey({
      publicKey: Buffer.from(backendPubkey),
      message: Buffer.from(hashedMessageRepo),
      signature: Buffer.from(Ed25519SignatureRepo),
    })

    const ix2 = await program.methods
      .mintRepoBadge(
        Array.from(repoNamePadded),
        Array.from(usernamePadded),
        1,
        11,
        Buffer.from([82, 117, 115, 116]),
        Buffer.from([]),
      )
      .accounts({
        dev: payer.publicKey,
        ghostConfig: ghostConfigPda,
        devState: devStatePda,
        repo_badge: repoBadgePda,
        instructionSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
        coreProgram: MPL_CORE_PROGRAM_ID,
      })
      .instruction()

    const tx = new Transaction().add(ix).add(ix2)
    const tx_sig = await sendAndConfirmTransaction(connection, tx, [payer])
  })
})

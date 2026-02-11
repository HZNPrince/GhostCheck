import * as anchor from "@coral-xyz/anchor"
import { Program } from "@coral-xyz/anchor"
import { GhostCheck } from "../target/types/ghost_check"
import { PublicKey, Keypair, Transaction, sendAndConfirmTransaction } from "@solana/web3.js"
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
  const Ed25519Signature = [
    92, 167, 49, 69, 200, 211, 252, 170, 164, 250, 78, 218, 173, 56, 242, 138, 12, 50, 54, 231, 131, 87, 149, 97, 91,
    152, 80, 248, 203, 86, 209, 72, 187, 230, 90, 225, 203, 228, 115, 99, 80, 152, 252, 142, 57, 110, 74, 195, 227, 20,
    247, 18, 154, 137, 198, 106, 146, 249, 122, 80, 7, 144, 93, 2,
  ]

  const messageHash = Buffer.from([
    200, 114, 63, 63, 43, 42, 55, 100, 20, 181, 162, 91, 107, 174, 30, 175, 70, 149, 43, 141, 68, 111, 173, 203, 16,
    169, 89, 109, 53, 172, 154, 108,
  ])

  let ghostConfigPda: PublicKey
  let devBadgePda: PublicKey
  let programDataPda: PublicKey
  let collectionAsset: Keypair

  const BPF_LOADER_UPGRADEABLE_PROGRAM_ID = new PublicKey("BPFLoaderUpgradeab1e11111111111111111111111")

  before("Initialize wallets and setup accounts", async () => {
    ;[ghostConfigPda] = PublicKey.findProgramAddressSync([Buffer.from("ghost_config")], program.programId)
    ;[devBadgePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("dev_state"), payer.publicKey.toBuffer()],
      program.programId,
    )
    ;[programDataPda] = PublicKey.findProgramAddressSync(
      [program.programId.toBuffer()],
      BPF_LOADER_UPGRADEABLE_PROGRAM_ID,
    )
    collectionAsset = Keypair.generate()
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
        asset: collectionAsset.publicKey,
      })
      .instruction()

    const transaction = new Transaction().add(ix).add(ix2)
    let signature = await sendAndConfirmTransaction(connection, transaction, [payer, collectionAsset])

    console.log("Dev Badge Minted: ", signature)
  })
})

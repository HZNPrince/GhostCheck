import * as anchor from "@coral-xyz/anchor"
import { Program } from "@coral-xyz/anchor"
import { GhostCheck } from "../target/types/ghost_check"
import { PublicKey, Keypair } from "@solana/web3.js"
import fs from "fs"
import path from "path"

describe("ghost_check", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)

  const program = anchor.workspace.GhostCheck as Program<GhostCheck>
  const user = provider.wallet
})

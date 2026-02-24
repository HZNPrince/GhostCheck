---
description: Rules and conventions for working on the GhostCheck project
---

## CRITICAL RULES

1. **Code editing:** The user may ask you to either teach them or write code directly. Follow their most recent instruction on this. When in doubt, ask.

2. **Always use async in Axum context.** The backend uses Tokio + Axum. Any external I/O (RPC calls, HTTP requests, DB queries) MUST use async/await. For Solana specifically:
   - Use `solana_rpc_client::nonblocking::rpc_client::RpcClient` (NOT the sync version)
   - All RPC calls need `.await`

3. **Teaching style.** When explaining code:
   - Explain WHY before showing WHAT
   - Highlight gotchas (like sync vs async, big-endian vs little-endian)
   - Reference existing patterns in the codebase when possible

## Project Conventions

- **Backend:** Rust (Axum), async, PostgreSQL via SQLx, Ed25519 signing
- **Frontend:** Rust (Leptos), WASM, JS interop via `solana.js` bridge
- **On-Chain:** Anchor framework, Metaplex Core for soulbound assets
- **Program ID:** `GQsPhnZApw9MY7khsbRLtL5mAGpmMn8wp8CFNDPTxGQr`
- **Borsh serialization:** little-endian for numbers (`from_le_bytes`)
- **Signer hashing:** big-endian for the signed message (`to_be_bytes`)

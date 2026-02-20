# GhostCheck 
<div align="center">
  <img src="images/frontend_images/landing.png" alt="GhostCheck Banner" width="100%" />

**The Anonymous On-chain Reputaion Protocol for Developers.**

[![Built with Anchor](https://img.shields.io/badge/Built_with-Anchor-blueviolet)](https://project-serum.github.io/anchor/)
[![Frontend: Leptos](https://img.shields.io/badge/Frontend-Leptos-orange)](https://leptos.dev/)
[![Backend: Axum](https://img.shields.io/badge/Backend-Axum-red)](https://github.com/tokio-rs/axum)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

</div>

---

## 📖 The Problem

In the current Web3 ecosystem, developer reputation is broken. DAOs, grant programs, and bounty platforms are forced to choose between two flawed options:

1.  **Doxxing Developers:** Forcing builders to share their real names or public GitHub profiles, destroying privacy.
2.  **"Trust Me Bro":** Accepting anonymous PDF resumes or claims of experience, which invites fraud and Sybil attacks.

## 💡 The Solution

**GhostCheck** bridges your Web2 reputation (GitHub) to Web3 (Solana) as a **soulbound, verifiable asset** without revealing your identity.

We act as an oracle, authenticating your GitHub stats, computing a reputation level, cryptographically signing the data via Ed25519, and minting it as an immutable **Metaplex Core Asset** on Solana.

---

##  UI Proofs

**[Live UI Proofs & Screenshots Directory](./images/frontend_images/)** 

---

## 🚀 Key Features

- **Verifiable Metrics:** We prove your GitHub Commits, PRs, and Stars via a trusted on-chain Ed25519 signature.
- **Soulbound Developer Badges:** Mint a unique, non-transferable asset that evolves with your "Proof of Code".
- **Repository Verification:** Project maintainers can mint "Repo Badges" to prove ownership and showcase repo stats on-chain.
- **Sybil-Resistant Vouching:** A peer-to-peer trust network where established developers (Level 3+) can vouch for newcomers on-chain.

---

## 🏗️ Architecture & Flow

GhostCheck is built purely in Rust across the entire stack.

<div align="center">
  
  ### 1. Frontend Architecture (Leptos + WASM)
  <img src="images/Frontend.png" alt="Frontend Architecture" width="80%" />

### 2. Backend Oracle (Axum + PostgreSQL)

  <img src="images/Backend.png" alt="Backend Architecture" width="80%" />

### 3. Smart Contract (Anchor + Metaplex Core)

  <img src="images/ghost_check.png" alt="On-Chain Architecture" width="80%" />
</div>

---

## 🛠 Tech Stack

- **Smart Contract:** Rust (Anchor Framework v0.30.1)
- **Frontend:** Rust (Leptos) + WASM + `solana.js` Javascript Bridge
- **Backend:** Rust (Axum) + SQLx + Reqwest
- **Data Models:** Metaplex Core (MPL Core Assets)
- **Security:** Ed25519 Instruction Introspection

---

## 📦 Prerequisites

- [Rust & Cargo](https://rustup.rs/) (Stable + Nightly for Leptos)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation)
- `cargo install trunk` (for running the frontend)

---

## 🏃‍♂️ Quick Start (Local Development)

### 1. Start Local Validator

```bash
solana-test-validator --bpf-program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s path/to/mpl_core.so
```

_(Ensure you have the Metaplex Core program deployed to your localnet)._

### 2. Build & Deploy Program

```bash
cd ghost_check
anchor build
anchor deploy
```

### 3. Backend Setup

Create a `.env` file in the `/backend` directory:

```env
DATABASE_URL=postgres://user:pass@localhost/ghostcheck
GITHUB_CLIENT_ID=your_github_oauth_app_id
GITHUB_CLIENT_SECRET=your_github_oauth_app_secret
GHOSTCHECK_SIGNER_SECRET=your_32_byte_ed25519_private_key_base58
```

Run the backend:

```bash
cd backend
cargo run
```

### 4. Frontend Setup

```bash
cd frontend
trunk serve
```

Visit `http://localhost:8080` to interact with the dApp!

---

## 🧪 Testing

Run the comprehensive 20+ Anchor test suite to verify the logic, vouch constraints, and Ed25519 integrations:

```bash
cd ghost_check
anchor test
```

---

## 🔮 Future Upgrades & Vision

We are actively building GhostCheck into the core reputation infrastructure on Solana.

**Coming Soon:**

- **Demo Video:** A full walkthrough of the MVP (Connect -> Fetch -> Mint -> Profile) is almost ready.
- **Search & Discovery Indexer:** An open API for searching developers by skills and wallet addresses.
- **Institute & DAO Portals:** B2B dashboards for DAOs and Grants programs (like Turbine or Rektoff) to automatically verify applicant criteria and funnel recruitment.
- **Incentivized Vouching:** Rewarding developers for participating in the peer-to-peer verification web of trust.

---

## 📜 License

MIT License.

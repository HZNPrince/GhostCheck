// solana.js — Bridge between Rust/WASM and Solana
// This file builds Solana transactions and sends them via Phantom wallet.
// It uses @solana/web3.js loaded via CDN in index.html.

// Browser-compatible replacement for Node.js Buffer.from("string")
const encoder = new TextEncoder();
function toBytes(str) {
  return encoder.encode(str);
}

const PROGRAM_ID = new solanaWeb3.PublicKey(
  "GQsPhnZApw9MY7khsbRLtL5mAGpmMn8wp8CFNDPTxGQr",
);
const MPL_CORE_PROGRAM_ID = new solanaWeb3.PublicKey(
  "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d",
);
const SYSVAR_INSTRUCTIONS = new solanaWeb3.PublicKey(
  "Sysvar1nstructions1111111111111111111111111",
);
const SYSTEM_PROGRAM = new solanaWeb3.PublicKey(
  "11111111111111111111111111111111",
);

// Anchor instruction discriminators ( 8 bytes : Of each instructions)
const MINT_DEV_BADGE_DISCRIMINATOR = new Uint8Array([
  102, 221, 55, 250, 237, 112, 154, 119,
]);
const MINT_REPO_BADGE_DISCRIMINATOR = new Uint8Array([
  43, 60, 10, 231, 230, 125, 118, 14,
]);
const UPDATE_DEV_BADGE_DISCRIMINATOR = new Uint8Array([
  115, 186, 3, 77, 214, 118, 70, 16,
]);
const UPDATE_REPO_BADGE_DISCRIMINATOR = new Uint8Array([
  74, 96, 123, 10, 162, 151, 119, 226,
]);
const VOUCH_FOR_DEV_DISCRIMINATOR = new Uint8Array([
  155, 55, 131, 60, 112, 41, 101, 7,
]);

const CONNECTION = new solanaWeb3.Connection(
  "http://localhost:8899",
  "confirmed",
);

// Helper: get Phantom provider
function getPhantom() {
  if (
    window.phantom &&
    window.phantom.solana &&
    window.phantom.solana.isPhantom
  ) {
    return window.phantom.solana;
  }
  throw new Error("Phantom wallet not found");
}

// Helper: find PDA (Program Derived Address)
function findPda(seeds) {
  return solanaWeb3.PublicKey.findProgramAddressSync(seeds, PROGRAM_ID);
}

// Helper: encode a u32 as 4 bytes little-endian (Anchor/Borsh uses little-endian)
function encodeU32LE(value) {
  const buf = new ArrayBuffer(4);
  new DataView(buf).setUint32(0, value, true); // true = little-endian
  return new Uint8Array(buf);
}

// Helper: encode a u64 as 8 bytes little-endian
function encodeU64LE(value) {
  const buf = new ArrayBuffer(8);
  const view = new DataView(buf);
  view.setUint32(0, value & 0xffffffff, true);
  view.setUint32(4, Math.floor(value / 0x100000000), true);
  return new Uint8Array(buf);
}

// Helper: encode a Vec<u8> as Borsh bytes (4-byte LE length prefix + data)
function encodeBorshBytes(data) {
  const len = encodeU32LE(data.length);
  const result = new Uint8Array(len.length + data.length);
  result.set(len, 0);
  result.set(data, len.length);
  return result;
}

// ============================================================
//  MINT DEV BADGE
// ============================================================
// Called from Rust via wasm-bindgen
// signature, message, publicKey, username are Uint8Array
// repoCount, totalCommits are numbers

window.buildAndSendDevBadgeTx = async function (
  signature, // Vec<u8> → Uint8Array (64 bytes, Ed25519 signature from backend)
  message, // Vec<u8> → Uint8Array (32 bytes, SHA256 hash that was signed)
  publicKey, // Vec<u8> → Uint8Array (32 bytes, backend's Ed25519 public key)
  username, // Vec<u8> → Uint8Array (32 bytes, hashed username)
  repoCount, // u32
  ownedRepoCount,
  totalStars,
  totalCommits, // u32
  prsMerged,
  issuesClosed,
  followers,
  accountAgeDays,
  reputationLevel,
) {
  console.log("buildAndSendDevBadgeTx called");
  console.log("  signature length:", signature.length);
  console.log("  message length:", message.length);
  console.log("  publicKey length:", publicKey.length);
  console.log("  username length:", username.length);
  console.log("  repoCount:", repoCount);
  console.log("  totalCommits:", totalCommits);

  const phantom = getPhantom();
  const walletPubkey = phantom.publicKey;

  if (!walletPubkey) {
    throw new Error("Wallet not connected");
  }

  // --- Instruction 0: Ed25519 Signature Verification ---
  const ed25519Ix = solanaWeb3.Ed25519Program.createInstructionWithPublicKey({
    publicKey: new Uint8Array(publicKey),
    message: new Uint8Array(message),
    signature: new Uint8Array(signature),
  });

  // --- Instruction 1: mintDevBadge ---
  // Derive all PDAs
  const [ghostConfigPda] = findPda([toBytes("ghost_config")]);
  const [devStatePda] = findPda([toBytes("dev_state"), walletPubkey.toBytes()]);
  const [devBadgePda] = findPda([toBytes("dev_badge"), walletPubkey.toBytes()]);

  // Serialize instruction data: discriminator + username([u8;32]) + 4 * 8 + 1  = 73
  const ixData = new Uint8Array(73);
  let offset = 0;
  ixData.set(MINT_DEV_BADGE_DISCRIMINATOR, offset);
  offset += 8;
  ixData.set(new Uint8Array(username), offset);
  offset += 32;
  ixData.set(encodeU32LE(repoCount), offset);
  offset += 4;
  ixData.set(encodeU32LE(ownedRepoCount), offset);
  offset += 4;
  ixData.set(encodeU32LE(totalStars), offset);
  offset += 4;
  ixData.set(encodeU32LE(totalCommits), offset);
  offset += 4;
  ixData.set(encodeU32LE(prsMerged), offset);
  offset += 4;
  ixData.set(encodeU32LE(issuesClosed), offset);
  offset += 4;
  ixData.set(encodeU32LE(followers), offset);
  offset += 4;
  ixData.set(encodeU32LE(accountAgeDays), offset);
  offset += 4;
  ixData[offset] = reputationLevel;

  // Build account metas (must match the order in the Anchor IDL)
  const mintDevIx = new solanaWeb3.TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: walletPubkey, isSigner: true, isWritable: true }, // dev
      { pubkey: ghostConfigPda, isSigner: false, isWritable: true }, // ghost_config
      { pubkey: devStatePda, isSigner: false, isWritable: true }, // dev_badge_account
      { pubkey: devBadgePda, isSigner: false, isWritable: true }, // asset
      { pubkey: SYSTEM_PROGRAM, isSigner: false, isWritable: false }, // system_program
      { pubkey: SYSVAR_INSTRUCTIONS, isSigner: false, isWritable: false }, // instruction_sysvar
      { pubkey: MPL_CORE_PROGRAM_ID, isSigner: false, isWritable: false }, // core_program
    ],
    data: ixData,
  });

  // Build and send transaction
  const transaction = new solanaWeb3.Transaction();
  transaction.add(ed25519Ix);
  transaction.add(mintDevIx);

  const { blockhash, lastValidBlockHeight } =
    await CONNECTION.getLatestBlockhash();
  transaction.recentBlockhash = blockhash;
  transaction.feePayer = walletPubkey;

  console.log("Sending transaction to Phantom for signing...");
  const signedTx = await phantom.signTransaction(transaction);
  console.log("Transaction signed! Sending to network...");

  const txSig = await CONNECTION.sendRawTransaction(signedTx.serialize());
  console.log("Transaction sent! Signature:", txSig);

  // Wait for confirmation
  await CONNECTION.confirmTransaction({
    signature: txSig,
    blockhash: blockhash,
    lastValidBlockHeight: lastValidBlockHeight,
  });
  console.log("Transaction confirmed!");

  return txSig;
};

// ============================================================
//  MINT REPO BADGE
// ============================================================
window.buildAndSendRepoBadgeTx = async function (
  signature, // Vec<u8> → Uint8Array (64 bytes)
  message, // Vec<u8> → Uint8Array (32 bytes, hashed message)
  publicKey, // Vec<u8> → Uint8Array (32 bytes, backend pubkey)
  repoNamePadded, // Vec<u8> → Uint8Array (32 bytes)
  usernamePadded, // Vec<u8> → Uint8Array (32 bytes, hashed username)
  stars, // u32
  commits, // u32
  forks,
  openIssues,
  isFork,
  lang1, // Vec<u8> → Uint8Array
  lang2, // Vec<u8> → Uint8Array
) {
  console.log("buildAndSendRepoBadgeTx called");

  const phantom = getPhantom();
  const walletPubkey = phantom.publicKey;

  if (!walletPubkey) {
    throw new Error("Wallet not connected");
  }

  // --- Instruction 0: Ed25519 Signature Verification ---
  const ed25519Ix = solanaWeb3.Ed25519Program.createInstructionWithPublicKey({
    publicKey: new Uint8Array(publicKey),
    message: new Uint8Array(message),
    signature: new Uint8Array(signature),
  });

  // --- Instruction 1: mintRepoBadge ---
  // Derive PDAs
  const [ghostConfigPda] = findPda([toBytes("ghost_config")]);
  const [devStatePda] = findPda([toBytes("dev_state"), walletPubkey.toBytes()]);
  const [devBadgePda] = findPda([toBytes("dev_badge"), walletPubkey.toBytes()]);
  const [repoStatePda] = findPda([
    toBytes("repo_state"),
    devBadgePda.toBytes(),
    new Uint8Array(repoNamePadded),
  ]);
  const [repoBadgePda] = findPda([
    toBytes("repo_badge"),
    devBadgePda.toBytes(),
    new Uint8Array(repoNamePadded),
  ]);

  // Serialize instruction data:
  // discriminator(8) + repo_name_padded([u8;32]) + username_padded([u8;32])
  // + stars(u32) + commits(u32) + lang1(borsh bytes) + lang2(borsh bytes)
  const lang1Bytes = encodeBorshBytes(new Uint8Array(lang1));
  const lang2Bytes = encodeBorshBytes(new Uint8Array(lang2));

  const totalLen =
    8 + 32 + 32 + 4 + 4 + 4 + 4 + 1 + lang1Bytes.length + lang2Bytes.length;
  const ixData = new Uint8Array(totalLen);
  let offset = 0;

  ixData.set(MINT_REPO_BADGE_DISCRIMINATOR, offset);
  offset += 8;
  ixData.set(new Uint8Array(repoNamePadded), offset);
  offset += 32;
  ixData.set(new Uint8Array(usernamePadded), offset);
  offset += 32;
  ixData.set(encodeU32LE(stars), offset);
  offset += 4;
  ixData.set(encodeU32LE(commits), offset);
  offset += 4;
  ixData.set(encodeU32LE(forks), offset);
  offset += 4;
  ixData.set(encodeU32LE(openIssues), offset);
  offset += 4;
  ixData[offset] = isFork;
  offset += 1;
  ixData.set(lang1Bytes, offset);
  offset += lang1Bytes.length;
  ixData.set(lang2Bytes, offset);

  const mintRepoIx = new solanaWeb3.TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: walletPubkey, isSigner: true, isWritable: true }, // dev
      { pubkey: ghostConfigPda, isSigner: false, isWritable: true }, // ghost_config
      { pubkey: devStatePda, isSigner: false, isWritable: true }, // dev_state
      { pubkey: devBadgePda, isSigner: false, isWritable: true }, // dev_badge
      { pubkey: repoStatePda, isSigner: false, isWritable: true }, // repo_state
      { pubkey: repoBadgePda, isSigner: false, isWritable: true }, // repo_badge
      { pubkey: SYSVAR_INSTRUCTIONS, isSigner: false, isWritable: false }, // instruction_sysvar
      { pubkey: MPL_CORE_PROGRAM_ID, isSigner: false, isWritable: false }, // core_program
      { pubkey: SYSTEM_PROGRAM, isSigner: false, isWritable: false }, // system_program
    ],
    data: ixData,
  });

  const transaction = new solanaWeb3.Transaction();
  transaction.add(ed25519Ix);
  transaction.add(mintRepoIx);

  const { blockhash, lastValidBlockHeight } =
    await CONNECTION.getLatestBlockhash();
  transaction.recentBlockhash = blockhash;
  transaction.feePayer = walletPubkey;

  console.log("Sending repo badge transaction to Phantom for signing...");
  const signedTx = await phantom.signTransaction(transaction);
  console.log("Repo badge tx signed! Sending to network...");

  const txSig = await CONNECTION.sendRawTransaction(signedTx.serialize());
  console.log("Repo badge tx sent! Signature:", txSig);

  await CONNECTION.confirmTransaction({
    signature: txSig,
    blockhash: blockhash,
    lastValidBlockHeight: lastValidBlockHeight,
  });
  console.log("Repo badge transaction confirmed!");

  return txSig;
};

window.buildAndSendUpdateDevBadgeTx = async function (
  signature,
  message,
  publicKey,
  username,
  repoCount,
  ownedRepoCount,
  totalStars,
  totalCommits,
  prsMerged,
  issuesClosed,
  followers,
  accountAgeDays,
  reputationLevel,
) {
  console.log("buildAndSendUpdateDevBadgeTx called");
  console.log("  signature length:", signature.length);
  console.log("  message length:", message.length);
  console.log("  publicKey length:", publicKey.length);
  console.log("  username length:", username.length);
  console.log("  repoCount:", repoCount);
  console.log("  totalCommits:", totalCommits);
  console.log("  totalStars: ", totalStars);
  console.log("  prsMerged: ", prsMerged);
  console.log("  issuesClosed: ", issuesClosed);
  console.log("  followers: ", followers);
  console.log("  rank: ", reputationLevel);

  const phantom = getPhantom();
  const walletPubkey = phantom.publicKey;

  if (!walletPubkey) {
    throw new Error("Wallet not connected");
  }

  // --- Instruction 0: Ed25519 Signature Verification ---
  const ed25519Ix = solanaWeb3.Ed25519Program.createInstructionWithPublicKey({
    publicKey: new Uint8Array(publicKey),
    message: new Uint8Array(message),
    signature: new Uint8Array(signature),
  });

  // --- Instruction 1: mintDevBadge ---
  // Derive all PDAs
  const [ghostConfigPda] = findPda([toBytes("ghost_config")]);
  const [devStatePda] = findPda([toBytes("dev_state"), walletPubkey.toBytes()]);
  const [devBadgePda] = findPda([toBytes("dev_badge"), walletPubkey.toBytes()]);

  // Serialize instruction data: discriminator + username([u8;32]) + 4 * 8 + 1  = 73
  const ixData = new Uint8Array(73);
  let offset = 0;
  ixData.set(UPDATE_DEV_BADGE_DISCRIMINATOR, offset);
  offset += 8;
  ixData.set(new Uint8Array(username), offset);
  offset += 32;
  ixData.set(encodeU32LE(repoCount), offset);
  offset += 4;
  ixData.set(encodeU32LE(ownedRepoCount), offset);
  offset += 4;
  ixData.set(encodeU32LE(totalStars), offset);
  offset += 4;
  ixData.set(encodeU32LE(totalCommits), offset);
  offset += 4;
  ixData.set(encodeU32LE(prsMerged), offset);
  offset += 4;
  ixData.set(encodeU32LE(issuesClosed), offset);
  offset += 4;
  ixData.set(encodeU32LE(followers), offset);
  offset += 4;
  ixData.set(encodeU32LE(accountAgeDays), offset);
  offset += 4;
  ixData[offset] = reputationLevel;

  // Build account metas (must match the order in the Anchor IDL)
  const mintDevIx = new solanaWeb3.TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: walletPubkey, isSigner: true, isWritable: true }, // dev
      { pubkey: ghostConfigPda, isSigner: false, isWritable: true }, // ghost_config
      { pubkey: devStatePda, isSigner: false, isWritable: true }, // dev_badge_account
      { pubkey: devBadgePda, isSigner: false, isWritable: true }, // asset
      { pubkey: SYSTEM_PROGRAM, isSigner: false, isWritable: false }, // system_program
      { pubkey: SYSVAR_INSTRUCTIONS, isSigner: false, isWritable: false }, // instruction_sysvar
    ],
    data: ixData,
  });

  // Build and send transaction
  const transaction = new solanaWeb3.Transaction();
  transaction.add(ed25519Ix);
  transaction.add(mintDevIx);

  const { blockhash, lastValidBlockHeight } =
    await CONNECTION.getLatestBlockhash();
  transaction.recentBlockhash = blockhash;
  transaction.feePayer = walletPubkey;

  console.log("Sending transaction to Phantom for signing...");
  const signedTx = await phantom.signTransaction(transaction);
  console.log("Transaction signed! Sending to network...");

  const txSig = await CONNECTION.sendRawTransaction(signedTx.serialize());
  console.log("Transaction sent! Signature:", txSig);

  // Wait for confirmation
  await CONNECTION.confirmTransaction({
    signature: txSig,
    blockhash: blockhash,
    lastValidBlockHeight: lastValidBlockHeight,
  });
  console.log("Transaction confirmed!");

  return txSig;
};

// Update Repo Badge Function

window.buildAndSendUpdateRepoBadgeTx = async function (
  signature, // Vec<u8> → Uint8Array (64 bytes)
  message, // Vec<u8> → Uint8Array (32 bytes, hashed message)
  publicKey, // Vec<u8> → Uint8Array (32 bytes, backend pubkey)
  repoNamePadded, // Vec<u8> → Uint8Array (32 bytes)
  usernameHashed, // Vec<u8> → Uint8Array (32 bytes, hashed username)
  stars, // u32
  commits, // u32
  forks,
  openIssues,
  lang1, // Vec<u8> → Uint8Array
  lang2, // Vec<u8> → Uint8Array
) {
  console.log("buildAndSendRepoBadgeTx called");

  const phantom = getPhantom();
  const walletPubkey = phantom.publicKey;

  if (!walletPubkey) {
    throw new Error("Wallet not connected");
  }

  // --- Instruction 0: Ed25519 Signature Verification ---
  const ed25519Ix = solanaWeb3.Ed25519Program.createInstructionWithPublicKey({
    publicKey: new Uint8Array(publicKey),
    message: new Uint8Array(message),
    signature: new Uint8Array(signature),
  });

  // --- Instruction 1: mintRepoBadge ---
  // Derive PDAs
  const [ghostConfigPda] = findPda([toBytes("ghost_config")]);
  const [devStatePda] = findPda([toBytes("dev_state"), walletPubkey.toBytes()]);
  const [devBadgePda] = findPda([toBytes("dev_badge"), walletPubkey.toBytes()]);
  const [repoStatePda] = findPda([
    toBytes("repo_state"),
    devBadgePda.toBytes(),
    new Uint8Array(repoNamePadded),
  ]);
  const [repoBadgePda] = findPda([
    toBytes("repo_badge"),
    devBadgePda.toBytes(),
    new Uint8Array(repoNamePadded),
  ]);

  // Serialize instruction data:
  // discriminator(8) + repo_name_padded([u8;32]) + username_padded([u8;32])
  // + stars(u32) + commits(u32) + lang1(borsh bytes) + lang2(borsh bytes)
  const lang1Bytes = encodeBorshBytes(new Uint8Array(lang1));
  const lang2Bytes = encodeBorshBytes(new Uint8Array(lang2));

  const totalLen =
    8 + 32 + 32 + 4 + 4 + 4 + 4 + lang1Bytes.length + lang2Bytes.length;
  const ixData = new Uint8Array(totalLen);
  let offset = 0;

  ixData.set(UPDATE_REPO_BADGE_DISCRIMINATOR, offset);
  offset += 8;
  ixData.set(new Uint8Array(repoNamePadded), offset);
  offset += 32;
  ixData.set(new Uint8Array(usernameHashed), offset);
  offset += 32;
  ixData.set(encodeU32LE(stars), offset);
  offset += 4;
  ixData.set(encodeU32LE(commits), offset);
  offset += 4;
  ixData.set(encodeU32LE(forks), offset);
  offset += 4;
  ixData.set(encodeU32LE(openIssues), offset);
  offset += 4;
  ixData.set(lang1Bytes, offset);
  offset += lang1Bytes.length;
  ixData.set(lang2Bytes, offset);

  const mintRepoIx = new solanaWeb3.TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: walletPubkey, isSigner: true, isWritable: true }, // dev
      { pubkey: ghostConfigPda, isSigner: false, isWritable: true }, // ghost_config
      { pubkey: devStatePda, isSigner: false, isWritable: true }, // dev_state
      { pubkey: devBadgePda, isSigner: false, isWritable: true }, // dev_badge
      { pubkey: repoStatePda, isSigner: false, isWritable: true }, // repo_state
      { pubkey: repoBadgePda, isSigner: false, isWritable: true }, // repo_badge
      { pubkey: SYSVAR_INSTRUCTIONS, isSigner: false, isWritable: false }, // instruction_sysvar
      { pubkey: MPL_CORE_PROGRAM_ID, isSigner: false, isWritable: false }, // core_program
      { pubkey: SYSTEM_PROGRAM, isSigner: false, isWritable: false }, // system_program
    ],
    data: ixData,
  });

  const transaction = new solanaWeb3.Transaction();
  transaction.add(ed25519Ix);
  transaction.add(mintRepoIx);

  const { blockhash, lastValidBlockHeight } =
    await CONNECTION.getLatestBlockhash();
  transaction.recentBlockhash = blockhash;
  transaction.feePayer = walletPubkey;

  console.log("Sending repo badge transaction to Phantom for signing...");
  const signedTx = await phantom.signTransaction(transaction);
  console.log("Repo badge tx signed! Sending to network...");

  const txSig = await CONNECTION.sendRawTransaction(signedTx.serialize());
  console.log("Repo badge tx sent! Signature:", txSig);

  await CONNECTION.confirmTransaction({
    signature: txSig,
    blockhash: blockhash,
    lastValidBlockHeight: lastValidBlockHeight,
  });
  console.log("Repo badge transaction confirmed!");

  return txSig;
};

window.buildAndSendVouchTx = async function (targetAddr) {
  // get the wallet
  console.log("buildAndSendVouchForDevTx called");
  const phantom = getPhantom();
  const walletPubKey = phantom.publicKey;

  if (!walletPubKey) {
    throw new error("Wallet Not Connected");
  }

  // Derive PDAs
  const [ghostConfigPda] = findPda([toBytes("ghost_config")]);
  const [voucherDevStatePda] = findPda([
    toBytes("dev_state"),
    walletPubKey.toBytes(),
  ]);
  const [targetDevStatePda] = findPda([
    toBytes("dev_state"),
    new Uint8Array(targetAddr),
  ]);
  const [vouchRecordPda] = findPda([
    toBytes("vouch_record"),
    walletPubKey.toBytes(),
    new Uint8Array(targetAddr),
  ]);

  // Ix data
  const ixData = new Uint8Array(8 + 32);
  ixData.set(VOUCH_FOR_DEV_DISCRIMINATOR, 0);
  ixData.set(new Uint8Array(targetAddr), 8);

  // Structure the instruction
  const vouchDevIx = new solanaWeb3.TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: walletPubKey, isSigner: true, isWritable: true },
      { pubkey: ghostConfigPda, isSigner: false, isWritable: true },
      { pubkey: voucherDevStatePda, isSigner: false, isWritable: false },
      { pubkey: vouchRecordPda, isSigner: false, isWritable: true },
      { pubkey: SYSTEM_PROGRAM, isSigner: false, isWritable: false },
    ],
    data: ixData,
  });

  // Send transaction
  const transaction = new solanaWeb3.Transaction().add(vouchDevIx);
  const { blockhash, lastValidBlockHeight } =
    await CONNECTION.getLatestBlockhash();
  transaction.recentBlockhash = blockhash;
  transaction.feePayer = walletPubKey;

  console.log("Sending vouch for dev transaction to Phantom for signing...");
  const signedTx = await phantom.signTransaction(transaction);
  console.log("vouch for dev tx signed! Sending to network...");

  const txSig = await CONNECTION.sendRawTransaction(signedTx.serialize());
  console.log("Vouch for Dev Tx sent, Sig : ", txSig);

  await CONNECTION.confirmTransaction({
    signature: txSig,
    blockhash: blockhash,
    lastValidBlockHeight: lastValidBlockHeight,
  });
  console.log("Vouch for dev Transaction confirmed !");

  return txSig;
};

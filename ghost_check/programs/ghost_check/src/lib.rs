use anchor_lang::prelude::*;

pub mod instructions;
pub use instructions::*;

pub mod errors;

pub mod state;

declare_id!("GQsPhnZApw9MY7khsbRLtL5mAGpmMn8wp8CFNDPTxGQr");

#[program]
pub mod ghost_check {
    use super::*;

    pub fn init_config(ctx: Context<InitConfig>, vkey_hash: [u8; 64]) -> Result<()> {
        process_init_config(ctx, vkey_hash)
    }

    pub fn verify_proof(
        ctx: Context<VerifyProof>,
        public_inputs: [u8; 32],
        zk_proof: Vec<u8>,
    ) -> Result<()> {
        process_verify_proof(ctx, public_inputs, zk_proof)
    }
}

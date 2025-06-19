use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;

#[cfg(test)]
mod tests;

mod associated_token_account_program {
    solana_pubkey::declare_id!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
}
mod system_program {
    solana_pubkey::declare_id!("11111111111111111111111111111111");
}
pub mod token_program {
    solana_pubkey::declare_id!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
}
pub mod token_2022_program {
    solana_pubkey::declare_id!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
}

/// Derives the Associated Token Account address for the given mint address.
/// This implementation was taken from [the associated token account repository](https://github.com/solana-program/associated-token-account/blob/main/interface/src/address.rs).
pub fn get_associated_token_address(
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
    token_program_id: &Pubkey,
) -> Pubkey {
    let (program_derived_address, _bump) = Pubkey::find_program_address(
        &[
            &wallet_address.to_bytes(),
            &token_program_id.to_bytes(),
            &token_mint_address.to_bytes(),
        ],
        &associated_token_account_program::id(),
    );
    program_derived_address
}

/// Creates an instruction to run the [`AssociatedTokenAccountInstruction`](https://github.com/solana-program/associated-token-account/blob/main/interface/src/instruction.rs)
/// in the SPL Associated Token Account program.
pub fn create_associated_token_account_instruction(
    funding_address: &Pubkey,
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
    token_program_id: &Pubkey,
) -> (Pubkey, Instruction) {
    let associated_account_address =
        get_associated_token_address(wallet_address, token_mint_address, token_program_id);
    let instruction = Instruction {
        program_id: associated_token_account_program::id(),
        accounts: vec![
            AccountMeta::new(*funding_address, true),
            AccountMeta::new(associated_account_address, false),
            AccountMeta::new_readonly(*wallet_address, false),
            AccountMeta::new_readonly(*token_mint_address, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(*token_program_id, false),
        ],
        data: vec![
            0, // SPL Associated Token Account program "create" instruction
        ],
    };
    (associated_account_address, instruction)
}

/// Creates an instruction to run the [`Transfer` instruction](https://github.com/solana-program/token/blob/main/interface/src/instruction.rs)
/// in the SPL Token program.
pub fn transfer_instruction_with_program_id(
    source_address: &Pubkey,
    destination_address: &Pubkey,
    authority_address: &Pubkey,
    amount: u64,
    token_program_id: &Pubkey,
) -> Instruction {
    Instruction {
        program_id: *token_program_id,
        accounts: vec![
            AccountMeta::new(*source_address, false),
            AccountMeta::new(*destination_address, false),
            AccountMeta::new_readonly(*authority_address, true),
        ],
        data: [vec![3], amount.to_le_bytes().to_vec()].concat(), // SPL token program "transfer" instruction
    }
}

use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;

mod system_program {
    solana_pubkey::declare_id!("11111111111111111111111111111111");
}
mod token_program {
    solana_pubkey::declare_id!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
}
mod associated_token_account_program {
    solana_pubkey::declare_id!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
}

pub fn get_associated_token_address(
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
) -> Pubkey {
    let (program_derived_address, _bump) = Pubkey::find_program_address(
        &[
            &wallet_address.to_bytes(),
            &token_program::id().to_bytes(),
            &token_mint_address.to_bytes(),
        ],
        &associated_token_account_program::id(),
    );
    program_derived_address
}

pub fn create_associated_token_account_instruction(
    funding_address: &Pubkey,
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
) -> Instruction {
    let associated_account_address =
        get_associated_token_address(wallet_address, token_mint_address);
    Instruction {
        program_id: associated_token_account_program::id(),
        accounts: vec![
            AccountMeta::new(*funding_address, true),
            AccountMeta::new(associated_account_address, false),
            AccountMeta::new_readonly(*wallet_address, false),
            AccountMeta::new_readonly(*token_mint_address, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(token_program::id(), false),
        ],
        data: vec![
            0, // SPL Associated Token Account program "create" instruction
        ],
    }
}

pub fn transfer_instruction(
    source_address: &Pubkey,
    destination_address: &Pubkey,
    authority_address: &Pubkey,
    amount: u64,
) -> Instruction {
    Instruction {
        program_id: token_program::id(),
        accounts: vec![
            AccountMeta::new(*source_address, false),
            AccountMeta::new(*destination_address, false),
            AccountMeta::new_readonly(*authority_address, true),
        ],
        data: [vec![3], amount.to_le_bytes().to_vec()].concat(), // SPL token program "transfer" instruction
    }
}

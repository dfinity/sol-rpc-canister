// TODO XC-297: Once the `js` feature flag is merged in the upstream `agave-xyz/solana-sdk repository`
//  and `solana-program/associated-token-account` and `solana-program/token` are updated to use the
//  newest versions of the Solana SDK crates, this module should be removed and the code from the
//  original repositories should be used instead.
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;

mod system_program {
    solana_pubkey::declare_id!("11111111111111111111111111111111");
}

mod token_2022_program {
    solana_pubkey::declare_id!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
}

mod associated_token_account_program {
    solana_pubkey::declare_id!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
}

// TODO XC-297: Replace usages with call to `get_associated_token_address` method in
//   `spl-associated-token-account-client` crate.
/// Derives the Associated Token Account address for the given mint address.
///
/// This implementation was taken from the `spl-associated-token-account-client` crate
/// [here](https://github.com/solana-program/associated-token-account/blob/109de0bf04dc033873941c6befed2a7ab07a93d9/interface/src/address.rs#L27)
/// to force usage of the DFINITY forks for the Solana SDK crates.
pub fn get_associated_token_address(
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
) -> Pubkey {
    let (program_derived_address, _bump) = Pubkey::find_program_address(
        &[
            &wallet_address.to_bytes(),
            &token_2022_program::id().to_bytes(),
            &token_mint_address.to_bytes(),
        ],
        &associated_token_account_program::id(),
    );
    program_derived_address
}

// TODO XC-297: Replace usages with call to `create_associated_token_account` method in
//   `spl-associated-token-account-client` crate.
/// Creates an instruction to run the
/// [`Create`](https://github.com/solana-program/associated-token-account/blob/109de0bf04dc033873941c6befed2a7ab07a93d9/program/src/instruction.rs#L18)
/// instruction in the SPL Associated Token Account program.
///
/// This implementation was taken from the `spl-associated-token-account-client` crate
/// [here](https://github.com/solana-program/associated-token-account/blob/109de0bf04dc033873941c6befed2a7ab07a93d9/interface/src/instruction.rs#L39)
/// to force usage of the DFINITY forks for the Solana SDK crates.
pub fn create_associated_token_account_instruction(
    funding_address: &Pubkey,
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
) -> (Pubkey, Instruction) {
    let associated_account_address =
        get_associated_token_address(wallet_address, token_mint_address);
    let instruction = Instruction {
        program_id: associated_token_account_program::id(),
        accounts: vec![
            AccountMeta::new(*funding_address, true),
            AccountMeta::new(associated_account_address, false),
            AccountMeta::new_readonly(*wallet_address, false),
            AccountMeta::new_readonly(*token_mint_address, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(token_2022_program::id(), false),
        ],
        data: vec![
            0, // SPL Associated Token Account program "create" instruction
        ],
    };
    (associated_account_address, instruction)
}

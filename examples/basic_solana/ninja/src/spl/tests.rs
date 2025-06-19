use crate::spl::{get_associated_token_address, token_2022_program, token_program};
use solana_pubkey::{pubkey, Pubkey};
use std::str::FromStr;

// USDC token which uses the legacy Token Program: https://solscan.io/token/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
const USDC_MINT_ADDRESS: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
// BonkEarn token which uses the Token 2022 Program: https://solscan.io/token/CKfatsPMUf8SkiURsDXs7eK6GWb4Jsd6UDbs7twMCWxo
const BERN_MINT_ADDRESS: &str = "CKfatsPMUf8SkiURsDXs7eK6GWb4Jsd6UDbs7twMCWxo";

const WALLET_ADDRESS: &str = "AAAGuCgkmxYDTiBvzx1QT5XEjqXPRtQaiEXQo4gatD2o";

#[test]
fn should_compute_ata_with_legacy_token_program() {
    let associated_token_address = get_associated_token_address(
        &Pubkey::from_str(WALLET_ADDRESS).unwrap(),
        &Pubkey::from_str(USDC_MINT_ADDRESS).unwrap(),
        &token_program::id(),
    );

    // The associated token address was obtained with the following command:
    // spl-token address --owner $WALLET_ADDRESS --token $MINT_ADDRESS --verbose --url mainnet-beta
    assert_eq!(
        associated_token_address,
        pubkey!("Cra8woRQhnHsGAmFWcCN1m7A9J44ykNfGpehi6dMBuKR")
    )
}

#[test]
fn should_compute_ata_with_token_2022_program() {
    let associated_token_address = get_associated_token_address(
        &Pubkey::from_str(WALLET_ADDRESS).unwrap(),
        &Pubkey::from_str(BERN_MINT_ADDRESS).unwrap(),
        &token_2022_program::id(),
    );

    // The associated token address was obtained with the following command:
    // spl-token address --owner $WALLET_ADDRESS --token $MINT_ADDRESS --verbose --url mainnet-beta
    assert_eq!(
        associated_token_address,
        pubkey!("GPtCoaz35vdCrFbyhxcRrkYvECrUkrBX6CoRZEv8EQDw")
    )
}

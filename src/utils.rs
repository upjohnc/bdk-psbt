use super::NETWORK;
use bdk_wallet::bitcoin::bip32::Xpriv;
use bdk_wallet::keys::bip39::Mnemonic;
use bdk_wallet::template::{Bip86, DescriptorTemplate};
use bdk_wallet::KeychainKind;

pub fn wallet_descriptor_from_mnemonic(seed_phrase: &str) -> [u8; 64] {
    let mnemonic = Mnemonic::parse(seed_phrase).expect("Seed phrase");
    let seed = mnemonic.to_seed("");
    seed
}

pub fn create_descriptor(seed: [u8; 64]) -> (String, String) {
    let xprv: Xpriv = Xpriv::new_master(NETWORK, &seed).expect("master key");
    println!("master private key {}", xprv);

    let (descriptor, key_map, _) = Bip86(xprv, KeychainKind::External)
        .build(NETWORK)
        .expect("external descriptor");
    let (change_descriptor, change_key_map, _) = Bip86(xprv, KeychainKind::Internal)
        .build(NETWORK)
        .expect("internal descriptor");
    (
        descriptor.to_string_with_secret(&key_map),
        change_descriptor.to_string_with_secret(&change_key_map),
    )
}

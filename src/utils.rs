use super::{
    DB_PATH_1, DB_PATH_2, NETWORK, PARALLEL_REQUESTS, SEED_PHRASE_1, SEED_PHRASE_2, STOP_GAP,
};
use bdk_esplora::esplora_client::{BlockingClient, Builder};
use bdk_esplora::EsploraExt;
use bdk_wallet::bitcoin::bip32::Xpriv;
use bdk_wallet::bitcoin::{Address, Amount};
use bdk_wallet::chain::spk_client::{
    FullScanRequestBuilder, FullScanResponse, SyncRequestBuilder, SyncResponse,
};
use bdk_wallet::descriptor::{Descriptor, DescriptorPublicKey};
use bdk_wallet::keys::bip39::Mnemonic;
use bdk_wallet::keys::DescriptorSecretKey;
use bdk_wallet::rusqlite::Connection;
use bdk_wallet::template::{Bip86, DescriptorTemplate};
use bdk_wallet::{AddressInfo, Wallet};
use bdk_wallet::{KeychainKind, PersistedWallet, SignOptions};
use std::collections::BTreeMap;
use std::str::FromStr;

pub fn wallet_descriptor_from_mnemonic(seed_phrase: &str) -> [u8; 64] {
    let mnemonic = Mnemonic::parse(seed_phrase).expect("Seed phrase");
    let seed = mnemonic.to_seed("");
    seed
}

pub struct WalletDescriptor {
    pub external: Descriptor<DescriptorPublicKey>,
    pub ext_key_map: BTreeMap<DescriptorPublicKey, DescriptorSecretKey>,
    pub internal: Descriptor<DescriptorPublicKey>,
    pub int_key_map: BTreeMap<DescriptorPublicKey, DescriptorSecretKey>,
}

impl WalletDescriptor {
    pub fn new(
        external: Descriptor<DescriptorPublicKey>,
        ext_key_map: BTreeMap<DescriptorPublicKey, DescriptorSecretKey>,
        internal: Descriptor<DescriptorPublicKey>,
        int_key_map: BTreeMap<DescriptorPublicKey, DescriptorSecretKey>,
    ) -> Self {
        Self {
            external,
            ext_key_map,
            internal,
            int_key_map,
        }
    }

    pub fn descriptor_string(&self) -> (String, String) {
        (
            self.external.to_string_with_secret(&self.ext_key_map),
            self.internal.to_string_with_secret(&self.int_key_map),
        )
    }
}

pub fn create_descriptor(seed: [u8; 64]) -> WalletDescriptor {
    let xprv: Xpriv = Xpriv::new_master(NETWORK, &seed).expect("master key");
    // println!("master private key {}", xprv);

    let (descriptor, key_map, _) = Bip86(xprv, KeychainKind::External)
        .build(NETWORK)
        .expect("external descriptor");
    let (change_descriptor, change_key_map, _) = Bip86(xprv, KeychainKind::Internal)
        .build(NETWORK)
        .expect("internal descriptor");
    WalletDescriptor::new(descriptor, key_map, change_descriptor, change_key_map)
}

pub fn get_wallet(
    wallet_descriptor: WalletDescriptor,
    db_path: &str,
) -> PersistedWallet<Connection> {
    let mut connection = Connection::open(db_path).expect("dbpath");
    let (priv_ext, priv_int) = wallet_descriptor.descriptor_string();
    let wallet_opt = Wallet::load()
        .descriptor(KeychainKind::External, Some(priv_ext.clone()))
        .descriptor(KeychainKind::Internal, Some(priv_int.clone()))
        .extract_keys()
        .check_network(NETWORK)
        .load_wallet(&mut connection)
        .expect("wallet opt created");

    let (mut wallet, is_new_wallet) = if let Some(load_wallet) = wallet_opt {
        (load_wallet, false)
    } else {
        (
            Wallet::create(priv_ext, priv_int)
                .network(NETWORK)
                .create_wallet(&mut connection)
                .expect("persisted wallet"),
            true,
        )
    };

    let client: BlockingClient = Builder::new("https://mutinynet.com/api").build_blocking();

    match is_new_wallet {
        true => {
            println!("Full scan");
            let full_scan_request: FullScanRequestBuilder<KeychainKind> = wallet.start_full_scan();
            let update: FullScanResponse<KeychainKind> = client
                .full_scan(full_scan_request, STOP_GAP, PARALLEL_REQUESTS)
                .expect("full scan");
            wallet.apply_update(update).unwrap();
            wallet.persist(&mut connection).expect("save to db");
        }
        false => {
            println!("Sync");
            let sync_request: SyncRequestBuilder<(KeychainKind, u32)> =
                wallet.start_sync_with_revealed_spks();
            let update: SyncResponse = client
                .sync(sync_request, PARALLEL_REQUESTS)
                .expect("create udpate");
            wallet.apply_update(update).unwrap();
            wallet.persist(&mut connection).expect("save to db");
        }
    }

    wallet
}

pub fn get_address(wallet: &mut PersistedWallet<Connection>, db_path: &str) -> AddressInfo {
    let address: AddressInfo = wallet.reveal_next_address(KeychainKind::External);

    let mut connection = Connection::open(db_path).expect("dbpath");
    wallet.persist(&mut connection).expect("good persist");
    address
}

#[allow(dead_code)]
pub fn send_from_two_wallets() {
    let seed_1 = wallet_descriptor_from_mnemonic(SEED_PHRASE_1);
    let wallet_descriptor = create_descriptor(seed_1);
    let mut wallet_1 = get_wallet(wallet_descriptor, DB_PATH_1);

    let seed_2 = wallet_descriptor_from_mnemonic(SEED_PHRASE_2);
    let wallet_descriptor = create_descriptor(seed_2);

    let wallet_2 = get_wallet(wallet_descriptor, DB_PATH_2);
    let address = Address::from_str("tb1qd28npep0s8frcm3y7dxqajkcy2m40eysplyr9v")
        .unwrap()
        .require_network(NETWORK)
        .unwrap();

    let utxo = wallet_2.list_unspent().next().unwrap();
    let psbt_input = wallet_2.get_psbt_input(utxo.clone(), None, false).unwrap();
    let foreign_utxo_satisfaction = wallet_2
        .public_descriptor(KeychainKind::External)
        .max_weight_to_satisfy()
        .unwrap();

    assert!(
        psbt_input.non_witness_utxo.is_none(),
        "`non_witness_utxo` should never be populated for taproot"
    );

    let mut builder = wallet_1.build_tx();
    builder
        .add_recipient(address.script_pubkey(), Amount::from_sat(35_000))
        .add_foreign_utxo(utxo.outpoint, psbt_input, foreign_utxo_satisfaction)
        .unwrap();
    let mut psbt = builder.finish().expect("finished psbt build");
    let _ = wallet_2
        .sign(&mut psbt, SignOptions::default())
        .expect("sign fine");
    let finalized = wallet_1
        .sign(&mut psbt, SignOptions::default())
        .expect("sign fine");
    assert!(finalized);

    let tx = psbt.extract_tx().expect("extract the transaction");
    let client: BlockingClient = Builder::new("https://mutinynet.com/api").build_blocking();
    let _ = client.broadcast(&tx);
    dbg!(tx.compute_txid());
}

mod utils;
use bdk_esplora::esplora_client::{BlockingClient, Builder};
use bdk_wallet::bitcoin::{Address, Amount, Network, Weight};
use bdk_wallet::{KeychainKind, SignOptions};
use std::str::FromStr;

const STOP_GAP: usize = 50;
const PARALLEL_REQUESTS: usize = 1;
const DB_PATH_1: &str = "./sqlite_db_wallet";
const DB_PATH_2: &str = "./sqlite_db_wallet_2";
const SEED_PHRASE_1: &str =
    "execute grunt bullet spawn panther until paper receive prison midnight tower orphan";
const SEED_PHRASE_2: &str =
    "rebel secret wide garment post onion amazing push inherit record exotic fold";
const NETWORK: Network = Network::Signet;

fn main() {
    println!("Start\n");
    let seed_1 = utils::wallet_descriptor_from_mnemonic(SEED_PHRASE_1);
    let wallet_descriptor = utils::create_descriptor(seed_1);
    let mut wallet_1 = utils::get_wallet(wallet_descriptor, DB_PATH_1);

    let seed_2 = utils::wallet_descriptor_from_mnemonic(SEED_PHRASE_2);
    let wallet_descriptor = utils::create_descriptor(seed_2);

    let wallet_2 = utils::get_wallet(wallet_descriptor, DB_PATH_2);
    let address = Address::from_str("tb1qd28npep0s8frcm3y7dxqajkcy2m40eysplyr9v")
        .unwrap()
        .require_network(Network::Signet)
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

    println!("\nEnd");
}

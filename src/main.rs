mod utils;
use bdk_wallet::bitcoin::{psbt::Input, Address, Amount, Network, Weight};
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
    let (ext_descriptor, int_descriptor) = wallet_descriptor.descriptor_string();

    // println!(
    //     "----------------  Descriptors  ------------------------------\nPrivate Key, External:\n{:?}\nPrivate Key, Internal:\n{:?}\n",
    //     ext_descriptor, int_descriptor,
    // );
    let mut wallet_1 = utils::get_wallet(wallet_descriptor, DB_PATH_1);
    // let mut builder_1 = wallet_1.build_tx();
    // let address = Address::from_str("tb1qd28npep0s8frcm3y7dxqajkcy2m40eysplyr9v")
    //     .unwrap()
    //     .require_network(Network::Signet)
    //     .unwrap();

    // let send_amount: Amount = Amount::from_sat(5000);
    // builder_1.add_recipient(address.script_pubkey(), send_amount);

    // let mut psbt_1 = builder_1.finish().expect("psbt_1");
    // dbg!(psbt_1.unsigned_tx);

    // // println!("address: {}", utils::get_address(&mut wallet_1, DB_PATH_1).address);
    // println!("wallet amount: {}", wallet_1.balance().total().to_sat());
    // for i in wallet_1.list_unspent() {
    //     println!("{:?}", i);
    // }

    let seed_2 = utils::wallet_descriptor_from_mnemonic(SEED_PHRASE_2);
    let wallet_descriptor = utils::create_descriptor(seed_2);
    let (ext_descriptor, int_descriptor) = wallet_descriptor.descriptor_string();

    // println!(
    //     "----------------  Descriptors  ------------------------------\nPrivate Key, External:\n{:?}\nPrivate Key, Internal:\n{:?}\n",
    //     ext_descriptor, int_descriptor,
    // );
    let mut wallet_2 = utils::get_wallet(wallet_descriptor, DB_PATH_2);
    // let mut builder_2 = wallet_2.build_tx();
    let address = Address::from_str("tb1qd28npep0s8frcm3y7dxqajkcy2m40eysplyr9v")
        .unwrap()
        .require_network(Network::Signet)
        .unwrap();

    let send_amount: Amount = Amount::from_sat(5000);
    // builder_2.add_recipient(address.script_pubkey(), send_amount);

    // let mut psbt_2 = builder_2.finish().expect("psbt_2");
    // dbg!(psbt_2.unsigned_tx);
    // println!(
    //     "address: {}",
    //     utils::get_address(&mut wallet_2, DB_PATH_2).address
    // );
    // println!("wallet amount: {}", wallet_2.balance().total().to_sat());
    // let seed_1 = utils::wallet_descriptor_from_mnemonic(SEED_PHRASE_1);
    // let wallet_descriptor = utils::create_descriptor(seed_1);
    // let mut wallet_1 = utils::get_wallet(wallet_descriptor, DB_PATH_1);
    // let i = wallet_1.list_unspent().next().expect("one item at least");
    // // // for i in wallet_2.list_unspent() {
    // let mut t = Input::default();

    // t.witness_utxo = Some(i.txout);
    // let z = i.outpoint;

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
        .add_recipient(address.script_pubkey(), Amount::from_sat(5_000))
        .add_foreign_utxo(utxo.outpoint, psbt_input, foreign_utxo_satisfaction)
        .unwrap();
    // let w = builder_2
    //     .add_foreign_utxo(z, t, Weight::ZERO)
    //     .expect("bulding well");
    // builder_2
    // .add_recipient(address.script_pubkey(), send_amount)
    // let mut builder = wallet1.build_tx();
    // builder_2
    //     .add_recipient(address.script_pubkey(), send_amount)
    //     .add_foreign_utxo(z, t, Weight::ZERO)
    //     // .add_foreign_utxo(utxo.outpoint, psbt_input, foreign_utxo_satisfaction)
    //     .unwrap();
    let mut result = builder.finish().expect("finished psbt build");
    let _ = wallet_2
        .sign(&mut result, SignOptions::default())
        .expect("sign fine");
    let _ = wallet_1
        .sign(&mut result, SignOptions::default())
        .expect("sign fine");
    dbg!(result);

    // println!("{:?}", z);
    //     // println!("{:?}", i.outpoint);
    // println!(
    //     "{:?}",
    //     t,
    //     // Input {
    //     //     witness_utxq: Some(i.txout)
    //     // }
    // );
    //     // println!("{:?}", i);
    // }
    println!("\nEnd");
}

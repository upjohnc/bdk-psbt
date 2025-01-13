mod utils;
use bdk_wallet::bitcoin::Network;

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
    // println!("address: {}", utils::get_address(&mut wallet_1, DB_PATH_1).address);
    println!("wallet amount: {}", wallet_1.balance().total().to_sat());

    let seed_2 = utils::wallet_descriptor_from_mnemonic(SEED_PHRASE_2);
    let wallet_descriptor = utils::create_descriptor(seed_2);
    let (ext_descriptor, int_descriptor) = wallet_descriptor.descriptor_string();

    // println!(
    //     "----------------  Descriptors  ------------------------------\nPrivate Key, External:\n{:?}\nPrivate Key, Internal:\n{:?}\n",
    //     ext_descriptor, int_descriptor,
    // );
    let mut wallet_2 = utils::get_wallet(wallet_descriptor, DB_PATH_2);
    // println!(
    //     "address: {}",
    //     utils::get_address(&mut wallet_2, DB_PATH_2).address
    // );
    println!("wallet amount: {}", wallet_2.balance().total().to_sat());
    println!("\nEnd");
}

mod utils;
use bdk_wallet::bitcoin::Network;
use clap::{Parser, Subcommand};
use std::process;

const STOP_GAP: usize = 50;
const PARALLEL_REQUESTS: usize = 1;
const DB_PATH_1: &str = "./sqlite_db_wallet";
const DB_PATH_2: &str = "./sqlite_db_wallet_2";
const SEED_PHRASE_1: &str =
    "execute grunt bullet spawn panther until paper receive prison midnight tower orphan";
const SEED_PHRASE_2: &str =
    "rebel secret wide garment post onion amazing push inherit record exotic fold";
const NETWORK: Network = Network::Signet;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Balance {
        #[arg(short, long)]
        wallet_number: u8,
    },
    NextAddress {
        #[arg(short, long)]
        wallet_number: u8,
    },
}

fn check_wallet_number(w: &u8) -> (&str, &str) {
    if ![1, 2].contains(&w) {
        println!("Choices are 1 or 2");
        process::exit(1);
    };
    let (seed, db_path) = match w {
        1 => (SEED_PHRASE_1, DB_PATH_1),
        2 => (SEED_PHRASE_2, DB_PATH_2),
        _ => ("not good", "not good"),
    };
    (seed, db_path)
}

// send utxo to an address
// send utxo back to mutiny
fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Balance {
            wallet_number: wallet,
        }) => {
            let (seed, db_path) = check_wallet_number(&wallet);
            println!("Wallet chosen {}", wallet);
            let seed = utils::wallet_descriptor_from_mnemonic(seed);
            let wallet_descriptor = utils::create_descriptor(seed);
            let wallet = utils::get_wallet(wallet_descriptor, db_path);
            println!(
                "Wallet confirmed balance: {}",
                wallet.balance().confirmed.to_sat()
            );
        }
        Some(Commands::NextAddress {
            wallet_number: wallet,
        }) => {
            let (seed, db_path) = check_wallet_number(&wallet);
            let seed = utils::wallet_descriptor_from_mnemonic(seed);
            let wallet_descriptor = utils::create_descriptor(seed);
            let mut wallet = utils::get_wallet(wallet_descriptor, db_path);
            let address = utils::get_address(&mut wallet, db_path);
            println!("Next Address:\n{}", address);
        }
        None => {
            println!("go for it");
        }
    }
}

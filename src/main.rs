mod utils;
use bdk_wallet::bitcoin::Network;

const SEED_PHRASE_1: &str =
    "execute grunt bullet spawn panther until paper receive prison midnight tower orphan";
const SEED_PHRASE_2: &str =
    "palm outer embrace test escape spend monster lunar inject taxi derive jealous";
const NETWORK: Network = Network::Signet;

fn main() {
    println!("Start\n");
    let seed = utils::wallet_descriptor_from_mnemonic(SEED_PHRASE_1);
    let (descriptor, change_descriptor) = utils::create_descriptor(seed);

    println!(
        "----------------  Descriptors  ------------------------------\nPrivate Key, External:\n{:?}\nPrivate Key, Internal:\n{:?}\n",
        descriptor, // privkey
        change_descriptor,
    );
    println!("\nEnd");
}

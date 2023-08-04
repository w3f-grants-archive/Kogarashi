use clap::{Parser, Subcommand};
use hex::FromHex;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use substrate_rpc::Wallet;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// init redjubjub wallet
    Init,
    /// get balance
    Balance { address: Option<String> },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init) => {
            println!("Start Wallet Generation...");
            let wallet = Wallet::generate();
            let mut file = File::create("key.kog").expect("fail to create key file");
            file.write_all(&wallet.seed()).expect("fail to store key");
            println!("Your SS58 Address: {:?}", wallet.public().to_string());
            println!("Your Wallet ID: {:?}", wallet.to_account_id());
            println!("Your Wallet Secret: {:?}", wallet.seed());
        }
        Some(Commands::Balance { address }) => match address {
            Some(x) => {
                println!("Get {:?} Balance", x);
            }
            None => {
                let mut f = File::open("key.kog").unwrap();
                let mut secret = vec![];
                f.read_to_end(&mut secret).unwrap();
                let seed: [u8; 32] = secret[..32].try_into().unwrap();
                let wallet = Wallet::from_seed(seed);
                println!("Your SS58 Address: {:?}", wallet.public().to_string());
                println!("Your Wallet ID: {:?}", wallet.to_account_id());
                println!("Your Wallet Secret: {:?}", wallet.seed());
            }
        },
        None => {}
    }
}

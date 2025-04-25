use clap::{command, Parser};
use client::MTreeClient;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use std::{fs, process::exit};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Args {
    /// Insert a leaf into the Merkle tree
    InsertLeaf(InsertLeafArgs),
    RootHash(RootHashArgs),
}

impl Args {
    fn program_id(&self) -> Pubkey {
        match self {
            Args::InsertLeaf(args) => args.program_id,
            Args::RootHash(args) => args.program_id,
        }
    }
    fn config_file(&self) -> Option<&str> {
        match self {
            Args::InsertLeaf(args) => args.config_file.as_deref(),
            Args::RootHash(args) => args.config_file.as_deref(),
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct InsertLeafArgs {
    #[arg(short, long)]
    program_id: Pubkey,
    #[arg(short, long)]
    config_file: Option<String>,
    data: String,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct RootHashArgs {
    #[arg(short, long)]
    program_id: Pubkey,
    #[arg(short, long)]
    config_file: Option<String>,
}

fn main() {
    let args = Args::parse();
    let program_id = args.program_id();
    let config = get_cli_config(args.config_file());
    let key = get_key_pair_from_config(&config);
    let client = MTreeClient::new(program_id, &config.json_rpc_url);

    match args {
        Args::InsertLeaf(args) => {
            let data = if args.data.starts_with("0x") {
                hex::decode(&args.data[2..]).unwrap_or_else(|_| {
                    eprintln!("error: Invalid hex string");
                    exit(1);
                })
            } else {
                args.data.as_bytes().to_vec()
            };

            let tx = client.insert_leaf(&key, data).unwrap_or_else(|err| {
                eprintln!("error: Failed to insert leaf:{:#}", err);
                exit(1);
            });
            println!("Transaction signature: {:#}", tx);

            let root_hash = client.get_tx_root_hash(tx).unwrap_or_else(|err| {
                eprintln!("error: Failed to get transaction root hash: {:#}", err);
                exit(1);
            });
            println!("Root hash: {:#}", hex::encode(root_hash));
        }
        Args::RootHash(_) => {
            let hash = client.get_root_hash().unwrap_or_else(|err| {
                eprintln!("error: Failed to get root hash: {:#}", err);
                exit(1);
            });
            println!("Root hash: {:#}", hex::encode(hash));
        }
    }
}

fn get_key_pair_from_config(config: &solana_cli_config::Config) -> Keypair {
    let path = &config.keypair_path;
    let key = fs::read_to_string(path).unwrap_or_else(|_| {
        eprintln!("error: Could not read keypair file `{}`", path);
        exit(1);
    });
    let key_bytes: Vec<u8> = serde_json::from_str(&key).unwrap_or_else(|_| {
        eprintln!("error: Invalid keypair file format");
        exit(1);
    });

    Keypair::from_bytes(&key_bytes).unwrap_or_else(|_| {
        eprintln!("error: Invalid keypair bytes");
        exit(1);
    })
}

fn get_cli_config(config_file: Option<&str>) -> solana_cli_config::Config {
    if let Some(config_file) = config_file {
        solana_cli_config::Config::load(config_file).unwrap_or_else(|_| {
            eprintln!("error: Could not find config file `{}`", config_file);
            exit(1);
        })
    } else if let Some(config_file) = &*solana_cli_config::CONFIG_FILE {
        solana_cli_config::Config::load(config_file).unwrap_or_default()
    } else {
        solana_cli_config::Config::default()
    }
}

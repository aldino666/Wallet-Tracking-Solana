use std::env;
use std::str::FromStr;
use solana_sdk::pubkey::Pubkey;
use anyhow::Context;

mod monitor;
mod parser;
mod token;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run -- <wallet_address>");
        println!("Exemple: cargo run -- 56S29mZ3wqvw8hATuUUFqKhGcSGYFASRRFNT38W8q7G3");
        return Ok(());
    }

    let target_wallet = Pubkey::from_str(&args[1])
        .context("Invalid wallet address provided")?;

    let rpc_url = "https://api.mainnet-beta.solana.com";
    let ws_url = "wss://api.mainnet-beta.solana.com";

    let monitor = monitor::WalletMonitor::new(rpc_url, ws_url, target_wallet)?;
    log::info!("Monitoring wallet: {}", target_wallet);
    
    // Lancement de l'écoute
    monitor.start().await?; 

    Ok(())
}
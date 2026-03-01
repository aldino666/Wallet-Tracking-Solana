use solana_client::{pubsub_client::PubsubClient, rpc_client::RpcClient};
use solana_sdk::{pubkey::Pubkey, commitment_config::CommitmentConfig};
use solana_client::rpc_config::{RpcTransactionConfig, RpcTransactionLogsConfig, RpcTransactionLogsFilter};
use std::sync::Arc;
use tokio::sync::Semaphore;
use crate::parser::{parse_transaction, ActionType};
use crate::token::fetch_token_info;
use solana_transaction_status::UiTransactionEncoding;
use std::str::FromStr;

pub struct WalletMonitor {
    pub rpc_client: Arc<RpcClient>,
    pub ws_url: String,
    pub target_wallet: Pubkey,
    pub semaphore: Arc<Semaphore>,
}

impl WalletMonitor {
    pub fn new(rpc_url: &str, ws_url: &str, target_wallet: Pubkey) -> anyhow::Result<Self> {
        let rpc_client = Arc::new(RpcClient::new(rpc_url.to_string()));
        // Limite à 10 requêtes RPC simultanées pour éviter le rate-limiting
        let semaphore = Arc::new(Semaphore::new(10)); 
        Ok(Self {
            rpc_client,
            ws_url: ws_url.to_string(),
            target_wallet,
            semaphore,
        })
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        log::info!("Starting multithreaded monitor for wallet: {}", self.target_wallet);

        let (_unsub, receiver) = PubsubClient::logs_subscribe(
            &self.ws_url,
            RpcTransactionLogsFilter::Mentions(vec![self.target_wallet.to_string()]),
            RpcTransactionLogsConfig {
                commitment: Some(CommitmentConfig::confirmed()),
            },
        )?;

        while let Ok(log_response) = receiver.recv() {
            let signature = log_response.value.signature;
            let rpc_client = Arc::clone(&self.rpc_client);
            let target_wallet = self.target_wallet;
            let semaphore = Arc::clone(&self.semaphore);

            // Spawn un nouveau thread (task) pour traiter la transaction en parallèle
            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                
                if let Err(e) = process_transaction(rpc_client, signature, target_wallet).await {
                    log::error!("Error processing transaction: {}", e);
                }
            });
        }
        Ok(())
    }
}

async fn process_transaction(rpc_client: Arc<RpcClient>, signature: String, target_wallet: Pubkey) -> anyhow::Result<()> {
    let config = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::JsonParsed),
        commitment: Some(CommitmentConfig::confirmed()),
        max_supported_transaction_version: Some(0),
    };

    let tx_sig = signature.parse()?;
    let tx = rpc_client.get_transaction_with_config(&tx_sig, config)?;
    
    let actions = parse_transaction(&tx, &target_wallet);
    for action in actions {
        let mint_pubkey = Pubkey::from_str(&action.token_mint).unwrap_or_default();
        let token_info = fetch_token_info(&rpc_client, &mint_pubkey).await;
        
        let symbol = match token_info {
            Ok(info) => info.symbol,
            Err(_) => "Unknown".to_string(),
        };

        let action_str = match action.action {
            ActionType::Buy => "🟢 BUY",
            ActionType::Sell => "🔴 SELL",
            _ => "UNKNOWN",
        };

        log::info!("Action: {} | Token: {} ({}) | Amount: {} | DEX: {} | Signature: {}", 
            action_str, symbol, action.token_mint, action.amount, action.dex, signature);
    }

    Ok(())
}
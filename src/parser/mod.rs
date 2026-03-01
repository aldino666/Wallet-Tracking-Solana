use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, option_serializer::OptionSerializer};

pub const RAYDIUM_AMM_V4: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
pub const ORCA_SWAP_V2: &str = "9W959DqmcGTu2kC9SghpCC2JzxeH9M89S6s5zX6U9L1i";
pub const METEORA_AMM: &str = "Eo7WjKq67rjJQSvSDB9UpXyP3od729fmH799yB5BpqLd";
pub const PHOENIX_V1: &str = "PhoeNiXNJ9bhP7th6vXgAn9N9h7Hym4atpZuhS7Ppt1";

#[derive(Debug, PartialEq, Clone)]
pub enum ActionType { Buy, Sell, LiquidityAdd, Transfer, Unknown }

#[derive(Debug, Clone)]
pub struct DecodedAction {
    pub action: ActionType,
    pub token_mint: String,
    pub amount: f64,
    pub dex: String,
}

pub fn parse_transaction(tx: &EncodedConfirmedTransactionWithStatusMeta, target_wallet: &Pubkey) -> Vec<DecodedAction> {
    let mut actions = Vec::new();
    let meta = match &tx.transaction.meta { Some(m) => m, None => return actions };
    let target_wallet_str = target_wallet.to_string();

    if let (OptionSerializer::Some(pre_balances), OptionSerializer::Some(post_balances)) = 
           (&meta.pre_token_balances, &meta.post_token_balances) {
        
        for post in post_balances {
            if let OptionSerializer::Some(owner) = &post.owner {
                if owner == &target_wallet_str {
                    let pre = pre_balances.iter().find(|p| p.mint == post.mint && p.owner == post.owner);
                    let pre_amount = pre.map(|p| p.ui_token_amount.ui_amount.unwrap_or(0.0)).unwrap_or(0.0);
                    let post_amount = post.ui_token_amount.ui_amount.unwrap_or(0.0);
                    let diff = post_amount - pre_amount;

                    if diff.abs() > 0.000001 {
                        let action = if diff > 0.0 { ActionType::Buy } else { ActionType::Sell };
                        let dex = identify_dex(&meta.log_messages);

                        actions.push(DecodedAction {
                            action,
                            token_mint: post.mint.clone(),
                            amount: diff.abs(),
                            dex,
                        });
                    }
                }
            }
        }
    }
    actions
}

fn identify_dex(logs: &OptionSerializer<Vec<String>>) -> String {
    if let OptionSerializer::Some(logs) = logs {
        for log in logs {
            if log.contains(RAYDIUM_AMM_V4) { return "Raydium".to_string(); }
            if log.contains(ORCA_SWAP_V2) { return "Orca".to_string(); }
            if log.contains(METEORA_AMM) { return "Meteora".to_string(); }
            if log.contains(PHOENIX_V1) { return "Phoenix".to_string(); }
        }
    }
    "Unknown DEX".to_string()
}
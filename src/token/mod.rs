use solana_sdk::pubkey::Pubkey;
use solana_client::rpc_client::RpcClient;
use mpl_token_metadata::accounts::Metadata;
use solana_sdk::account::Account;

pub enum TokenType { StandardSPL, Token2022, Unknown }
pub struct TokenInfo { pub mint: Pubkey, pub token_type: TokenType, pub name: String, pub symbol: String }

pub fn get_token_type(account: &Account) -> TokenType {
    if account.owner == spl_token::id() { TokenType::StandardSPL } 
    else if account.owner == spl_token_2022::id() { TokenType::Token2022 } 
    else { TokenType::Unknown }
}

pub async fn fetch_token_info(rpc_client: &RpcClient, mint_pubkey: &Pubkey) -> anyhow::Result<TokenInfo> {
    let account = rpc_client.get_account(mint_pubkey)?;
    let token_type = get_token_type(&account);

    let (name, symbol) = match token_type {
        TokenType::StandardSPL | TokenType::Token2022 => {
            let (metadata_pubkey, _) = Metadata::find_pda(mint_pubkey);
            if let Ok(metadata_account) = rpc_client.get_account(&metadata_pubkey) {
                if let Ok(metadata) = Metadata::from_bytes(&metadata_account.data) {
                    (metadata.name.trim_matches(char::from(0)).to_string(), 
                     metadata.symbol.trim_matches(char::from(0)).to_string())
                } else { ("Unknown".to_string(), "???".to_string()) }
            } else { ("Unknown".to_string(), "???".to_string()) }
        }
        _ => ("Not a Token".to_string(), "N/A".to_string()),
    };

    Ok(TokenInfo { mint: *mint_pubkey, token_type, name, symbol })
}
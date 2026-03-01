use crate::parser::{parse_transaction, ActionType};
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction, UiTransaction, UiMessage,
    UiRawMessage, UiTransactionStatusMeta, option_serializer::OptionSerializer,
    EncodedTransactionWithStatusMeta, UiTransactionTokenBalance
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_buy_action() {
        let target_wallet = Pubkey::new_unique();
        let target_wallet_str = target_wallet.to_string();
        let mint_str = "EPjFW3F2DB2S6MNitvXqST8no679G3fG1Qv1t493XARy"; // USDC

        let pre_token_balances = vec![
            UiTransactionTokenBalance {
                account_index: 0,
                mint: mint_str.to_string(),
                owner: OptionSerializer::Some(target_wallet_str.clone()),
                ui_token_amount: solana_account_decoder::parse_token::UiTokenAmount {
                    ui_amount: Some(100.0), decimals: 6, amount: "100".to_string(), ui_amount_string: "100".to_string(),
                },
                program_id: OptionSerializer::None,
            }
        ];

        let post_token_balances = vec![
            UiTransactionTokenBalance {
                account_index: 0,
                mint: mint_str.to_string(),
                owner: OptionSerializer::Some(target_wallet_str.clone()),
                ui_token_amount: solana_account_decoder::parse_token::UiTokenAmount {
                    ui_amount: Some(150.0), decimals: 6, amount: "150".to_string(), ui_amount_string: "150".to_string(),
                },
                program_id: OptionSerializer::None,
            }
        ];
        
        let meta = UiTransactionStatusMeta {
            err: None, status: Ok(()), fee: 5000,
            pre_balances: vec![], post_balances: vec![],
            inner_instructions: OptionSerializer::None,
            log_messages: OptionSerializer::Some(vec![
                "Program 675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8 invoke [1]".to_string(),
                "Program log: Instruction: Swap".to_string(),
            ]),
            pre_token_balances: OptionSerializer::Some(pre_token_balances),
            post_token_balances: OptionSerializer::Some(post_token_balances),
            rewards: OptionSerializer::None, loaded_addresses: OptionSerializer::None,
            return_data: OptionSerializer::None, compute_units_consumed: OptionSerializer::None,
        };

        let tx = EncodedConfirmedTransactionWithStatusMeta {
            slot: 1,
            transaction: EncodedTransactionWithStatusMeta {
                transaction: EncodedTransaction::Json(UiTransaction {
                    signatures: vec!["mock_sig".to_string()],
                    message: UiMessage::Raw(UiRawMessage {
                        header: solana_sdk::message::MessageHeader {
                            num_required_signatures: 1, num_readonly_signed_accounts: 0, num_readonly_unsigned_accounts: 0,
                        },
                        account_keys: vec![target_wallet.to_string()],
                        recent_blockhash: "mock_blockhash".to_string(),
                        instructions: vec![], address_table_lookups: None,
                    }),
                }),
                meta: Some(meta), version: None,
            },
            block_time: Some(123456789),
        };

        let actions = parse_transaction(&tx, &target_wallet);
        assert!(!actions.is_empty());
        assert_eq!(actions[0].action, ActionType::Buy);
    }
}
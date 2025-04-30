use core::fmt;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use solana_transaction_error::TransactionError;

use crate::{BASE_FEE, NATIVE_PROGRAMS, TokenAmount};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NativeAccount {
    pub address: String,
    pub index: u8,
    pub pre_balance: u64,
    pub post_balance: u64,
    pub amount_transacted: u64,
    pub spam: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParsedTokenAccount {
    pub ata_address: String,
    pub account_index: u8,
    pub pre_token_balance_amount: u64,
    pub post_token_balance_amount: u64,
    pub mint_decimals: u8,
    pub amount_transacted: u64,
    pub low_value: bool,
    pub ui_string_amount: String,
}

#[derive(Serialize, Deserialize)]
pub struct LogMessage {
    pub message: String,
    pub has_invalid_chars: bool,
    pub invalid_chars: Vec<(u16, char)>,
}

#[derive(Serialize, Deserialize)]
pub struct RpcTransactionOutcome {
    pub signer: NativeAccount,
    pub native_accounts: Vec<NativeAccount>,
    pub token_accounts: HashMap<String, ParsedTokenAccount>,
    pub logs: Vec<LogMessage>,
}

impl RpcTransactionOutcome {
    pub fn logs(&self) -> &[LogMessage] {
        self.logs.as_slice()
    }

    fn process_native(
        tx: &RpcTransaction,
        token_accounts: &HashMap<String, ParsedTokenAccount>,
        mints: &[String],
    ) -> Vec<NativeAccount> {
        let mut accounts = Vec::<NativeAccount>::new();

        tx.transaction
            .message
            .account_keys
            .iter()
            .skip(1)
            .enumerate()
            .for_each(|(index, account)| {
                if !NATIVE_PROGRAMS
                    .iter()
                    .any(|native_account| native_account.as_bytes() == account.as_bytes())
                    && token_accounts.get(account).is_none()
                    && mints.binary_search(account).is_err()
                {
                    let pre_balance = tx.meta.pre_balances[index];
                    let post_balance = tx.meta.post_balances[index];
                    let amount_transacted = post_balance.saturating_sub(pre_balance);

                    // Most operations require transaction amounts of more than
                    // 5000 lamports which is also higher that the rent required for rent
                    // exemption or creating an ATA.
                    let spam = amount_transacted < BASE_FEE;

                    let set_outcome = NativeAccount {
                        address: account.clone(),
                        index: index as u8,
                        pre_balance,
                        post_balance,
                        amount_transacted,
                        spam,
                    };

                    accounts.push(set_outcome);
                }
            });

        accounts
    }

    pub fn parse(tx: RpcTransaction) -> Self {
        let pre_balance = tx.meta.pre_balances[0];
        let post_balance = tx.meta.post_balances[0];
        let amount_transacted = pre_balance.saturating_sub(post_balance);

        let signer = NativeAccount {
            address: tx.transaction.message.account_keys[0].clone(),
            index: 0u8,
            pre_balance,
            post_balance,
            amount_transacted,
            spam: false,
        };
        let mut token_accounts = HashMap::<String, ParsedTokenAccount>::new();
        let mut mints = Vec::<String>::new();

        tx.meta
            .pre_token_balances
            .iter()
            .enumerate()
            .for_each(|(index, token_balance)| {
                mints.push(token_balance.mint.clone());
                mints.dedup();

                let ata_address = tx.transaction.message.account_keys
                    [token_balance.account_index as usize]
                    .clone();
                let pre_token_balance = token_balance.ui_token_amount.ui_amount;
                let post_token_balance =
                    tx.meta.post_token_balances[index].ui_token_amount.ui_amount;
                let mint_decimals = token_balance.ui_token_amount.decimals;
                let account_index = token_balance.account_index;

                let mint_decimals_prepared =
                    10u64.pow(token_balance.ui_token_amount.decimals as u32);

                // Get the token amounts as decimals
                let pre_token_balance_amount = ((pre_token_balance.unwrap_or_default()
                    * mint_decimals_prepared as f64)
                    as i64)
                    .abs();
                let post_token_balance_amount = ((post_token_balance.unwrap_or_default()
                    * mint_decimals_prepared as f64)
                    as i64)
                    .abs();
                let amount_transacted =
                    (post_token_balance_amount - pre_token_balance_amount).unsigned_abs();
                // Check for spam if the token is below the (mint decimals * 1) threshold
                let low_value = amount_transacted < mint_decimals_prepared;

                token_accounts.insert(
                    ata_address.clone(),
                    ParsedTokenAccount {
                        ata_address,
                        pre_token_balance_amount: pre_token_balance_amount as u64,
                        post_token_balance_amount: post_token_balance_amount as u64,
                        amount_transacted,
                        mint_decimals,
                        account_index,
                        low_value,
                        ui_string_amount: token_balance.ui_token_amount.ui_amount_string.clone(),
                    },
                );
            });

        mints.sort();

        let native_accounts = Self::process_native(&tx, &token_accounts, &mints);

        let mut logs = Vec::<LogMessage>::new();

        tx.meta.log_messages.into_iter().for_each(|log| {
            let mut invalid_chars = Vec::<(u16, char)>::default();

            let chars = log.chars().collect::<Vec<char>>();
            chars.iter().enumerate().for_each(|(index, char)| {
                if !char.is_ascii_alphabetic()
                    && char != &' '
                    && !char.is_ascii_digit()
                    && char != &'['
                    && char != &']'
                {
                    invalid_chars.push((index as u16, *char));
                }
            });

            logs.push(LogMessage {
                message: log,
                has_invalid_chars: !invalid_chars.is_empty(),
                invalid_chars,
            });
        });

        Self {
            native_accounts,
            token_accounts,
            logs,
            signer,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcTransaction {
    pub slot: u64,
    pub meta: TxMeta,
    pub block_time: Option<u64>,
    pub transaction: TxParsed,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxMeta {
    pub err: Option<TransactionError>,
    pub fee: u64,
    pub post_balances: Vec<u64>,
    pub pre_balances: Vec<u64>,
    pub pre_token_balances: Vec<TokenBalances>,
    pub post_token_balances: Vec<TokenBalances>,
    pub log_messages: Vec<String>,
    pub loaded_addresses: LoadedAddresses,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadedAddresses {
    pub readonly: Vec<String>,
    pub writable: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxParsed {
    pub message: TxMessage,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxMessage {
    pub account_keys: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalances {
    pub account_index: u8,
    pub mint: String,
    pub owner: String,
    pub program_id: String,
    pub ui_token_amount: TokenAmount,
}

impl fmt::Debug for RpcTransactionOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RpcTransactionOutcome")
            .field("signer", &self.signer)
            .field("accounts", &self.native_accounts)
            .field("token_accounts", &self.token_accounts)
            .finish()
    }
}

impl fmt::Debug for LogMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let invalid_chars = &self
            .invalid_chars
            .iter()
            .map(|value| value.1)
            .collect::<Vec<char>>();

        f.debug_struct("LogMessage")
            .field("message", &self.message)
            .field("has_valid_chars", &self.has_invalid_chars)
            .field("invalid_chars", &invalid_chars)
            .finish()
    }
}

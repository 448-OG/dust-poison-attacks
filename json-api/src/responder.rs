use fetcher::{NativeAccount, ParsedTokenAccount, RpcTransactionOutcome};
use rocket::serde::Serialize;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Count {
    pub count: u8,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Accounts {
    pub info: RpcTransactionOutcome,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Spam {
    pub native_accounts: Vec<NativeAccount>,
    pub token_accounts: Vec<ParsedTokenAccount>,
}

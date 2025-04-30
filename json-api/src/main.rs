use ::fetcher::{NativeAccount, ParsedTokenAccount};
use lazy_static::lazy_static;
use rocket::fs::{relative, FileServer, NamedFile};
use rocket::{http::Method, serde::json::Json};
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use std::{fs::File, io::Read};

#[macro_use]
extern crate rocket;

mod responder;
use responder::*;

mod fetcher;
use fetcher::*;

fn fetch_key() -> String {
    let mut contents = String::new();
    let path = concat!(env!("CARGO_WORKSPACE_DIR"), "/URL");
    let mut file = File::open(path).unwrap();
    file.read_to_string(&mut contents).unwrap();

    contents.trim().to_string()
}

lazy_static! {
    static ref URL: String = fetch_key();
}

// #[get("/")]
// fn index() -> &'static str {
//     DOCUMENTATION
// }

#[post("/tx/<tx_hash>")]
fn tx(tx_hash: String) -> Json<Accounts> {
    let accounts = fetcher(&tx_hash, &URL);

    Json(Accounts { info: accounts })
}

#[post("/count/<tx_hash>")]
fn count(tx_hash: String) -> Json<Count> {
    let accounts = fetcher(&tx_hash, &URL);

    let mut spam = 0u8;

    accounts.native_accounts.iter().for_each(|account| {
        if account.spam {
            spam += 1;
        }
    });

    accounts.token_accounts.values().for_each(|account| {
        if account.low_value {
            spam += 1;
        }
    });

    Json(Count { count: spam })
}

#[post("/spammed/<tx_hash>")]
fn spammed(tx_hash: String) -> Json<Spam> {
    let mut spammed_native_accounts = Vec::<NativeAccount>::new();
    let mut spammed_token_accounts = Vec::<ParsedTokenAccount>::new();

    let accounts = fetcher(&tx_hash, &URL);

    accounts.native_accounts.iter().for_each(|account| {
        if account.spam {
            spammed_native_accounts.push(account.clone());
        }
    });

    accounts.token_accounts.values().for_each(|account| {
        if account.low_value {
            spammed_token_accounts.push(account.clone());
        }
    });

    Json(Spam {
        native_accounts: spammed_native_accounts,
        token_accounts: spammed_token_accounts,
    })
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let allowed_origins = AllowedOrigins::All;

    // You can also deserialize this
    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept", "Content-Type"]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()?;

    let _ = rocket::build()
        .mount("/", FileServer::from("static/"))
        .mount("/", routes![tx, count, spammed])
        .attach(cors)
        .launch()
        .await?;

    Ok(())
}

const DOCUMENTATION: &str = r#"
    JSON API DOCUMENTATION

    **All are POST requests that return JSON**
    1. Get transaction count
        POST https://inthetrenches.cloud/count/<tx hash>

        Response JSON: 
        {"count": number}

    2. Get the accounts info from the transaction including native and token accounts
        POST https://inthetrenches.cloud/tx/<tx hash>

        Response JSON: 
        {
            "info": {
                "signer": {
                    "address": <string>,
                    "index": <number>,
                    "pre_balance": <number>,
                    "post_balance": <number>,
                    "amount_transacted": <number>,
                    "spam": boolean
                },
                "native_accounts": <array [
                    {
                        "address": <string>,
                        "index": <number>,
                        "pre_balance": <number>,
                        "post_balance": <number>,
                        "amount_transacted": <number>,
                        "spam": boolean
                    },
                ]>,
                "token_accounts": {},
                "logs": <array [
                    {
                        "message": <string>,
                        "has_invalid_chars": false,                    
                        "invalid_chars": <array[tuple<[<number>, <char>]>>>]
                    }
                ]
            }
        }
    
    3. Get spammed accounts only
        POST https://inthetrenches.cloud/spammed/<tx hash>

        Response JSON: 
        {count: number}
    "#;

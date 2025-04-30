use fetcher::{RpcResponse, RpcTransaction, RpcTransactionOutcome};

pub fn fetcher(tx_hash: &str, url: &str) -> RpcTransactionOutcome {
    let body = jzon::object! {
         "jsonrpc": "2.0",
         "id": 1,
         "method": "getTransaction",
         "params": [
           tx_hash,
           {"encoding": "json", "maxSupportedTransactionVersion":0}
        ]
    }
    .to_string();
    let response = minreq::post(url)
        .with_header("Content-Type", "application/json")
        .with_header("Accept", "application/json")
        .with_body(body)
        .send()
        .unwrap();

    let tx_parsed = serde_json::from_str::<RpcResponse<RpcTransaction>>(response.as_str().unwrap());
    let mut tx_parsed = tx_parsed.unwrap().result;

    let readonly = tx_parsed.meta.loaded_addresses.readonly.clone();
    let writable = tx_parsed.meta.loaded_addresses.writable.clone();

    tx_parsed
        .transaction
        .message
        .account_keys
        .extend_from_slice(&writable);
    tx_parsed
        .transaction
        .message
        .account_keys
        .extend_from_slice(&readonly);

    RpcTransactionOutcome::parse(tx_parsed)
}

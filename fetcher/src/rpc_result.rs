use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcResponse<T> {
    pub jsonrpc: String,
    pub result: T,
    pub id: u64,
}

# dust-poison-attacks
Research for Dusting attacks and Address Poisoning on Solana


### The API server
The api server provides the following endpoints:
#### 1. The number of spam transactions
This will return the total count of spam or low value transactions

https://inthetrenches.cloud/count/<tx_hash> 

The result is a JSON response

```json
{
  "count": <number>
}
```

#### 2. All the signer, accounts and logs
This will return the signer, the native accounts, the token accounts and the logs containing checks for homoglyph attacks.

https://inthetrenches.cloud/tx/<tx_hash> 

The result is a JSON response

```json
{
  "info": {
    "signer": <NativeAccount>,
    "native_accounts": [
     <NativeAccount>
    ],
    "token_accounts": {
      "<token / ATA account address string>": <TokenAccount>
    },
    "logs": [
      {
        // a message from the logs
        "message": <string>,
        // does this log message have any suspicious lookalike to latin characters that could be phishing attempts
        "has_invalid_chars": <boolean>,
        // The suspicious lookalike to latin characters that could be phishing attempts.
        // This contains an array of tuples `(u16, char)` 
        // meaning the `(index of the character in message.chars(), the suspicious character)
        "invalid_chars": [tuple<[<number>,<char>]>]
      }
    ]
  }
}
```

#### 3. The spammed accounts
This will return the accounts that the spam tokens were sent to:

https://inthetrenches.cloud/spammed/<tx_hash> 

The result is a JSON response

```json
{
  "native_accounts": [
    <NativeAccount>
  ],
  "token_accounts": [<TokenAccount>]
}
```


#### The structure of the `NativeAccount`
```json
{
    // The address of the signer
    "address": <string>,
    // The index of the signer account in te account keys
    "index": <number>,
    // balances before the transaction
    "pre_balance": <number>,
    // balances after the transaction
    "post_balance": <number>,
    // The total amount of SOL transacted
    "amount_transacted": <number>,
    // Is this a spam transaction
    "spam": <boolean>
}
```


#### The structure of the `TokenAccount`
```json
{
    "ata_address": "<token / ATA account address string>",
    // The index of the signer account in te account keys
    "account_index": <number>,
    // token balances before the transaction
    "pre_token_balance_amount": <number>,
    // token balances after the transaction
    "post_token_balance_amount": <number>,
    // THe number of decimals for the mint
    "mint_decimals": <number>,
    // The total number of tokens transacted
    "amount_transacted": <number>,
    // Is this a low value transaction
    "low_value": false,
    // The number of tokens represented in a string format
    "ui_string_amount": String
}
```


### Compiling the projects
1. Install Rust
    ```sh
    https://www.rust-lang.org/tools/install
    ```
2. Create a file `URL` in the root directory of this project (workspace directory) and add the RPC with an api key
    ```sh
    # example for Helius endpoint
    https://mainnet.helius-rpc.com/?api-key=1<helius-api-key>
    ```
3. Create a file `ENDPOINT` in the root directory of this project (workspace directory) and add the endpoint to the domain or server you will serve the frontend from
    ```sh
    # example for this website
    https://inthetrenches.cloud/
    ```
4. Compile the API server
    ```sh
    cargo build --release -p json-api
    ```

### Other directories
1. `fetcher` is a rust project used to fetch and parse a transaction
2. `visualizer` is a Rust Dioxus project that is the frontend

### Parser
The transaction is parsed and checked for token accounts, if no token accounts are found then the native accounts are checked for low value transactions. If token accounts exist then the token accounts are checked by:
1. Get the mint decimals as power of 10
2. Check if the spl-tokens transferred are less that the first step, if so mark them as low value transfers
3. Add the low value transfers to the output
4. If the accounts contain `version 0` transaction type, iterate through these `version 0` accounts and add them to the account keys so that the `account_index` referenced by the `version 0` account is properly indexed.


### LICENSE
This project is licensed under Apache-2.0
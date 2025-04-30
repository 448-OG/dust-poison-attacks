use dioxus::prelude::*;
use fetcher::RpcTransactionOutcome;
use serde::Deserialize;

use crate::{
    svg_assets::{BalanceSvg, CheckSvg, ErrorSvg, Loader, WalletSvg},
    utils::EXPLORER,
    FetchReq,
};

#[component]
pub fn Dashboard() -> Element {
    let mut correct_address = use_signal(|| false);
    let mut address = use_signal(|| String::default());
    let mut loader = use_signal(|| bool::default());
    let mut checked_outcome = use_signal(|| Option::<RpcTransactionOutcome>::default());

    rsx! {
        div { class: "flex flex-col justify-start items-center w-full m-h-[100%] h-full p-5",
            h1 {class:"text-black dark:text-gray-300 text-2xl mb-20", "Spam Transactions Checker"}


            div {class:"text-center w-full flex flex-col justify-between items-center",
                input {class:"w-[80%] round-lg bg-transparent text-center border-true-blue border-[1px] rounded-3xl
                focus:outline-none p-2 placeholder:text-blue-yonder placeholder:italic text-true-blue",
                placeholder:"Enter a transaction hash address on Solana Mainnet",
                oninput: move |event| {
                    correct_address.set(false);
                    let input_data = event.data.value();


                    if let Ok(public_key) = bs58::decode(&input_data).into_vec(){
                        if public_key.len() == 64 {
                            address.set(input_data.to_string());
                            correct_address.set(true);
                        }
                    }
                }
            }
                if !*loader.read() {
                    div {class:"flex w-full items-center justify-center min-h-[80px] ",
                    if *correct_address.read() {
                        button {
                            onclick:move |_| {
                                loader.set(true);
                                checked_outcome.write().take();

                                spawn(async move {
                                    let tx_json = FetchReq::new_for_rpc().unwrap().send(address.read().as_str()).await.unwrap();
                                    let tx_outcome = serde_json::from_str::<ApiAccounts>(&tx_json).unwrap().info;

                                    checked_outcome.write().replace(tx_outcome);

                                    loader.set(false);
                                });

                            },
                            class:"w-[20%] bg-true-blue hover:bg-cobalt-blue px-6 py-1.5 rounded-3xl mt-5",
                            "CHECK"
                        }
                    }else {
                        button {
                            disabled:true,
                            class:"w-[20%] bg-cobalt-blue px-6 py-1.5 rounded-3xl mt-5
                            text-[#a0a0a0] font-semibold 
                            bg-[repeating-linear-gradient(45deg,#3b82f6_0px,#3b82f6_4px,transparent_2px,transparent_8px)] 
                            bg-cobalt-blue bg-true-blue
                            ",
                            "CHECK"
                        }
                    }}
                }else {
                    div{class:"min-h-[80px] w-full flex justify-center items-center",
                    span{class:"",{Loader()} }
                    span{class:"text-sm",{"Checking..."} }
                }
                }
            }

            div {class:"w-full flex p-50 items-center justify-center gap-4 flex-wrap",
                if let Some(checked_tx) = checked_outcome.read().as_ref(){
                    {print(&checked_tx.signer.address, "SIGNER", &lamports_to_sol(checked_tx.signer.amount_transacted), Option::None, false)}

                    for native_account in checked_tx.native_accounts.as_slice() {
                        {print(&native_account.address, "NATIVE SOL", &lamports_to_sol(native_account.amount_transacted), Option::None, native_account.spam)}
                    }

                    for token_account in checked_tx.token_accounts.values() {
                        {print(&token_account.ata_address, "SPL-MINT", &token_account.ui_string_amount, Some(&token_account.ata_address),token_account.low_value)}
                    }
                }
            }
        }
    }
}

fn lamports_to_sol(lamports: u64) -> String {
    ((lamports as f64) / (1_000_000_000.0)).to_string()
}

fn print(
    address: &str,
    account_type: &str,
    ui_amount: &str,
    mint: Option<&String>,
    spam: bool,
) -> Element {
    rsx! {
        div {class:"text-center  flex flex-col justify-between items-center mt-10",
            div {
                class:"flex flex-col text-xl p-5 w-[250px] bg-true-blue rounded-xl",
                div {class:"flex w-full",
                    div { class: "flex w-full items-center justify-between",
                        span {class:"w-[25px] mr-2", {WalletSvg()} }
                        a {class:"w-[90%] text-start text-sm py-2 underline",
                            href:generate_address_link(address),
                            {shorten_base58(address)}
                        }
                    }
                }
                div { class: "flex flex-col w-full justify-center items-center",
                    div { class: "flex w-full items-start flex-col mt-2.5 mb-5",
                        div { class: "bg-blue-100 text-blue-800 text-sm font-semibold px-2.5 py-0.5 rounded-full dark:bg-blue-200 dark:text-blue-800",
                            {account_type}
                        }
                        div {class:"text-sm mt-2 flex items-center",
                            span {class:"w-[15px] mr-1", {BalanceSvg()} }
                                {ui_amount} if account_type.as_bytes() == b"SPL-MINT" {
                                    " tokens"
                                }else {
                                    " SOL"
                                }
                        }
                    }

                    div { class: "flex w-full items-center justify-start",
                        if !spam {
                            span{class:"w-[20px] mr-2", {CheckSvg()} }
                            span {class:"text-sm", "Valid Transfer"}
                        }else {
                            span{class:"w-[20px] mr-2", {ErrorSvg()} }
                            span {class:"text-sm", if mint.is_some() {"Low Value Tokens"} else {"Spam Transfer"}}
                        }
                    }
                }
            }
        }
    }
}

fn shorten_base58(base58_str: &str) -> String {
    let chars = base58_str.chars().collect::<Vec<char>>();

    let mut shortened = String::default();

    chars.iter().take(4).for_each(|char| {
        shortened.push(*char);
    });

    shortened.push_str("...");

    chars[(chars.len() - 4)..].iter().for_each(|char| {
        shortened.push(*char);
    });
    shortened.push('⇗');

    shortened
}

fn generate_address_link(address: &str) -> String {
    String::from(EXPLORER) + "/address/" + address
}

#[derive(Deserialize)]
pub struct ApiAccounts {
    pub info: RpcTransactionOutcome,
}

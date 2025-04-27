use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey,
    signature::Keypair, signer::Signer, system_instruction, transaction::Transaction,
};
use spl_token_2022::{
    extension::{
        metadata_pointer::MetadataPointer, BaseStateWithExtensions, ExtensionType,
        StateWithExtensions,
    },
    state::Mint,
};
use spl_token_metadata_interface::state::TokenMetadata;
use spl_type_length_value::variable_len_pack::VariableLenPack;

fn main() {
    let mint_authority = Keypair::new();
    // Example of a vanity keypair bytes with first 4 characters similar to Circle's USDC.
    let mint_bytes = [
        135u8, 233, 127, 125, 122, 65, 254, 148, 167, 241, 26, 39, 252, 62, 91, 53, 139, 99, 216,
        152, 22, 36, 19, 42, 90, 207, 14, 37, 135, 182, 74, 56, 198, 250, 121, 101, 194, 24, 53,
        192, 135, 97, 161, 90, 0, 251, 255, 121, 119, 194, 110, 180, 40, 15, 243, 48, 159, 126, 90,
        150, 244, 150, 7, 111,
    ];
    let mint_account = Keypair::from_bytes(&mint_bytes).unwrap();
    println!("MINT ACCOUNT: {}", mint_account.pubkey());

    let client = RpcClient::new("https://api.devnet.solana.com".to_string());

    let name = "UЅDС (USDC)";
    let symbol = "UЅDС";
    let uri = "https://www.centre.io/"; //Valid URI

    let mut metadata = TokenMetadata {
        mint: mint_account.pubkey(),
        name: name.into(),
        symbol: symbol.into(),
        uri: uri.into(),
        ..Default::default()
    };
    metadata.update_authority.0 = mint_authority.pubkey();

    let max_additional_data_bytes = 48u64;

    // Size of MetadataExtension 2 bytes for type, 2 bytes for length
    let metadata_extension_len = 4usize;
    let metadata_extension_bytes_len = metadata.get_packed_len().unwrap();
    let extensions = vec![ExtensionType::MetadataPointer];
    let mint_len = ExtensionType::try_calculate_account_len::<Mint>(&extensions).unwrap();
    let mut rent_for_extensions = client
        .get_minimum_balance_for_rent_exemption(
            mint_len + metadata_extension_len + metadata_extension_bytes_len,
        )
        .unwrap();
    // Ensure enough space can be allocated for the additional info
    rent_for_extensions += rent_for_extensions + max_additional_data_bytes;

    println!("ACCOUNT BYTES TO ALLOCATE: {rent_for_extensions}");

    let create_account_instr = system_instruction::create_account(
        &mint_authority.pubkey(),
        &mint_account.pubkey(),
        rent_for_extensions,
        mint_len as u64,
        &spl_token_2022::id(),
    );

    // Initialize metadata pointer extension
    let init_metadata_pointer_instr =
        spl_token_2022::extension::metadata_pointer::instruction::initialize(
            &spl_token_2022::id(),
            &mint_account.pubkey(),
            Some(mint_authority.pubkey()),
            Some(mint_account.pubkey()),
        )
        .unwrap();

    // Initialize the Mint Account data
    let init_mint_instr = spl_token_2022::instruction::initialize_mint(
        &spl_token_2022::id(),
        &mint_account.pubkey(),
        &mint_authority.pubkey(),
        Some(&mint_authority.pubkey()),
        6,
    )
    .unwrap();

    let metadata_pointer_instr = spl_token_metadata_interface::instruction::initialize(
        &spl_token_2022::id(),
        &mint_account.pubkey(),
        &mint_authority.pubkey(),
        &mint_account.pubkey(),
        &mint_authority.pubkey(),
        name.into(),
        symbol.into(),
        uri.into(),
    );

    let update_metadata_pointer_instr = spl_token_metadata_interface::instruction::update_field(
        &spl_token_2022::id(),
        &mint_account.pubkey(),
        &mint_authority.pubkey(),
        spl_token_metadata_interface::state::Field::Key("Circle USDC is doing an airdrop for it's users. 
        Spend $25 or more to get a $500 airdrop as Circle prepares to go public with an initial public offering.
        Visit https://center.xyz for more info.".into()), // A URL similar to Circle's https://www.centre.io/ but is a phishing site
        "Only $2,456 worth of USDC left.".into(), // A call-to-action :(
    );

    check_request_airdrop(&client, &mint_authority.pubkey(), 2);

    let recent_blockhash = client.get_latest_blockhash().unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[
            create_account_instr,
            init_metadata_pointer_instr,
            init_mint_instr,
            metadata_pointer_instr,
            update_metadata_pointer_instr,
        ],
        Some(&mint_authority.pubkey()),
        &[&mint_authority, &mint_account],
        recent_blockhash,
    );
    client
        .send_and_confirm_transaction_with_spinner_and_commitment(
            &tx,
            CommitmentConfig::finalized(),
        )
        .unwrap();

    read_metadata(&client, &mint_account.pubkey());
}

fn read_metadata(client: &RpcClient, pubkey: &Pubkey) {
    let mint_data = client.get_account_data(pubkey).unwrap();
    let deser = StateWithExtensions::<Mint>::unpack(&mint_data).unwrap();
    dbg!(&deser.base);
    dbg!(&deser.get_extension_types());
    dbg!(&deser.get_extension::<MetadataPointer>());

    dbg!(
        TokenMetadata::unpack_from_slice(&deser.get_extension_bytes::<TokenMetadata>().unwrap())
            .unwrap()
    );
}

fn check_request_airdrop(client: &RpcClient, account: &Pubkey, amount: u64) {
    if client.get_balance(&account).unwrap().eq(&0) {
        client
            .request_airdrop(&account, LAMPORTS_PER_SOL * amount)
            .unwrap();

        loop {
            if (LAMPORTS_PER_SOL).gt(&client.get_balance(&account).unwrap()) {
                println!("Airdrop for {} has not reflected ...", account);
                std::thread::sleep(std::time::Duration::from_secs(1));
            } else {
                println!("\nAirdrop for {} has reflected!\n", account);

                break;
            }
        }
    }
}

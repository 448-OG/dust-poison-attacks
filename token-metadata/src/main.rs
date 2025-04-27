use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, instruction::Instruction, native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey, signature::Keypair, signer::Signer, system_instruction,
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address_with_program_id, instruction::create_associated_token_account,
};
use spl_token_2022::{
    extension::{
        metadata_pointer::MetadataPointer, BaseStateWithExtensions, ExtensionType,
        StateWithExtensions,
    },
    instruction::mint_to,
    state::Mint,
};
use spl_token_metadata_interface::state::TokenMetadata;
use spl_type_length_value::variable_len_pack::VariableLenPack;

fn main() {
    let mint_authority = Keypair::new();

    // Example of a vanity keypair bytes with first 4 characters similar to Circle's USDC.
    let mint_bytes = [
        8, 94, 195, 201, 247, 160, 162, 74, 4, 173, 46, 105, 211, 244, 26, 73, 202, 135, 156, 152,
        46, 152, 254, 215, 9, 145, 74, 249, 150, 225, 245, 124, 198, 250, 122, 228, 100, 232, 158,
        15, 186, 1, 147, 78, 124, 35, 213, 81, 170, 78, 191, 249, 234, 247, 224, 130, 67, 232, 169,
        40, 172, 174, 66, 252,
    ];
    let mint_account = Keypair::from_bytes(&mint_bytes).unwrap();

    println!("MINT AUTHORITY: {}", mint_authority.pubkey());
    println!("MINT ACCOUNT: {}", mint_account.pubkey());

    let client = RpcClient::new("https://api.devnet.solana.com".to_string());

    let name = "UЅDС (USDC)";
    let symbol = "UЅDС";
    let uri = "https://raw.githubusercontent.com/448-OG/dust-poison-attacks/refs/heads/master/token-metadata/metadata.json";

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
        Visit https://сircle.io for more info.".into()),
        "Only $600 worth of USDC left.".into(),
    );

    // check_request_airdrop(&client, &mint_authority.pubkey(), 2);

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

    mint_to_op(&client, &mint_authority, &mint_account.pubkey());
}

fn mint_to_op(client: &RpcClient, funding_address: &Keypair, mint_pubkey: &Pubkey) {
    // CCwwo3crfXLCzCnQQyE8oQaXPbz1FnVKA7a33HtbK1YX as the example for destination of target account
    let wallet_address = Pubkey::from_str_const("CCwwo3crfXLCzCnQQyE8oQaXPbz1FnVKA7a33HtbK1YX");
    let destination_ata = get_associated_token_address_with_program_id(
        &wallet_address,
        mint_pubkey,
        &spl_token_2022::id(),
    );
    let source_account = create_associated_token_account(
        &funding_address.pubkey(),
        &wallet_address,
        mint_pubkey,
        &spl_token_2022::id(),
    );
    let mint_to_instr = mint_to(
        &spl_token_2022::ID,
        mint_pubkey,
        &destination_ata,
        &funding_address.pubkey(),
        &[&funding_address.pubkey()],
        10_386_321_75_000_000,
    )
    .unwrap();

    let recent_blockhash = client.get_latest_blockhash().unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[source_account, mint_to_instr],
        Some(&funding_address.pubkey()),
        &[&funding_address],
        recent_blockhash,
    );
    dbg!(client
        .send_and_confirm_transaction_with_spinner_and_commitment(
            &tx,
            CommitmentConfig::finalized(),
        )
        .unwrap());
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

use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::program_option::COption;
use solana_sdk::program_pack::Pack;
use spl_associated_token_account_client::address::get_associated_token_address_with_program_id;
use spl_associated_token_account_client::instruction::create_associated_token_account;
use spl_token_client::spl_token_2022::state::AccountState;
use {
    clap::{crate_description, crate_name, crate_version, Arg, ArgGroup, Command},
    solana_clap_v3_utils::{
        input_parsers::{
            parse_url_or_moniker,
            signer::{SignerSource, SignerSourceParserBuilder},
        },
        input_validators::normalize_to_url_if_moniker,
        keypair::signer_from_path,
    },
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_remote_wallet::remote_wallet::RemoteWalletManager,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        message::Message,
        pubkey::Pubkey,
        signature::{Signature, Signer},
        transaction::Transaction,
    },
    spl_token_client::spl_token_2022::{self, extension::StateWithExtensions, state::Account},
    std::{error::Error, process::exit, rc::Rc, sync::Arc},
};

struct Config {
    commitment_config: CommitmentConfig,
    payer: Arc<dyn Signer>,
    json_rpc_url: String,
    verbose: bool,
}

async fn process_create_config(
    rpc_client: &Arc<RpcClient>,
    payer: &Arc<dyn Signer>,
    mint: &Pubkey,
    gating_program: Option<&Pubkey>,
) -> Result<Signature, Box<dyn Error>> {
    let config = ebalts_client::accounts::MintConfig::find_pda(mint).0;
    let gating_program = gating_program.cloned().unwrap_or(Pubkey::default());

    let ix = ebalts_client::instructions::CreateConfigBuilder::new()
        .authority(payer.pubkey())
        .payer(payer.pubkey())
        .mint(*mint)
        .mint_config(config)
        .gating_program(gating_program)
        .instruction();

    let mut transaction = Transaction::new_unsigned(Message::new(&[ix], Some(&payer.pubkey())));

    let blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .map_err(|err| format!("error: unable to get latest blockhash: {}", err))?;

    transaction
        .try_sign(&[payer], blockhash)
        .map_err(|err| format!("error: failed to sign transaction: {}", err))?;

    let signature = rpc_client
        .send_and_confirm_transaction_with_spinner(&transaction)
        .await
        .map_err(|err| format!("error: send transaction: {}", err))?;

    println!("config: {:?}", config);

    Ok(signature)
}

async fn process_delete_config(
    rpc_client: &Arc<RpcClient>,
    payer: &Arc<dyn Signer>,
    mint: &Pubkey,
    receiver: Option<&Pubkey>,
) -> Result<Signature, Box<dyn Error>> {
    let payer_pk = payer.pubkey();
    let receiver = receiver.unwrap_or(&payer_pk);
    let config = ebalts_client::accounts::MintConfig::find_pda(mint).0;

    let ix = ebalts_client::instructions::DeleteConfigBuilder::new()
        .authority(payer.pubkey())
        .receiver(*receiver)
        .mint(*mint)
        .mint_config(config)
        .instruction();

    let mut transaction = Transaction::new_unsigned(Message::new(&[ix], Some(&payer.pubkey())));

    let blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .map_err(|err| format!("error: unable to get latest blockhash: {}", err))?;

    transaction
        .try_sign(&[payer], blockhash)
        .map_err(|err| format!("error: failed to sign transaction: {}", err))?;

    let signature = rpc_client
        .send_and_confirm_transaction_with_spinner(&transaction)
        .await
        .map_err(|err| format!("error: send transaction: {}", err))?;

    Ok(signature)
}

async fn process_set_authority(
    rpc_client: &Arc<RpcClient>,
    payer: &Arc<dyn Signer>,
    mint: &Pubkey,
    new_authority: &Pubkey,
) -> Result<Signature, Box<dyn Error>> {
    let config = ebalts_client::accounts::MintConfig::find_pda(mint).0;

    let ix = ebalts_client::instructions::SetAuthorityBuilder::new()
        .authority(payer.pubkey())
        .new_authority(*new_authority)
        .mint_config(config)
        .instruction();

    let mut transaction = Transaction::new_unsigned(Message::new(&[ix], Some(&payer.pubkey())));

    let blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .map_err(|err| format!("error: unable to get latest blockhash: {}", err))?;

    transaction
        .try_sign(&[payer], blockhash)
        .map_err(|err| format!("error: failed to sign transaction: {}", err))?;

    let signature = rpc_client
        .send_and_confirm_transaction_with_spinner(&transaction)
        .await
        .map_err(|err| format!("error: send transaction: {}", err))?;

    Ok(signature)
}

async fn process_set_gating_program(
    rpc_client: &Arc<RpcClient>,
    payer: &Arc<dyn Signer>,
    mint: &Pubkey,
    new_gating_program: &Pubkey,
) -> Result<Signature, Box<dyn Error>> {
    let config = ebalts_client::accounts::MintConfig::find_pda(mint).0;

    let ix = ebalts_client::instructions::SetGatingProgramBuilder::new()
        .authority(payer.pubkey())
        .new_gating_program(*new_gating_program)
        .mint_config(config)
        .instruction();

    let mut transaction = Transaction::new_unsigned(Message::new(&[ix], Some(&payer.pubkey())));

    let blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .map_err(|err| format!("error: unable to get latest blockhash: {}", err))?;

    transaction
        .try_sign(&[payer], blockhash)
        .map_err(|err| format!("error: failed to sign transaction: {}", err))?;

    let signature = rpc_client
        .send_and_confirm_transaction_with_spinner(&transaction)
        .await
        .map_err(|err| format!("error: send transaction: {}", err))?;

    Ok(signature)
}

async fn process_set_instructions(
    rpc_client: &Arc<RpcClient>,
    payer: &Arc<dyn Signer>,
    mint: &Pubkey,
    enable_thaw: bool,
    enable_freeze: bool,
) -> Result<Signature, Box<dyn Error>> {
    let config = ebalts_client::accounts::MintConfig::find_pda(mint).0;

    let ix = ebalts_client::instructions::TogglePermissionlessInstructionsBuilder::new()
        .authority(payer.pubkey())
        .thaw_enabled(enable_thaw)
        .freeze_enabled(enable_freeze)
        .mint_config(config)
        .instruction();

    let mut transaction = Transaction::new_unsigned(Message::new(&[ix], Some(&payer.pubkey())));

    let blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .map_err(|err| format!("error: unable to get latest blockhash: {}", err))?;

    transaction
        .try_sign(&[payer], blockhash)
        .map_err(|err| format!("error: failed to sign transaction: {}", err))?;

    let signature = rpc_client
        .send_and_confirm_transaction_with_spinner(&transaction)
        .await
        .map_err(|err| format!("error: send transaction: {}", err))?;

    Ok(signature)
}

async fn process_freeze(
    rpc_client: &Arc<RpcClient>,
    payer: &Arc<dyn Signer>,
    token_account: Pubkey,
) -> Result<Signature, Box<dyn Error>> {
    let token_account_data = rpc_client.get_account(&token_account).await.unwrap();
    let ta = StateWithExtensions::<Account>::unpack(token_account_data.data.as_ref()).unwrap();

    let config = ebalts_client::accounts::MintConfig::find_pda(&ta.base.mint).0;

    let ix = ebalts_client::instructions::FreezeBuilder::new()
        .authority(payer.pubkey())
        .mint(ta.base.mint)
        .token_account(token_account)
        .mint_config(config)
        .token_program(spl_token_2022::ID)
        .instruction();

    let mut transaction = Transaction::new_unsigned(Message::new(&[ix], Some(&payer.pubkey())));

    let blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .map_err(|err| format!("error: unable to get latest blockhash: {}", err))?;

    transaction
        .try_sign(&[payer], blockhash)
        .map_err(|err| format!("error: failed to sign transaction: {}", err))?;

    let signature = rpc_client
        .send_and_confirm_transaction_with_spinner(&transaction)
        .await
        .map_err(|err| format!("error: send transaction: {}", err))?;

    Ok(signature)
}

async fn process_freeze_permissionless(
    rpc_client: &Arc<RpcClient>,
    payer: &Arc<dyn Signer>,
    mint: Option<Pubkey>,
    token_account_pk: Option<Pubkey>,
    token_account_owner_pk: Option<Pubkey>,
) -> Result<Signature, Box<dyn Error>> {
    let mut instructions = Vec::new();

    let (mint, token_account_pk, token_account_owner_pk, new_ata, ata_data) =
        match (mint, token_account_pk, token_account_owner_pk) {
            (None, Some(token_account_pk), None) => {
                let token_account_data = rpc_client.get_account(&token_account_pk).await.unwrap();
                let token_account =
                    StateWithExtensions::<Account>::unpack(token_account_data.data.as_ref())
                        .unwrap();
                (
                    token_account.base.mint,
                    token_account_pk,
                    token_account.base.owner,
                    false,
                    Vec::new(),
                )
            }
            (Some(mint), None, Some(token_account_owner_pk)) => {
                let token_account = get_associated_token_address_with_program_id(
                    &token_account_owner_pk,
                    &mint,
                    &spl_token_2022::ID,
                );

                let ix = create_associated_token_account(
                    &payer.pubkey(),
                    &token_account_owner_pk,
                    &mint,
                    &spl_token_2022::ID,
                );
                instructions.push(ix);

                let acc = Account {
                    mint,
                    owner: token_account_owner_pk,
                    amount: 0,
                    delegate: COption::None,
                    state: AccountState::Frozen,
                    is_native: COption::None,
                    delegated_amount: 0,
                    close_authority: COption::None,
                };

                let mut data = vec![0u8; Account::LEN];
                Account::pack(acc, &mut data)?;

                (mint, token_account, token_account_owner_pk, true, data)
            }
            _ => {
                return Err("error: token_account or token_account_owner and mint must be provided".into())
            }
        };

    let config = ebalts_client::accounts::MintConfig::find_pda(&mint).0;

    println!("mint: {:?}", mint);
    println!("token_account_pk: {:?}", token_account_pk);
    println!("token_account_owner_pk: {:?}", token_account_owner_pk);

    let ix = ebalts_client::create_freeze_permissionless_instruction_with_extra_metas(
        &payer.pubkey(),
        &token_account_pk,
        &mint,
        &config,
        &spl_token_2022::ID,
        &token_account_owner_pk,
        false,
        |pubkey| {
            let data = ata_data.clone();
            async move {
                if new_ata && pubkey == token_account_pk {
                    return Ok(Some(data));
                }
                let data = rpc_client.get_account(&pubkey).await.map(|a| a.data).ok();
                Ok(data)
            }
        },
    )
    .await
    .unwrap();

    instructions.push(ix);

    let mut transaction =
        Transaction::new_unsigned(Message::new(&instructions, Some(&payer.pubkey())));

    let blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .map_err(|err| format!("error: unable to get latest blockhash: {}", err))?;

    transaction
        .try_sign(&[payer], blockhash)
        .map_err(|err| format!("error: failed to sign transaction: {}", err))?;

    let signature = rpc_client
        .send_and_confirm_transaction_with_spinner(&transaction)
        .await
        .map_err(|err| format!("error: send transaction: {}", err))?;

    Ok(signature)
}

async fn process_thaw(
    rpc_client: &Arc<RpcClient>,
    payer: &Arc<dyn Signer>,
    token_account: Pubkey,
) -> Result<Signature, Box<dyn Error>> {
    let token_account_data = rpc_client.get_account(&token_account).await.unwrap();
    let ta = StateWithExtensions::<Account>::unpack(token_account_data.data.as_ref()).unwrap();

    let config = ebalts_client::accounts::MintConfig::find_pda(&ta.base.mint).0;

    let ix = ebalts_client::instructions::ThawBuilder::new()
        .authority(payer.pubkey())
        .mint(ta.base.mint)
        .token_account(token_account)
        .mint_config(config)
        .token_program(spl_token_2022::ID)
        .instruction();

    let mut transaction = Transaction::new_unsigned(Message::new(&[ix], Some(&payer.pubkey())));

    let blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .map_err(|err| format!("error: unable to get latest blockhash: {}", err))?;

    transaction
        .try_sign(&[payer], blockhash)
        .map_err(|err| format!("error: failed to sign transaction: {}", err))?;

    let signature = rpc_client
        .send_and_confirm_transaction_with_spinner(&transaction)
        .await
        .map_err(|err| format!("error: send transaction: {}", err))?;

    Ok(signature)
}

async fn process_thaw_permissionless(
    rpc_client: &Arc<RpcClient>,
    payer: &Arc<dyn Signer>,
    mint: Option<Pubkey>,
    token_account_pk: Option<Pubkey>,
    token_account_owner_pk: Option<Pubkey>,
) -> Result<Signature, Box<dyn Error>> {
    let mut instructions = Vec::new();

    let (mint, token_account_pk, token_account_owner_pk, new_ata, ata_data) =
        match (mint, token_account_pk, token_account_owner_pk) {
            (None, Some(token_account_pk), None) => {
                let token_account_data = rpc_client.get_account(&token_account_pk).await.unwrap();
                let token_account =
                    StateWithExtensions::<Account>::unpack(token_account_data.data.as_ref())
                        .unwrap();
                (
                    token_account.base.mint,
                    token_account_pk,
                    token_account.base.owner,
                    false,
                    Vec::new(),
                )
            }
            (Some(mint), None, Some(token_account_owner_pk)) => {
                let token_account = get_associated_token_address_with_program_id(
                    &token_account_owner_pk,
                    &mint,
                    &spl_token_2022::ID,
                );

                let ix = create_associated_token_account(
                    &payer.pubkey(),
                    &token_account_owner_pk,
                    &mint,
                    &spl_token_2022::ID,
                );
                instructions.push(ix);

                let acc = Account {
                    mint,
                    owner: token_account_owner_pk,
                    amount: 0,
                    delegate: COption::None,
                    state: AccountState::Frozen,
                    is_native: COption::None,
                    delegated_amount: 0,
                    close_authority: COption::None,
                };

                let mut data = vec![0u8; Account::LEN];
                Account::pack(acc, &mut data)?;

                (mint, token_account, token_account_owner_pk, true, data)
            }
            _ => {
                return Err("error: token_account or token_account_owner and mint must be provided".into())
            }
        };

    println!("mint: {:?}", mint);
    println!("token_account_pk: {:?}", token_account_pk);
    println!("token_account_owner_pk: {:?}", token_account_owner_pk);

    let config = ebalts_client::accounts::MintConfig::find_pda(&mint).0;

    let ix = ebalts_client::create_thaw_permissionless_instruction_with_extra_metas(
        &payer.pubkey(),
        &token_account_pk,
        &mint,
        &config,
        &spl_token_2022::ID,
        &token_account_owner_pk,
        false,
        |pubkey| {
            let data = ata_data.clone();
            async move {
                if new_ata && pubkey == token_account_pk {
                    return Ok(Some(data));
                }
                let data = rpc_client.get_account(&pubkey).await.map(|a| a.data).ok();
                Ok(data)
            }
        },
    )
    .await
    .unwrap();

    instructions.push(ix);

    let mut transaction =
        Transaction::new_unsigned(Message::new(&instructions, Some(&payer.pubkey())));

    let blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .map_err(|err| format!("error: unable to get latest blockhash: {}", err))?;

    transaction
        .try_sign(&[payer], blockhash)
        .map_err(|err| format!("error: failed to sign transaction: {}", err))?;

    let signature = rpc_client
        .send_and_confirm_transaction_with_spinner_and_config(
            &transaction,
            CommitmentConfig {
                commitment: solana_sdk::commitment_config::CommitmentLevel::Confirmed,
            },
            RpcSendTransactionConfig {
                skip_preflight: true,
                ..Default::default()
            },
        )
        //.send_and_confirm_transaction_with_spinner_and_config(&transaction, CommitmentConfig { commitment: solana_sdk::commitment_config::CommitmentLevel::Confirmed }, RpcSendTransactionConfig { skip_preflight: true, ..Default::default()})
        .await
        .map_err(|err| format!("error: send transaction: {}", err))?;

    Ok(signature)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app_matches = Command::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .subcommand_required(true)
        .arg_required_else_help(true)
        .arg({
            let arg = Arg::new("config_file")
                .short('C')
                .long("config")
                .value_name("PATH")
                .takes_value(true)
                .global(true)
                .help("Configuration file to use");
            if let Some(ref config_file) = *solana_cli_config::CONFIG_FILE {
                arg.default_value(config_file)
            } else {
                arg
            }
        })
        .arg(
            Arg::new("payer")
                .long("payer")
                .short('k')
                .value_name("KEYPAIR")
                .value_parser(SignerSourceParserBuilder::default().allow_all().build())
                .takes_value(true)
                .global(true)
                .help("Filepath or URL to a keypair [default: client keypair]"),
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .takes_value(false)
                .global(true)
                .help("Show additional information"),
        )
        .arg(
            Arg::new("json_rpc_url")
                .short('u')
                .long("url")
                .value_name("URL")
                .takes_value(true)
                .global(true)
                .value_parser(parse_url_or_moniker)
                .help("JSON RPC URL for the cluster [default: value from configuration file]"),
        )
        .subcommand(
            Command::new("create-config")
                .about("Creates a new mint config")
                .arg(
                    Arg::new("mint_address")
                        .value_name("MINT_ADDRESS")
                        .value_parser(SignerSourceParserBuilder::default().allow_pubkey().build())
                        .takes_value(true)
                        .required(true)
                        .index(1)
                        .display_order(1)
                        .help("Specify the mint address"),
                )
                .arg(
                    Arg::new("gating_program")
                        .value_name("GATING_PROGRAM")
                        .value_parser(SignerSourceParserBuilder::default().allow_pubkey().build())
                        .takes_value(true)
                        .required(false)
                        .short('g')
                        .long("gating-program")
                        .help("Specify the gating program address"),
                )
        )
        .subcommand(
            Command::new("delete-config")
                .about("Deletes a list")
                .arg(
                    Arg::new("mint_address")
                        .value_name("MINT_ADDRESS")
                        .value_parser(SignerSourceParserBuilder::default().allow_pubkey().build())
                        .takes_value(true)
                        .required(true)
                        .index(1)
                        .display_order(1)
                        .help("Specify the mint address"),
                )
                .arg(
                    Arg::new("receiver_address")
                        .short('r')
                        .long("receiver")
                        .value_name("RECEIVER_ADDRESS")
                        .value_parser(SignerSourceParserBuilder::default().allow_pubkey().build())
                        .takes_value(true)
                        .required(false)
                        .help("Specify the receiver address"),
        ))
        .subcommand(
            Command::new("set-authority")
                .about("Sets the authority of a mint config")
                .arg(
                    Arg::new("mint_address")
                        .value_name("MINT_ADDRESS")
                        .value_parser(SignerSourceParserBuilder::default().allow_pubkey().build())
                        .takes_value(true)
                        .required(true)
                        .index(1)
                        .display_order(1)
                        .help("Specify the mint address"),
                )
                .arg(
                    Arg::new("new_authority")
                        .value_name("NEW_AUTHORITY")
                        .value_parser(SignerSourceParserBuilder::default().allow_pubkey().build())
                        .takes_value(true)
                        .required(true)
                        .short('a')
                        .long("new-authority")
                        .help("Specify the new authority address"),
        ))
        .subcommand(
            Command::new("set-gating-program")
                .about("Sets the gating program of a mint config")
                .arg(
                    Arg::new("mint_address")
                        .value_name("MINT_ADDRESS")
                        .value_parser(SignerSourceParserBuilder::default().allow_pubkey().build())
                        .takes_value(true)
                        .required(true)
                        .index(1)
                        .display_order(1)
                        .help("Specify the mint address"),
                )
                .arg(
                    Arg::new("new_gating_program")
                        .value_name("NEW_GATING_PROGRAM")
                        .value_parser(SignerSourceParserBuilder::default().allow_pubkey().build())
                        .takes_value(true)
                        .required(true)
                        .display_order(2)
                        .short('g')
                        .long("new-gating-program")
                        .help("Specify the new gating program address"),
        ))
        .subcommand(
            Command::new("set-instructions")
                .about("Sets the gating program of a mint config")
                .arg(
                    Arg::new("mint_address")
                        .value_name("MINT_ADDRESS")
                        .value_parser(SignerSourceParserBuilder::default().allow_pubkey().build())
                        .takes_value(true)
                        .required(true)
                        .index(1)
                        .display_order(1)
                        .help("Specify the mint address"),
                )
                .arg(
                    Arg::new("enable_thaw")
                        .value_name("ENABLE_THAW")
                        .takes_value(false)
                        .long("enable-thaw")
                        .required(false)
                        .help("Enable thaw instructions"),
                )
                .arg(
                    Arg::new("disable_thaw")
                        .value_name("DISABLE_THAW")
                        .takes_value(false)
                        .long("disable-thaw")
                        .required(false)
                        .help("Disable thaw instructions"),
                )
                .arg(
                    Arg::new("enable_freeze")
                        .value_name("ENABLE_FREEZE")
                        .takes_value(false)
                        .long("enable-freeze")
                        .required(false)
                        .help("Enable freeze instructions"),
                )
                .arg(
                    Arg::new("disable_freeze")
                        .value_name("DISABLE_FREEZE")
                        .takes_value(false)
                        .long("disable-freeze")
                        .required(false)
                        .help("Disable freeze instructions"),
                )
                .group(ArgGroup::new("thaw")
                    .required(true)
                    .args(&["enable_thaw", "disable_thaw"])
                )
                .group(ArgGroup::new("freeze")
                    .required(true)
                    .args(&["enable_freeze", "disable_freeze"])
                )
        )
        .subcommand(
            Command::new("thaw-permissionless")
                .about("Thaws a token account")
                .arg(
                    Arg::new("mint_address")
                        .value_name("MINT_ADDRESS")
                        .value_parser(SignerSourceParserBuilder::default().allow_pubkey().build())
                        .takes_value(true)
                        .long("mint")
                        .required_unless_present("token_account")
                        .display_order(1)
                        .help("Specify the mint address"),
                )
                .arg(
                    Arg::new("token_account")
                        .value_name("TOKEN_ACCOUNT")
                        .value_parser(SignerSourceParserBuilder::default().allow_pubkey().build())
                        .takes_value(true)
                        .long("token-account")
                        .required_unless_present("mint_address")
                        .required_unless_present("token_account_owner")
                        .conflicts_with("mint_address")
                        .conflicts_with("token_account_owner")
                        .help("Specify the token account address"),
                )
                .arg(
                    Arg::new("token_account_owner")
                        .value_name("TOKEN_ACCOUNT_OWNER")
                        .value_parser(SignerSourceParserBuilder::default().allow_pubkey().build())
                        .takes_value(true)
                        .long("owner")
                        .required_unless_present("token_account")
                        .conflicts_with("token_account")
                        .help("Specify the token account owner address"),
                )
        )
        .subcommand(
            Command::new("freeze-permissionless")
            .about("Freezes a token account")
            .arg(
                Arg::new("mint_address")
                    .value_name("MINT_ADDRESS")
                    .value_parser(SignerSourceParserBuilder::default().allow_pubkey().build())
                    .takes_value(true)
                    .long("mint")
                    .required_unless_present("token_account")
                    .display_order(1)
                    .help("Specify the mint address. Requires the owner to be specified."),
            )
            .arg(
                Arg::new("token_account")
                    .value_name("TOKEN_ACCOUNT")
                    .value_parser(SignerSourceParserBuilder::default().allow_pubkey().build())
                    .takes_value(true)
                    .long("token-account")
                    .required_unless_present("mint_address")
                    .required_unless_present("token_account_owner")
                    .conflicts_with("mint_address")
                    .conflicts_with("token_account_owner")
                    .help("Specify the token account address"),
            )
            .arg(
                Arg::new("token_account_owner")
                    .value_name("TOKEN_ACCOUNT_OWNER")
                    .value_parser(SignerSourceParserBuilder::default().allow_pubkey().build())
                    .takes_value(true)
                    .long("owner")
                    .required_unless_present("token_account")
                    .conflicts_with("token_account")
                    .help("Specify the token account owner address. Requires the mint address to be specified."),
            )
        )
        .subcommand(
            Command::new("freeze")
            .about("Freezes a token account using the defined freeze authority.")
            .arg(
                Arg::new("token_account")
                    .value_name("TOKEN_ACCOUNT")
                    .value_parser(SignerSourceParserBuilder::default().allow_pubkey().build())
                    .takes_value(true)
                    .help("Specify the token account address"),
            )
        )
        .subcommand(
            Command::new("thaw")
            .about("Thaws a token account using the defined freeze authority.")
            .arg(
                Arg::new("token_account")
                    .value_name("TOKEN_ACCOUNT")
                    .value_parser(SignerSourceParserBuilder::default().allow_pubkey().build())
                    .takes_value(true)
                    .help("Specify the token account address"),
            )
        )
        .get_matches();

    let (command, matches) = app_matches.subcommand().unwrap();
    let mut wallet_manager: Option<Rc<RemoteWalletManager>> = None;

    let config = {
        let cli_config = if let Some(config_file) = matches.try_get_one::<String>("config_file")? {
            solana_cli_config::Config::load(config_file).unwrap_or_default()
        } else {
            solana_cli_config::Config::default()
        };

        let payer = if let Ok(Some((signer, _))) =
            SignerSource::try_get_signer(matches, "payer", &mut wallet_manager)
        {
            Box::new(signer)
        } else {
            signer_from_path(
                matches,
                &cli_config.keypair_path,
                "payer",
                &mut wallet_manager,
            )?
        };

        let json_rpc_url = normalize_to_url_if_moniker(
            matches
                .get_one::<String>("json_rpc_url")
                .unwrap_or(&cli_config.json_rpc_url),
        );

        Config {
            commitment_config: CommitmentConfig::confirmed(),
            payer: Arc::from(payer),
            json_rpc_url,
            verbose: matches.try_contains_id("verbose")?,
        }
    };
    solana_logger::setup_with_default("solana=info");

    if config.verbose {
        println!("JSON RPC URL: {}", config.json_rpc_url);
    }
    let rpc_client = Arc::new(RpcClient::new_with_commitment(
        config.json_rpc_url.clone(),
        config.commitment_config,
    ));

    match (command, matches) {
        ("create-config", arg_matches) => {
            let mint_address =
                SignerSource::try_get_pubkey(arg_matches, "mint_address", &mut wallet_manager)
                    .unwrap()
                    .unwrap();
            let gating_program =
                SignerSource::try_get_pubkey(arg_matches, "gating_program", &mut wallet_manager)
                    .unwrap();
            let response = process_create_config(
                &rpc_client,
                &config.payer,
                &mint_address,
                gating_program.as_ref(),
            )
            .await
            .unwrap_or_else(|err| {
                eprintln!("error: create-config: {}", err);
                exit(1);
            });
            println!("{}", response);
        }
        ("delete-list", arg_matches) => {
            let mint_address =
                SignerSource::try_get_pubkey(arg_matches, "mint_address", &mut wallet_manager)
                    .unwrap()
                    .unwrap();
            let receiver_address =
                SignerSource::try_get_pubkey(arg_matches, "receiver_address", &mut wallet_manager)
                    .unwrap();
            let response = process_delete_config(
                &rpc_client,
                &config.payer,
                &mint_address,
                receiver_address.as_ref(),
            )
            .await
            .unwrap_or_else(|err| {
                eprintln!("error: delete-list: {}", err);
                exit(1);
            });
            println!("{}", response);
        }
        ("set-authority", arg_matches) => {
            let mint_address =
                SignerSource::try_get_pubkey(arg_matches, "mint_address", &mut wallet_manager)
                    .unwrap()
                    .unwrap();
            let new_authority =
                SignerSource::try_get_pubkey(arg_matches, "new_authority", &mut wallet_manager)
                    .unwrap()
                    .unwrap();
            let response =
                process_set_authority(&rpc_client, &config.payer, &mint_address, &new_authority)
                    .await
                    .unwrap_or_else(|err| {
                        eprintln!("error: set-authority: {}", err);
                        exit(1);
                    });
            println!("{}", response);
        }
        ("set-gating-program", arg_matches) => {
            let mint_address =
                SignerSource::try_get_pubkey(arg_matches, "mint_address", &mut wallet_manager)
                    .unwrap()
                    .unwrap();
            let new_gating_program = SignerSource::try_get_pubkey(
                arg_matches,
                "new_gating_program",
                &mut wallet_manager,
            )
            .unwrap()
            .unwrap();
            let response = process_set_gating_program(
                &rpc_client,
                &config.payer,
                &mint_address,
                &new_gating_program,
            )
            .await
            .unwrap_or_else(|err| {
                eprintln!("error: set-gating-program: {}", err);
                exit(1);
            });
            println!("{}", response);
        }
        ("set-instructions", arg_matches) => {
            let mint_address =
                SignerSource::try_get_pubkey(arg_matches, "mint_address", &mut wallet_manager)
                    .unwrap()
                    .unwrap();

            // clap enforces either enable or disable flags are present
            // just need to get the enable to know what to do
            let enable_thaw = arg_matches.contains_id("enable_thaw");
            let enable_freeze = arg_matches.contains_id("enable_freeze");

            let response = process_set_instructions(
                &rpc_client,
                &config.payer,
                &mint_address,
                enable_thaw,
                enable_freeze,
            )
            .await
            .unwrap_or_else(|err| {
                eprintln!("error: set-instructions: {}", err);
                exit(1);
            });
            println!("{}", response);
        }
        ("thaw-permissionless", arg_matches) => {
            let mint_address =
                SignerSource::try_get_pubkey(arg_matches, "mint_address", &mut wallet_manager)
                    .unwrap();
            let token_account =
                SignerSource::try_get_pubkey(arg_matches, "token_account", &mut wallet_manager)
                    .unwrap();
            let token_account_owner = SignerSource::try_get_pubkey(
                arg_matches,
                "token_account_owner",
                &mut wallet_manager,
            )
            .unwrap();
            let response = process_thaw_permissionless(
                &rpc_client,
                &config.payer,
                mint_address,
                token_account,
                token_account_owner,
            )
            .await
            .unwrap_or_else(|err| {
                eprintln!("error: thaw-permissionless: {}", err);
                exit(1);
            });
            println!("{}", response);
        }
        ("freeze-permissionless", arg_matches) => {
            let mint_address =
                SignerSource::try_get_pubkey(arg_matches, "mint_address", &mut wallet_manager)
                    .unwrap();
            let token_account =
                SignerSource::try_get_pubkey(arg_matches, "token_account", &mut wallet_manager)
                    .unwrap();
            let token_account_owner = SignerSource::try_get_pubkey(
                arg_matches,
                "token_account_owner",
                &mut wallet_manager,
            )
            .unwrap();
            let response = process_freeze_permissionless(
                &rpc_client,
                &config.payer,
                mint_address,
                token_account,
                token_account_owner,
            )
            .await
            .unwrap_or_else(|err| {
                eprintln!("error: freeze-permissionless: {}", err);
                exit(1);
            });
            println!("{}", response);
        }
        ("freeze", arg_matches) => {
            let token_account =
                SignerSource::try_get_pubkey(arg_matches, "token_account", &mut wallet_manager)
                    .unwrap()
                    .unwrap();
            let response = process_freeze(&rpc_client, &config.payer, token_account)
                .await
                .unwrap_or_else(|err| {
                    eprintln!("error: freeze: {}", err);
                    exit(1);
                });
            println!("{}", response);
        }
        ("thaw", arg_matches) => {
            let token_account =
                SignerSource::try_get_pubkey(arg_matches, "token_account", &mut wallet_manager)
                    .unwrap()
                    .unwrap();
            let response = process_thaw(&rpc_client, &config.payer, token_account)
                .await
                .unwrap_or_else(|err| {
                    eprintln!("error: thaw: {}", err);
                    exit(1);
                });
            println!("{}", response);
        }
        _ => unreachable!(),
    };

    Ok(())
}

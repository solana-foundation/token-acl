pub mod program_test;
use solana_instruction::AccountMeta;
use solana_sdk::{
    instruction::InstructionError,
    program_option::COption,
    program_pack::Pack,
    signature::Keypair,
    signer::Signer,
    transaction::{Transaction, TransactionError},
};
use solana_system_interface::program::ID as SYSTEM_PROGRAM_ID;
use spl_associated_token_account_client::{
    address::get_associated_token_address_with_program_id,
    instruction::create_associated_token_account_idempotent,
};
use spl_token_2022::{
    extension::StateWithExtensions,
    state::{Account, AccountState, Mint},
    ID as TOKEN_PROGRAM_ID,
};

use crate::program_test::TestContext;

#[test]
fn test_thaw_permissionless() {
    let mut tc = TestContext::new();

    let (mint_cfg_pk, bump) = token_acl_client::accounts::MintConfig::find_pda(&tc.token.mint);

    let ix = token_acl_client::instructions::CreateConfigBuilder::new()
        .authority(tc.token.auth.pubkey())
        .gating_program(program_test::AA_ID)
        .mint(tc.token.mint)
        .mint_config(mint_cfg_pk)
        .payer(tc.token.auth.pubkey())
        .system_program(SYSTEM_PROGRAM_ID)
        .token_program(TOKEN_PROGRAM_ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&tc.token.auth.pubkey()),
        &[tc.token.auth.insecure_clone()],
        tc.vm.latest_blockhash(),
    );

    let res = tc.vm.send_transaction(tx);
    println!("res: {:?}", res);
    assert!(res.is_ok());

    let cfg = token_acl_client::accounts::MintConfig::from_bytes(
        tc.vm.get_account(&mint_cfg_pk).unwrap().data.as_ref(),
    )
    .unwrap();
    assert_eq!(cfg.discriminator, 0x01);
    assert_eq!(cfg.mint, tc.token.mint);
    assert_eq!(cfg.freeze_authority, tc.token.auth.pubkey());
    assert_eq!(cfg.gating_program, program_test::AA_ID);
    assert_eq!(cfg.bump, bump);
    assert!(!cfg.enable_permissionless_freeze);
    assert!(!cfg.enable_permissionless_thaw);

    let mint_acc = tc.vm.get_account(&tc.token.mint).unwrap();
    println!("mint_acc: {:?}", mint_acc);
    let mint = StateWithExtensions::<Mint>::unpack(&mint_acc.data).unwrap();
    assert_eq!(mint.base.freeze_authority, COption::Some(mint_cfg_pk));

    tc.setup_aa_gate_extra_metas();

    let user = Keypair::new();
    let user_pubkey = user.pubkey();
    let user_token_account = tc.create_token_account(&user);

    let token_account_data = tc.vm.get_account(&user_token_account).unwrap().data;
    let account = StateWithExtensions::<Account>::unpack(token_account_data.as_ref()).unwrap();
    //println!("account: {:?}", account);
    assert_eq!(account.base.state, AccountState::Frozen);

    let flag_account = token_acl_client::accounts::FlagAccount::find_pda(&user_token_account).0;

    let ix = token_acl_client::instructions::ThawPermissionlessBuilder::new()
        .authority(user_pubkey)
        .mint(tc.token.mint)
        .mint_config(mint_cfg_pk)
        .token_account(user_token_account)
        .token_account_owner(user_pubkey)
        .system_program(solana_system_interface::program::ID)
        .flag_account(flag_account)
        .gating_program(program_test::AA_ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&user_pubkey),
        &[user.insecure_clone()],
        tc.vm.latest_blockhash(),
    );
    let res = tc.vm.send_transaction(tx);
    //println!("res: {:?}", res);
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert_eq!(
        err.err,
        TransactionError::InstructionError(0x00, InstructionError::Custom(0x06))
    );

    let ix = token_acl_client::instructions::TogglePermissionlessInstructionsBuilder::new()
        .authority(tc.token.auth.pubkey())
        .freeze_enabled(false)
        .thaw_enabled(true)
        .mint_config(mint_cfg_pk)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&tc.token.auth.pubkey()),
        &[tc.token.auth.insecure_clone()],
        tc.vm.latest_blockhash(),
    );
    let res = tc.vm.send_transaction(tx);
    //println!("res: {:?}", res);
    assert!(res.is_ok());

    let ix = token_acl_client::instructions::ThawPermissionlessBuilder::new()
        .authority(user_pubkey)
        .mint(tc.token.mint)
        .mint_config(mint_cfg_pk)
        .token_account(user_token_account)
        .token_account_owner(user_pubkey)
        .gating_program(program_test::AA_ID)
        .system_program(solana_system_interface::program::ID)
        .flag_account(flag_account)
        .instruction();

    tc.vm.expire_blockhash();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&user_pubkey),
        &[user.insecure_clone()],
        tc.vm.latest_blockhash(),
    );
    let res = tc.vm.send_transaction(tx);
    println!("res: {:?}", res);
    assert!(res.is_ok());

    let token_account_data = tc.vm.get_account(&user_token_account).unwrap().data;
    let account = StateWithExtensions::<Account>::unpack(token_account_data.as_ref()).unwrap();
    //println!("account: {:?}", account);
    assert_eq!(account.base.state, AccountState::Initialized);
}

#[tokio::test]
async fn test_create_ata_and_thaw_permissionless() {
    let mut tc = TestContext::new();

    let mint_cfg_pk = tc.setup_token_acl(&program_test::AA_WD_ID);

    tc.setup_aa_wd_gate_extra_metas();

    let ix = token_acl_client::instructions::TogglePermissionlessInstructionsBuilder::new()
        .authority(tc.token.auth.pubkey())
        .freeze_enabled(false)
        .thaw_enabled(true)
        .mint_config(mint_cfg_pk)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&tc.token.auth.pubkey()),
        &[tc.token.auth.insecure_clone()],
        tc.vm.latest_blockhash(),
    );
    let res = tc.vm.send_transaction(tx);
    //println!("res: {:?}", res);
    assert!(res.is_ok());

    let user = Keypair::new();
    let user_pubkey = user.pubkey();

    let mut instructions = Vec::new();

    let res = tc.vm.airdrop(&user.pubkey(), 1_000_000_000);
    assert!(res.is_ok());

    let token_account = get_associated_token_address_with_program_id(
        &user_pubkey,
        &tc.token.mint,
        &spl_token_2022::ID,
    );

    let ix = create_associated_token_account_idempotent(
        &user_pubkey,
        &user_pubkey,
        &tc.token.mint,
        &spl_token_2022::ID,
    );
    instructions.push(ix);

    let acc = Account {
        mint: tc.token.mint,
        owner: user_pubkey,
        amount: 0,
        delegate: COption::None,
        state: AccountState::Frozen,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    };

    let mut data = vec![0u8; Account::LEN];
    let res = Account::pack(acc, &mut data);
    assert!(res.is_ok());

    let ix = token_acl_client::create_thaw_permissionless_instruction_with_extra_metas(
        &user_pubkey,
        &token_account,
        &tc.token.mint,
        &mint_cfg_pk,
        &spl_token_2022::ID,
        &user_pubkey,
        false,
        |pubkey| {
            let data = data.clone();
            let data2 = tc.vm.get_account(&pubkey);
            async move {
                if pubkey == token_account {
                    return Ok(Some(data));
                }
                Ok(data2.map(|a| a.data.clone()))
            }
        },
    )
    .await
    .unwrap();

    instructions.push(ix);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&user_pubkey),
        &[user.insecure_clone()],
        tc.vm.latest_blockhash(),
    );
    let res = tc.vm.send_transaction(tx);
    assert!(res.is_ok());

    let token_account_data = tc.vm.get_account(&token_account).unwrap().data;
    let account = StateWithExtensions::<Account>::unpack(token_account_data.as_ref()).unwrap();
    //println!("account: {:?}", account);
    assert_eq!(account.base.state, AccountState::Initialized);

    // expire bh so we can submit same instructions
    tc.vm.expire_blockhash();

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&user_pubkey),
        &[user.insecure_clone()],
        tc.vm.latest_blockhash(),
    );
    let res = tc.vm.send_transaction(tx);
    println!("res: {:?}", res);
    //should err because not idempotent
    assert!(res.is_err());

    let ix2 = token_acl_client::create_thaw_permissionless_instruction_with_extra_metas(
        &user_pubkey,
        &token_account,
        &tc.token.mint,
        &mint_cfg_pk,
        &spl_token_2022::ID,
        &user_pubkey,
        true,
        |pubkey| {
            let data = data.clone();
            let data2 = tc.vm.get_account(&pubkey);
            async move {
                if pubkey == token_account {
                    return Ok(Some(data));
                }
                Ok(data2.map(|a| a.data.clone()))
            }
        },
    )
    .await
    .unwrap();

    println!("ix2 data: {:?}", ix2.data);

    instructions.remove(1);
    //instructions.push(ix2);

    let tx = Transaction::new_signed_with_payer(
        &[ix2],
        Some(&user_pubkey),
        &[user.insecure_clone()],
        tc.vm.latest_blockhash(),
    );
    let res = tc.vm.send_transaction(tx);
    println!("res: {:?}", res);
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_thaw_permissionless_always_block() {
    let mut tc = TestContext::new();
    let mint_cfg_pk = tc.setup_token_acl(&program_test::AB_ID);

    tc.setup_ab_gate_extra_metas();

    let user = Keypair::new();
    let user_pubkey = user.pubkey();
    let user_token_account = tc.create_token_account(&user);

    let flag_account = token_acl_client::accounts::FlagAccount::find_pda(&user_token_account).0;

    let ix = token_acl_client::instructions::ThawPermissionlessBuilder::new()
        .authority(user_pubkey)
        .mint(tc.token.mint)
        .mint_config(mint_cfg_pk)
        .token_account(user_token_account)
        .token_account_owner(user_pubkey)
        .gating_program(program_test::AB_ID)
        .system_program(solana_system_interface::program::ID)
        .flag_account(flag_account)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&user_pubkey),
        &[user.insecure_clone()],
        tc.vm.latest_blockhash(),
    );
    let res = tc.vm.send_transaction(tx);
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert_eq!(
        err.err,
        TransactionError::InstructionError(0x00, InstructionError::Custom(0x06))
    );

    let ix = token_acl_client::instructions::TogglePermissionlessInstructionsBuilder::new()
        .authority(tc.token.auth.pubkey())
        .freeze_enabled(false)
        .thaw_enabled(true)
        .mint_config(mint_cfg_pk)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&tc.token.auth.pubkey()),
        &[tc.token.auth.insecure_clone()],
        tc.vm.latest_blockhash(),
    );
    let res = tc.vm.send_transaction(tx);
    assert!(res.is_ok());

    let ix = token_acl_client::create_thaw_permissionless_instruction_with_extra_metas(
        &user_pubkey,
        &user_token_account,
        &tc.token.mint,
        &mint_cfg_pk,
        &spl_token_2022::ID,
        &user_pubkey,
        false,
        |pubkey| {
            println!("pubkey: {:?}", pubkey);
            let acc = tc.vm.get_account(&pubkey);
            async move {
                match acc {
                    Some(a) => Ok(Some(a.data)),
                    None => Ok(None),
                }
            }
        },
    )
    .await
    .unwrap();

    tc.vm.expire_blockhash();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&user_pubkey),
        &[user.insecure_clone()],
        tc.vm.latest_blockhash(),
    );
    let res = tc.vm.send_transaction(tx);
    println!("res: {:?}", res);
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert_eq!(
        err.err,
        TransactionError::InstructionError(0x00, InstructionError::Custom(999999999))
    );

    let token_account_data = tc.vm.get_account(&user_token_account).unwrap().data;
    let account = StateWithExtensions::<Account>::unpack(token_account_data.as_ref()).unwrap();
    //println!("account: {:?}", account);
    assert_eq!(account.base.state, AccountState::Frozen);
}

#[tokio::test]
async fn test_thaw_permissionless_always_allow_with_deps() {
    let mut tc = TestContext::new();
    let mint_cfg_pk = tc.setup_token_acl(&program_test::AA_WD_ID);

    tc.setup_aa_wd_gate_extra_metas();

    let user = Keypair::new();
    let user_pubkey = user.pubkey();
    let user_token_account = tc.create_token_account(&user);

    let ix = token_acl_client::instructions::TogglePermissionlessInstructionsBuilder::new()
        .authority(tc.token.auth.pubkey())
        .freeze_enabled(false)
        .thaw_enabled(true)
        .mint_config(mint_cfg_pk)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&tc.token.auth.pubkey()),
        &[tc.token.auth.insecure_clone()],
        tc.vm.latest_blockhash(),
    );
    let res = tc.vm.send_transaction(tx);
    //println!("res: {:?}", res);
    assert!(res.is_ok());

    let flag_account = token_acl_client::accounts::FlagAccount::find_pda(&user_token_account).0;
    let ix = token_acl_client::instructions::ThawPermissionlessBuilder::new()
        .authority(user_pubkey)
        .mint(tc.token.mint)
        .mint_config(mint_cfg_pk)
        .token_account(user_token_account)
        .token_account_owner(user_pubkey)
        .gating_program(program_test::AA_WD_ID)
        .system_program(solana_system_interface::program::ID)
        .flag_account(flag_account)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&user_pubkey),
        &[user.insecure_clone()],
        tc.vm.latest_blockhash(),
    );
    let res = tc.vm.send_transaction(tx);
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert_eq!(
        err.err,
        TransactionError::InstructionError(0x00, InstructionError::NotEnoughAccountKeys)
    );

    let ix = token_acl_client::instructions::ThawPermissionlessBuilder::new()
        .authority(user_pubkey)
        .mint(tc.token.mint)
        .mint_config(mint_cfg_pk)
        .token_account(user_token_account)
        .token_account_owner(user_pubkey)
        .token_program(spl_token_2022::ID)
        .gating_program(program_test::AA_WD_ID)
        .system_program(solana_system_interface::program::ID)
        .flag_account(flag_account)
        .add_remaining_account(AccountMeta::new(
            token_acl_interface::get_thaw_extra_account_metas_address(
                &tc.token.mint,
                &program_test::AA_WD_ID,
            ),
            false,
        ))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&user_pubkey),
        &[user.insecure_clone()],
        tc.vm.latest_blockhash(),
    );
    let res = tc.vm.send_transaction(tx);
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert_eq!(
        err.err,
        TransactionError::InstructionError(0x00, InstructionError::Custom(2_724_315_840)) // https://github.com/solana-program/libraries/blob/main/tlv-account-resolution/src/error.rs#L19
    );

    let extra_account_metas_address = token_acl_interface::get_thaw_extra_account_metas_address(
        &tc.token.mint,
        &program_test::AA_WD_ID,
    );
    let ata = get_associated_token_address_with_program_id(
        &user_pubkey,
        &tc.token.mint,
        &spl_token_2022::ID,
    );

    println!("ata: {:?}", ata);
    println!("mint_cfg_pk: {:?}", mint_cfg_pk);
    println!("user_pubkey: {:?}", user_pubkey);
    println!("user_token_account: {:?}", user_token_account);
    println!("tc.token.mint: {:?}", tc.token.mint);
    println!("spl_token_2022::ID: {:?}", spl_token_2022::ID);
    println!("extra_account_metas: {:?}", extra_account_metas_address);
    println!(
        "account: {:?}",
        tc.vm.get_account(&extra_account_metas_address)
    );

    let flag_account = token_acl_client::accounts::FlagAccount::find_pda(&user_token_account).0;
    println!("flag_account: {:?}", flag_account);

    let cb = solana_compute_budget_interface::ComputeBudgetInstruction::set_compute_unit_limit(
        1_400_000,
    );
    let ix = token_acl_client::create_thaw_permissionless_instruction_with_extra_metas(
        &user_pubkey,
        &user_token_account,
        &tc.token.mint,
        &mint_cfg_pk,
        &spl_token_2022::ID,
        &user_pubkey,
        false,
        |pubkey| {
            println!("pubkey: {:?}", pubkey);
            let acc = tc.vm.get_account(&pubkey);
            async move {
                match acc {
                    Some(a) => Ok(Some(a.data)),
                    None => Ok(None),
                }
            }
        },
    )
    .await
    .unwrap();

    tc.vm.expire_blockhash();

    let tx = Transaction::new_signed_with_payer(
        &[cb, ix],
        Some(&user_pubkey),
        &[user.insecure_clone()],
        tc.vm.latest_blockhash(),
    );
    let res = tc.vm.send_transaction(tx);
    println!("res: {:?}", res);
    assert!(res.is_ok());

    let token_account_data = tc.vm.get_account(&user_token_account).unwrap().data;
    let account = StateWithExtensions::<Account>::unpack(token_account_data.as_ref()).unwrap();
    //println!("account: {:?}", account);
    assert_eq!(account.base.state, AccountState::Initialized);
}

pub mod program_test;
use solana_instruction::AccountMeta;
use solana_sdk::{
    instruction::InstructionError,
    program_option::COption,
    signature::Keypair,
    signer::Signer,
    transaction::{Transaction, TransactionError},
};
use solana_system_interface::program::ID as SYSTEM_PROGRAM_ID;
use spl_associated_token_account_client::address::get_associated_token_address_with_program_id;
use spl_token_2022::{
    extension::StateWithExtensions,
    state::{Account, AccountState, Mint},
    ID as TOKEN_PROGRAM_ID,
};

use crate::program_test::TestContext;

#[test]
fn test_thaw_permissionless() {
    let mut tc = TestContext::new();

    let (mint_cfg_pk, bump) = ebalts_client::accounts::MintConfig::find_pda(&tc.token.mint);

    let ix = ebalts_client::instructions::CreateConfigBuilder::new()
        .authority(tc.token.auth.pubkey())
        .gating_program(program_test::AA_ID)
        .mint(tc.token.mint.clone())
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

    let cfg = ebalts_client::accounts::MintConfig::from_bytes(
        tc.vm.get_account(&mint_cfg_pk).unwrap().data.as_ref(),
    )
    .unwrap();
    assert_eq!(cfg.discriminator, 0x01);
    assert_eq!(cfg.mint, tc.token.mint);
    assert_eq!(cfg.freeze_authority, tc.token.auth.pubkey());
    assert_eq!(cfg.gating_program, program_test::AA_ID);
    assert_eq!(cfg.bump, bump);
    assert_eq!(cfg.enable_permissionless_freeze, false);
    assert_eq!(cfg.enable_permissionless_thaw, false);

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

    let ix = ebalts_client::instructions::ThawPermissionlessBuilder::new()
        .authority(user_pubkey)
        .mint(tc.token.mint.clone())
        .mint_config(mint_cfg_pk.clone())
        .token_account(user_token_account.clone())
        .token_account_owner(user_pubkey.clone())
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

    let ix = ebalts_client::instructions::TogglePermissionlessInstructionsBuilder::new()
        .authority(tc.token.auth.pubkey())
        .freeze_enabled(false)
        .thaw_enabled(true)
        .mint_config(mint_cfg_pk.clone())
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

    let ix = ebalts_client::instructions::ThawPermissionlessBuilder::new()
        .authority(user_pubkey)
        .mint(tc.token.mint.clone())
        .mint_config(mint_cfg_pk.clone())
        .token_account(user_token_account.clone())
        .token_account_owner(user_pubkey.clone())
        .gating_program(program_test::AA_ID)
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
async fn test_thaw_permissionless_always_block() {
    let mut tc = TestContext::new();
    let mint_cfg_pk = tc.setup_ebalts(&program_test::AB_ID);

    tc.setup_ab_gate_extra_metas();

    let user = Keypair::new();
    let user_pubkey = user.pubkey();
    let user_token_account = tc.create_token_account(&user);


    let ix = ebalts_client::instructions::ThawPermissionlessBuilder::new()
        .authority(user_pubkey)
        .mint(tc.token.mint.clone())
        .mint_config(mint_cfg_pk.clone())
        .token_account(user_token_account.clone())
        .token_account_owner(user_pubkey.clone())
        .gating_program(program_test::AB_ID)
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

    let ix = ebalts_client::instructions::TogglePermissionlessInstructionsBuilder::new()
        .authority(tc.token.auth.pubkey())
        .freeze_enabled(false)
        .thaw_enabled(true)
        .mint_config(mint_cfg_pk.clone())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&tc.token.auth.pubkey()),
        &[tc.token.auth.insecure_clone()],
        tc.vm.latest_blockhash(),
    );
    let res = tc.vm.send_transaction(tx);
    assert!(res.is_ok());


    let ix = ebalts_client::create_thaw_permissionless_instruction_with_extra_metas(
        &user_pubkey,
        &user_token_account,
        &tc.token.mint,
        &mint_cfg_pk,
        &spl_token_2022::ID,
        &user_pubkey,
        |pubkey| {
            println!("pubkey: {:?}", pubkey);
            let data = tc.vm.get_account(&pubkey).unwrap().data;
            async move {
                Ok(Some(data))
            }
        }
    ).await.unwrap();

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
    let mint_cfg_pk = tc.setup_ebalts(&program_test::AA_WD_ID);

    tc.setup_aa_wd_gate_extra_metas();

    let user = Keypair::new();
    let user_pubkey = user.pubkey();
    let user_token_account = tc.create_token_account(&user);

    let ix = ebalts_client::instructions::TogglePermissionlessInstructionsBuilder::new()
        .authority(tc.token.auth.pubkey())
        .freeze_enabled(false)
        .thaw_enabled(true)
        .mint_config(mint_cfg_pk.clone())
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

    
    let ix = ebalts_client::instructions::ThawPermissionlessBuilder::new()
        .authority(user_pubkey)
        .mint(tc.token.mint.clone())
        .mint_config(mint_cfg_pk.clone())
        .token_account(user_token_account.clone())
        .token_account_owner(user_pubkey.clone())
        .gating_program(program_test::AA_WD_ID)
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
    
    let ix = ebalts_client::instructions::ThawPermissionlessBuilder::new()
        .authority(user_pubkey)
        .mint(tc.token.mint.clone())
        .mint_config(mint_cfg_pk.clone())
        .token_account(user_token_account.clone())
        .token_account_owner(user_pubkey.clone())
        .token_program(spl_token_2022::ID)
        .gating_program(program_test::AA_WD_ID)
        .add_remaining_account(AccountMeta::new(ebalts_interface::get_thaw_extra_account_metas_address(
            &tc.token.mint,
            &program_test::AA_WD_ID,
        ), false))
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

    let extra_account_metas_address = ebalts_interface::get_thaw_extra_account_metas_address(
        &tc.token.mint,
        &program_test::AA_WD_ID,
    );
    let ata = get_associated_token_address_with_program_id(&user_pubkey, &tc.token.mint, &spl_token_2022::ID);
    
    println!("ata: {:?}", ata);
    println!("mint_cfg_pk: {:?}", mint_cfg_pk);
    println!("user_pubkey: {:?}", user_pubkey);
    println!("user_token_account: {:?}", user_token_account);
    println!("tc.token.mint: {:?}", tc.token.mint);
    println!("spl_token_2022::ID: {:?}", spl_token_2022::ID);
    println!("extra_account_metas: {:?}", extra_account_metas_address);
    println!("account: {:?}", tc.vm.get_account(&extra_account_metas_address));

    let cb = solana_compute_budget_interface::ComputeBudgetInstruction::set_compute_unit_limit(1_400_000);
    let ix = ebalts_client::create_thaw_permissionless_instruction_with_extra_metas(
        &user_pubkey,
        &user_token_account,
        &tc.token.mint,
        &mint_cfg_pk,
        &spl_token_2022::ID,
        &user_pubkey,
        |pubkey| {
            println!("pubkey: {:?}", pubkey);
            let acc = tc.vm.get_account(&pubkey);
            async move {
                match acc {
                    Some(a) => Ok(Some(a.data)),
                    None => Ok(None),
                }
            }
        }
    ).await.unwrap();

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

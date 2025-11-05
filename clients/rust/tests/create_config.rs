pub mod program_test;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};
use solana_system_interface::program::ID as SYSTEM_PROGRAM_ID;
use spl_token_2022::ID as TOKEN_PROGRAM_ID;

use crate::program_test::TestContext;

#[test]
fn test_create_mint_config() {
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
}

#[test]
fn test_create_mint_config_invalid_account() {
    let mut tc = TestContext::new();

    let new_token_context = TestContext::create_token(&mut tc.vm);

    let (mint_cfg_pk, _bump) =
        token_acl_client::accounts::MintConfig::find_pda(&new_token_context.mint);
    let (mint_cfg_pk_orig, _bump_orig) =
        token_acl_client::accounts::MintConfig::find_pda(&tc.token.mint);

    println!("expected mint_cfg_pk: {:?}", mint_cfg_pk_orig);
    println!("actual mint_cfg_pk: {:?}", mint_cfg_pk);

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
    assert!(res.is_err());

    let acc = tc.vm.get_account(&mint_cfg_pk);

    assert!(acc.is_none());
}

#[test]
fn test_create_mint_config_invalid_non_pda() {
    let mut tc = TestContext::new();

    let mint_cfg_kp = Keypair::new();
    let mint_cfg_pk = mint_cfg_kp.pubkey();

    let _ = tc.vm.airdrop(&mint_cfg_pk, 1_000_000_000);

    //println!("expected mint_cfg_pk: {:?}", mint_cfg_pk_orig);
    //println!("actual mint_cfg_pk: {:?}", mint_cfg_pk);

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
        Some(&mint_cfg_kp.pubkey()),
        &[tc.token.auth.insecure_clone(), mint_cfg_kp.insecure_clone()],
        tc.vm.latest_blockhash(),
    );

    let res = tc.vm.send_transaction(tx);
    println!("res: {:?}", res);
    assert!(res.is_err());
}

#[test]
fn test_create_mint_config_with_existing_config() {
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

    tc.vm.expire_blockhash();

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
    assert!(res.is_err());

}

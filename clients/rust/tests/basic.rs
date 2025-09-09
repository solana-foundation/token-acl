pub mod program_test;
use solana_sdk::{
    program_option::COption, signature::Keypair, signer::Signer, transaction::Transaction,
};
use spl_token_2022::{
    extension::StateWithExtensions,
    state::{Account, AccountState, Mint},
    ID as TOKEN_PROGRAM_ID,
};

use crate::program_test::TestContext;

#[test]
fn test_set_authority() {
    let mut tc = TestContext::new();
    let mint_cfg_pk = tc.setup_ebalts(&program_test::AA_ID);

    let mint_config = ebalts_client::accounts::MintConfig::from_bytes(
        tc.vm.get_account(&mint_cfg_pk).unwrap().data.as_ref(),
    )
    .unwrap();
    assert_eq!(mint_config.freeze_authority, tc.token.auth.pubkey());

    let new_authority = Keypair::new();
    let new_authority_pubkey = new_authority.pubkey();

    let ix = ebalts_client::instructions::SetAuthorityBuilder::new()
        .authority(tc.token.auth.pubkey())
        .new_authority(new_authority_pubkey)
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

    let mint_config = ebalts_client::accounts::MintConfig::from_bytes(
        tc.vm.get_account(&mint_cfg_pk).unwrap().data.as_ref(),
    )
    .unwrap();
    assert_eq!(mint_config.freeze_authority, new_authority_pubkey);
}

#[test]
fn test_set_gating_program() {
    let mut tc = TestContext::new();
    let mint_cfg_pk = tc.setup_ebalts(&program_test::AA_ID);

    let mint_config = ebalts_client::accounts::MintConfig::from_bytes(
        tc.vm.get_account(&mint_cfg_pk).unwrap().data.as_ref(),
    )
    .unwrap();
    assert_eq!(mint_config.gating_program, program_test::AA_ID);

    let new_gating_program = Keypair::new();
    let new_gating_program_pubkey = new_gating_program.pubkey();

    let ix = ebalts_client::instructions::SetGatingProgramBuilder::new()
        .authority(tc.token.auth.pubkey())
        .new_gating_program(new_gating_program_pubkey)
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
}

#[test]
fn test_toggle_permissionless_instructions() {
    let mut tc = TestContext::new();
    let mint_cfg_pk = tc.setup_ebalts(&program_test::AA_ID);

    let mint_config = ebalts_client::accounts::MintConfig::from_bytes(
        tc.vm.get_account(&mint_cfg_pk).unwrap().data.as_ref(),
    )
    .unwrap();
    assert!(!mint_config.enable_permissionless_freeze);
    assert!(!mint_config.enable_permissionless_thaw);

    let ix = ebalts_client::instructions::TogglePermissionlessInstructionsBuilder::new()
        .authority(tc.token.auth.pubkey())
        .freeze_enabled(true)
        .thaw_enabled(false)
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

    let mint_config = ebalts_client::accounts::MintConfig::from_bytes(
        tc.vm.get_account(&mint_cfg_pk).unwrap().data.as_ref(),
    )
    .unwrap();
    assert!(mint_config.enable_permissionless_freeze);
    assert!(!mint_config.enable_permissionless_thaw);

    let ix = ebalts_client::instructions::TogglePermissionlessInstructionsBuilder::new()
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

    let mint_config = ebalts_client::accounts::MintConfig::from_bytes(
        tc.vm.get_account(&mint_cfg_pk).unwrap().data.as_ref(),
    )
    .unwrap();
    assert!(!mint_config.enable_permissionless_freeze);
    assert!(mint_config.enable_permissionless_thaw);

    let ix = ebalts_client::instructions::TogglePermissionlessInstructionsBuilder::new()
        .authority(tc.token.auth.pubkey())
        .freeze_enabled(true)
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

    let mint_config = ebalts_client::accounts::MintConfig::from_bytes(
        tc.vm.get_account(&mint_cfg_pk).unwrap().data.as_ref(),
    )
    .unwrap();
    assert!(mint_config.enable_permissionless_freeze);
    assert!(mint_config.enable_permissionless_thaw);
}

#[test]
fn test_thaw_permissioned() {
    let mut tc = TestContext::new();
    let mint_cfg_pk = tc.setup_ebalts(&program_test::AA_ID);

    let user_kp = Keypair::new();
    let user_ata = tc.create_token_account(&user_kp);

    let user_ta = tc.vm.get_account(&user_ata).unwrap();
    let account = StateWithExtensions::<Account>::unpack(user_ta.data.as_ref()).unwrap();
    assert_eq!(account.base.state, AccountState::Frozen);

    let ix = ebalts_client::instructions::ThawBuilder::new()
        .authority(tc.token.auth.pubkey())
        .mint(tc.token.mint)
        .mint_config(mint_cfg_pk)
        .token_account(user_ata)
        .token_program(TOKEN_PROGRAM_ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&tc.token.auth.pubkey()),
        &[tc.token.auth.insecure_clone()],
        tc.vm.latest_blockhash(),
    );
    let res = tc.vm.send_transaction(tx);
    assert!(res.is_ok());

    let user_ta = tc.vm.get_account(&user_ata).unwrap();
    let account = StateWithExtensions::<Account>::unpack(user_ta.data.as_ref()).unwrap();
    assert_eq!(account.base.state, AccountState::Initialized);
}

#[test]
fn test_freeze_permissioned() {
    let mut tc = TestContext::new();
    let mint_cfg_pk = tc.setup_ebalts(&program_test::AA_ID);

    let user_kp = Keypair::new();
    let user_ata = tc.create_token_account(&user_kp);

    let ix = ebalts_client::instructions::ThawBuilder::new()
        .authority(tc.token.auth.pubkey())
        .mint(tc.token.mint)
        .mint_config(mint_cfg_pk)
        .token_account(user_ata)
        .token_program(TOKEN_PROGRAM_ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&tc.token.auth.pubkey()),
        &[tc.token.auth.insecure_clone()],
        tc.vm.latest_blockhash(),
    );
    let res = tc.vm.send_transaction(tx);
    assert!(res.is_ok());

    let user_ta = tc.vm.get_account(&user_ata).unwrap();
    let account = StateWithExtensions::<Account>::unpack(user_ta.data.as_ref()).unwrap();
    assert_eq!(account.base.state, AccountState::Initialized);

    let ix = ebalts_client::instructions::FreezeBuilder::new()
        .authority(tc.token.auth.pubkey())
        .mint(tc.token.mint)
        .mint_config(mint_cfg_pk)
        .token_account(user_ata)
        .token_program(TOKEN_PROGRAM_ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&tc.token.auth.pubkey()),
        &[tc.token.auth.insecure_clone()],
        tc.vm.latest_blockhash(),
    );
    let res = tc.vm.send_transaction(tx);
    assert!(res.is_ok());

    let user_ta = tc.vm.get_account(&user_ata).unwrap();
    let account = StateWithExtensions::<Account>::unpack(user_ta.data.as_ref()).unwrap();
    assert_eq!(account.base.state, AccountState::Frozen);
}

#[test]
fn test_delete_config() {
    let mut tc = TestContext::new();
    let mint_cfg_pk = tc.setup_ebalts(&program_test::AA_ID);

    let mint = tc.vm.get_account(&tc.token.mint).unwrap();
    let mint = StateWithExtensions::<Mint>::unpack(mint.data.as_ref()).unwrap();
    assert_eq!(mint.base.freeze_authority, COption::Some(mint_cfg_pk));

    let new_freeze_authority = Keypair::new();
    let new_freeze_authority_pubkey = new_freeze_authority.pubkey();

    let ix = ebalts_client::instructions::DeleteConfigBuilder::new()
        .authority(tc.token.auth.pubkey())
        .receiver(tc.token.auth.pubkey())
        .mint(tc.token.mint)
        .mint_config(mint_cfg_pk)
        .new_freeze_authority(new_freeze_authority_pubkey)
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

    let mint = tc.vm.get_account(&tc.token.mint).unwrap();
    let mint = StateWithExtensions::<Mint>::unpack(mint.data.as_ref()).unwrap();
    assert_eq!(
        mint.base.freeze_authority,
        COption::Some(new_freeze_authority_pubkey)
    );
}

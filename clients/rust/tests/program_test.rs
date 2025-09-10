use litesvm::LiteSVM;
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use solana_sdk::transaction::Transaction;
use solana_sdk::{signature::Keypair, signer::Signer};
use solana_system_interface::instruction::create_account;
use solana_system_interface::program::ID;
use spl_associated_token_account_client::address::get_associated_token_address_with_program_id;
use spl_associated_token_account_client::instruction::create_associated_token_account;
use spl_token_2022::extension::default_account_state::instruction::initialize_default_account_state;
use spl_token_2022::extension::ExtensionType;
use spl_token_2022::instruction::initialize_mint2;
use spl_token_2022::state::{AccountState, Mint};

pub const AA_ID: Pubkey = Pubkey::from_str_const("Eba1ts11111111111111111111111111111111111112");
pub const AB_ID: Pubkey = Pubkey::from_str_const("Eba1ts11111111111111111111111111111111111113");
pub const AA_WD_ID: Pubkey = Pubkey::from_str_const("Eba1ts11111111111111111111111111111111111114");

pub struct TestContext {
    pub vm: LiteSVM,
    pub token: TokenContext,
}

pub struct TokenContext {
    pub mint: Pubkey,
    pub auth: Keypair,
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TestContext {
    pub fn new() -> Self {
        let mut vm = LiteSVM::new();

        // current path
        let current_dir = std::env::current_dir().unwrap();

        let res = vm.add_program_from_file(
            token_acl_client::programs::TOKEN_ACL_ID,
            current_dir.join("tests/fixtures/token_acl.so"),
        );
        assert!(res.is_ok());

        let res = vm.add_program_from_file(
            AA_ID,
            current_dir.join("tests/fixtures/always_allow_gate_program.so"),
        );
        assert!(res.is_ok());

        let res = vm.add_program_from_file(
            AB_ID,
            current_dir.join("tests/fixtures/always_block_gate_program.so"),
        );
        assert!(res.is_ok());

        let res = vm.add_program_from_file(
            AA_WD_ID,
            current_dir.join("tests/fixtures/always_allow_with_deps_gate_program.so"),
        );
        assert!(res.is_ok());

        //let auth = Keypair::new();
        //let tokenKp = Keypair::new();
        //let auth_pubkey = auth.pubkey();

        let token = Self::create_token(&mut vm);

        Self { vm, token }
    }

    fn create_token(vm: &mut LiteSVM) -> TokenContext {
        let auth = Keypair::new();
        let auth_pubkey = auth.pubkey();

        let res = vm.airdrop(&auth_pubkey, 1_000_000_000_000);
        assert!(res.is_ok());

        let mint_size =
            ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::DefaultAccountState])
                .unwrap();
        let mint_kp = Keypair::new();
        let mint_pk = mint_kp.pubkey();
        let token_program_id = &spl_token_2022::ID;
        let payer_pk = auth.pubkey();

        let ix1 = create_account(
            &payer_pk,
            &mint_pk,
            vm.minimum_balance_for_rent_exemption(mint_size),
            mint_size as u64,
            token_program_id,
        );

        let ix2 =
            initialize_default_account_state(token_program_id, &mint_pk, &AccountState::Frozen)
                .unwrap();

        let ix3 = initialize_mint2(
            token_program_id,
            &mint_pk,
            &auth_pubkey,
            Some(&auth_pubkey),
            6,
        )
        .unwrap();

        let block_hash = vm.latest_blockhash();
        let tx = Transaction::new_signed_with_payer(
            &[ix1, ix2, ix3],
            Some(&payer_pk),
            &[auth.insecure_clone(), mint_kp],
            block_hash,
        );
        let res = vm.send_transaction(tx);
        assert!(res.is_ok());

        TokenContext {
            mint: mint_pk,
            auth,
        }
    }

    fn create_token_account_with_params(
        vm: &mut LiteSVM,
        mint: &Pubkey,
        owner: &Keypair,
    ) -> Pubkey {
        let token_program_id = &spl_token_2022::ID;
        let payer_pk = owner.pubkey();

        let res = vm.airdrop(&payer_pk, 1_000_000_000);
        assert!(res.is_ok());

        let token_account =
            get_associated_token_address_with_program_id(&owner.pubkey(), mint, token_program_id);

        let ix = create_associated_token_account(&payer_pk, &payer_pk, mint, token_program_id);

        let block_hash = vm.latest_blockhash();
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer_pk),
            &[owner.insecure_clone()],
            block_hash,
        );

        let res = vm.send_transaction(tx);
        assert!(res.is_ok());

        token_account
    }

    pub fn create_token_account(&mut self, owner: &Keypair) -> Pubkey {
        Self::create_token_account_with_params(&mut self.vm, &self.token.mint, owner)
    }

    pub fn get_setup_extra_metas_ix(&self, payer: &Pubkey, gating_program: &Pubkey) -> Instruction {
        Instruction::new_with_bytes(
            *gating_program,
            &[1, 1, 1, 1, 1, 1, 1, 1],
            vec![
                AccountMeta::new(*payer, true),
                AccountMeta::new_readonly(self.token.mint, false),
                AccountMeta::new(
                    token_acl_interface::get_thaw_extra_account_metas_address(
                        &self.token.mint,
                        gating_program,
                    ),
                    false,
                ),
                AccountMeta::new(
                    token_acl_interface::get_freeze_extra_account_metas_address(
                        &self.token.mint,
                        gating_program,
                    ),
                    false,
                ),
                AccountMeta::new_readonly(solana_system_interface::program::ID, false),
            ],
        )
    }

    pub fn setup_token_acl(&mut self, gating_program: &Pubkey) -> Pubkey {
        let (mint_cfg_pk, _) = token_acl_client::accounts::MintConfig::find_pda(&self.token.mint);

        let ix = token_acl_client::instructions::CreateConfigBuilder::new()
            .authority(self.token.auth.pubkey())
            .gating_program(*gating_program)
            .mint(self.token.mint)
            .mint_config(mint_cfg_pk)
            .payer(self.token.auth.pubkey())
            .system_program(ID)
            .token_program(spl_token_2022::ID)
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&self.token.auth.pubkey()),
            &[self.token.auth.insecure_clone()],
            self.vm.latest_blockhash(),
        );
        let res = self.vm.send_transaction(tx);

        assert!(res.is_ok());

        mint_cfg_pk
    }

    pub fn setup_aa_gate_extra_metas(&mut self) {
        let setup_extra_metas_ix = self.get_setup_extra_metas_ix(&self.token.auth.pubkey(), &AA_ID);
        let tx = Transaction::new_signed_with_payer(
            &[setup_extra_metas_ix],
            Some(&self.token.auth.pubkey()),
            &[self.token.auth.insecure_clone()],
            self.vm.latest_blockhash(),
        );
        let res = self.vm.send_transaction(tx);
        assert!(res.is_ok());
    }

    pub fn setup_ab_gate_extra_metas(&mut self) {
        let setup_extra_metas_ix = self.get_setup_extra_metas_ix(&self.token.auth.pubkey(), &AB_ID);
        let tx = Transaction::new_signed_with_payer(
            &[setup_extra_metas_ix],
            Some(&self.token.auth.pubkey()),
            &[self.token.auth.insecure_clone()],
            self.vm.latest_blockhash(),
        );
        let res = self.vm.send_transaction(tx);
        assert!(res.is_ok());
    }

    pub fn setup_aa_wd_gate_extra_metas(&mut self) {
        let setup_extra_metas_ix =
            self.get_setup_extra_metas_ix(&self.token.auth.pubkey(), &AA_WD_ID);
        let tx = Transaction::new_signed_with_payer(
            &[setup_extra_metas_ix],
            Some(&self.token.auth.pubkey()),
            &[self.token.auth.insecure_clone()],
            self.vm.latest_blockhash(),
        );
        let res = self.vm.send_transaction(tx);
        println!("res: {:?}", res);
        assert!(res.is_ok());
    }

    pub fn thaw(&mut self, token_account: &Pubkey) {
        let ix = token_acl_client::instructions::ThawBuilder::new()
            .authority(self.token.auth.pubkey())
            .mint(self.token.mint)
            .mint_config(token_acl_client::accounts::MintConfig::find_pda(&self.token.mint).0)
            .token_account(*token_account)
            .token_program(spl_token_2022::ID)
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&self.token.auth.pubkey()),
            &[self.token.auth.insecure_clone()],
            self.vm.latest_blockhash(),
        );
        let res = self.vm.send_transaction(tx);
        assert!(res.is_ok());
    }

    pub fn freeze(&mut self, token_account: &Pubkey) {
        let ix = token_acl_client::instructions::FreezeBuilder::new()
            .authority(self.token.auth.pubkey())
            .mint(self.token.mint)
            .mint_config(token_acl_client::accounts::MintConfig::find_pda(&self.token.mint).0)
            .token_account(*token_account)
            .token_program(spl_token_2022::ID)
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&self.token.auth.pubkey()),
            &[self.token.auth.insecure_clone()],
            self.vm.latest_blockhash(),
        );
        let res = self.vm.send_transaction(tx);
        assert!(res.is_ok());
    }
}

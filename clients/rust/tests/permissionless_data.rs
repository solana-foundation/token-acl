//! Payload forwarding for permissionless thaw/freeze.
//!
//! Uses the `examples/require-data` gate, which only allows the call when the
//! payload after its 8-byte discriminator equals `EXPECTED_PAYLOAD` and which
//! declares an extra account meta derived from the payload with a
//! `Seed::InstructionData` seed.
pub mod program_test;

use litesvm::types::TransactionMetadata;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use solana_sdk::{
    instruction::InstructionError,
    signature::Keypair,
    signer::Signer,
    transaction::{Transaction, TransactionError},
};
use spl_discriminator::SplDiscriminate;
use spl_token_2022_interface::{
    extension::StateWithExtensions,
    state::{Account, AccountState},
    ID as TOKEN_PROGRAM_ID,
};
use token_acl_interface::instruction::{
    CanFreezePermissionlessInstruction, CanThawPermissionlessInstruction,
};

use crate::program_test::{TestContext, AA_ID, AA_WD_ID, AB_ID, RD_ID};

/// Mirrors `require_data_gate_program::EXPECTED_PAYLOAD`.
const EXPECTED_PAYLOAD: [u8; 32] = *b"sRFC37 passthrough test payload!";
/// Mirrors `require_data_gate_program::CustomErrors::PayloadMismatch`.
const PAYLOAD_MISMATCH: u32 = 0x5001;
/// Always-block gate error.
const ALWAYS_BLOCK: u32 = 999_999_999;
/// spl-tlv-account-resolution `AccountResolutionError::InstructionDataTooSmall`.
const TLV_INSTRUCTION_DATA_TOO_SMALL: u32 = 2_724_315_851;
/// spl-tlv-account-resolution `AccountResolutionError::IncorrectAccount`.
const TLV_INCORRECT_ACCOUNT: u32 = 2_724_315_840;

fn payload_pda(payload: &[u8]) -> Pubkey {
    Pubkey::find_program_address(&[b"payload", payload], &RD_ID).0
}

fn wrong_payload() -> [u8; 32] {
    let mut p = EXPECTED_PAYLOAD;
    p[31] ^= 0xff;
    p
}

#[derive(Clone, Copy, PartialEq)]
enum Op {
    Thaw,
    Freeze,
}

struct Setup {
    tc: TestContext,
    mint_cfg: Pubkey,
    user: Keypair,
    token_account: Pubkey,
}

/// Creates a Token ACL mint gated by `gate`, enables the permissionless `op`
/// and creates a token account in the state `op` acts on (frozen for thaw,
/// thawed for freeze).
fn setup(gate: &Pubkey, op: Op) -> Setup {
    let mut tc = TestContext::new();
    let mint_cfg = tc.setup_token_acl(gate);
    tc.setup_gate_extra_metas(gate);

    let ix = token_acl_client::instructions::TogglePermissionlessInstructionsBuilder::new()
        .authority(tc.token.auth.pubkey())
        .freeze_enabled(op == Op::Freeze)
        .thaw_enabled(op == Op::Thaw)
        .mint_config(mint_cfg)
        .instruction();
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&tc.token.auth.pubkey()),
        &[tc.token.auth.insecure_clone()],
        tc.vm.latest_blockhash(),
    );
    assert!(tc.vm.send_transaction(tx).is_ok());

    let user = Keypair::new();
    let token_account = tc.create_token_account(&user);
    if op == Op::Freeze {
        tc.thaw(&token_account);
    }

    Setup {
        tc,
        mint_cfg,
        user,
        token_account,
    }
}

impl Setup {
    async fn build(&self, op: Op, idempotent: bool, data: &[u8]) -> Result<Instruction, String> {
        let vm = &self.tc.vm;
        let fetch = |pubkey: Pubkey| {
            let acc = vm.get_account(&pubkey);
            async move { Ok(acc.map(|a| a.data)) }
        };
        let user = self.user.pubkey();
        let res = match op {
            Op::Thaw => {
                token_acl_client::create_thaw_permissionless_instruction_with_extra_metas_and_data(
                    &user,
                    &self.token_account,
                    &self.tc.token.mint,
                    &self.mint_cfg,
                    &TOKEN_PROGRAM_ID,
                    &user,
                    idempotent,
                    data,
                    fetch,
                )
                .await
            }
            Op::Freeze => {
                token_acl_client::create_freeze_permissionless_instruction_with_extra_metas_and_data(
                    &user,
                    &self.token_account,
                    &self.tc.token.mint,
                    &self.mint_cfg,
                    &TOKEN_PROGRAM_ID,
                    &user,
                    idempotent,
                    data,
                    fetch,
                )
                .await
            }
        };
        res.map_err(|e| e.to_string())
    }

    /// Builds with the legacy (no data) helpers.
    async fn build_legacy(&self, op: Op, idempotent: bool) -> Instruction {
        let vm = &self.tc.vm;
        let fetch = |pubkey: Pubkey| {
            let acc = vm.get_account(&pubkey);
            async move { Ok(acc.map(|a| a.data)) }
        };
        let user = self.user.pubkey();
        match op {
            Op::Thaw => token_acl_client::create_thaw_permissionless_instruction_with_extra_metas(
                &user,
                &self.token_account,
                &self.tc.token.mint,
                &self.mint_cfg,
                &TOKEN_PROGRAM_ID,
                &user,
                idempotent,
                fetch,
            )
            .await
            .unwrap(),
            Op::Freeze => {
                token_acl_client::create_freeze_permissionless_instruction_with_extra_metas(
                    &user,
                    &self.token_account,
                    &self.tc.token.mint,
                    &self.mint_cfg,
                    &TOKEN_PROGRAM_ID,
                    &user,
                    idempotent,
                    fetch,
                )
                .await
                .unwrap()
            }
        }
    }

    fn send(&mut self, ix: Instruction) -> Result<TransactionMetadata, TransactionError> {
        self.tc.vm.expire_blockhash();
        let cb = solana_compute_budget_interface::ComputeBudgetInstruction::set_compute_unit_limit(
            1_400_000,
        );
        let tx = Transaction::new_signed_with_payer(
            &[cb, ix],
            Some(&self.user.pubkey()),
            &[self.user.insecure_clone()],
            self.tc.vm.latest_blockhash(),
        );
        let res = self.tc.vm.send_transaction(tx);
        match res {
            Ok(meta) => Ok(meta),
            Err(failed) => {
                println!("{}", failed.meta.pretty_logs());
                Err(failed.err)
            }
        }
    }

    fn state(&self) -> AccountState {
        let data = self.tc.vm.get_account(&self.token_account).unwrap().data;
        StateWithExtensions::<Account>::unpack(&data)
            .unwrap()
            .base
            .state
    }
}

fn target_state(op: Op) -> AccountState {
    match op {
        Op::Thaw => AccountState::Initialized,
        Op::Freeze => AccountState::Frozen,
    }
}

fn initial_state(op: Op) -> AccountState {
    match op {
        Op::Thaw => AccountState::Frozen,
        Op::Freeze => AccountState::Initialized,
    }
}

fn gate_discriminator(op: Op) -> &'static [u8] {
    match op {
        Op::Thaw => CanThawPermissionlessInstruction::SPL_DISCRIMINATOR_SLICE,
        Op::Freeze => CanFreezePermissionlessInstruction::SPL_DISCRIMINATOR_SLICE,
    }
}

/// Returns the data of every CPI Token ACL made into the gating program.
fn gate_cpi_data(meta: &TransactionMetadata, ix: &Instruction, gate: &Pubkey) -> Vec<Vec<u8>> {
    // LiteSVM reports inner instructions with program ids indexed into the
    // message account keys; rebuild the same message to map them back.
    let message = solana_sdk::message::Message::new(
        &[
            solana_compute_budget_interface::ComputeBudgetInstruction::set_compute_unit_limit(
                1_400_000,
            ),
            ix.clone(),
        ],
        Some(&ix.accounts[0].pubkey),
    );
    meta.inner_instructions
        .iter()
        .flatten()
        .filter(|inner| message.account_keys[inner.instruction.program_id_index as usize] == *gate)
        .map(|inner| inner.instruction.data.clone())
        .collect()
}

fn token_acl_discriminator(op: Op, idempotent: bool) -> u8 {
    match (op, idempotent) {
        (Op::Thaw, false) => 6,
        (Op::Freeze, false) => 7,
        (Op::Thaw, true) => 9,
        (Op::Freeze, true) => 10,
    }
}

async fn correct_data_passes(op: Op, idempotent: bool) {
    let mut s = setup(&RD_ID, op);
    let ix = s.build(op, idempotent, &EXPECTED_PAYLOAD).await.unwrap();

    // Token ACL instruction: 1-byte discriminator || payload.
    assert_eq!(ix.data[0], token_acl_discriminator(op, idempotent));
    assert_eq!(&ix.data[1..], &EXPECTED_PAYLOAD);
    // The payload-derived PDA was resolved off-chain from the payload.
    assert!(ix
        .accounts
        .iter()
        .any(|m| m.pubkey == payload_pda(&EXPECTED_PAYLOAD)));

    let meta = s.send(ix.clone()).expect("correct payload must pass");
    assert_eq!(s.state(), target_state(op));

    // The gate received sRFC37 discriminator || payload, verbatim.
    let cpis = gate_cpi_data(&meta, &ix, &RD_ID);
    assert_eq!(cpis.len(), 1);
    assert_eq!(&cpis[0][..8], gate_discriminator(op));
    assert_eq!(&cpis[0][8..], &EXPECTED_PAYLOAD);

    if idempotent {
        // Already in the target state: early return, gate not called, the
        // payload is irrelevant (even a missing one).
        let ix = s.build(op, true, &EXPECTED_PAYLOAD).await.unwrap();
        let mut no_data = ix.clone();
        no_data.data.truncate(1);
        let meta = s.send(no_data.clone()).expect("idempotent no-op must pass");
        assert!(gate_cpi_data(&meta, &no_data, &RD_ID).is_empty());
        assert_eq!(s.state(), target_state(op));
    }
}

async fn missing_data_fails(op: Op, idempotent: bool) {
    let mut s = setup(&RD_ID, op);

    // Off-chain resolution cannot derive the payload PDA without a payload.
    assert!(s.build(op, idempotent, &[]).await.is_err());
    // Neither can the legacy helper (which forwards no data).
    let vm = &s.tc.vm;
    let fetch = |pubkey: Pubkey| {
        let acc = vm.get_account(&pubkey);
        async move { Ok(acc.map(|a| a.data)) }
    };
    let legacy = match op {
        Op::Thaw => token_acl_client::create_thaw_permissionless_instruction_with_extra_metas(
            &s.user.pubkey(),
            &s.token_account,
            &s.tc.token.mint,
            &s.mint_cfg,
            &TOKEN_PROGRAM_ID,
            &s.user.pubkey(),
            idempotent,
            fetch,
        )
        .await
        .is_err(),
        Op::Freeze => token_acl_client::create_freeze_permissionless_instruction_with_extra_metas(
            &s.user.pubkey(),
            &s.token_account,
            &s.tc.token.mint,
            &s.mint_cfg,
            &TOKEN_PROGRAM_ID,
            &s.user.pubkey(),
            idempotent,
            fetch,
        )
        .await
        .is_err(),
    };
    assert!(legacy);

    // On-chain: correct accounts but the payload stripped. Token ACL's own
    // extra-meta resolution runs on the (now too short) gate instruction.
    let mut ix = s.build(op, idempotent, &EXPECTED_PAYLOAD).await.unwrap();
    ix.data.truncate(1);
    let err = s.send(ix).unwrap_err();
    assert_eq!(
        err,
        TransactionError::InstructionError(
            1,
            InstructionError::Custom(TLV_INSTRUCTION_DATA_TOO_SMALL)
        )
    );
    assert_eq!(s.state(), initial_state(op));
}

async fn wrong_data_fails(op: Op, idempotent: bool) {
    let mut s = setup(&RD_ID, op);

    // Same length, different bytes: metas resolve consistently, gate rejects.
    let ix = s.build(op, idempotent, &wrong_payload()).await.unwrap();
    let err = s.send(ix).unwrap_err();
    assert_eq!(
        err,
        TransactionError::InstructionError(1, InstructionError::Custom(PAYLOAD_MISMATCH))
    );

    // Expected payload plus a trailing byte: forwarded verbatim, gate rejects.
    let mut long = EXPECTED_PAYLOAD.to_vec();
    long.push(0);
    let ix = s.build(op, idempotent, &long).await.unwrap();
    let err = s.send(ix).unwrap_err();
    assert_eq!(
        err,
        TransactionError::InstructionError(1, InstructionError::Custom(PAYLOAD_MISMATCH))
    );

    // Expected payload but extra metas resolved for a different payload:
    // Token ACL's on-chain resolution (which reads the forwarded payload)
    // rejects the mismatched account before the gate runs.
    let good = s.build(op, idempotent, &EXPECTED_PAYLOAD).await.unwrap();
    let mut ix = s.build(op, idempotent, &wrong_payload()).await.unwrap();
    ix.data = good.data;
    let err = s.send(ix).unwrap_err();
    assert_eq!(
        err,
        TransactionError::InstructionError(1, InstructionError::Custom(TLV_INCORRECT_ACCOUNT))
    );

    assert_eq!(s.state(), initial_state(op));
}

#[tokio::test]
async fn test_thaw_permissionless_with_correct_data() {
    correct_data_passes(Op::Thaw, false).await;
}

#[tokio::test]
async fn test_thaw_permissionless_idempotent_with_correct_data() {
    correct_data_passes(Op::Thaw, true).await;
}

#[tokio::test]
async fn test_freeze_permissionless_with_correct_data() {
    correct_data_passes(Op::Freeze, false).await;
}

#[tokio::test]
async fn test_freeze_permissionless_idempotent_with_correct_data() {
    correct_data_passes(Op::Freeze, true).await;
}

#[tokio::test]
async fn test_thaw_permissionless_missing_data_fails() {
    missing_data_fails(Op::Thaw, false).await;
}

#[tokio::test]
async fn test_thaw_permissionless_idempotent_missing_data_fails() {
    missing_data_fails(Op::Thaw, true).await;
}

#[tokio::test]
async fn test_freeze_permissionless_missing_data_fails() {
    missing_data_fails(Op::Freeze, false).await;
}

#[tokio::test]
async fn test_freeze_permissionless_idempotent_missing_data_fails() {
    missing_data_fails(Op::Freeze, true).await;
}

#[tokio::test]
async fn test_thaw_permissionless_wrong_data_fails() {
    wrong_data_fails(Op::Thaw, false).await;
}

#[tokio::test]
async fn test_thaw_permissionless_idempotent_wrong_data_fails() {
    wrong_data_fails(Op::Thaw, true).await;
}

#[tokio::test]
async fn test_freeze_permissionless_wrong_data_fails() {
    wrong_data_fails(Op::Freeze, false).await;
}

#[tokio::test]
async fn test_freeze_permissionless_idempotent_wrong_data_fails() {
    wrong_data_fails(Op::Freeze, true).await;
}

/// Existing gates keep working with no data, the `_with_data` helpers with an
/// empty payload build byte-identical instructions to the legacy helpers, and
/// the gate CPI carries only the 8-byte discriminator (as before).
async fn existing_gate_without_data(gate: Pubkey, op: Op, idempotent: bool) {
    let mut s = setup(&gate, op);

    let legacy = s.build_legacy(op, idempotent).await;
    let with_empty = s.build(op, idempotent, &[]).await.unwrap();
    assert_eq!(legacy, with_empty);
    assert_eq!(legacy.data, vec![token_acl_discriminator(op, idempotent)]);

    let res = s.send(legacy.clone());
    if gate == AB_ID {
        assert_eq!(
            res.unwrap_err(),
            TransactionError::InstructionError(1, InstructionError::Custom(ALWAYS_BLOCK))
        );
        assert_eq!(s.state(), initial_state(op));
    } else {
        let meta = res.expect("existing gate must pass without data");
        assert_eq!(s.state(), target_state(op));
        assert_eq!(
            gate_cpi_data(&meta, &legacy, &gate),
            vec![gate_discriminator(op).to_vec()]
        );
    }
}

#[tokio::test]
async fn test_existing_gates_without_data() {
    for gate in [AA_ID, AB_ID, AA_WD_ID] {
        for op in [Op::Thaw, Op::Freeze] {
            for idempotent in [false, true] {
                existing_gate_without_data(gate, op, idempotent).await;
            }
        }
    }
}

/// Gates that ignore the payload are unaffected by one being forwarded.
#[tokio::test]
async fn test_existing_gate_ignores_forwarded_data() {
    for gate in [AA_ID, AA_WD_ID] {
        for op in [Op::Thaw, Op::Freeze] {
            let mut s = setup(&gate, op);
            let ix = s.build(op, false, b"ignored").await.unwrap();
            let meta = s.send(ix.clone()).expect("payload-agnostic gate must pass");
            assert_eq!(s.state(), target_state(op));
            let cpis = gate_cpi_data(&meta, &ix, &gate);
            assert_eq!(cpis.len(), 1);
            assert_eq!(&cpis[0][8..], b"ignored");
        }
    }
}

/// The generated builders still work without data, and `instruction_with_data`
/// only appends the payload.
#[test]
fn test_builders_with_and_without_data() {
    let k = Pubkey::new_unique();
    let mut b = token_acl_client::instructions::ThawPermissionlessBuilder::new();
    b.authority(k)
        .mint(k)
        .token_account(k)
        .flag_account(k)
        .token_account_owner(k)
        .mint_config(k)
        .gating_program(k);
    assert_eq!(b.instruction().data, vec![6]);
    assert_eq!(b.instruction_with_data(&[]), b.instruction());
    assert_eq!(b.instruction_with_data(&[1, 2, 3]).data, vec![6, 1, 2, 3]);

    let mut b = token_acl_client::instructions::FreezePermissionlessIdempotentBuilder::new();
    b.authority(k)
        .mint(k)
        .token_account(k)
        .flag_account(k)
        .token_account_owner(k)
        .mint_config(k)
        .gating_program(k);
    assert_eq!(b.instruction().data, vec![10]);
    assert_eq!(b.instruction_with_data(&[4]).data, vec![10, 4]);
}

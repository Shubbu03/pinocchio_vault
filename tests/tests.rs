use mollusk_svm::result::{Check, ProgramResult};
use mollusk_svm::{program, Mollusk};
use solana_sdk::account::Account;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
extern crate alloc;
use alloc::vec;

use pinocchio_vault::instructions::*;
use pinocchio_vault::states::{to_bytes, DataLen as _, VaultState};
use solana_sdk::rent::Rent;
use solana_sdk::sysvar::SysvarSerialize as _;

pub const PROGRAM: Pubkey = pubkey!("63vgRZotq9C4krvqWcVjWHgw1gaZTXuYu76sSbosq6ca");

pub const RENT: Pubkey = pubkey!("SysvarRent111111111111111111111111111111111");

pub const PAYER: Pubkey = pubkey!("EcgxCCyx5YrFTN6WeQ9ioX6CGZVgWsbyXxzNSAZDzdVT");

pub fn mollusk() -> Mollusk {
    let mollusk = Mollusk::new(
        &PROGRAM,
        "target/sbpf-solana-solana/release/pinocchio_vault",
    );
    mollusk
}

#[test]
fn test_close() {
    let mollusk = mollusk();

    // System program and account
    let (system_program, system_account) = program::keyed_account_for_system_program();

    // Derive PDA and bump
    let (vault_state_pda, bump) =
        Pubkey::find_program_address(&[VaultState::SEED.as_bytes(), &PAYER.to_bytes()], &PROGRAM);

    // Base accounts and rent sysvar
    let payer_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);
    let vault_state_account = Account::new(0, 0, &system_program);
    let min_balance = mollusk.sysvars.rent.minimum_balance(Rent::size_of());
    let mut rent_account = Account::new(min_balance, Rent::size_of(), &RENT);
    rent_account.data = get_rent_data();

    // ---------- 1) Initialize ----------
    let init_ix_accounts = vec![
        AccountMeta::new(PAYER, true),
        AccountMeta::new(vault_state_pda, false),
        AccountMeta::new_readonly(RENT, false),
        AccountMeta::new_readonly(system_program, false),
    ];
    let init_ix_data = Init { bump };
    let mut ser_init_ix_data = vec![0]; // discriminator for init
    ser_init_ix_data.extend_from_slice(unsafe { to_bytes(&init_ix_data) });

    let init_instruction =
        Instruction::new_with_bytes(PROGRAM, &ser_init_ix_data, init_ix_accounts);

    let init_tx_accounts = &vec![
        (PAYER, payer_account.clone()),
        (vault_state_pda, vault_state_account.clone()),
        (RENT, rent_account.clone()),
        (system_program, system_account.clone()),
    ];

    let init_res = mollusk.process_and_validate_instruction(
        &init_instruction,
        init_tx_accounts,
        &[Check::success()],
    );
    assert!(init_res.program_result == ProgramResult::Success);

    // ---------- 2) Deposit ----------
    let rent_exempt_lamports = mollusk.sysvars.rent.minimum_balance(VaultState::LEN);
    let deposit_amount = 50_000_000; // small deposit

    let deposit_ix_accounts = vec![
        AccountMeta::new(PAYER, true),
        AccountMeta::new(vault_state_pda, false),
        AccountMeta::new_readonly(system_program, false),
    ];
    let deposit_ix_data = Deposit {
        amount: deposit_amount,
        bump,
    };
    let mut ser_deposit_ix_data = vec![1];
    ser_deposit_ix_data.extend_from_slice(unsafe { to_bytes(&deposit_ix_data) });

    let deposit_instruction =
        Instruction::new_with_bytes(PROGRAM, &ser_deposit_ix_data, deposit_ix_accounts);

    let payer_pre_deposit = Account::new(
        1 * LAMPORTS_PER_SOL - rent_exempt_lamports,
        0,
        &system_program,
    );

    let vault_state = VaultState {
        owner: PAYER.to_bytes(),
    };
    let mut vault_pre_deposit = Account::new(rent_exempt_lamports, VaultState::LEN, &PROGRAM);
    vault_pre_deposit.data = unsafe { to_bytes(&vault_state) }.to_vec();

    let deposit_tx_accounts = &vec![
        (PAYER, payer_pre_deposit),
        (vault_state_pda, vault_pre_deposit),
        (system_program, system_account.clone()),
    ];

    let deposit_res = mollusk.process_and_validate_instruction(
        &deposit_instruction,
        deposit_tx_accounts,
        &[Check::success()],
    );
    assert!(deposit_res.program_result == ProgramResult::Success);

    // ---------- 3) Close ----------
    let close_ix_accounts = vec![
        AccountMeta::new(PAYER, true),
        AccountMeta::new(vault_state_pda, false),
        AccountMeta::new_readonly(system_program, false),
    ];
    let close_ix_data = Close { bump };
    let mut ser_close_ix_data = vec![3];
    ser_close_ix_data.extend_from_slice(unsafe { to_bytes(&close_ix_data) });

    let close_instruction =
        Instruction::new_with_bytes(PROGRAM, &ser_close_ix_data, close_ix_accounts);

    // Pre-close state (after deposit)
    let payer_pre_close = Account::new(
        1 * LAMPORTS_PER_SOL - rent_exempt_lamports - deposit_amount,
        0,
        &system_program,
    );
    let mut vault_pre_close = Account::new(
        rent_exempt_lamports + deposit_amount,
        VaultState::LEN,
        &PROGRAM,
    );
    vault_pre_close.data = unsafe { to_bytes(&vault_state) }.to_vec();

    let close_tx_accounts = &vec![
        (PAYER, payer_pre_close),
        (vault_state_pda, vault_pre_close),
        (system_program, system_account.clone()),
    ];

    let close_res = mollusk.process_and_validate_instruction(
        &close_instruction,
        close_tx_accounts,
        &[Check::success()],
    );
    assert!(close_res.program_result == ProgramResult::Success);
}

pub fn get_rent_data() -> Vec<u8> {
    let rent = Rent::default();
    unsafe {
        core::slice::from_raw_parts(&rent as *const Rent as *const u8, Rent::size_of()).to_vec()
    }
}

#[test]
fn test_initialize() {
    let mollusk = mollusk();

    //system program and system account
    let (system_program, system_account) = program::keyed_account_for_system_program();

    // Create the PDA
    let (vault_state_pda, bump) =
        Pubkey::find_program_address(&[VaultState::SEED.as_bytes(), &PAYER.to_bytes()], &PROGRAM);

    //Initialize the accounts
    let payer_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);
    let vault_state_account = Account::new(0, 0, &system_program);
    let min_balance = mollusk.sysvars.rent.minimum_balance(Rent::size_of());
    let mut rent_account = Account::new(min_balance, Rent::size_of(), &RENT);
    rent_account.data = get_rent_data();

    //Push the accounts in to the instruction_accounts vec!
    let ix_accounts = vec![
        AccountMeta::new(PAYER, true),
        AccountMeta::new(vault_state_pda, false),
        AccountMeta::new_readonly(RENT, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    // Create the instruction data
    let ix_data = Init { bump };

    // Ix discriminator = 0
    let mut ser_ix_data = vec![0];

    // Serialize the instruction data
    ser_ix_data.extend_from_slice(unsafe { to_bytes(&ix_data) });

    // Create instruction
    let instruction = Instruction::new_with_bytes(PROGRAM, &ser_ix_data, ix_accounts);

    // Create tx_accounts vec
    let tx_accounts = &vec![
        (PAYER, payer_account.clone()),
        (vault_state_pda, vault_state_account.clone()),
        (RENT, rent_account.clone()),
        (system_program, system_account.clone()),
    ];

    let init_res =
        mollusk.process_and_validate_instruction(&instruction, tx_accounts, &[Check::success()]);

    assert!(init_res.program_result == ProgramResult::Success);
}

#[test]
fn test_deposit() {
    let mollusk = mollusk();

    // First, we need to initialize the vault (prerequisite)
    let (system_program, system_account) = program::keyed_account_for_system_program();
    let (vault_state_pda, bump) =
        Pubkey::find_program_address(&[VaultState::SEED.as_bytes(), &PAYER.to_bytes()], &PROGRAM);

    // Initialize vault first
    let payer_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);
    let vault_state_account = Account::new(0, 0, &system_program); // Empty initially
    let min_balance = mollusk.sysvars.rent.minimum_balance(Rent::size_of());
    let mut rent_account = Account::new(min_balance, Rent::size_of(), &RENT);
    rent_account.data = get_rent_data();

    // Initialize vault
    let init_ix_accounts = vec![
        AccountMeta::new(PAYER, true),
        AccountMeta::new(vault_state_pda, false),
        AccountMeta::new_readonly(RENT, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let init_ix_data = Init { bump };
    let mut ser_init_ix_data = vec![0]; // discriminator for init
    ser_init_ix_data.extend_from_slice(unsafe { to_bytes(&init_ix_data) });

    let init_instruction =
        Instruction::new_with_bytes(PROGRAM, &ser_init_ix_data, init_ix_accounts);
    let init_tx_accounts = &vec![
        (PAYER, payer_account.clone()),
        (vault_state_pda, vault_state_account.clone()),
        (RENT, rent_account.clone()),
        (system_program, system_account.clone()),
    ];

    let init_res = mollusk.process_and_validate_instruction(
        &init_instruction,
        init_tx_accounts,
        &[Check::success()],
    );
    assert!(init_res.program_result == ProgramResult::Success);

    // Now test deposit
    let deposit_amount = 100_000_000; // 0.1 SOL in lamports
    let deposit_ix_data = Deposit {
        amount: deposit_amount,
        bump,
    };

    let mut ser_deposit_ix_data = vec![1]; // discriminator for deposit
    ser_deposit_ix_data.extend_from_slice(unsafe { to_bytes(&deposit_ix_data) });

    let deposit_ix_accounts = vec![
        AccountMeta::new(PAYER, true),
        AccountMeta::new(vault_state_pda, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let deposit_instruction =
        Instruction::new_with_bytes(PROGRAM, &ser_deposit_ix_data, deposit_ix_accounts);

    // Set up accounts as they should be AFTER initialization but BEFORE deposit
    // The vault account now has:
    // 1. Rent-exempt lamports (from init)
    // 2. Our program as owner (from init)
    // 3. VaultState data with owner pubkey (from init)
    let rent_exempt_lamports = mollusk.sysvars.rent.minimum_balance(VaultState::LEN);

    // Pre-deposit balances (post-init state)
    let payer_pre_deposit = Account::new(
        1 * LAMPORTS_PER_SOL - rent_exempt_lamports,
        0,
        &system_program,
    );

    // Vault state data and account (post-init)
    let vault_state = VaultState {
        owner: PAYER.to_bytes(),
    };
    let mut vault_pre_deposit = Account::new(rent_exempt_lamports, VaultState::LEN, &PROGRAM);
    vault_pre_deposit.data = unsafe { to_bytes(&vault_state) }.to_vec();

    let deposit_tx_accounts = &vec![
        (PAYER, payer_pre_deposit),
        (vault_state_pda, vault_pre_deposit),
        (system_program, system_account.clone()),
    ];

    let deposit_res = mollusk.process_and_validate_instruction(
        &deposit_instruction,
        deposit_tx_accounts,
        &[Check::success()],
    );

    assert!(deposit_res.program_result == ProgramResult::Success);
}

#[test]
fn test_withdraw() {
    let mollusk = mollusk();

    // System program and account
    let (system_program, system_account) = program::keyed_account_for_system_program();

    // Derive PDA and bump
    let (vault_state_pda, bump) =
        Pubkey::find_program_address(&[VaultState::SEED.as_bytes(), &PAYER.to_bytes()], &PROGRAM);

    // Base accounts and rent sysvar
    let payer_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);
    let vault_state_account = Account::new(0, 0, &system_program);
    let min_balance = mollusk.sysvars.rent.minimum_balance(Rent::size_of());
    let mut rent_account = Account::new(min_balance, Rent::size_of(), &RENT);
    rent_account.data = get_rent_data();

    // ---------- 1) Initialize ----------
    let init_ix_accounts = vec![
        AccountMeta::new(PAYER, true),
        AccountMeta::new(vault_state_pda, false),
        AccountMeta::new_readonly(RENT, false),
        AccountMeta::new_readonly(system_program, false),
    ];
    let init_ix_data = Init { bump };
    let mut ser_init_ix_data = vec![0]; // discriminator for init
    ser_init_ix_data.extend_from_slice(unsafe { to_bytes(&init_ix_data) });

    let init_instruction =
        Instruction::new_with_bytes(PROGRAM, &ser_init_ix_data, init_ix_accounts);

    let init_tx_accounts = &vec![
        (PAYER, payer_account.clone()),
        (vault_state_pda, vault_state_account.clone()),
        (RENT, rent_account.clone()),
        (system_program, system_account.clone()),
    ];

    let init_res = mollusk.process_and_validate_instruction(
        &init_instruction,
        init_tx_accounts,
        &[Check::success()],
    );
    assert!(init_res.program_result == ProgramResult::Success);

    // ---------- 2) Deposit ----------
    let rent_exempt_lamports = mollusk.sysvars.rent.minimum_balance(VaultState::LEN);
    let deposit_amount = 200_000_000; // 0.2 SOL

    let deposit_ix_accounts = vec![
        AccountMeta::new(PAYER, true),
        AccountMeta::new(vault_state_pda, false),
        AccountMeta::new_readonly(system_program, false),
    ];
    let deposit_ix_data = Deposit {
        amount: deposit_amount,
        bump,
    };
    let mut ser_deposit_ix_data = vec![1]; // discriminator for deposit
    ser_deposit_ix_data.extend_from_slice(unsafe { to_bytes(&deposit_ix_data) });

    let deposit_instruction =
        Instruction::new_with_bytes(PROGRAM, &ser_deposit_ix_data, deposit_ix_accounts);

    // Pre-deposit state (post-init)
    let payer_pre_deposit = Account::new(
        1 * LAMPORTS_PER_SOL - rent_exempt_lamports,
        0,
        &system_program,
    );

    let vault_state = VaultState {
        owner: PAYER.to_bytes(),
    };
    let mut vault_pre_deposit = Account::new(rent_exempt_lamports, VaultState::LEN, &PROGRAM);
    vault_pre_deposit.data = unsafe { to_bytes(&vault_state) }.to_vec();

    let deposit_tx_accounts = &vec![
        (PAYER, payer_pre_deposit),
        (vault_state_pda, vault_pre_deposit),
        (system_program, system_account.clone()),
    ];

    let deposit_res = mollusk.process_and_validate_instruction(
        &deposit_instruction,
        deposit_tx_accounts,
        &[Check::success()],
    );
    assert!(deposit_res.program_result == ProgramResult::Success);

    // ---------- 3) Withdraw ----------
    // Choose an amount that keeps the vault ≥ rent_exempt after withdrawal
    let withdraw_amount = 100_000_000; // 0.1 SOL

    let withdraw_ix_accounts = vec![
        AccountMeta::new(PAYER, true),
        AccountMeta::new(vault_state_pda, false),
        AccountMeta::new_readonly(RENT, false),
        AccountMeta::new_readonly(system_program, false),
    ];
    let withdraw_ix_data = Withdraw {
        amount: withdraw_amount,
        bump,
    };
    let mut ser_withdraw_ix_data = vec![2]; // discriminator for withdraw
    ser_withdraw_ix_data.extend_from_slice(unsafe { to_bytes(&withdraw_ix_data) });

    let withdraw_instruction =
        Instruction::new_with_bytes(PROGRAM, &ser_withdraw_ix_data, withdraw_ix_accounts);

    // Pre-withdraw state (post-deposit)
    let payer_pre_withdraw = Account::new(
        1 * LAMPORTS_PER_SOL - rent_exempt_lamports - deposit_amount,
        0,
        &system_program,
    );

    let mut vault_pre_withdraw = Account::new(
        rent_exempt_lamports + deposit_amount,
        VaultState::LEN,
        &PROGRAM,
    );
    vault_pre_withdraw.data = unsafe { to_bytes(&vault_state) }.to_vec();

    // Ensure remaining ≥ rent-exempt after withdraw
    assert!((rent_exempt_lamports + deposit_amount) - withdraw_amount >= rent_exempt_lamports);

    let withdraw_tx_accounts = &vec![
        (PAYER, payer_pre_withdraw),
        (vault_state_pda, vault_pre_withdraw),
        (RENT, rent_account.clone()),
        (system_program, system_account.clone()),
    ];

    let withdraw_res = mollusk.process_and_validate_instruction(
        &withdraw_instruction,
        withdraw_tx_accounts,
        &[Check::success()],
    );
    assert!(withdraw_res.program_result == ProgramResult::Success);
}

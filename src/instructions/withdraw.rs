use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    sysvars::rent::Rent,
    ProgramResult,
};

use pinocchio_system::instructions::Transfer;

use crate::states::{load_acc_unchecked, load_ix_data, DataLen, VaultState};

#[repr(C)]
pub struct Withdraw {
    pub amount: u64,
    pub bump: u8,
}

impl DataLen for Withdraw {
    const LEN: usize = core::mem::size_of::<Withdraw>();
}

pub fn withdraw_from_vault(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [user, vault, rent_sysvar, _] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !user.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    };

    let withdraw_ix_data = unsafe { load_ix_data::<Withdraw>(data)? };

    if withdraw_ix_data.amount.eq(&0) {
        return Err(ProgramError::InvalidInstructionData);
    };

    VaultState::validate_pda(withdraw_ix_data.bump, vault.key(), user.key())?;

    let vault_state = unsafe { load_acc_unchecked::<VaultState>(vault.borrow_data_unchecked()) }?;

    if vault_state.owner != *user.key() {
        return Err(ProgramError::InvalidAccountOwner);
    }

    //check if enough for this withdrwal
    if vault.lamports() < withdraw_ix_data.amount {
        return Err(ProgramError::InsufficientFunds);
    };

    //check min amount after withdrawl should be enough for rent
    let rent = Rent::from_account_info(rent_sysvar)?;
    let min_balance = rent.minimum_balance(VaultState::LEN);

    if vault.lamports() - withdraw_ix_data.amount < min_balance {
        return Err(ProgramError::InsufficientFunds);
    }

    let bump_bytes = [withdraw_ix_data.bump];

    let signer_seeds = [
        Seed::from(VaultState::SEED.as_bytes()),
        Seed::from(user.key()),
        Seed::from(&bump_bytes[..]),
    ];

    let signers = &[Signer::from(&signer_seeds[..])];

    Transfer {
        from: vault,
        to: user,
        lamports: withdraw_ix_data.amount,
    }
    .invoke_signed(signers)?;

    Ok(())
}

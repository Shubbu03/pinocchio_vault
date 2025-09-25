use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, sysvars::rent::Rent, ProgramResult,
};

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

    // Move lamports directly. Since our program owns `vault` and it carries data,
    // using a CPI to the system program would fail (system transfer requires empty data).
    unsafe {
        let vault_lamports = vault.borrow_mut_lamports_unchecked();
        let user_lamports = user.borrow_mut_lamports_unchecked();

        *vault_lamports = (*vault_lamports)
            .checked_sub(withdraw_ix_data.amount)
            .ok_or(ProgramError::InsufficientFunds)?;
        *user_lamports = (*user_lamports)
            .checked_add(withdraw_ix_data.amount)
            .ok_or(ProgramError::InsufficientFunds)?;
    }

    Ok(())
}

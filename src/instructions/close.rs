use crate::states::{load_acc_unchecked, load_ix_data, DataLen, VaultState};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

#[repr(C)]
pub struct Close {
    pub bump: u8,
}

impl DataLen for Close {
    const LEN: usize = core::mem::size_of::<Close>();
}

pub fn close_vault(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [user, vault, _] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !user.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    };

    let close_ix_data = unsafe { load_ix_data::<Close>(data)? };

    VaultState::validate_pda(close_ix_data.bump, vault.key(), user.key())?;

    let vault_state = unsafe { load_acc_unchecked::<VaultState>(vault.borrow_data_unchecked()) }?;

    if vault_state.owner != *user.key() {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Move all lamports directly; system transfer would reject `from` with data.
    unsafe {
        let vault_lamports = vault.borrow_mut_lamports_unchecked();
        let user_lamports = user.borrow_mut_lamports_unchecked();
        let amount = *vault_lamports;
        *vault_lamports = 0;
        *user_lamports = (*user_lamports)
            .checked_add(amount)
            .ok_or(ProgramError::InsufficientFunds)?;
    }

    // After draining lamports, Solana runtime would garbage collect rent-ineligible accounts.
    Ok(())
}

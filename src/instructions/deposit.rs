use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

use pinocchio_system::instructions::Transfer;

use crate::states::{load_acc_unchecked, load_ix_data, DataLen, VaultState};

#[repr(C)]
pub struct Deposit {
    pub amount: u64,
    pub bump: u8,
}

impl DataLen for Deposit {
    const LEN: usize = core::mem::size_of::<Deposit>();
}

pub fn deposit_to_vault(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [user, vault, _] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !user.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    //ix data parsing
    let deposit_data = unsafe { load_ix_data::<Deposit>(data)? };

    if deposit_data.amount.eq(&0) {
        return Err(ProgramError::InvalidInstructionData);
    };

    //validating pda
    VaultState::validate_pda(deposit_data.bump, vault.key(), user.key())?;

    //reading state for auth
    let vault_state = unsafe { load_acc_unchecked::<VaultState>(vault.borrow_data_unchecked()) }?;

    if vault_state.owner != *user.key() {
        return Err(ProgramError::InvalidAccountOwner);
    }

    //actual transfer
    Transfer {
        from: user,
        to: vault,
        lamports: deposit_data.amount,
    }
    .invoke()?;

    Ok(())
}

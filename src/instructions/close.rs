use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    ProgramResult,
};
use pinocchio_system::instructions::Transfer;

use crate::states::{load_acc_unchecked, load_ix_data, DataLen, VaultState};

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

    let ix_bump = [close_ix_data.bump];

    let signer_seeds = &[
        Seed::from(VaultState::SEED.as_bytes()),
        Seed::from(user.key()),
        Seed::from(&ix_bump[..]),
    ];

    let signer = &[Signer::from(&signer_seeds[..])];

    //transfer all remaining amt to user
    Transfer {
        from: vault,
        to: user,
        lamports: vault.lamports(),
    }
    .invoke_signed(signer)?;

    //close the vault account
    // AFTER ALL LAMPORTS IS MOVED OUT OF VAULT , IT BECOMES RENT-INELIGIBLE AND SOLANA RUNTIME WILL GARBAGE COLLECT IT AUTOMATICALLY.

    //BUT FOR PROD, WE CAN ASSIGN THE VAULT TO THE SYSTEM_PROGRAM-
    // unsafe {
    //     vault.assign(&solana_program::system_program::ID);
    // };
    Ok(())
}

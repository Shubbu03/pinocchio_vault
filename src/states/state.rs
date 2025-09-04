use super::utils::DataLen;
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    ProgramResult,
};

use crate::errors::MyProgramError;
use crate::states::utils::load_acc_mut_unchecked;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VaultState {
    pub owner: Pubkey,
}

impl DataLen for VaultState {
    const LEN: usize = core::mem::size_of::<VaultState>();
}

impl VaultState {
    pub const SEED: &'static str = "vault";

    pub fn validate_pda(bump: u8, pda: &Pubkey, owner: &Pubkey) -> Result<(), ProgramError> {
        let seed_with_bump = &[Self::SEED.as_bytes(), owner, &[bump]];
        let derived = pubkey::create_program_address(seed_with_bump, &crate::ID)?;
        if derived != *pda {
            return Err(MyProgramError::PdaMismatch.into());
        }
        Ok(())
    }

    pub fn initialize(vault_acc: &AccountInfo, owner: &Pubkey) -> ProgramResult {
        let vault_state =
            unsafe { load_acc_mut_unchecked::<VaultState>(vault_acc.borrow_mut_data_unchecked())? };

        vault_state.owner = *owner;

        Ok(())
    }
}

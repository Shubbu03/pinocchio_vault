use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    sysvars::rent::Rent,
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::states::{load_ix_data, DataLen, VaultState};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Init {
    pub bump: u8,
}

impl DataLen for Init {
    const LEN: usize = core::mem::size_of::<Init>();
}

pub fn init_vault(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [user, vault_pda, rent, _] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !user.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    };

    let init_ix_data = unsafe { load_ix_data::<Init>(data)? };

    if !vault_pda.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    };

    VaultState::validate_pda(init_ix_data.bump, vault_pda.key(), user.key())?;

    let rent = Rent::from_account_info(rent)?;

    let bump_bytes = [init_ix_data.bump];

    let signer_seeds = [
        Seed::from(VaultState::SEED.as_bytes()),
        Seed::from(user.key()),
        Seed::from(&bump_bytes[..]),
    ];

    let signer = [Signer::from(&signer_seeds[..])];

    CreateAccount {
        from: user,
        to: vault_pda,
        space: VaultState::LEN as u64,
        owner: &crate::ID,
        lamports: rent.minimum_balance(VaultState::LEN),
    }
    .invoke_signed(&signer)?;

    VaultState::initialize(vault_pda, user.key())?;

    Ok(())
}

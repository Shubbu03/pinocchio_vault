use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::find_program_address,
    ProgramResult,
};
use pinocchio_system::instructions::Transfer;

//--------------------- 1. Defining the Accounts Struct ---------------------
pub struct DepositAccounts<'a> {
    pub owner: &'a AccountInfo,
    pub vault: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for DepositAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // 1. destructing to check all accounts present or not
        let [owner, vault, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 2. checking accounts, was done automatically in anchor, here have to do manually

        // a. owner should be the signer
        if !owner.is_signer() {
            return Err(ProgramError::InvalidAccountOwner);
        };

        // b. vault account should be owned by the system program
        if !vault.is_owned_by(&pinocchio_system::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        };

        // c. vault should not have any prev amount, it should be zero as its initialised here
        if vault.lamports() != 0 {
            return Err(ProgramError::InvalidAccountData);
        };

        // 3. PDA derivation & validation
        let (vault_key, _) = find_program_address(&[b"vault", owner.key()], &crate::ID);
        if vault.key().ne(&vault_key) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // 4. returning the accounts
        Ok(Self { owner, vault })
    }
}
//-----------------------------------------------------------------------------

//--------------------- 2. Defining the Instruction Struct ---------------------
pub struct DepositInstructions {
    pub amount: u64,
}

impl<'a> TryFrom<&'a [u8]> for DepositInstructions {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() != size_of::<u64>() {
            return Err(ProgramError::InvalidInstructionData);
        }

        //deserialising raw bytes(byte array) to meaningful data
        let amount = u64::from_le_bytes(data.try_into().unwrap());

        // the amount deposited by user should not be 0
        if amount.eq(&0) {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self { amount })
    }
}
//-----------------------------------------------------------------------------

//--------------------- 3. Defining the actual ix logic ---------------------
pub struct Deposit<'a> {
    pub accounts: DepositAccounts<'a>,
    pub instruction_data: DepositInstructions,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Deposit<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let accounts = DepositAccounts::try_from(accounts)?;
        let instruction_data = DepositInstructions::try_from(data)?;

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

impl<'a> Deposit<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    //transfering user declared amount from users acc to vault acc
    pub fn process(&mut self) -> ProgramResult {
        Transfer {
            from: self.accounts.owner,
            to: self.accounts.vault,
            lamports: self.instruction_data.amount,
        }
        .invoke()?;

        Ok(())
    }
}
//-----------------------------------------------------------------------------

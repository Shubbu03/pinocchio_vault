use pinocchio::program_error::ProgramError;

pub mod close;
pub mod deposit;
pub mod withdraw;
pub mod init;

pub use init::*;
pub use close::*;
pub use deposit::*;
pub use withdraw::*;

#[repr(u8)]
pub enum ProgramInstruction {
    Init,
    Deposit,
    Withdraw,
    Close,
}

impl TryFrom<&u8> for ProgramInstruction {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            0 => Ok(ProgramInstruction::Init),
            1 => Ok(ProgramInstruction::Deposit),
            2 => Ok(ProgramInstruction::Withdraw),
            3 => Ok(ProgramInstruction::Close),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

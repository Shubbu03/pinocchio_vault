use pinocchio::{
    account_info::AccountInfo, no_allocator, program_entrypoint, program_error::ProgramError,
    pubkey::Pubkey, ProgramResult,
};

program_entrypoint!(process_instruction);
no_allocator!();
pub mod instructions;
pub use instructions::*;

// bytes correspond to: Bx1whbMpQLtHRjSoT7zhacngfi4UiFE76qfWtQqHiBS9
pub const ID: [u8; 32] = [
    159, 31, 92, 107, 12, 64, 93, 172, 71, 79, 222, 157, 106, 113, 198, 114, 115, 137, 79, 34, 224,
    79, 20, 251, 234, 161, 213, 170, 133, 10, 132, 125,
];

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data.split_first() {
        Some((Deposit::DISCRIMINATOR, data)) => Deposit::try_from((data, accounts))?.process(),
        Some((Withdraw::DISCRIMINATOR, _)) => Withdraw::try_from(accounts)?.process(),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

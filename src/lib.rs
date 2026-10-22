use {
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program::invoke_signed,
        program_error::ProgramError,
        program_pack::Pack,
        pubkey::Pubkey,
    },
    spl_token::{
        instruction::transfer_checked,
        state::{Account, Mint},
    },
};

solana_program::entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let source_account = next_account_info(account_info_iter)?;
    let mint_account = next_account_info(account_info_iter)?;
    let destination_account = next_account_info(account_info_iter)?;
    let authority_account = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    if token_program.key != &spl_token::id() {
        msg!("Token program mismatch");
        return Err(ProgramError::IncorrectProgramId);
    }

    let (expected_authority, bump_seed) =
        Pubkey::find_program_address(&[b"authority"], program_id);

    if authority_account.key != &expected_authority {
        msg!("Authority PDA mismatch");
        return Err(ProgramError::InvalidAccountData);
    }

    if source_account.owner != token_program.key {
        msg!("Source token account owner mismatch");
        return Err(ProgramError::IncorrectProgramId);
    }

    if mint_account.owner != token_program.key {
        msg!("Mint account owner mismatch");
        return Err(ProgramError::IncorrectProgramId);
    }

    if destination_account.owner != token_program.key {
        msg!("Destination token account owner mismatch");
        return Err(ProgramError::IncorrectProgramId);
    }

    let source_token = Account::unpack(&source_account.try_borrow_data()?)?;
    let mint = Mint::unpack(&mint_account.try_borrow_data()?)?;

    if source_token.owner != expected_authority {
        msg!("Source token account is not owned by the PDA authority");
        return Err(ProgramError::InvalidAccountData);
    }

    if source_token.mint != *mint_account.key {
        msg!("Source token account mint mismatch");
        return Err(ProgramError::InvalidAccountData);
    }

    let destination_token = Account::unpack(&destination_account.try_borrow_data()?)?;
    if destination_token.mint != *mint_account.key {
        msg!("Destination token account mint mismatch");
        return Err(ProgramError::InvalidAccountData);
    }

    let amount = 5_000u64;
    let transfer_instruction = transfer_checked(
        token_program.key,
        source_account.key,
        mint_account.key,
        destination_account.key,
        authority_account.key,
        &[],
        amount,
        mint.decimals,
    )?;

    msg!("Transferring {} base units from source to destination", amount);

    invoke_signed(
        &transfer_instruction,
        &[
            source_account.clone(),
            mint_account.clone(),
            destination_account.clone(),
            authority_account.clone(),
        ],
        &[&[b"authority", &[bump_seed]]],
    )?;

    Ok(())
}
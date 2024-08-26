use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    rent::Rent,
    sysvar::Sysvar,
    msg,
};

// my program start point
entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = instruction_data[0];

    // instruction sets
    match instruction {
        0 => create_account(program_id, accounts),
        1 => deposit(accounts, instruction_data),
        2 => withdraw(program_id, accounts),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

fn create_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let initializer = next_account_info(accounts_iter)?;
    let vault_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    let (vault_pubkey, bump_seed) = Pubkey::find_program_address(&[b"vault", initializer.key.as_ref()], program_id);
    if vault_pubkey != *vault_account.key {
        return Err(ProgramError::InvalidArgument);
    }

    let seeds: &[&[u8]] = &[b"vault", initializer.key.as_ref(), &[bump_seed]];  

    let lamports = Rent::default().minimum_balance(0);

    let ix = system_instruction::create_account(
        initializer.key,
        vault_account.key,
        lamports,
        0,
        program_id,
    );

    invoke_signed(
        &ix,
        &[initializer.clone(), vault_account.clone(), system_program.clone()],
        &[seeds],  
    )?;

    Ok(())
}


fn deposit(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let depositor = next_account_info(accounts_iter)?;
    let vault_account = next_account_info(accounts_iter)?;

    if !vault_account.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());

    **vault_account.try_borrow_mut_lamports()? += amount;
    **depositor.try_borrow_mut_lamports()? -= amount;

    Ok(())
}

fn withdraw(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let initializer = next_account_info(accounts_iter)?;
    let vault_account = next_account_info(accounts_iter)?;
    let recipient = next_account_info(accounts_iter)?;

    let (vault_pubkey, bump_seed) = Pubkey::find_program_address(&[b"vault", initializer.key.as_ref()], program_id);
    if vault_pubkey != *vault_account.key {
        return Err(ProgramError::InvalidArgument);
    }

    let seeds = &[b"vault", initializer.key.as_ref(), &[bump_seed]];

    let vault_balance = **vault_account.try_borrow_mut_lamports()?;
    let withdrawal_amount = vault_balance / 10;  // Withdraw 10% TODO: Add error handling and test this more 

    invoke_signed(
        &system_instruction::transfer(vault_account.key, recipient.key, withdrawal_amount),
        &[vault_account.clone(), recipient.clone()],
        &[seeds], // 
    )?;

    Ok(())
}


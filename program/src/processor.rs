use crate::{
    assertions::{assert_signer, assert_system_program},
    error::MtreeError,
    info::{find_info_pda, find_sub_tree_pda, MTreeInfo, INFO_SEED},
    mtree::{
        hash_leaf,
        path::{get_child_index, get_path_to_root},
        sub_tree::{SubTree, SUB_TREE_LEAF_SIZE, SUB_TREE_SIZE},
        Hash, SubTreeId,
    },
};
use borsh::{BorshDeserialize as _, BorshSerialize as _};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar as _,
};

pub fn insert_leaf(program_id: &Pubkey, accounts: &[AccountInfo], leaf: Vec<u8>) -> ProgramResult {
    let accounts_iterator = &mut accounts.iter();

    let sender = next_account_info(accounts_iterator)?;
    assert_signer("sender", sender)?;

    let info_acc = next_account_info(accounts_iterator)?;
    let sys = next_account_info(accounts_iterator)?;
    assert_system_program(sys)?;

    let rent = Rent::get()?;
    let (mut info, _) = get_or_init_info(info_acc, sender, sys, program_id, &rent)?;

    if info.root_hash != Hash::default() {
        transfer_commission(info_acc, sender, &rent)?;
    }

    let path = get_path_to_root(info.tree_id);

    let last_sub_tree_id = path[0];
    let last_node_acc = next_account_info(accounts_iterator)?;

    let mut leaf_sub_tree = get_or_init_sub_tree(
        sender,
        info_acc,
        last_node_acc,
        last_sub_tree_id,
        sys,
        program_id,
        &rent,
    )?;

    let mut is_new_sub_tree = leaf_sub_tree.is_empty();

    if leaf_sub_tree.is_full() {
        return Err(MtreeError::SubTreeFull.into());
    }
    leaf_sub_tree.insert_leaf(hash_leaf(leaf));

    if leaf_sub_tree.is_full() {
        info.tree_id = last_sub_tree_id + 1;
    }

    let mut root_hash = leaf_sub_tree.root_hash();
    let mut child_id = last_sub_tree_id;

    for tree_id in path.iter().skip(1) {
        let sub_tree_acc = next_account_info(accounts_iterator)?;
        let mut sub_tree = load_sub_tree(sub_tree_acc, *tree_id, program_id)?;
        let child_index = get_child_index(child_id);

        if is_new_sub_tree {
            is_new_sub_tree = false;
            if let Some(leaf) = sub_tree.get_leaf(child_index) {
                leaf_sub_tree.insert_leaf(leaf);
                root_hash = leaf_sub_tree.root_hash();
            }
        }

        sub_tree.update_leaf(child_index, root_hash);
        sub_tree.serialize(&mut *sub_tree_acc.try_borrow_mut_data()?)?;
        root_hash = sub_tree.root_hash();
        child_id = *tree_id;
    }

    leaf_sub_tree.serialize(&mut *last_node_acc.try_borrow_mut_data()?)?;

    info.root_hash = root_hash;
    info.serialize(&mut *info_acc.try_borrow_mut_data()?)?;
    msg!("Hash:{:?}", hex::encode(info.root_hash));
    Ok(())
}

fn load_sub_tree(
    sub_tree_acc: &AccountInfo,
    id: SubTreeId,
    program_id: &Pubkey,
) -> Result<SubTree, ProgramError> {
    let node_key = find_sub_tree_pda(id, program_id);
    if *sub_tree_acc.key != node_key.0 {
        return Err(MtreeError::InvalidNodeAccount.into());
    }

    if sub_tree_acc.data_is_empty() {
        return Err(MtreeError::UninitializedSubTree.into());
    }

    let data = sub_tree_acc.try_borrow_data()?;
    SubTree::try_from_slice(data.as_ref()).map_err(|_| ProgramError::InvalidAccountData)
}

fn get_or_init_sub_tree<'a>(
    sender_acc: &AccountInfo<'a>,
    info_acc: &AccountInfo<'a>,
    sub_tree_acc: &AccountInfo<'a>,
    id: SubTreeId,
    sys: &AccountInfo<'a>,
    program_id: &Pubkey,
    rent: &Rent,
) -> Result<SubTree, ProgramError> {
    let node_key = find_sub_tree_pda(id, program_id);
    if *sub_tree_acc.key != node_key.0 {
        return Err(MtreeError::InvalidNodeAccount.into());
    }

    if !sub_tree_acc.data_is_empty() {
        let data = sub_tree_acc.try_borrow_data()?;
        return SubTree::try_from_slice(data.as_ref())
            .map_err(|_| ProgramError::InvalidAccountData);
    }

    let rent = rent.minimum_balance(SUB_TREE_SIZE);
    invoke_signed(
        &system_instruction::create_account(
            sender_acc.key,
            sub_tree_acc.key,
            1,
            SUB_TREE_SIZE as u64,
            program_id,
        ),
        &[sender_acc.clone(), sub_tree_acc.clone(), sys.clone()],
        &[&[&id.to_be_bytes()[..], &[node_key.1]]],
    )?;
    **info_acc.try_borrow_mut_lamports()? -= rent;
    **sub_tree_acc.try_borrow_mut_lamports()? += rent;

    Ok(SubTree::default())
}

fn transfer_commission<'a>(
    info_acc: &AccountInfo<'a>,
    sender: &AccountInfo<'a>,
    rent: &Rent,
) -> ProgramResult {
    let commission = rent.minimum_balance(SUB_TREE_LEAF_SIZE);
    invoke_signed(
        &system_instruction::transfer(sender.key, info_acc.key, commission),
        &[sender.clone(), info_acc.clone()],
        &[],
    )
}

fn get_or_init_info<'a>(
    info: &AccountInfo<'a>,
    sender: &AccountInfo<'a>,
    sys: &AccountInfo<'a>,
    program_id: &Pubkey,
    rent: &Rent,
) -> Result<(MTreeInfo, u8), ProgramError> {
    let info_key = find_info_pda(program_id);
    if *info.key != info_key.0 {
        return Err(MtreeError::InvalidInfoAccount.into());
    }

    if !info.data_is_empty() {
        let data = info.try_borrow_data()?;
        return Ok((
            MTreeInfo::try_from_slice(data.as_ref())
                .map_err(|_| ProgramError::InvalidAccountData)?,
            info_key.1,
        ));
    }

    let lamports = rent.minimum_balance(MTreeInfo::LEN) + rent.minimum_balance(SUB_TREE_SIZE);

    invoke_signed(
        &system_instruction::create_account(
            sender.key,
            info.key,
            lamports,
            MTreeInfo::LEN as u64,
            program_id,
        ),
        &[sender.clone(), info.clone(), sys.clone()],
        &[&[INFO_SEED, &[info_key.1]]],
    )?;

    Ok((MTreeInfo::default(), info_key.1))
}

use crate::{
    assertions::{assert_signer, assert_system_program},
    events::MTreeEvent,
    tree::{find_mtree_pda, hash_leaf, MTree, MAX_LEAFS, PDA_SEED},
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar as _,
    msg,
};

pub fn insert_leaf(program_id: &Pubkey, accounts: &[AccountInfo], leaf: Vec<u8>) -> ProgramResult {
    let accounts_iterator = &mut accounts.iter();

    let sender = next_account_info(accounts_iterator)?;
    assert_signer("sender", sender)?;

    let tree_acc = next_account_info(accounts_iterator)?;

    let sys = next_account_info(accounts_iterator)?;
    assert_system_program(sys)?;

    msg!("Insert leaf");
    let mut tree = get_or_init_tree::<MAX_LEAFS>(tree_acc, sender, sys, program_id)?;
    msg!("Tree initialized");

    tree.insert_leaf(hash_leaf(leaf))?;
    msg!("Leaf inserted");
    let root_hash = tree.root_hash();
    msg!("Root hash: {:?}", root_hash);
    MTreeEvent::NewRootHash(root_hash).send()?;
    Ok(())
}

fn get_or_init_tree<'a, const LEAFS: usize>(
    tree: &AccountInfo<'a>,
    sender: &AccountInfo<'a>,
    sys: &AccountInfo<'a>,
    program_id: &Pubkey,
) -> Result<MTree<'a, LEAFS>, ProgramError> {
    let (key, bump) = find_mtree_pda(program_id);
    if key != *tree.key {
        return Err(ProgramError::InvalidArgument);
    }

    if !tree.data_is_empty() {
        return MTree::map_acc(tree);
    }

    let tree_size = MTree::<LEAFS>::SIZE;
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(tree_size);

    invoke_signed(
        &system_instruction::create_account(
            sender.key,
            tree.key,
            lamports,
            tree_size as u64,
            program_id,
        ),
        &[sender.clone(), tree.clone(), sys.clone()],
        &[&[PDA_SEED, &[bump]]],
    )?;

    MTree::map_acc(tree)
}

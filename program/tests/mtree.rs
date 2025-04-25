use borsh::BorshDeserialize as _;
use solana_program_mtree::{
    info::{find_info_pda, find_sub_tree_pda, MTreeInfo},
    instruction::encode::make_insert_leaf_instruction,
    mtree::{
        hash_leaf,
        sub_tree::{SubTree, SUB_TREE_LEAFS},
        SubTreeId,
    },
};
use solana_program_test::{BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::{signer::Signer, transaction::Transaction};

#[tokio::test]
pub async fn test_init() {
    let mut context = ProgramTest::new("solana_program_mtree", solana_program_mtree::ID, None)
        .start_with_context()
        .await;

    let info = get_info(&mut context).await;
    assert!(info.is_none());
    let root_sub_tree = get_sub_tree(&mut context, 0).await;
    assert!(root_sub_tree.is_none());

    let test_data = "test_data".as_bytes().to_vec();

    insert_leaf(&mut context, test_data.clone(), 0)
        .await
        .unwrap();

    let info = get_info(&mut context).await;
    assert!(info.is_some());
    let info = info.unwrap();
    assert_eq!(info.tree_id, 0);

    let mut tree = SubTree::new();
    tree.insert_leaf(hash_leaf(test_data));
    assert_eq!(tree.root_hash(), info.root_hash);
}

#[tokio::test]
pub async fn test_full_root_sub_tree() {
    let mut context = ProgramTest::new("solana_program_mtree", solana_program_mtree::ID, None)
        .start_with_context()
        .await;

    let mut expected_tree = SubTree::new();
    for i in 0..SUB_TREE_LEAFS - 1 {
        let test_data = format!("test_data_{}", i).as_bytes().to_vec();
        insert_leaf(&mut context, test_data.clone(), 0)
            .await
            .unwrap();
        expected_tree.insert_leaf(hash_leaf(test_data));
    }

    let info = get_info(&mut context).await;
    assert!(info.is_some());
    let info = info.unwrap();
    assert_eq!(info.tree_id, 1);
    assert_eq!(info.root_hash, expected_tree.root_hash());
}

#[tokio::test]
pub async fn test_overflow_root_sub_tree() {
    let mut context = ProgramTest::new("solana_program_mtree", solana_program_mtree::ID, None)
        .start_with_context()
        .await;

    for i in 0..SUB_TREE_LEAFS - 1 {
        let test_data = format!("test_data_{}", i).as_bytes().to_vec();
        insert_leaf(&mut context, test_data.clone(), 0)
            .await
            .unwrap();
    }

    assert!(
        insert_leaf(&mut context, "test_data".as_bytes().to_vec(), 0)
            .await
            .is_err()
    );
}

#[tokio::test]
pub async fn insert_to_wrong_sub_tree() {
    let mut context = ProgramTest::new("solana_program_mtree", solana_program_mtree::ID, None)
        .start_with_context()
        .await;

    let test_data = "test_data".as_bytes().to_vec();
    assert!(insert_leaf(&mut context, test_data.clone(), 1)
        .await
        .is_err());
}

#[tokio::test]
pub async fn test_full_tree() {
    let mut context = ProgramTest::new("solana_program_mtree", solana_program_mtree::ID, None)
        .start_with_context()
        .await;

    let mut root_sub_tree = SubTree::new();

    for i in 0..SUB_TREE_LEAFS - 1 {
        let test_data = format!("test_data_{}", i).as_bytes().to_vec();
        insert_leaf(&mut context, test_data.clone(), 0)
            .await
            .unwrap();
        root_sub_tree.insert_leaf(hash_leaf(test_data));
    }

    let moved_to_leaf = root_sub_tree.get_leaf(0).unwrap();

    let mut sub_tree = SubTree::new();
    sub_tree.insert_leaf(moved_to_leaf);

    for i in 0..SUB_TREE_LEAFS - 2 {
        let test_data = format!("test_data_{}", i).as_bytes().to_vec();
        insert_leaf(&mut context, test_data.clone(), 1)
            .await
            .unwrap();
        sub_tree.insert_leaf(hash_leaf(test_data));
    }
    root_sub_tree.insert_leaf(sub_tree.root_hash());
    let info = get_info(&mut context).await.unwrap();

    assert_eq!(info.root_hash, root_sub_tree.root_hash());
}

async fn insert_leaf(
    context: &mut ProgramTestContext,
    leaf: Vec<u8>,
    id: SubTreeId,
) -> Result<(), BanksClientError> {
    let insert_instruction =
        make_insert_leaf_instruction(solana_program_mtree::ID, context.payer.pubkey(), leaf, id)
            .unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[insert_instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await
}

async fn get_info(context: &mut ProgramTestContext) -> Option<MTreeInfo> {
    let acc = context
        .banks_client
        .get_account(find_info_pda(&solana_program_mtree::ID).0)
        .await
        .unwrap()?;

    if acc.data.is_empty() {
        return None;
    }

    let info = MTreeInfo::try_from_slice(&acc.data).unwrap();
    Some(info)
}

async fn get_sub_tree(context: &mut ProgramTestContext, id: SubTreeId) -> Option<SubTree> {
    let acc = context
        .banks_client
        .get_account(find_sub_tree_pda(id, &solana_program_mtree::ID).0)
        .await
        .unwrap()?;

    if acc.data.is_empty() {
        return None;
    }

    let sub_tree = SubTree::try_from_slice(&acc.data).unwrap();
    Some(sub_tree)
}

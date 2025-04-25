use borsh::BorshDeserialize as _;
use solana_program_mtree::{
    info::{find_info_pda, find_sub_tree_pda, MTreeInfo},
    instruction::encode::make_insert_leaf_instruction,
    mtree::{hash_leaf, sub_tree::SubTree, SubTreeId},
};
use solana_program_test::{ProgramTest, ProgramTestContext};
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

    insert_leaf(&mut context, test_data.clone(), 0).await;

    let info = get_info(&mut context).await;
    assert!(info.is_some());
    let info = info.unwrap();
    assert_eq!(info.node_id, 0);

    let mut tree = SubTree::new();
    tree.insert_leaf(hash_leaf(test_data));
    assert_eq!(tree.root_hash(), info.root_hash);
}

async fn insert_leaf(context: &mut ProgramTestContext, leaf: Vec<u8>, id: SubTreeId) {
    let insert_instruction =
        make_insert_leaf_instruction(solana_program_mtree::ID, context.payer.pubkey(), leaf, id)
            .unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[insert_instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();
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

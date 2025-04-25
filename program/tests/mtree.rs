use std::ops::Sub;

use borsh::BorshDeserialize as _;
use solana_program_mtree::{info::{find_info_pda, find_sub_tree_pda, MTreeInfo}, mtree::{sub_tree::SubTree, SubTreeId}};
use solana_program_test::{ProgramTest, ProgramTestContext};

#[tokio::test]
pub async fn test_init() {
    let mut context = ProgramTest::new("solana_program_mtree", solana_program_mtree::ID, None)
        .start_with_context()
        .await;

    let info = get_info(&mut context).await;
    assert!(info.is_none());
    let root_sub_tree = get_sub_tree(&mut context, 0).await;




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

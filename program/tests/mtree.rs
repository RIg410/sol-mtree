use solana_program_mtree::{
    instruction::encode::make_insert_leaf_instruction,
    tree::{find_mtree_pda, hash_leaf, join, Hash, LEAF_SIZE, ROOT_OFFSET},
};
use solana_program_test::{BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::{signer::Signer, transaction::Transaction};

#[tokio::test]
pub async fn test_init() {
    let mut context = ProgramTest::new("solana_program_mtree", solana_program_mtree::ID, None)
        .start_with_context()
        .await;

    let test_data = "test_data".as_bytes().to_vec();
    insert_leaf(&mut context, test_data.clone()).await.unwrap();

    let root_hash = get_root_hash(&mut context).await.unwrap();
    assert_eq!(root_hash, join(&hash_leaf(test_data.clone()), &Hash::default()));

    let test_data2 = "test_data2".as_bytes().to_vec();
    insert_leaf(&mut context, test_data2.clone()).await.unwrap();

    let root_hash = get_root_hash(&mut context).await.unwrap();

    assert_eq!(
        root_hash,
        join(&hash_leaf(test_data), &hash_leaf(test_data2))
    );
}

async fn insert_leaf(
    context: &mut ProgramTestContext,
    leaf: Vec<u8>,
) -> Result<(), BanksClientError> {
    let insert_instruction =
        make_insert_leaf_instruction(solana_program_mtree::ID, context.payer.pubkey(), leaf)
            .unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[insert_instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await
}

async fn get_root_hash(context: &mut ProgramTestContext) -> Option<Hash> {
    let acc = context
        .banks_client
        .get_account(find_mtree_pda(&solana_program_mtree::ID).0)
        .await
        .unwrap()?;

    if acc.data.is_empty() {
        return None;
    }

    let mut root = Hash::default();
    root.copy_from_slice(&acc.data[ROOT_OFFSET..ROOT_OFFSET + LEAF_SIZE]);
    Some(root)
}

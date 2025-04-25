use solana_program_test::ProgramTest;

#[tokio::test]
pub async fn test_init() {
     let mut context = ProgramTest::new(
        "solana_program_mtree",
        solana_program_mtree::ID,
        None,
    )
    .start_with_context()
    .await;



}
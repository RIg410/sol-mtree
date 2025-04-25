use eyre::Error;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_program_mtree::events::MTreeEvent;
use solana_program_mtree::instruction::encode::make_insert_leaf_instruction;
use solana_program_mtree::tree::{find_mtree_pda, LEAF_SIZE, ROOT_OFFSET, Hash};
use solana_sdk::signature::Signature;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};
use solana_transaction_status::UiTransactionEncoding;

pub struct MTreeClient {
    program_id: Pubkey,
    client: RpcClient,
}

impl MTreeClient {
    pub fn new(program_id: Pubkey, url: &str) -> Self {
        Self {
            program_id,
            client: RpcClient::new(url),
        }
    }

    pub fn get_root_hash(&self) -> Result<Hash, Error> {
        let (tree_pda, _) = find_mtree_pda(&self.program_id);
        let account = self.client.get_account(&tree_pda)?;

        let mut root = Hash::default();
        if account.data.len() > 0 {
            root.copy_from_slice(&account.data[ROOT_OFFSET..ROOT_OFFSET + LEAF_SIZE]);
        }

        Ok(root)
    }

    pub fn insert_leaf(&self, payer: &Keypair, data: Vec<u8>) -> Result<Signature, Error> {
        let insert_ix = make_insert_leaf_instruction(self.program_id, payer.pubkey(), data)?;

        let recent_blockhash = self.client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[insert_ix],
            Some(&payer.pubkey()),
            &[payer],
            recent_blockhash,
        );

        Ok(self.client.send_and_confirm_transaction(&transaction)?)
    }

    pub fn get_tx_root_hash(&self, tx: Signature) -> Result<Hash, Error> {
        let tx = self
            .client
            .get_transaction(&tx, UiTransactionEncoding::Json)?;
        let meta = tx
            .transaction
            .meta
            .ok_or_else(|| eyre::eyre!("No transaction meta"))?;
        let logs = meta
            .log_messages
            .ok_or_else(|| eyre::eyre!("No log messages"))?;

        let log = logs
            .iter()
            .filter_map(|log| log.strip_prefix("Program log: "))
            .filter_map(MTreeEvent::decode)
            .next()
            .ok_or_else(|| eyre::eyre!("No MTreeEvent"))?;

        let MTreeEvent::NewRootHash(hash) = log;
        Ok(hash)
    }
}

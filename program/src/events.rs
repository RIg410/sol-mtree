use crate::mtree::Hash;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub enum MTreeEvent {
    NewRootHash(Hash),
}

impl MTreeEvent {
    pub fn send(&self) -> ProgramResult {
        let data = hex::encode(self.try_to_vec()?);
        msg!("EVENT:{}", data);
        Ok(())
    }

    #[cfg(feature = "decode-event")]
    pub fn decode<S: AsRef<str>>(log: S) -> Option<Self> {
        let log = log.as_ref();
        if let Some(data) = log.strip_prefix("EVENT:") {
            let bytes = hex::decode(data).ok()?;
            let event = MTreeEvent::try_from_slice(&bytes).ok()?;
            Some(event)
        } else {
            None
        }
    }
}

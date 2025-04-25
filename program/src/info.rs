use crate::mtree::{Hash, SubTreeId};
use borsh::{BorshDeserialize, BorshSerialize};
use core::mem::size_of;
use solana_program::pubkey::Pubkey;

// use shank::ShankAccount;
// use solana_program::account_info::AccountInfo;
// use solana_program::entrypoint::ProgramResult;
// use solana_program::msg;
// use solana_program::program_error::ProgramError;
// use solana_program::pubkey::Pubkey;
// use crate::error::MtreeError;

pub fn find_sub_tree_pda(node_id: SubTreeId, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[&node_id.to_be_bytes()[..]], program_id)
}

pub const INFO_SEED: &[u8] = b"info";

pub fn find_info_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[INFO_SEED], program_id)
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct Info {
    pub node_id: SubTreeId,
    pub root_hash: Hash,
}

impl Info {
    pub const LEN: usize = size_of::<SubTreeId>() + size_of::<Hash>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_borsh_info_size() {
        let info = Info::default();
        let size = info.try_to_vec().unwrap().len();
        assert_eq!(size, Info::LEN);
    }
}

// #[derive(Clone, BorshSerialize, BorshDeserialize, Debug)]
// pub enum Key {
//     Uninitialized,
//     Counter,
// }

// #[repr(C)]
// #[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
// pub struct Counter {
//     pub key: Key,
//     pub authority: Pubkey,
//     pub value: u32,
// }

// impl Counter {
//     pub const LEN: usize = 1 + 32 + 4;

//     pub fn seeds(authority: &Pubkey) -> Vec<&[u8]> {
//         vec!["counter".as_bytes(), authority.as_ref()]
//     }

//     pub fn find_pda(authority: &Pubkey) -> (Pubkey, u8) {
//         Pubkey::find_program_address(&Self::seeds(authority), &crate::ID)
//     }

//     pub fn load(account: &AccountInfo) -> Result<Self, ProgramError> {
//         let mut bytes: &[u8] = &(*account.data).borrow();
//         Counter::deserialize(&mut bytes).map_err(|error| {
//             msg!("Error: {}", error);
//             MtreeError::DeserializationError.into()
//         })
//     }

//     pub fn save(&self, account: &AccountInfo) -> ProgramResult {
//         borsh::to_writer(&mut account.data.borrow_mut()[..], self).map_err(|error| {
//             msg!("Error: {}", error);
//             MtreeError::SerializationError.into()
//         })
//     }
// }

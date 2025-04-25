use solana_program::account_info::AccountInfo;
use solana_program::hash::hashv;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use std::cell::RefCell;
use std::fmt::Debug;
use std::mem::size_of;
use std::rc::Rc;

pub type Hash = [u8; 32];

pub const MAX_LEAFS: usize = 256;
const _: () = assert!(MAX_LEAFS % 2 == 0, "TREE_LEAFS_COUNT must be even");

pub const PDA_SEED: &[u8] = b"mtree";

pub fn find_mtree_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PDA_SEED], program_id)
}

pub const ROOT_OFFSET: usize = 0;
const LENGTH_OFFSET: usize = 32;
const LENGTH_SIZE: usize = size_of::<u32>();
const FIRST_LEAF_OFFSET: usize = 36;
pub const LEAF_SIZE: usize = size_of::<Hash>();

/// layout of the MTree account
/// |0..32: root_hash|32..36: length|36..2048 * 32: leafs|
pub struct MTree<'a, const LEAFS: usize> {
    data: Rc<RefCell<&'a mut [u8]>>,
}

impl<'a, const LEAFS: usize> MTree<'a, LEAFS> {
    pub const SIZE: usize = 36 + LEAFS * size_of::<Hash>();

    pub fn new(data: Rc<RefCell<&'a mut [u8]>>) -> Self {
        Self { data }
    }

    pub fn map_acc(acc: &AccountInfo<'a>) -> Result<Self, ProgramError> {
        Ok(MTree::new(acc.data.clone()))
    }

    pub fn root_hash(&self) -> Hash {
        let data = self.data.borrow();
        let mut root_hash = Hash::default();
        root_hash.copy_from_slice(&data[ROOT_OFFSET..LEAF_SIZE]);
        root_hash
    }

    pub fn len(&self) -> usize {
        let data = self.data.borrow();
        let mut count = [0u8; LENGTH_SIZE];
        count.copy_from_slice(&data[LENGTH_OFFSET..LENGTH_OFFSET + LENGTH_SIZE]);
        u32::from_be_bytes(count) as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn set_len(&self, len: u32) {
        let mut data = self.data.borrow_mut();
        let count_bytes = len.to_be_bytes();
        data[LENGTH_OFFSET..LENGTH_OFFSET + LENGTH_SIZE].copy_from_slice(&count_bytes);
    }

    pub fn is_full(&self) -> bool {
        self.len() >= LEAFS
    }

    pub fn is_init(&self) -> bool {
        !self.data.borrow().is_empty()
    }

    pub fn get_leaf(&self, index: usize) -> Result<Hash, ProgramError> {
        if index >= self.len() {
            return Err(ProgramError::InvalidArgument);
        }

        let data = self.data.borrow();
        let mut leaf = Hash::default();
        leaf.copy_from_slice(
            &data[FIRST_LEAF_OFFSET + index * size_of::<Hash>()
                ..FIRST_LEAF_OFFSET + index * size_of::<Hash>() + LEAF_SIZE],
        );
        Ok(leaf)
    }

    pub fn insert_leaf(&mut self, leaf: Hash) -> Result<(), ProgramError> {
        let count = self.len();
        {
            let mut data = self.data.borrow_mut();
            if count >= LEAFS {
                return Err(ProgramError::InvalidArgument);
            }

            let leaf_offset = FIRST_LEAF_OFFSET + count * size_of::<Hash>();
            data[leaf_offset..leaf_offset + LEAF_SIZE].copy_from_slice(&leaf);
        }
        let len = count + 1;
        self.set_len(len as u32);
        self.update_root(len)?;
        Ok(())
    }

    fn update_root(&self, len: usize) -> Result<(), ProgramError> {
        if len == 0 {
            return Err(ProgramError::InvalidArgument);
        }

        let mut data = self.data.borrow_mut();
        let root_hash = calculate_root(&data.as_ref()[FIRST_LEAF_OFFSET..], 0, len)?;

        data[ROOT_OFFSET..LEAF_SIZE].copy_from_slice(&root_hash);
        Ok(())
    }
}

fn calculate_root(leafs: &[u8], start: usize, count: usize) -> Result<Hash, ProgramError> {
    if count == 0 {
        return Err(ProgramError::InvalidArgument);
    }

    if count == 1 {
        return Ok(join(
            &leafs[start * LEAF_SIZE..(start + 1) * LEAF_SIZE],
            &Hash::default(),
        ));
    }
    if count == 2 {
        return Ok(join(
            &leafs[start * LEAF_SIZE..(start + 1) * LEAF_SIZE],
            &leafs[(start + 1) * LEAF_SIZE..(start + 2) * LEAF_SIZE],
        ));
    }

    let left_count = count / 2;
    let right_count = count - left_count;

    let left_hash = calculate_root(leafs, start, left_count)?;
    let right_hash = calculate_root(leafs, start + left_count, right_count)?;

    Ok(join(&left_hash, &right_hash))
}

impl<const LEAFS: usize> Debug for MTree<'_, LEAFS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.len();
        let mut leafs = Vec::new();
        for i in 0..len {
            if let Ok(leaf) = self.get_leaf(i) {
                leafs.push(hex::encode(leaf));
            }
        }

        f.debug_struct("MTree")
            .field("root_hash", &hex::encode(self.root_hash()))
            .field("len", &self.len())
            .field("fields", &leafs)
            .finish()
    }
}

pub fn hash_leaf(leaf: Vec<u8>) -> Hash {
    hashv(&[leaf.as_slice()]).to_bytes()
}

pub fn join(left: &[u8], right: &[u8]) -> Hash {
    hashv(&[left, right]).to_bytes()
}

#[cfg(test)]
mod test_mtree {
    use super::*;

    fn idx(i: usize) -> Hash {
        let mut hash = [0u8; 32];
        hash[0] = i as u8;
        hash
    }

    #[test]
    fn test_mtree() {
        let mut acc = vec![0u8; MTree::<10>::SIZE];
        let data = Rc::new(RefCell::new(&mut acc[..]));
        let mut mtree = MTree::<10>::new(data);

        assert_eq!(mtree.len(), 0);
        assert!(!mtree.is_full());
        assert!(mtree.is_init());

        mtree.insert_leaf(idx(0)).unwrap();
        assert_eq!(mtree.len(), 1);

        let root = join(&idx(0), &Hash::default());
        assert_eq!(mtree.root_hash(), root);

        mtree.insert_leaf(idx(1)).unwrap();
        assert_eq!(mtree.len(), 2);
        let root = join(&idx(0), &idx(1));
        assert_eq!(mtree.root_hash(), root);

        mtree.insert_leaf(idx(2)).unwrap();
        assert_eq!(mtree.len(), 3);
        let root = join(&join(&idx(0), &Hash::default()), &join(&idx(1), &idx(2)));
        assert_eq!(mtree.root_hash(), root);

        mtree.insert_leaf(idx(3)).unwrap();
        assert_eq!(mtree.len(), 4);
        let root = join(&join(&idx(0), &idx(1)), &join(&idx(2), &idx(3)));
        assert_eq!(mtree.root_hash(), root);

        mtree.insert_leaf(idx(4)).unwrap();
        assert_eq!(mtree.len(), 5);
        mtree.insert_leaf(idx(5)).unwrap();
        assert_eq!(mtree.len(), 6);
        mtree.insert_leaf(idx(6)).unwrap();
        assert_eq!(mtree.len(), 7);
        mtree.insert_leaf(idx(7)).unwrap();
        assert_eq!(mtree.len(), 8);
        mtree.insert_leaf(idx(8)).unwrap();
        assert_eq!(mtree.len(), 9);
        mtree.insert_leaf(idx(9)).unwrap();
        assert_eq!(mtree.len(), 10);
        assert!(mtree.is_full());

        let left = join(
            &join(&idx(0), &idx(1)),
            &join(&join(&idx(2), &Hash::default()), &join(&idx(3), &idx(4))),
        );

        let right = join(
            &join(&idx(5), &idx(6)),
            &join(&join(&idx(7), &Hash::default()), &join(&idx(8), &idx(9))),
        );
        assert_eq!(mtree.root_hash(), join(&left, &right));
    }
}

use solana_program::hash::hashv;

pub mod path;
pub mod sub_tree;

pub type SubTreeId = u32;
pub type Hash = [u8; 32];

pub fn hash_leaf(leaf: Vec<u8>) -> Hash {
    hashv(&[leaf.as_slice()]).to_bytes()
}

pub fn join_hashes(left: &Hash, right: &Hash) -> Hash {
    hashv(&[left, right]).to_bytes()
}

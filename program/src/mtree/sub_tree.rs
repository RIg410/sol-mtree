use super::{join_hashes, Hash};
use borsh::{BorshDeserialize, BorshSerialize};

pub const SUB_TREE_LEAFS: usize = 8;

const ELEMENTS_IN_SUB_TREE: usize = 2 * SUB_TREE_LEAFS - 1;
pub const SUB_TREE_LEAF_SIZE: usize = SUB_TREE_SIZE / SUB_TREE_LEAFS;

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct SubTree {
    nodes: Box<[Hash]>,
    next_index: u32,
}

pub const SUB_TREE_SIZE: usize = ELEMENTS_IN_SUB_TREE * std::mem::size_of::<Hash>() // size of nodes
    + std::mem::size_of::<u32>() // size of node_index
    + std::mem::size_of::<u32>(); // size of nodes length

impl SubTree {
    pub fn new() -> Self {
        let nodes = vec![Hash::default(); ELEMENTS_IN_SUB_TREE];
        let leaves_start = SUB_TREE_LEAFS - 1;

        let mut tree = Self {
            nodes: nodes.into_boxed_slice(),
            next_index: leaves_start as u32,
        };
        tree.recompute();
        tree
    }

    pub fn recompute(&mut self) {
        let leaves_start = SUB_TREE_LEAFS - 1;
        for i in (0..leaves_start).rev() {
            let left = &self.nodes[2 * i + 1];
            let right = &self.nodes[2 * i + 2];
            self.nodes[i] = join_hashes(left, right);
        }
    }

    pub fn is_full(&self) -> bool {
        self.next_index == ELEMENTS_IN_SUB_TREE as u32 - 1
    }

    pub fn root_hash(&self) -> Hash {
        self.nodes[0]
    }

    pub fn update_leaf(&mut self, index: usize, new: Hash) -> bool {
        if index >= SUB_TREE_LEAFS - 1 && index < ELEMENTS_IN_SUB_TREE {
            self.nodes[index] = new;
            self.update_up(index);
            return true;
        }

        return false;
    }

    pub fn insert_leaf(&mut self, leaf: Hash) -> bool {
        if self.is_full() {
            return false;
        }
        let index = self.next_index as usize;

        self.nodes[index] = leaf;
        self.next_index += 1;

        self.update_up(index);
        return true;
    }

    fn update_up(&mut self, index: usize) {
        let mut i = index;
        while i > 0 {
            let parent_index = (i - 1) / 2;
            let left = &self.nodes[2 * parent_index + 1];
            let right = &self.nodes[2 * parent_index + 2];
            self.nodes[parent_index] = join_hashes(left, right);
            i = parent_index;
        }
    }
}

impl Default for SubTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_tree_leafs_is_even() {
        assert_eq!(SUB_TREE_LEAFS % 2, 0, "SUB_TREE_LEAFS must be even");
        assert_eq!(
            ELEMENTS_IN_SUB_TREE,
            2 * SUB_TREE_LEAFS - 1,
            "ELEMENTS_IN_SUB_TREE calculation is correct"
        );
    }

    #[test]
    fn test_node_borsh_size() {
        let node = SubTree::new();
        let size = node.try_to_vec().unwrap().len();
        assert_eq!(size, SUB_TREE_SIZE);
    }

    #[test]
    fn test_sub_tree_new() {
        let tree: SubTree = SubTree::new();

        assert_eq!(tree.next_index, SUB_TREE_LEAFS as u32 - 1);

        assert_eq!(tree.nodes.len(), ELEMENTS_IN_SUB_TREE);

        for i in (SUB_TREE_LEAFS - 1)..ELEMENTS_IN_SUB_TREE {
            assert_eq!(tree.nodes[i], Hash::default());
        }

        for i in (0..SUB_TREE_LEAFS - 1).rev() {
            let expected = join_hashes(&tree.nodes[2 * i + 1], &tree.nodes[2 * i + 2]);
            assert_eq!(tree.nodes[i], expected);
        }

        assert_eq!(tree.root_hash(), tree.nodes[0]);
    }

    fn idx_hash(i: usize) -> Hash {
        let mut hash = [0u8; 32];
        hash[0] = i as u8;
        hash
    }

    #[test]
    fn test_insert_leaf() {
        let mut expected_tree = SubTree::new();
        let mut tree = SubTree::new();
        assert!(!tree.is_full());

        for i in 0..SUB_TREE_LEAFS - 1 {
            assert!(tree.insert_leaf(idx_hash(i)));
            expected_tree.nodes[expected_tree.next_index as usize] = idx_hash(i);
            expected_tree.next_index += 1;
            expected_tree.recompute();
            assert_eq!(tree.nodes, expected_tree.nodes);
        }
        assert!(tree.is_full());
    }

    #[test]
    fn test_update_leaf() {
        let mut tree = SubTree::new();

        for i in 0..SUB_TREE_LEAFS - 1 {
            tree.insert_leaf(idx_hash(i));
        }

        assert!(!tree.update_leaf(SUB_TREE_LEAFS - 2, idx_hash(100)));
        assert!(!tree.update_leaf(ELEMENTS_IN_SUB_TREE, idx_hash(100)));

        let update_idx = SUB_TREE_LEAFS - 1;
        let original_root = tree.root_hash();
        assert!(tree.update_leaf(update_idx, idx_hash(99)));

        assert_ne!(original_root, tree.root_hash());

        let before_update = tree.root_hash();
        assert!(tree.update_leaf(update_idx, idx_hash(88)));
        assert_ne!(before_update, tree.root_hash());
    }
}

use super::{sub_tree::SUB_TREE_LEAFS, SubTreeId};

pub fn get_path_to_root(index: SubTreeId) -> Vec<SubTreeId> {
    let mut path = Vec::new();
    let mut current = index;

    path.push(current);

    while current > 0 {
        current = (current - 1) / SUB_TREE_LEAFS as SubTreeId;
        path.push(current);
    }

    path
}

pub fn get_child_index(node_id: SubTreeId) -> usize {
    if node_id == 0 {
        return 0;
    }
    ((node_id - 1) % SUB_TREE_LEAFS as SubTreeId) as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_path_to_root() {
        assert_eq!(get_path_to_root(22), vec![22, 2, 0]);
        assert_eq!(get_path_to_root(28), vec![28, 3, 0]);
        assert_eq!(get_path_to_root(0), vec![0]);
        assert_eq!(get_path_to_root(1), vec![1, 0]);
        assert_eq!(get_path_to_root(88), vec![88, 10, 1, 0]);
        assert_eq!(get_path_to_root(12), vec![12, 1, 0]);
    }

    #[test]
    fn test_get_child_index() {
        assert_eq!(get_child_index(9), 0); 
        assert_eq!(get_child_index(10), 1);
        assert_eq!(get_child_index(11), 2); 
        assert_eq!(get_child_index(12), 3); 
        assert_eq!(get_child_index(13), 4); 
        assert_eq!(get_child_index(14), 5);
        assert_eq!(get_child_index(15), 6);
        assert_eq!(get_child_index(16), 7);
        assert_eq!(get_child_index(17), 0);
        assert_eq!(get_child_index(2), 1); 
        assert_eq!(get_child_index(0), 0); 
    }
}

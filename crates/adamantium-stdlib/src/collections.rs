//! Collection helpers shared by Adamantium `List` bindings.

pub fn contains<T: PartialEq>(values: &[T], value: &T) -> bool {
    values.contains(value)
}

pub fn index_of<T: PartialEq>(values: &[T], value: &T) -> Option<usize> {
    values.iter().position(|candidate| candidate == value)
}

pub fn reversed<T: Clone>(values: &[T]) -> Vec<T> {
    values.iter().rev().cloned().collect()
}

pub fn sorted<T: Ord + Clone>(values: &[T]) -> Vec<T> {
    let mut result = values.to_vec();
    result.sort();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collection_operations_do_not_modify_the_input() {
        let input = [3, 1, 2];
        assert!(contains(&input, &1));
        assert_eq!(index_of(&input, &2), Some(2));
        assert_eq!(reversed(&input), [2, 1, 3]);
        assert_eq!(sorted(&input), [1, 2, 3]);
        assert_eq!(input, [3, 1, 2]);
    }
}

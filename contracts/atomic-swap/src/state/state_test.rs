/*
Testing for contract, state, and migration.
*/

#[cfg(test)]
mod tests {
    use crate::state::*;
    use cosmwasm_std::testing::MockStorage;
    use cosmwasm_std::{Binary, Addr};

    /// Dummy atomic swap entry
    fn dummy_swap() -> AtomicSwap {
        AtomicSwap {
            recipient: Addr::unchecked("recip"),
            source: Addr::unchecked("source"),
            expires: Default::default(),
            hash: Binary("hash".into()),
            balance: Default::default(),
        }
    }

    /// Testing no swaps of queried id
    #[test]
    fn test_no_swap_ids() {
        let storage = MockStorage::new();
        let ids = all_swap_ids(&storage, None, 10).unwrap();
        assert_eq!(0, ids.len());
    }

    /// Testing return all swaps id on storage
    #[test]
    fn test_all_swap_ids() {
        let mut storage = MockStorage::new();
        SWAPS.save(&mut storage, "lazy", &dummy_swap()).unwrap();
        SWAPS.save(&mut storage, "assign", &dummy_swap()).unwrap();
        SWAPS.save(&mut storage, "zen", &dummy_swap()).unwrap();

        let ids = all_swap_ids(&storage, None, 10).unwrap();
        assert_eq!(3, ids.len());
        assert_eq!(
            vec!["assign".to_string(), "lazy".to_string(), "zen".to_string()],
            ids
        )
    }
}
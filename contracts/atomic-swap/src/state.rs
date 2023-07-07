/*
The atomic swap contract also keeps track of a state such that it can track all swap offers that
are currently pending. If the swap offer has expired, it will get removed from the state. That
said, one can still query the swap offer using block info, which would be permanent on the chain.
*/

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, BlockInfo, Order, StdResult, Storage};

use cw_storage_plus::{Bound, Map};
use cw20::{Balance, Expiration};


/// Atomic swap offer representation.
#[cw_serde]
pub struct AtomicSwap {
    /// This is the sha-256 hash of the preimage
    pub hash      : Binary,
    pub recipient : Addr,
    pub source    : Addr,
    pub expires   : Expiration,
    /// Balance in native tokens, or cw20 token
    pub balance   : Balance,
}

/// Atomic swap can check itself whether it has expired or not with block info
impl AtomicSwap {
    pub fn is_expired(&self, block: &BlockInfo) -> bool {
        self.expires.is_expired(block)
    }
}

/// The cache storage on the smart contract to keep track of swap offers
pub const SWAPS: Map<&str, AtomicSwap> = Map::new("atomic_swap");

/// This returns the list of ids for all active swaps
pub fn all_swap_ids<'a>(
    storage: &dyn Storage,
    start: Option<Bound<'a, &'a str>>,
    limit: usize,
) -> StdResult<Vec<String>> {
    SWAPS
        .keys(storage, start, None, Order::Ascending)
        .take(limit)
        .collect()
}


/// -------------------- UNIT TESTS -------------------- ///
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::MockStorage;
    use cosmwasm_std::Binary;

    /// Test asserting no swap ids mean storage swap is empty
    #[test]
    fn test_no_swap_ids() {
        let storage = MockStorage::new();
        let ids = all_swap_ids(&storage, None, 10).unwrap();
        assert_eq!(0, ids.len());
    }

    fn dummy_swap() -> AtomicSwap {
        AtomicSwap {
            recipient: Addr::unchecked("recip"),
            source: Addr::unchecked("source"),
            expires: Default::default(),
            hash: Binary("hash".into()),
            balance: Default::default(),
        }
    }

    /// Testing a filled storage all of its swap ids
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
/*
Testing for contract, state, and migration.
*/

#[cfg(test)]
mod tests {
    use crate::migrate::*;
    use cw2::{get_contract_version, set_contract_version};
    use cosmwasm_std::testing::MockStorage;

    /// Testing - accepting identical versions to migrate
    #[test]
    fn accepts_identical_version() {
        let mut storage = MockStorage::new();
        set_contract_version(&mut storage, "demo", "0.1.2").unwrap();
        // ensure this matches
        ensure_from_older_version(&mut storage, "demo", "0.1.2").unwrap();
    }

    /// Testing - accepting and updating newer version (backward compatibility)
    #[test]
    fn accepts_and_updates_on_newer_version() {
        let mut storage = MockStorage::new();
        set_contract_version(&mut storage, "demo", "0.4.0").unwrap();
        // ensure this matches
        let original_version = ensure_from_older_version(
            &mut storage, "demo", "0.4.2").unwrap();

        // check the original version is returned
        assert_eq!(original_version.to_string(), "0.4.0".to_string());

        // check the version is updated
        let stored = get_contract_version(&storage).unwrap();
        assert_eq!(stored.contract, "demo".to_string());
        assert_eq!(stored.version, "0.4.2".to_string());
    }

    /// Testing name mismatch
    #[test]
    fn errors_on_name_mismatch() {
        let mut storage = MockStorage::new();
        set_contract_version(&mut storage, "demo", "0.1.2").unwrap();
        // ensure this matches
        let err = ensure_from_older_version(
            &mut storage, "cw20-base", "0.1.2").unwrap_err();
        assert!(err.to_string().contains("cw20-base"), "{}", err);
        assert!(err.to_string().contains("demo"), "{}", err);
    }

    /// Testing disallowing new versions to be migrated to older ones
    #[test]
    fn errors_on_older_version() {
        let mut storage = MockStorage::new();
        set_contract_version(&mut storage, "demo", "0.10.2").unwrap();
        // ensure this matches
        let err = ensure_from_older_version(
            &mut storage, "demo", "0.9.7").unwrap_err();
        assert!(err.to_string().contains("0.10.2"), "{}", err);
        assert!(err.to_string().contains("0.9.7"), "{}", err);
    }

    /// Testing disallowing migration to broken versions
    #[test]
    fn errors_on_broken_version() {
        let mut storage = MockStorage::new();
        let err = ensure_from_older_version(
            &mut storage, "demo", "0.a.7").unwrap_err();
        assert!(
            err.to_string().contains("unexpected character 'a'"),
            "{}",
            err
        );
    }
}
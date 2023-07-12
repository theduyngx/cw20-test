/*
Atomic swap migration. It for now has similar implementation to Cw20-base, by CosmWasm.
*/

use cosmwasm_std::{StdError, StdResult, Storage};
use cw2::{get_contract_version, set_contract_version};
use semver::Version;

/// This function not only validates that the right contract and version can be migrated, but also
/// updates the contract version from the original (stored) version to the new version.
/// It returns the original version for the convenience of doing external checks.
pub fn ensure_from_older_version(
    storage: &mut dyn Storage,
    name: &str,
    new_version: &str,
) -> StdResult<Version> {
    let version: Version = new_version.parse().map_err(from_semver)?;
    let stored = get_contract_version(storage)?;
    let storage_version: Version = stored.version.parse().map_err(from_semver)?;

    if name != stored.contract {
        let msg = format!("Cannot migrate from {} to {}", stored.contract, name);
        return Err(StdError::generic_err(msg));
    }

    if storage_version > version {
        let msg = format!(
            "Cannot migrate from newer version ({}) to older ({})",
            stored.version, new_version
        );
        return Err(StdError::generic_err(msg));
    }
    if storage_version < version {
        // we don't need to save anything if migrating from the same version
        set_contract_version(storage, name, new_version)?;
    }

    Ok(storage_version)
}

/// semver error
fn from_semver(err: semver::Error) -> StdError {
    StdError::generic_err(format!("Semver: {}", err))
}

/// Unit tests
#[cfg(test)]
mod migrate_test;
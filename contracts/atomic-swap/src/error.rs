/*
Error types to handle failed smart contract operations.
*/

use cosmwasm_std::StdError;
use thiserror::Error;

/// Atomic swap smart contract error type
#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    /// Standard error (?)
    #[error("{0}")]
    Std(#[from] StdError),

    /// Command parsing error
    #[error("Hash parse error: {0}")]
    ParseError(String),

    /// Error where swap id (of either sender or recipient) is not valid
    #[error("Invalid atomic swap id")]
    InvalidId {},

    /// Error where the preimage is not valid (probably not UTF-8?)
    #[error("Invalid preimage")]
    InvalidPreimage {},

    /// Error where the hash is not valid
    #[error("Invalid hash ({0} chars): must be 64 characters")]
    InvalidHash(usize),

    /// Zero balance error - smart contracts do not allow empty swaps
    #[error("Send some coins to create an atomic swap")]
    EmptyBalance {},

    /// Not expired swap error - used for refund since locked tokens before expiration cannot
    /// be refunded
    #[error("Atomic swap not yet expired")]
    NotExpired,

    /// Expired swap error - used for create / release since swap cannot be done if timeout
    #[error("Expired atomic swap")]
    Expired,

    /// Smart contract is already in another SWAP - with this implementation, there can only be
    /// a single swap for a smart contract at a time
    #[error("Atomic swap already exists")]
    AlreadyExists,

    /// Recipient does not match with the specified in Create
    #[error("Recipient is not authorized")]
    RecipientUnauthorized,
}
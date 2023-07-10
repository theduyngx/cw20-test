/*
The request messages sent to the blockchain server to an atomic swap smart contract.
*/

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin};
use cw20::{Cw20Coin, Cw20ReceiveMsg, Expiration};
use crate::error::ContractError;


/// Instantiate message for the atomic swap does not inherently require anything other than
/// its own existence (at least for now). So we won't need to pass in anything.
#[cw_serde]
pub struct InstantiateMsg {}

/// The Execute message. For now, it includes:
/// * `Create`  - creating a swap request
/// * `Release` - sends agreed upon tokens to the recipient
/// * `Refund`  - cancels the swap and retrieve all remaining tokens
/// * `Receive` - Handling the receiving end
#[cw_serde]
pub enum ExecuteMsg {
    Create(CreateMsg),
    /// Release sends all tokens to the recipient.
    Release {
        id: String,
        /// This is the preimage, must be exactly 32 bytes in hex (64 chars)
        /// to release: sha256(from_hex(preimage)) == from_hex(hash)
        preimage: String,
    },
    /// Refund returns all remaining tokens to the original sender,
    Refund {
        id: String,
    },
    /// Receive is required in any Cw20 implementation in order to manage the Send/Receive flow.
    /// In this case, it will be called to verify that the other end has received the initiator's
    /// message, and in atomic-swap will mirror create to also lock the tokens of recipient
    Receive {
        id: String,
        msg: Cw20ReceiveMsg,
    },
}

/// Receive message is basically just the create message, for whatever reason
#[cw_serde]
pub enum ReceiveMsg {
    Create(CreateMsg),
}

/// The create message
#[cw_serde]
pub struct CreateMsg {
    /// id is a human-readable name for the swap to use later.
    /// 3-20 bytes of utf-8 text
    pub id: String,
    /// This is hex-encoded sha-256 hash of the preimage (must be 32*2 = 64 chars)
    pub hash: String,
    /// If approved, funds go to the recipient
    pub recipient: String,
    /// You can set expiration at time or at block height the contract is valid at.
    /// After the contract is expired, it can be returned to the original funder.
    pub expires: Expiration,
}

/// Check whether human-readable smart contract's id is valid or not
pub fn is_valid_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    ! (bytes.len() < 3 || bytes.len() > 20)
}


/// Validating that the recipient is indeed the specified one by the sender.
/// # Arguments
/// * `init` - the specified recipient by sender
/// * `curr` - the current recipient that calls Receive
/// # Returns
/// * nothing on Ok
/// * error on Err
pub fn validate_recipient(init: &str, curr: &Addr) -> Result<(), ContractError> {
    let curr: &str = curr.as_str();
    if init == curr {
        Ok(())
    }
    else {
        Err(ContractError::RecipientUnauthorized)
    }
}


pub fn validate_sender() {
    
}

/// Query message
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Show all open swaps. Return type is ListResponse.
    #[returns(ListResponse)]
    List {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Returns the details of the named swap, error if not created.
    /// Return type: DetailsResponse.
    #[returns(DetailsResponse)]
    Details { id: String },
}

/// The list response, which is essentially just a vector of swap ids
#[cw_serde]
pub struct ListResponse {
    /// List all open swap ids
    pub swaps: Vec<String>,
}

/// The individual swap detail response
#[cw_serde]
pub struct DetailsResponse {
    /// Id of this swap
    pub id: String,
    /// This is hex-encoded sha-256 hash of the preimage (must be 32*2 = 64 chars)
    pub hash: String,
    /// If released, funds go to the recipient
    pub recipient: String,
    /// If refunded, funds go to the source
    pub source: String,
    /// Once a swap is expired, it can be returned to the original source (via "refund").
    pub expires: Expiration,
    /// Balance in native tokens or cw20 token, with human-readable address
    pub balance: BalanceHuman,
}

#[cw_serde]
pub enum BalanceHuman {
    Native(Vec<Coin>),
    Cw20(Cw20Coin),
}
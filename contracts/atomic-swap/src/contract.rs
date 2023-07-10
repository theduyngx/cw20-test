/*
Smart contract for token atomic swap on the CosmWasm network.
Mechanism: the atomic swap starts with initiator first sending a certain amount of tokens onto the atomic swap 
smart contract, and other end will receive: "Send this id this amount of your coin in exchange for said id's amount 
of sent funds before this expiration". Remember that it is P2P, so we have a definitive sender and receipient.
*/

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Addr, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, SubMsg, WasmMsg, from_binary, to_binary
};
use sha2::{Digest, Sha256};

use cw_storage_plus::Bound;
use cw2::set_contract_version;
use cw20::{
    Balance, Cw20Coin, Cw20CoinVerified, Cw20ExecuteMsg, Cw20ReceiveMsg
};

use crate::error::ContractError;
use crate::state::{all_swap_ids, AtomicSwap, SWAPS};
use crate::msg::{
    is_valid_name, BalanceHuman, CreateMsg, DetailsResponse, ExecuteMsg, InstantiateMsg,
    ListResponse, QueryMsg, ReceiveMsg,
};

// Version info, for migration info
const CONTRACT_NAME: &str = "crates.io:atomic-swap";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


/// Instantiation - default does not have any setup
/// An atomic swap contract should only be seen as an extension to a full-fledged Cw20 contract.
/// This is because it should only be used for the swapping itself, rather than handling a lot
/// of executions and instantiation logic.
/// # Arguments
/// * `deps` - mutable dependency which has the storage (state) of the chain
/// * `env`  - environment variables which include block information
/// * `info` - initiator's information (including their address and balance)
/// * `msg`  - the instantiate message
/// # Returns
///   Default response
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps  : DepsMut,
    _env  : Env,
    _info : MessageInfo,
    _msg  : InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // No setup
    Ok(Response::default())
}


/// Execute - similar to any Cw Contract - check which Msg it is and execute accordingly.
/// # Arguments
/// * `deps` - mutable dependency which has the storage (state) of the chain
/// * `env`  - environment variables which include block information
/// * `info` - initiator's information (including their address and balance)
/// * `msg`  - the instantiate message
/// # Returns
/// * the execute response
/// * the error type Err
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps : DepsMut,
    env  : Env,
    info : MessageInfo,
    msg  : ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {

        // create - swap creation for native tokens
        // first, we send the funds to the contract, which will be stored in info storage
        ExecuteMsg::Create(msg) => {
            let sent_funds = info.funds.clone();
            execute_create(deps, env, info, msg, Balance::from(sent_funds))
        }

        // release - release the sent funds
        // it requires the contract's id and a preimage, which is the input to the hash; think of preimage
        // as the password that will be hashed on the smart contract (read more in execute_release)
        ExecuteMsg::Release {
            id,
            preimage
        } => execute_release(deps, env, id, preimage),

        // refund - cancel transaction
        // it only requires the contract's id to let it return the funds back
        ExecuteMsg::Refund {
            id
        } => execute_refund(deps, env, id),

        // receive - same with create but for Cw20 tokens
        ExecuteMsg::Receive(msg) => execute_receive(deps, env, info, msg),
    }
}


/// Create a swap - the message info will include the swap id named by initiator when creating the offer.
/// # Arguments
/// * `deps`    - mutable dependency which has the storage (state) of the chain
/// * `env`     - environment variables which include block information
/// * `info`    - initiator's information (including their address and balance)
/// * `msg`     - the create message
/// * `balance` - the sent funds from initiator
/// # Returns
/// * the create response
/// * the error type Err
pub fn execute_create(
    deps    : DepsMut,
    env     : Env,
    info    : MessageInfo,
    msg     : CreateMsg,
    balance : Balance,
) -> Result<Response, ContractError> {
    if !is_valid_name(&msg.id) {
        return Err(ContractError::InvalidId {});
    }

    // this ignores 0 value coins, must have one or more with positive balance
    // ignoring zero-value is a common standard among Cw tokens
    if balance.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }

    // Ensure this is 32 bytes hex-encoded, and decode
    let hash = parse_hex_32(&msg.hash)?;

    // Ensure that the swap has not expired
    // remember that Expiration struct will automatically update to the block once it expires
    if msg.expires.is_expired(&env.block) {
        return Err(ContractError::Expired {});
    }

    // validate recipient address
    // the smart contract does not allow same sender and recipient
    let recipient = deps.api.addr_validate(&msg.recipient)?;
    if recipient == info.sender {
        return Err(ContractError::SameSenderRecipient);
    }

    // create an atomic swap unit
    let swap = AtomicSwap {
        hash: Binary(hash),     // the preimage hash (initially stored in create msg)
        recipient,              // the recipient's smart contract
        source: info.sender,    // the sender's smart contract
        expires: msg.expires,   // expiration
        balance,                // the balance which is sender's already sent funds on the contract
    };

    // Try to store it in SWAP, fail if the id already exists (unmodifiable swaps - they're atomic)
    SWAPS.update(deps.storage, &msg.id, |existing| match existing {
        None => Ok(swap),
        Some(_) => Err(ContractError::AlreadyExists {}),
    })?;

    // return the response
    let res = Response::new()
        .add_attribute("action", "create")
        .add_attribute("id", msg.id)
        .add_attribute("hash", msg.hash)
        .add_attribute("recipient", msg.recipient);
    Ok(res)
}


/// Receive - this is identical to Create, but the difference is, this is used for Cw20 tokens,
/// instead of native tokens.
/// # Arguments
/// * `deps`    - mutable dependency which has the storage (state) of the chain
/// * `env`     - environment variables which include block information
/// * `info`    - initiator's information (including their address and balance)
/// * `wrapper` - the Cw20 receive message (including a sender, amount, and the create msg)
///               it is wrapped in binary (as it appears so)
/// # Returns
/// * the execute response
pub fn execute_receive(
    deps    : DepsMut,
    env     : Env,
    info    : MessageInfo,
    wrapper : Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let unwrapped: ReceiveMsg = from_binary(&wrapper.msg)?;
    let token = Cw20CoinVerified {
        address: info.sender,
        amount : wrapper.amount,
    };
    // we need to update the info... so the original sender is the one authorizing with these tokens
    let org_info = MessageInfo {
        sender : deps.api.addr_validate(&wrapper.sender)?,
        funds  : info.funds,
    };
    // we unwrap the wrapper message such that we can call create again
    // once we've converted the Cw20 Receive Message to the Create Message, we can call create
    let ReceiveMsg::Create(msg) = unwrapped;
    execute_create(deps, env, org_info, msg, Balance::Cw20(token))
}


/// Release - both ends have successfully locked their tokens.
/// Since this is release phase, it can only be called when the preimage has indeed been publicized,
/// which only occurs when both parties have locked their tokens on the smart contract.
/// # Arguments
/// * `deps`     - mutable dependency which has the storage (state) of the chain
/// * `env`      - environment variables which include block information
/// * `id`       - sender's smart contract ID
/// * `preimage` - the password before hashed to allow the release of tokens
/// # Returns
/// * the execute response
/// * the error type Err
pub fn execute_release(
    deps     : DepsMut,
    env      : Env,
    id       : String,
    preimage : String,
) -> Result<Response, ContractError> {
    let swap = SWAPS.load(deps.storage, &id)?;
    if swap.is_expired(&env.block) {
        return Err(ContractError::Expired {});
    }

    // check whether the preimage matches the hash or not
    let hash = Sha256::digest(&parse_hex_32(&preimage)?);
    if hash.as_slice() != swap.hash.as_slice() {
        return Err(ContractError::InvalidPreimage {});
    }

    // Delete the swap on storage
    SWAPS.remove(deps.storage, &id);

    // Send the tokens out
    let msgs = send_tokens(&swap.recipient, swap.balance)?;
    Ok(Response::new()
        .add_submessages(msgs)
        .add_attribute("action", "release")
        .add_attribute("id", id)
        .add_attribute("preimage", preimage)
        .add_attribute("to", swap.recipient.to_string()))
}


/// Refund - refund can only occur when the swap has expired.
/// # Arguments
/// * `deps` - mutable dependency which has the storage (state) of the chain
/// * `env`  - environment variables which include block information
/// * `id`   - human-readable swap id
/// # Returns
/// * the execute response
/// * the error type Err
pub fn execute_refund(
    deps : DepsMut, 
    env  : Env, 
    id   : String
) -> Result<Response, ContractError> {
    let swap = SWAPS.load(deps.storage, &id)?;

    // refund is not possible if the swap has not expired
    if !swap.is_expired(&env.block) {
        return Err(ContractError::NotExpired {});
    }

    // We delete the swap
    SWAPS.remove(deps.storage, &id);

    // and send the tokens back to the source (initiator)
    let msgs = send_tokens(&swap.source, swap.balance)?;
    Ok(Response::new()
        .add_submessages(msgs)
        .add_attribute("action", "refund")
        .add_attribute("id", id)
        .add_attribute("to", swap.source.to_string()))
}

/// Parse hex 32-byte string to ensure that it is of correct format. Helper function so private.
/// # Arguments
/// * `data` - the 32-byte string
/// # Returns
/// * array of bytes (u8)
/// * the error type Err
fn parse_hex_32(data: &str) -> Result<Vec<u8>, ContractError> {
    match hex::decode(data) {
        Ok(bin) => 
            if bin.len() == 32 { Ok(bin) } 
            else { Err(ContractError::InvalidHash(bin.len() * 2)) }
        Err(e) => Err(ContractError::ParseError(e.to_string())),
    }
}


/// Get the required messages for sending a specific amount of token already on the contract to the specified
/// address. This is used when releasing the locked tokens, or refunding back to initiator.
/// # Arguments
/// * `to`     - the specified destination address to send tokens to
/// * `amount` - the balance on smart contract
/// # Returns
/// * array of bytes (u8)
/// * the error type Err
fn send_tokens(to: &Addr, amount: Balance) -> StdResult<Vec<SubMsg>> {
    // sending zero amount
    if amount.is_empty() {
        Ok(vec![])
    }
    // sending some other amount
    else {
        match amount {

            // native coin will simply use the standard Bank Send message (it is compatible to it)
            Balance::Native(coins) => {
                let msg = BankMsg::Send {
                    to_address: to.into(),
                    amount: coins.into_vec(),
                };
                Ok(vec![SubMsg::new(msg)])
            }

            // Cw20 coin will be sent in a different, more sophisticated way
            // This has to do with how different smart contracts (even if internally logically) communicate
            Balance::Cw20(coin) => {
                let msg = Cw20ExecuteMsg::Transfer {
                    recipient: to.into(),
                    amount: coin.amount,
                };
                let exec = WasmMsg::Execute {
                    contract_addr: coin.address.into(),
                    msg: to_binary(&msg)?,
                    funds: vec![],
                };
                Ok(vec![SubMsg::new(exec)])
            }
        }
    }
}


/// Query - there are 2 types of queries: listing and retrieving details of a specified smart contract
/// # Arguments
/// * `deps` - mutable dependency which has the storage (state) of the chain
/// * `_env` - environment variables which include block information
/// * `msg`  - the query message
/// # Returns
/// * array of bytes (u8)
/// * the error type Err
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {

        // listing is retrieving the list of swaps starting after a specific point with a limit
        QueryMsg::List {
            start_after,
            limit
        } => to_binary(&query_list(deps, start_after, limit)?),

        // details is simply the details of a swap, indexed by human-readable swap's id
        QueryMsg::Details {
            id
        } => to_binary(&query_details(deps, id)?),
    }
}


/// Querying details of a swap; query by its human-readable id.
/// # Arguments
/// * `deps` - mutable dependency which has the storage (state) of the chain
/// * `id`   - swap id
/// # Returns
///   The details of the swap
fn query_details(deps: Deps, id: String) -> StdResult<DetailsResponse> {
    // load is a mapping method that takes in a storage and a key
    // in this case, the id is the swap id named by the initiator, and value being AtomicSwap
    // SWAPS = Map<swap_id:String, pending:AtomicSwap>
    let swap = SWAPS.load(deps.storage, &id)?;

    // Convert balance to human balance
    let balance_human = match swap.balance {
        Balance::Native(coins) => BalanceHuman::Native(coins.into_vec()),
        Balance::Cw20(coin) => BalanceHuman::Cw20(Cw20Coin {
            address: coin.address.into(),
            amount: coin.amount,
        }),
    };

    // return the details of the swap
    let details = DetailsResponse {
        id,
        hash: hex::encode(swap.hash.as_slice()),
        recipient: swap.recipient.into(),
        source: swap.source.into(),
        expires: swap.expires,
        balance: balance_human,
    };
    Ok(details)
}


// Settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

/// Querying a list of swaps
/// # Arguments
/// * `deps`        - mutable dependency which has the storage (state) of the chain
/// * `start_after` - the starting point of query
/// * `limit`       - the list size limit
/// # Returns
///   The list of responses
fn query_list(
    deps        : Deps,
    start_after : Option<String>,
    limit       : Option<u32>,
) -> StdResult<ListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.as_ref().map(|s| Bound::exclusive(s.as_str()));

    Ok(ListResponse {
        swaps: all_swap_ids(deps.storage, start, limit)?,
    })
}
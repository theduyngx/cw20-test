use crate::msg::MigrateMsg;

use cosmwasm_std::{
    Deps, DepsMut, Env, MessageInfo, Response, StdResult, Binary, to_binary, entry_point
};
use cw2::set_contract_version;
use cw20_base::allowances::{
    execute_transfer_from, execute_send_from, execute_burn_from,
    execute_increase_allowance, execute_decrease_allowance, query_allowance
};
use cw20_base::contract::{
    execute_transfer, execute_burn, execute_send, execute_mint, execute_update_marketing, execute_upload_logo, 
    query_balance, query_token_info, query_minter, query_marketing_info, query_download_logo, execute_update_minter
};
use cw20_base::ContractError;
use cw20_base::enumerable::{query_owner_allowances, query_all_accounts, query_spender_allowances};
use cw20_base::msg::{
    InstantiateMsg, ExecuteMsg, QueryMsg
};

const CONTRACT_NAME: &str = "crates.io::eames-token";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


/// Instantiate - calling cw20_base instantiation
/// ### Arguments
/// * `deps` - mutable dependency which has the storage (state) of the chain
/// * `env`  - environment variables which include block information
/// * `info` - message info, such as sender/initiator and denomination
/// * `msg`  - the instantiate message
/// ### Returns
/// * the instantiate response on Ok
/// * the error type on Err
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps : DepsMut,
    env  : Env,
    info : MessageInfo,
    msg  : InstantiateMsg
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw20_base::contract::instantiate(deps, env, info, msg)
}


/// Execute - calling cw20_base execute function. Arguments are identical to that of Instantiate.
/// ### Arguments
/// * `deps` - mutable dependency which has the storage (state) of the chain
/// * `env`  - environment variables which include block information
/// * `info` - message info, such as sender/initiator and denomination
/// * `msg`  - the execute message
/// ### Returns
/// * the execute response on Ok
/// * the error type on Err
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps : DepsMut,
    env  : Env,
    info : MessageInfo,
    msg  : ExecuteMsg
) -> Result<Response, ContractError> {
    // pattern matching message type
    match msg {

        // transfer action (initiator is sender)
        ExecuteMsg::Transfer {
            recipient,
            amount
        } => execute_transfer(deps, env, info, recipient, amount),

        // burn action (initiator's amount will get burnt)
        ExecuteMsg::Burn {
            amount
        } => execute_burn(deps, env, info, amount),

        // send action - transfer with an extra message as instruction for the smart contract
        ExecuteMsg::Send {
            contract,
            amount,
            msg
        } => execute_send(deps, env, info, contract, amount, msg),

        // increase allowance action - initiator increases another contract's total allowance to spend
        // on behalf of them
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount, 
            expires 
        } => execute_increase_allowance(deps, env, info, spender, amount, expires),
        
        // decrease allownace action (similar to increase)
        ExecuteMsg::DecreaseAllowance { 
            spender, 
            amount, 
            expires 
        } => execute_decrease_allowance(deps, env, info, spender, amount, expires),

        // transfer from action - uses allowance to let another transfer their money
        // as such, sender (initiator) is the allowed party, and owner is the true token owner
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount
        } => execute_transfer_from(deps, env, info, owner, recipient, amount),

        // send from action - similar to transfer from but with send
        ExecuteMsg::SendFrom {
            owner,
            contract,
            amount,
            msg 
        } => execute_send_from(deps, env, info, owner, contract, amount, msg),

        // burn from action - similar to transfer from but with burn
        ExecuteMsg::BurnFrom { 
            owner, 
            amount 
        } => execute_burn_from(deps, env, info, owner, amount),

        // mint action - the recipient is one to get the award with amount
        ExecuteMsg::Mint { 
            recipient, 
            amount 
        } => execute_mint(deps, env, info, recipient, amount),

        // update minter (probably to update the forefront minter on the block)
        ExecuteMsg::UpdateMinter {
            new_minter
        } => execute_update_minter(deps, env, info, new_minter),

        // marketing stuffs (not important)
        ExecuteMsg::UpdateMarketing {
            project,
            description,
            marketing
        } => execute_update_marketing(deps, env, info, project, description, marketing),

        ExecuteMsg::UploadLogo(logo) => execute_upload_logo(deps, env, info, logo),
    }
}


/// Query - calling cw20_base function.
/// ### Arguments
/// * `deps` - mutable dependency which has the storage (state) of the chain
/// * `_env` - environment variables which include block information
/// * `msg`  - the execute message
/// ### Returns
/// Serialized binary representing the portable queried response
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps,
    _env: Env,
    msg : QueryMsg
) -> StdResult<Binary> {
    match msg {

        // querying balance of a particular address
        QueryMsg::Balance { 
            address 
        } => to_binary(&query_balance(deps, address)?),
        
        // querying the token info on the blockchain
        QueryMsg::TokenInfo {
        } => to_binary(&query_token_info(deps)?),

        // querying the forefront minter (probably)
        QueryMsg::Minter {
        } => to_binary(&query_minter(deps)?),

        // querying a spender's allowance with a particular owner
        QueryMsg::Allowance {
            owner,
            spender
        } => to_binary(&query_allowance(deps, owner, spender)?),

        // querying all allownaces from a particular owner
        QueryMsg::AllAllowances {
            owner,
            start_after,
            limit
        } => to_binary(&query_owner_allowances(deps, owner, start_after, limit)?),

        // querying all allowances of a particular spender
        QueryMsg::AllSpenderAllowances {
            spender,
            start_after,
            limit
        } => to_binary(&query_spender_allowances(deps, spender, start_after, limit)?),

        // querying all accounts with stamp and limit on the blockchain
        QueryMsg::AllAccounts {
            start_after,
            limit
        } => to_binary(&query_all_accounts(deps, start_after, limit)?),

        // querying marketing information (not important)
        QueryMsg::MarketingInfo {
        } => to_binary(&query_marketing_info(deps)?),

        QueryMsg::DownloadLogo {
        } => to_binary(&query_download_logo(deps)?),
    }
}


/// Migrate - contract migration. Contract migration essentially allows a contract to have its ID changed
/// (internal logic of the wasm file) without having to create a new contract. CosmWasm, unlike Ethereum - 
/// most contracts implement the same standard (i.e. Cw20) so no need to upload the whole thing. Also if 
/// the underlying logic remains similar, we can do very flexible things with it, such as migration.
/// ### Arguments
/// * `_deps` - mutable dependency which has the storage (state) of the chain
/// * `_env`  - environment variables which include block information
/// * `_msg`  - the execute message
/// ### Returns
/// * the execute response on Ok
/// * the error type on Err
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

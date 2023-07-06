use cosmwasm_schema::write_api;
use atomic_swap::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};


/// Write schemas for the messages of atomic swap smart contract
fn main() {
    write_api! {
        instantiate : InstantiateMsg,
        query       : QueryMsg,
        execute     : ExecuteMsg,
    }
}
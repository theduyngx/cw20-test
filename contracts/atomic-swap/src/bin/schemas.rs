use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use atomic_swap::msg::{InstantiateMsg, QueryMsg, ExecuteMsg};


/// This will create the json schemas for the different types of messages, including Instantiate,
/// Execute, and Query. Create, and follow the generated schema to create a client request.
fn main() {
    // get the current crate directory
    let mut out_dir = current_dir().unwrap();
    // create a new one called schema
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    // export to schema directory all the json schemas to create these request messages to server
    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg)    , &out_dir);
    export_schema(&schema_for!(QueryMsg)      , &out_dir);
}

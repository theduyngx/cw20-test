use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Migrate message to initiate contract migration
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

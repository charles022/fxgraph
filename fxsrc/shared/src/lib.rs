use serde::{Deserialize, Serialize};
use polars::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppState {
    pub message: String,
    pub counter: u32,
}

/// Helper to create a consistent DataFrame on Client or Server
pub fn create_example_df() -> PolarsResult<DataFrame> {
    df!(
        "names" => ["A", "B", "C"],
        "values" => [10, 20, 30],
        "active" => [true, false, true]
    )
}

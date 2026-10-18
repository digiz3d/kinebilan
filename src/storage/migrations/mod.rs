mod v0_to_v1;

use serde_json::Value;

pub type Step = fn(Value) -> Result<Value, serde_json::Error>;

/// Returns the step that upgrades a save file from `version` to `version + 1`.
pub fn step(version: u32) -> Option<Step> {
    match version {
        0 => Some(v0_to_v1::migrate),
        _ => None,
    }
}

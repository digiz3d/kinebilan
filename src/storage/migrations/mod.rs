mod v0_to_v1;
mod v1_to_v2;
mod v2_to_v3;
mod v3_to_v4;

use serde_json::Value;

pub type Step = fn(Value) -> Result<Value, serde_json::Error>;

/// Returns the step that upgrades a save file from `version` to `version + 1`.
pub fn step(version: u32) -> Option<Step> {
    match version {
        0 => Some(v0_to_v1::migrate),
        1 => Some(v1_to_v2::migrate),
        2 => Some(v2_to_v3::migrate),
        3 => Some(v3_to_v4::migrate),
        _ => None,
    }
}

#[cfg(test)]
pub mod test_support {
    use serde_json::Value;

    /// Runs the step from `from` to `from + 1` on `input` and checks the whole output against `expected`.
    pub fn assert_migration(from: u32, input: Value, expected: Value) {
        assert_eq!(expected["version"], from + 1, "the expected fixture must be at version {}", from + 1);
        let step = super::step(from).expect("a migration should exist for this version");
        let migrated = step(input).expect("migration should succeed");
        assert_eq!(migrated, expected);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_every_version_up_to_current_has_a_step() {
        for version in 0..crate::storage::CURRENT_VERSION {
            assert!(step(version).is_some(), "missing migration from version {version}");
        }
        assert!(step(crate::storage::CURRENT_VERSION).is_none());
    }
}

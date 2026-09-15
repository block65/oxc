use crate::rule::{DefaultRuleConfig, Rule};
use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoWideningObjectKeys(Box<NoWideningObjectKeysConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoWideningObjectKeysConfig {
    /// Helper used instead of Object.keys.
    pub keys: String,
    /// Helper used instead of Object.entries.
    pub entries: String,
}
impl Default for NoWideningObjectKeysConfig {
    fn default() -> Self {
        Self { keys: "typedKeys".into(), entries: "typedEntries".into() }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow Object.keys and Object.entries where a declared finite key type is lost.
    ///
    /// ### Why is this bad?
    ///
    /// Erasing a useful type leaves callers unable to check the values they use.
    /// The native tsgolint checker performs this analysis alongside other type-aware rules.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// const levels = { off: 0, warn: 1 };
    /// const names = Object.keys(levels);
    /// const pairs = Object.entries(levels);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const levels = { off: 0, warn: 1 };
    /// const names = typedKeys(levels);
    /// const pairs = typedEntries(levels);
    /// ```
    NoWideningObjectKeys(tsgolint),
    typescript,
    restriction,
    config = NoWideningObjectKeysConfig,
    version = "1.82.0",
    short_description = "Disallow Object.keys and Object.entries where a declared finite key type is lost.",
);

impl Rule for NoWideningObjectKeys {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }
    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_default_config() {
        let rule = NoWideningObjectKeys::default();
        let config = rule.to_configuration().unwrap().unwrap();

        assert_eq!(config["keys"], json!("typedKeys"));
        assert_eq!(config["entries"], json!("typedEntries"));
    }

    #[test]
    fn test_from_configuration() {
        let config_value = json!([{
            "keys": "objectKeysFrom",
            "entries": "objectEntriesFrom"
        }]);

        let rule = NoWideningObjectKeys::from_configuration(config_value).unwrap();

        assert_eq!(rule.0.keys, "objectKeysFrom");
        assert_eq!(rule.0.entries, "objectEntriesFrom");
    }

    #[test]
    fn test_round_trip() {
        let original_config = json!([{
            "keys": "objectKeysFrom",
            "entries": "objectEntriesFrom"
        }]);

        let rule = NoWideningObjectKeys::from_configuration(original_config).unwrap();
        let serialized = rule.to_configuration().unwrap().unwrap();

        assert_eq!(serialized["keys"], json!("objectKeysFrom"));
        assert_eq!(serialized["entries"], json!("objectEntriesFrom"));
    }
}

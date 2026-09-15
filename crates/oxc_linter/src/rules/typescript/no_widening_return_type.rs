use crate::rule::{DefaultRuleConfig, Rule};
use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoWideningReturnType(Box<NoWideningReturnTypeConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoWideningReturnTypeConfig {
    /// Allow a named object return type to express a contract.
    pub contracts: bool,
}
impl Default for NoWideningReturnTypeConfig {
    fn default() -> Self {
        Self { contracts: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow return annotations wider than the values produced by the body.
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
    /// function supported(): boolean {
    ///   return true;
    /// }
    ///
    /// function session(): { name: string; secure: boolean } {
    ///   return { name: 'sid', secure: true };
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// function supported(): true {
    ///   return true;
    /// }
    ///
    /// interface Cookie {
    ///   name: string;
    ///   secure: boolean;
    /// }
    ///
    /// function session(): Cookie {
    ///   return { name: 'sid', secure: true };
    /// }
    /// ```
    NoWideningReturnType(tsgolint),
    typescript,
    restriction,
    config = NoWideningReturnTypeConfig,
    version = "1.82.0",
    short_description = "Disallow return annotations wider than the values produced by the body.",
);

impl Rule for NoWideningReturnType {
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
        let rule = NoWideningReturnType::default();
        let config = rule.to_configuration().unwrap().unwrap();

        assert_eq!(config["contracts"], json!(true));
    }

    #[test]
    fn test_from_configuration() {
        let config_value = json!([{
            "contracts": false
        }]);

        let rule = NoWideningReturnType::from_configuration(config_value).unwrap();

        assert!(!rule.0.contracts);
    }

    #[test]
    fn test_round_trip() {
        let original_config = json!([{
            "contracts": false
        }]);

        let rule = NoWideningReturnType::from_configuration(original_config).unwrap();
        let serialized = rule.to_configuration().unwrap().unwrap();

        assert_eq!(serialized["contracts"], json!(false));
    }
}

use crate::rule::Rule;
use oxc_macros::declare_oxc_lint;

#[derive(Debug, Default, Clone)]
pub struct NoOpenStringFallback;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow an inline string literal as the fallback for an open `string`.
    ///
    /// ### Why is this bad?
    ///
    /// A literal after `??` or `??=`, or as a destructuring default, that fills a
    /// plain `string` invents a value the source did not have, and the placeholder
    /// usually just fails later, in validation. A literal that is a member of the
    /// operand's literal union chooses a default instead, and a named value is a
    /// typed default. The native tsgolint checker performs this analysis alongside
    /// other type-aware rules.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// declare const tenant: string | undefined;
    /// const name = tenant ?? "conformance-tenant";
    ///
    /// const [, iterations = "0"] = serialised.split("$");
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// declare const level: "debug" | "info" | undefined;
    /// const chosen = level ?? "debug";
    ///
    /// declare const tenant: string | undefined;
    /// const name = tenant ?? DEFAULT_TENANT;
    /// ```
    NoOpenStringFallback(tsgolint),
    typescript,
    restriction,
    version = "1.87.0",
    short_description = "Disallow an inline string literal as the fallback for an open `string`.",
);

impl Rule for NoOpenStringFallback {}

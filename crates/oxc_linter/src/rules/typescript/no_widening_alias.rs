use crate::rule::Rule;
use oxc_macros::declare_oxc_lint;

#[derive(Debug, Default, Clone)]
pub struct NoWideningAlias;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow const aliases that erase the initializer type or its finite keys.
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
    /// declare const source: 'ready' | 'done';
    /// const alias: string = source;
    ///
    /// const messages = { state: { id: 'state' } };
    /// const byCode: Record<string, { id: string }> = messages;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// declare const source: 'ready' | 'done';
    /// const alias: 'ready' | 'done' = source;
    ///
    /// const messages = { state: { id: 'state' } };
    /// const byCode = messages;
    /// ```
    NoWideningAlias(tsgolint),
    typescript,
    restriction,
    version = "1.82.0",
    short_description = "Disallow const aliases that erase the initializer type or its finite keys.",
);

impl Rule for NoWideningAlias {}

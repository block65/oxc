use crate::rule::Rule;
use oxc_macros::declare_oxc_lint;

#[derive(Debug, Default, Clone)]
pub struct DefineMessagesKeys;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require a finite key type for defineMessages: a union of string or number literals.
    /// A call with no type argument passes when it resolves to an overload with a `const`
    /// type parameter, such as react-intl 12's, which already keeps the literal keys.
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
    /// import { defineMessages } from 'react-intl';
    ///
    /// const otherMessages = defineMessages<string>({
    ///   greeting: { id: 'greeting' },
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// import { defineMessages } from 'react-intl';
    ///
    /// const messages = defineMessages<'greeting' | 'farewell'>({
    ///   greeting: { id: 'greeting' },
    ///   farewell: { id: 'farewell' },
    /// });
    /// ```
    DefineMessagesKeys(tsgolint),
    typescript,
    restriction,
    version = "1.82.0",
    short_description = "Require a finite literal key type for defineMessages.",
);

impl Rule for DefineMessagesKeys {}

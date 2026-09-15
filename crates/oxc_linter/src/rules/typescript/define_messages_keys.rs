use crate::rule::Rule;
use oxc_macros::declare_oxc_lint;

#[derive(Debug, Default, Clone)]
pub struct DefineMessagesKeys;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require an explicit finite string-literal key type for defineMessages.
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
    /// const messages = defineMessages({
    ///   greeting: { id: 'greeting' },
    /// });
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
    short_description = "Require an explicit finite string-literal key type for defineMessages.",
);

impl Rule for DefineMessagesKeys {}

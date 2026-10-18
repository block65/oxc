use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::is_node_value_not_dom_node,
};

fn prefer_query_selector_diagnostic(
    good_method: &str,
    bad_method: &str,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `.{good_method}()` over `.{bad_method}()`."))
        .with_help("It's better to use the same method to query DOM elements. This helps keep consistency and it lends itself to future improvements (e.g. more specific selectors).")
        .with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PreferQuerySelector {
    /// When set to `true`, allows using `.getElementById()` and `.getElementsByClassName()` when called with a variable or expression.
    /// This avoids the need to manually compose a CSS selector string, which can be less readable.
    allow_with_variables: bool,
}

fn get_preferred_identifier_name(ident_name: &str) -> Option<&'static str> {
    match ident_name {
        "getElementById" => Some("querySelector"),
        "getElementsByClassName" | "getElementsByTagName" | "getElementsByName" => {
            Some("querySelectorAll")
        }
        _ => None,
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer `.querySelector()` over `.getElementById()`. And prefer `.querySelectorAll()`
    /// over `.getElementsByClassName()`, `.getElementsByTagName()`, and `.getElementsByName()`.
    ///
    /// ### Why is this bad?
    ///
    /// - Using `.querySelector()` and `.querySelectorAll()` is more flexible and allows for more specific selectors.
    /// - It's better to use the same method to query DOM elements. This helps keep consistency and it lends itself to future improvements (e.g. more specific selectors).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// document.getElementById('foo');
    /// document.getElementsByClassName('foo bar');
    /// document.getElementsByTagName('main');
    /// document.getElementsByClassName(fn());
    /// document.getElementsByName('foo');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// document.querySelector('#foo');
    /// document.querySelector('.bar');
    /// document.querySelector('main #foo .bar');
    /// document.querySelectorAll('.foo.bar');
    /// document.querySelectorAll('li a');
    /// document.querySelector('li').querySelectorAll('a');
    /// ```
    ///
    /// Examples of **correct** code for this rule with `{ "allowWithVariables": true }`:
    /// ```javascript
    /// document.getElementById(someId);
    /// document.getElementsByClassName(someClass);
    /// document.getElementsByClassName(`${someClass}`);
    /// ```
    PreferQuerySelector,
    unicorn,
    pedantic,
    conditional_fix,
    config = PreferQuerySelector,
    version = "0.0.15",
    short_description = "Prefer `.querySelector()` over `.getElementById()`, and `.querySelectorAll()` over `.getElementsByClassName()` and `.getElementsByTagName()`.",
);

impl Rule for PreferQuerySelector {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if call_expr.optional || call_expr.arguments.len() != 1 {
            return;
        }

        let Some(member_expr) = call_expr.callee.as_member_expression() else {
            return;
        };

        if member_expr.optional()
            || member_expr.is_computed()
            || is_node_value_not_dom_node(member_expr.object())
        {
            return;
        }

        let Some(argument_expr) = call_expr.arguments[0].as_expression() else {
            return;
        };

        let Some((property_span, property_name)) = member_expr.static_property_info() else {
            return;
        };

        if self.allow_with_variables
            && matches!(property_name, "getElementById" | "getElementsByClassName")
            && is_non_literal_argument(argument_expr)
        {
            return;
        }

        if let Some(preferred_selector) = get_preferred_identifier_name(property_name) {
            let diagnostic =
                prefer_query_selector_diagnostic(preferred_selector, property_name, property_span);

            let literal_value = match argument_expr {
                Expression::StringLiteral(literal) => Some(literal.value.as_str()),
                Expression::TemplateLiteral(literal) => {
                    if literal.expressions.is_empty() {
                        literal.quasis.first().unwrap().value.cooked.as_deref()
                    } else {
                        None
                    }
                }
                _ => None,
            };

            // A dynamic argument would need `CSS.escape()`, and a literal that is not a plain
            // identifier would change meaning as a selector, so neither is rewritten
            let argument = literal_value.and_then(|value| selector_for(property_name, value));
            let Some(argument) = argument else {
                return ctx.diagnostic(diagnostic);
            };

            ctx.diagnostic_with_fix(diagnostic, |fixer| {
                let source_text = fixer.source_range(argument_expr.span());
                let quotes_symbol = source_text.chars().next().unwrap();
                let argument = if property_name == "getElementsByName" {
                    let inner_quote = if quotes_symbol == '\'' { '"' } else { '\'' };
                    format!("[name={inner_quote}{argument}{inner_quote}]")
                } else {
                    argument
                };
                let span = property_span.merge(argument_expr.span());
                fixer.replace(
                    span,
                    format!("{preferred_selector}({quotes_symbol}{argument}{quotes_symbol}"),
                )
            });
        }
    }
}

/// The selector equivalent to `value` passed to `method`, or `None` when no selector
/// matches the same elements without escaping
fn selector_for(method: &str, value: &str) -> Option<String> {
    match method {
        "getElementById" => is_css_identifier(value).then(|| format!("#{value}")),
        "getElementsByClassName" => {
            // `getElementsByClassName` matches elements having ALL the listed
            // classes, so multiple classes become a compound selector
            // (`.foo.bar`), not a descendant selector (`.foo .bar`).
            let classes = value.split_whitespace().collect::<Vec<_>>();
            (!classes.is_empty() && classes.iter().all(|class| is_css_identifier(class)))
                .then(|| format!(".{}", classes.join(".")))
        }
        "getElementsByTagName" => {
            (value == "*" || is_css_identifier(value)).then(|| value.to_string())
        }
        "getElementsByName" => (!value.is_empty()
            && !value.contains(['"', '\'', '\\', '\n', '\r', '`', '$']))
        .then(|| value.to_string()),
        _ => None,
    }
}

/// Whether `value` reads as itself in a selector: ASCII letters, digits, `_` and `-`,
/// not starting with a digit or with `-` and a digit
fn is_css_identifier(value: &str) -> bool {
    let bytes = value.as_bytes();
    let start = match bytes {
        [b'-', b'-', ..] => 2,
        [b'-', rest @ ..] => {
            if rest.first().is_none_or(u8::is_ascii_digit) {
                return false;
            }
            1
        }
        [first, ..] if first.is_ascii_digit() => return false,
        [] => return false,
        _ => 0,
    };
    bytes[start..].iter().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn is_non_literal_argument(expr: &Expression) -> bool {
    match expr.get_inner_expression() {
        Expression::NullLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::BinaryExpression(_) => false,
        Expression::TemplateLiteral(template) => {
            !template.expressions.is_empty()
                && template
                    .quasis
                    .iter()
                    .all(|quasi| quasi.value.cooked.is_none_or(|cooked| cooked.trim().is_empty()))
        }
        _ => true,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("new document.getElementById(foo);", None),
        ("getElementById(foo);", None),
        ("document['getElementById'](bar);", None),
        ("document[getElementById](bar);", None),
        ("document.foo(bar);", None),
        ("document.getElementById();", None),
        ("document?.getElementById('foo');", None),
        ("document.getElementById?.('foo');", None),
        (r#"document.getElementsByClassName("foo", "bar");"#, None),
        (r#"document.getElementById(...["id"]);"#, None),
        (r##"document.querySelector("#foo");"##, None),
        (r#"document.querySelector(".bar");"#, None),
        (r#"document.querySelector("main #foo .bar");"#, None),
        (r#"document.querySelectorAll(".foo .bar");"#, None),
        (r#"document.querySelectorAll("li a");"#, None),
        (r#"document.querySelector("li").querySelectorAll("a");"#, None),
        ("document.getElementsByName();", None),
        (
            "document.getElementById(someId);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementsByClassName(someClass);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementsByClassName(fn());",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementsByClassName(`${someClass}`);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementById(`${someId}`);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementById(obj.id);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementById(someId as string);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
    ];

    let fail = vec![
        (r#"document.getElementById("foo");"#, None),
        (r#"document.getElementsByClassName("foo");"#, None),
        (r#"document.getElementsByClassName("foo bar");"#, None),
        (r#"document.getElementsByTagName("foo");"#, None),
        (r#"document.getElementById("");"#, None),
        ("document.getElementById('foo');", None),
        ("document.getElementsByClassName('foo');", None),
        ("document.getElementsByClassName('foo bar');", None),
        ("document.getElementsByTagName('foo');", None),
        ("document.getElementsByClassName('');", None),
        ("document.getElementById(`foo`);", None),
        ("document.getElementsByClassName(`foo`);", None),
        ("document.getElementsByClassName(`foo bar`);", None),
        ("document.getElementsByTagName(`foo`);", None),
        ("document.getElementsByTagName(``);", None),
        ("document.getElementsByClassName(`${fn()}`);", None),
        ("document.getElementsByClassName(`foo ${undefined}`);", None),
        ("document.getElementsByClassName(null);", None),
        ("document.getElementsByTagName(null);", None),
        ("document.getElementsByClassName(fn());", None),
        (r#"document.getElementsByClassName("foo" + fn());"#, None),
        (r#"document.getElementsByClassName(foo + "bar");"#, None),
        (
            r#"for (const div of document.body.getElementById("id").getElementsByClassName("class")) {
                console.log(div.getElementsByTagName("div"));
            }"#,
            None,
        ),
        ("e.getElementById(3)", None),
        (r#"document.getElementsByName("foo");"#, None),
        ("document.getElementsByName('foo');", None),
        ("document.getElementsByName(`foo`);", None),
        ("document.getElementsByName(`${'foo'}`);", None),
        ("document.getElementsByName(null);", None),
        (r#"document.getElementsByName("");"#, None),
        (r#"document.getElementsByName(foo + "bar");"#, None),
        (r#"document.getElementsByName("multiple name should be fixable");"#, None),
        (
            "document.getElementById(someId);",
            Some(serde_json::json!([{ "allowWithVariables": false }])),
        ),
        (
            r#"document.getElementById("foo");"#,
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            r#"document.getElementById("foo" as string);"#,
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            r#"document.getElementsByClassName("foo");"#,
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            r#"document.getElementsByClassName("foo"!);"#,
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            r#"document.getElementsByTagName("foo");"#,
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            r#"document.getElementsByName("foo");"#,
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementsByTagName(someTag);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementsByName(someName);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementsByClassName(null);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementsByClassName(`foo`);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            r#"document.getElementsByClassName(variable + "x");"#,
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            r#"document.getElementsByClassName("foo" + fn());"#,
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            r#"document.getElementsByClassName(foo + "bar");"#,
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementById(`x${someId}`);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementsByClassName(`foo ${someClass}`);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
    ];

    let fix = vec![
        ("document.getElementsByTagName('foo');", "document.querySelectorAll('foo');"),
        ("document.getElementsByClassName(`foo bar`);", "document.querySelectorAll(`.foo.bar`);"),
        ("document.getElementsByClassName(null);", "document.getElementsByClassName(null);"),
        ("document.getElementsByTagName(`   `);", "document.getElementsByTagName(`   `);"),
        ("document.getElementsByTagName('*');", "document.querySelectorAll('*');"),
        ("document.getElementById(123);", "document.getElementById(123);"),
        ("document.getElementById(`id`);", "document.querySelector(`#id`);"),
        ("document.getElementById(obj.id);", "document.getElementById(obj.id);"),
        ("document.getElementById(getId());", "document.getElementById(getId());"),
        ("document.getElementById(`${foo}`);", "document.getElementById(`${foo}`);"),
        ("document.getElementById(searchInputId);", "document.getElementById(searchInputId);"),
        (
            r#"document.getElementById("address.line1");"#,
            r#"document.getElementById("address.line1");"#,
        ),
        (r#"document.getElementById("1st");"#, r#"document.getElementById("1st");"#),
        (r#"document.getElementById("-1st");"#, r#"document.getElementById("-1st");"#),
        (r#"document.getElementById(" foo ");"#, r#"document.getElementById(" foo ");"#),
        (r#"document.getElementById("");"#, r#"document.getElementById("");"#),
        (
            r#"document.getElementById("-foo_bar-2");"#,
            r##"document.querySelector("#-foo_bar-2");"##,
        ),
        (
            r#"document.getElementsByClassName("foo sm:bar");"#,
            r#"document.getElementsByClassName("foo sm:bar");"#,
        ),
        (r#"document.getElementsByName("foo");"#, r#"document.querySelectorAll("[name='foo']");"#),
        ("document.getElementsByName('foo');", r#"document.querySelectorAll('[name="foo"]');"#),
        ("document.getElementsByName(`foo`);", "document.querySelectorAll(`[name='foo']`);"),
        ("document.getElementsByName(null);", "document.getElementsByName(null);"),
        (r#"document.getElementsByName("");"#, r#"document.getElementsByName("");"#),
        (r#"document.getElementsByName("it's");"#, r#"document.getElementsByName("it's");"#),
        (
            r#"document.getElementsByName("multiple name should be fixable");"#,
            r#"document.querySelectorAll("[name='multiple name should be fixable']");"#,
        ),
        // We do not fix these.
        (
            "document.getElementsByClassName(foo + \"bar\");",
            "document.getElementsByClassName(foo + \"bar\");",
        ),
        ("document.getElementsByClassName(fn());", "document.getElementsByClassName(fn());"),
        ("document.getElementsByName(`${'foo'}`);", "document.getElementsByName(`${'foo'}`);"),
        (
            r#"document.getElementsByName(foo + "bar");"#,
            r#"document.getElementsByName(foo + "bar");"#,
        ),
    ];

    Tester::new(PreferQuerySelector::NAME, PreferQuerySelector::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

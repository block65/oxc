use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    ast_util::get_declaration_of_variable,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::{DefaultRuleConfig, Rule},
};

fn no_typeof_undefined_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Compare with `undefined` directly instead of using `typeof`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoTypeofUndefined {
    /// If set to `true`, also report `typeof x === "undefined"` when `x` may be a global
    /// variable that is not declared (commonly checked via `typeof foo === "undefined"`).
    check_global_variables: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow `typeof` comparisons with `undefined`.
    ///
    /// ### Why is this bad?
    ///
    /// Checking if a value is `undefined` by using `typeof value === 'undefined'` is needlessly verbose. It's generally better to compare against `undefined` directly. The only time `typeof` is needed is when a global variable potentially does not exists, in which case, using `globalThis.value === undefined` may be better.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// typeof foo === 'undefined';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// foo === undefined;
    /// ```
    ///
    /// ### Fixes
    ///
    /// The fix is a suggestion rather than a safe fix where the variable may be
    /// an undeclared global, as `foo === undefined` throws a `ReferenceError`
    /// for one. In TypeScript, it is also a suggestion inside a generic
    /// function or class: `typeof value === "undefined"` narrows a value of an
    /// unconstrained type parameter `T` to `undefined`, and
    /// `value === undefined` does not, so the fix can introduce type errors.
    NoTypeofUndefined,
    unicorn,
    pedantic,
    recommended,
    fix_or_suggestion,
    config = NoTypeofUndefined,
    version = "0.0.18",
    short_description = "Disallow `typeof` comparisons with `undefined`.",
);

impl Rule for NoTypeofUndefined {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(bin_expr) = node.kind() else {
            return;
        };

        let operator = match bin_expr.operator {
            BinaryOperator::StrictEquality | BinaryOperator::Equality => "===",
            BinaryOperator::StrictInequality | BinaryOperator::Inequality => "!==",
            _ => return,
        };

        let Expression::UnaryExpression(unary_expr) = &bin_expr.left else {
            return;
        };

        if unary_expr.operator != UnaryOperator::Typeof {
            return;
        }

        if !bin_expr.right.is_specific_string_literal("undefined") {
            return;
        }

        let is_global_variable = is_global_variable(&unary_expr.argument, ctx);

        if !self.check_global_variables && is_global_variable {
            return;
        }

        if is_global_variable || in_generic_scope(node, ctx) {
            ctx.diagnostic_with_suggestion(
                no_typeof_undefined_diagnostic(bin_expr.span),
                |fixer| generate_fix(fixer, &unary_expr.argument, operator, bin_expr.span, ctx),
            );
        } else {
            ctx.diagnostic_with_fix(no_typeof_undefined_diagnostic(bin_expr.span), |fixer| {
                generate_fix(fixer, &unary_expr.argument, operator, bin_expr.span, ctx)
            });
        }
    }

    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }
}

fn is_global_variable<'a>(ident: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
    let Expression::Identifier(ident) = ident else {
        return false;
    };

    get_declaration_of_variable(ident, ctx).is_none()
}

/// Returns `true` if `node` is inside a function or class that declares type
/// parameters, where an operand's type may be a type parameter.
fn in_generic_scope(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    ctx.nodes().ancestor_kinds(node.id()).any(|kind| match kind {
        AstKind::Function(func) => func.type_parameters.is_some(),
        AstKind::ArrowFunctionExpression(arrow) => arrow.type_parameters.is_some(),
        AstKind::Class(class) => class.type_parameters.is_some(),
        _ => false,
    })
}

fn generate_fix<'a>(
    fixer: RuleFixer<'_, 'a>,
    argument: &Expression<'a>,
    operator: &str,
    span: Span,
    ctx: &LintContext<'a>,
) -> RuleFix {
    let argument_text = ctx.source_range(argument.span());
    fixer.replace(span, format!("{argument_text} {operator} undefined"))
}

#[test]
fn test() {
    use crate::{
        fixer::FixKind,
        tester::{ExpectFixTestCase, Tester},
    };

    let pass = vec![
        (r"typeof a.b", None),
        (r#"typeof a.b > "undefined""#, None),
        (r#"a.b === "undefined""#, None),
        (r#"void a.b === "undefined""#, None),
        (r#"+a.b === "undefined""#, None),
        (r#"++a.b === "undefined""#, None),
        (r#"a.b++ === "undefined""#, None),
        (r"foo === undefined", None),
        (r#"typeof a.b === "string""#, None),
        (r#"typeof foo === "undefined""#, None),
        (r#"foo = 2; typeof foo === "undefined""#, None),
        (r#"/* globals foo: readonly */ typeof foo === "undefined""#, None),
        (r#"/* globals globalThis: readonly */ typeof globalThis === "undefined""#, None),
        (r#""undefined" === typeof a.b"#, None),
        (r#"const UNDEFINED = "undefined"; typeof a.b === UNDEFINED"#, None),
        (r"typeof a.b === `undefined`", None),
    ];

    let fail = vec![
        (r#"typeof a.b === "undefined""#, None),
        (r#"typeof a.b !== "undefined""#, None),
        (r#"typeof a.b == "undefined""#, None),
        (r#"typeof a.b != "undefined""#, None),
        (r"typeof a.b == 'undefined'", None),
        (r#"let foo; typeof foo === "undefined""#, None),
        (r#"const foo = 1; typeof foo === "undefined""#, None),
        (r#"var foo; typeof foo === "undefined""#, None),
        (r#"var foo; var foo; typeof foo === "undefined""#, None),
        (r#"for (const foo of bar) typeof foo === "undefined";"#, None),
        (r#"function foo() {typeof foo === "undefined"}"#, None),
        (r#"function foo(bar) {typeof bar === "undefined"}"#, None),
        (r#"function foo({bar}) {typeof bar === "undefined"}"#, None),
        (r#"function foo([bar]) {typeof bar === "undefined"}"#, None),
        (r#"typeof foo.bar === "undefined""#, None),
        (
            r#"let foo; typeof foo === "undefined""#,
            Some(serde_json::json!([{ "checkGlobalVariables": false }])),
        ),
        (
            r#"typeof foo === "undefined""#,
            Some(serde_json::json!([{ "checkGlobalVariables": true }])),
        ),
    ];

    let fix = vec![
        (r#"typeof a.b === "undefined""#, r"a.b === undefined", None),
        (r#"typeof a.b !== "undefined""#, r"a.b !== undefined", None),
        (r#"typeof a.b == "undefined""#, r"a.b === undefined", None),
        (r#"typeof a.b != "undefined""#, r"a.b !== undefined", None),
        (r"typeof a.b == 'undefined'", r"a.b === undefined", None),
        (r#"let foo; typeof foo === "undefined""#, r"let foo; foo === undefined", None),
        (r#"const foo = 1; typeof foo === "undefined""#, r"const foo = 1; foo === undefined", None),
        (r#"var foo; typeof foo === "undefined""#, r"var foo; foo === undefined", None),
        (r#"typeof foo.bar === "undefined""#, r"foo.bar === undefined", None),
        (
            r#"typeof foo === "undefined""#,
            r"foo === undefined",
            Some(serde_json::json!([{ "checkGlobalVariables": true }])),
        ),
    ];

    let mut fix = fix.into_iter().map(ExpectFixTestCase::from).collect::<Vec<_>>();
    // Inside a generic function or class, `x === undefined` may not narrow `x`
    // as `typeof x === "undefined"` does, so the fix is only a suggestion.
    let generic = [
        "function f<T>(val: T) { return typeof val === 'undefined' ? val : String(val); }",
        "const f = <T,>(val: T) => typeof val !== 'undefined';",
        "function f<T>(o: { v: T }) { return typeof o.v === 'undefined'; }",
        "class C<T> { m(val: T) { return typeof val === 'undefined'; } }",
        "function f<T>(val: T) { return () => typeof val === 'undefined'; }",
        "const o = { m<T>(val: T) { return typeof val === 'undefined'; } };",
    ];
    for source in generic {
        fix.push((source, source, None, FixKind::SafeFix).into());
    }
    fix.extend([
        (
            "function f<T>(val: T) { return typeof val === 'undefined' ? val : String(val); }",
            "function f<T>(val: T) { return val === undefined ? val : String(val); }",
            None,
            FixKind::Suggestion,
        )
            .into(),
        // Outside a generic scope, the fix stays safe.
        (
            "function f(val: unknown) { return typeof val === 'undefined'; }",
            "function f(val: unknown) { return val === undefined; }",
            None,
            FixKind::SafeFix,
        )
            .into(),
    ]);

    Tester::new(NoTypeofUndefined::NAME, NoTypeofUndefined::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

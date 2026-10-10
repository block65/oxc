use cow_utils::CowUtils;
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn uppercase_prefix(span: Span, prefix: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected number literal prefix in uppercase.")
        .with_help(format!("Use lowercase for the number literal prefix `{prefix}`."))
        .with_label(span)
}

fn uppercase_exponential_notation(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected exponential notation in uppercase.")
        .with_help("Use lowercase for `e` in exponential notations.")
        .with_label(span)
}

fn hexadecimal_digits(span: Span, case: HexadecimalValue) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unexpected hexadecimal digits in {}.", case.opposite()))
        .with_help(format!("Use {} for hexadecimal digits.", case.as_str()))
        .with_label(span)
}

fn uppercase_prefix_and_hexadecimal_digits(
    span: Span,
    prefix: &str,
    case: HexadecimalValue,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Unexpected number literal prefix in uppercase and hexadecimal digits in {}.",
        case.opposite()
    ))
    .with_help(format!(
        "Use lowercase for the number literal prefix `{prefix}` and {} for hexadecimal digits.",
        case.as_str()
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum HexadecimalValue {
    #[default]
    Uppercase,
    Lowercase,
}

impl HexadecimalValue {
    fn as_str(self) -> &'static str {
        match self {
            Self::Uppercase => "uppercase",
            Self::Lowercase => "lowercase",
        }
    }

    fn opposite(self) -> &'static str {
        match self {
            Self::Uppercase => "lowercase",
            Self::Lowercase => "uppercase",
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NumberLiteralCase {
    /// The case to enforce for hexadecimal digits.
    hexadecimal_value: HexadecimalValue,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces proper case for numeric literals.
    ///
    /// ### Why is this bad?
    ///
    /// When both an identifier and a numeric literal are in
    /// lower case, it can be hard to differentiate between them.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// <!-- prettier-ignore-start -->
    /// ```javascript
    /// const foo = 0XFF;
    /// const foo = 0xff;
    /// const foo = 0Xff;
    /// const foo = 0Xffn;
    ///
    /// const foo = 0B10;
    /// const foo = 0B10n;
    ///
    /// const foo = 0O76;
    /// const foo = 0O76n;
    ///
    /// const foo = 2E-5;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const foo = 0xFF;
    /// const foo = 0b10;
    /// const foo = 0o76;
    /// const foo = 0xFFn;
    /// const foo = 2e+5;
    /// ```
    /// <!-- prettier-ignore-end -->
    ///
    /// Examples of **correct** code for this rule with `{ "hexadecimalValue": "lowercase" }`:
    /// ```javascript
    /// const foo = 0xff;
    /// const foo = 0xffn;
    /// ```
    NumberLiteralCase,
    unicorn,
    style,
    fix,
    config = NumberLiteralCase,
    version = "0.0.18",
    short_description = "This rule enforces proper case for numeric literals.",
);

impl Rule for NumberLiteralCase {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (raw_literal, raw_span) = match node.kind() {
            AstKind::NumericLiteral(number) => (number.raw.as_ref().unwrap().as_str(), number.span),
            AstKind::BigIntLiteral(number) => {
                let span = number.span;
                (span.source_text(ctx.source_text()), span)
            }
            _ => return,
        };

        if let Some((diagnostic, fixed_literal)) =
            check_number_literal(raw_literal, raw_span, self.hexadecimal_value)
        {
            ctx.diagnostic_with_fix(diagnostic, |fixer| fixer.replace(raw_span, fixed_literal));
        }
    }
}

#[expect(clippy::cast_possible_truncation)]
fn check_number_literal(
    number_literal: &str,
    raw_span: Span,
    case: HexadecimalValue,
) -> Option<(OxcDiagnostic, String)> {
    if number_literal.starts_with("0B") || number_literal.starts_with("0O") {
        return Some((
            uppercase_prefix(
                Span::new(raw_span.start + 1, raw_span.start + 2),
                if number_literal.starts_with("0B") { "0b" } else { "0o" },
            ),
            number_literal.cow_to_ascii_lowercase().into_owned(),
        ));
    }
    if number_literal.starts_with("0X") || number_literal.starts_with("0x") {
        let has_uppercase_prefix = number_literal.starts_with("0X");
        let has_wrong_case_digits = number_literal[2..].chars().any(|c| match case {
            HexadecimalValue::Uppercase => ('a'..='f').contains(&c),
            HexadecimalValue::Lowercase => ('A'..='F').contains(&c),
        });
        if has_uppercase_prefix && has_wrong_case_digits {
            return Some((
                uppercase_prefix_and_hexadecimal_digits(raw_span, "0x", case),
                "0x".to_owned() + &digits_to_case(&number_literal[2..], case),
            ));
        }
        if has_uppercase_prefix {
            return Some((
                uppercase_prefix(Span::new(raw_span.start + 1, raw_span.start + 2), "0x"),
                "0x".to_owned() + &number_literal[2..],
            ));
        }
        if has_wrong_case_digits {
            return Some((
                hexadecimal_digits(Span::new(raw_span.start + 2, raw_span.end), case),
                "0x".to_owned() + &digits_to_case(&number_literal[2..], case),
            ));
        }
        return None;
    }
    if let Some(index) = number_literal.find('E') {
        let char_position = raw_span.start + index as u32;
        return Some((
            uppercase_exponential_notation(Span::sized(char_position, 1)),
            number_literal.cow_to_ascii_lowercase().into_owned(),
        ));
    }
    None
}

fn digits_to_case(digits: &str, case: HexadecimalValue) -> String {
    match case {
        // A BigInt's `n` suffix is lowercase too.
        HexadecimalValue::Lowercase => digits.cow_to_ascii_lowercase().into_owned(),
        HexadecimalValue::Uppercase => {
            let mut result = digits.cow_to_ascii_uppercase().into_owned();
            if result.ends_with('N') {
                result.truncate(result.len() - 1);
                result.push('n');
            }
            result
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let lowercase = Some(serde_json::json!([{ "hexadecimalValue": "lowercase" }]));
    let uppercase = Some(serde_json::json!([{ "hexadecimalValue": "uppercase" }]));

    let pass = vec![
        ("const foo = 0xABCDEF", uppercase.clone()),
        ("const foo = 0xabcdef", lowercase.clone()),
        ("const foo = 0xabcdefn", lowercase.clone()),
        ("const foo = 0xdeed_beef", lowercase.clone()),
        ("const foo = 0b10", lowercase.clone()),
        ("const foo = 1.2e3", lowercase.clone()),
        ("const foo = 1234", lowercase.clone()),
    ];
    let pass_default = vec![
        "var foo = 0777",
        "var foo = 0888",
        "const foo = 1234",
        "const foo = 0b10",
        "const foo = 0o1234567",
        "const foo = 0xABCDEF",
        "const foo = 1234n",
        "const foo = 0b10n",
        "const foo = 0o1234567n",
        "const foo = 0xABCDEFn",
        "const foo = NaN",
        "const foo = +Infinity",
        "const foo = -Infinity",
        "const foo = 1.2e3",
        "const foo = 1.2e-3",
        "const foo = 1.2e+3",
        "const foo = '0Xff'",
        "const foo = '0Xffn'",
        "const foo = 123_456",
        "const foo = 0b10_10",
        "const foo = 0o1_234_567",
        "const foo = 0xDEED_BEEF",
        "const foo = 123_456n",
        "const foo = 0b10_10n",
        "const foo = 0o1_234_567n",
        "const foo = 0xDEED_BEEFn",
    ];

    let fail_options = vec![
        ("const foo = 0xABCDEF", lowercase.clone()),
        ("const foo = 0XaBcDeF", lowercase.clone()),
        ("const foo = 0XaBcDeFn", lowercase.clone()),
        ("const foo = 0XABCDEF", lowercase.clone()),
        ("const foo = 0xdeED_BEEFn", lowercase.clone()),
        ("const foo = 0B10", lowercase.clone()),
        ("const foo = 1.2E3", lowercase.clone()),
        ("const foo = 0xabcdef", uppercase.clone()),
    ];
    let fail_default = vec![
        "const foo = 0B10",
        "const foo = 0O1234567",
        "const foo = 0XaBcDeF",
        "const foo = 0B10n",
        "const foo = 0O1234567n",
        "const foo = 0XaBcDeFn",
        "const foo = 0B0n",
        "const foo = 0O0n",
        "const foo = 0X0n",
        "const foo = 1.2E3",
        "const foo = 1.2E-3",
        "const foo = 1.2E+3",
        "
            const foo = 255;

            if (foo === 0xff) {
                console.log('invalid');
            }
        ",
        "const foo = 0XdeEd_Beefn",
        "console.log(BigInt(0B10 + 1.2E+3) + 0XdeEd_Beefn)",
    ];

    let fix = vec![
        ("const foo = 0xABCDEF", "const foo = 0xabcdef", lowercase.clone()),
        ("const foo = 0XaBcDeF", "const foo = 0xabcdef", lowercase.clone()),
        ("const foo = 0XaBcDeFn", "const foo = 0xabcdefn", lowercase.clone()),
        ("const foo = 0XABCDEF", "const foo = 0xabcdef", lowercase.clone()),
        ("const foo = 0xdeED_BEEFn", "const foo = 0xdeed_beefn", lowercase.clone()),
        ("const foo = 0B10", "const foo = 0b10", lowercase.clone()),
        ("const foo = 1.2E3", "const foo = 1.2e3", lowercase),
        ("const foo = 0xabcdef", "const foo = 0xABCDEF", uppercase),
        ("const foo = 0B10", "const foo = 0b10", None),
        ("const foo = 0O1234567", "const foo = 0o1234567", None),
        ("const foo = 0XaBcDeF", "const foo = 0xABCDEF", None),
        ("const foo = 0B10n", "const foo = 0b10n", None),
        ("const foo = 0O1234567n", "const foo = 0o1234567n", None),
        ("const foo = 0XaBcDeFn", "const foo = 0xABCDEFn", None),
        ("const foo = 0B0n", "const foo = 0b0n", None),
        ("const foo = 0O0n", "const foo = 0o0n", None),
        ("const foo = 0X0n", "const foo = 0x0n", None),
        ("const foo = 1.2E3", "const foo = 1.2e3", None),
        ("const foo = 1.2E-3", "const foo = 1.2e-3", None),
        ("const foo = 1.2E+3", "const foo = 1.2e+3", None),
        (
            "
            const foo = 255;

            if (foo === 0xff) {
                console.log('invalid');
            }
            ",
            "
            const foo = 255;

            if (foo === 0xFF) {
                console.log('invalid');
            }
            ",
            None,
        ),
        ("const foo = 0XdeEd_Beefn", "const foo = 0xDEED_BEEFn", None),
        (
            "console.log(BigInt(0B10 + 1.2E+3) + 0XdeEd_Beefn)",
            "console.log(BigInt(0b10 + 1.2e+3) + 0xDEED_BEEFn)",
            None,
        ),
    ];

    let pass: Vec<_> = pass_default.into_iter().map(|code| (code, None)).chain(pass).collect();
    let fail: Vec<_> =
        fail_default.into_iter().map(|code| (code, None)).chain(fail_options).collect();

    Tester::new(NumberLiteralCase::NAME, NumberLiteralCase::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

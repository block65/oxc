# @block65/oxlint

oxlint with four added type-aware rules and four changed upstream rules. The
four added rules are implemented in @block65/oxlint-tsgolint, and `package.json` declares the
oldest build of it they work with as an optional peer dependency on
`oxlint-tsgolint`. Linux x64 and arm64 only, the platforms Block65 runs. The
package version names the upstream release it is built on, as described under
Versioning.

```yaml
# pnpm-workspace.yaml
overrides:
  oxlint: npm:@block65/oxlint@[version]
  oxlint-tsgolint: npm:@block65/oxlint-tsgolint@[version]
```

## Added rules

All four are `typescript/` rules in the `restriction` category. They run in
tsgolint, so they need `--type-aware`. Each reports a place where a finite
type is erased.

- `define-messages-keys`: `defineMessages` called without an explicit
  string-literal union as its type argument.
- `no-widening-alias`: a `const` whose annotation is wider than its
  initializer, such as `string` over a literal union or `Record<string, T>`
  over an object with known keys.
- `no-widening-object-keys`: `Object.keys` or `Object.entries` on a value with
  a finite key type. Options `keys` and `entries` name the helpers to use
  instead; the defaults are `typedKeys` and `typedEntries`.
- `no-widening-return-type`: a return annotation wider than what the body
  returns, such as `boolean` over `return true`. Option `contracts` (default
  `true`) allows a named object type as the annotation.

## Changed rules

`typescript/no-inferrable-types` reports a primitive annotation it considers
redundant and suggests deleting it. On a `const` holding a literal the
annotation is not redundant: `const a: number = 5` is typed `number`, while
`const a = 5` is typed `5`. Deleting it narrows the type, and on an exported
declaration that narrows what every other file sees.

In this build the rule stops reporting that case for `number`, `string`,
`boolean` and `bigint`. It still reports `const a: 5 = 5`, where the
annotation matches what inference already gives, and `let`, `var`, parameters
and properties, which widen on their own.

Three `unicorn/` rules report what they did upstream, but `--fix` applies
fewer of their rewrites. Where a rewrite can change behaviour or types, it is
offered as a suggestion instead, which `--fix-suggestions` applies:

- `prefer-single-call` merges `push()` and `unshift()` calls only on a
  variable known to be an array: annotated `T[]` or `Array<T>`, or
  initialised with an array and never reassigned. Any other receiver may have
  a `push()` that takes one argument, such as a stream's.
- `no-typeof-undefined` does not rewrite inside a generic function or class.
  `typeof value === "undefined"` narrows a value of an unconstrained type
  parameter to `undefined`, and `value === undefined` does not.
- `no-useless-undefined` does not rewrite these in TypeScript files:
  `return undefined` and `() => undefined`, which change an inferred return
  type from `undefined` to `void`; a parameter's `= undefined` default, which
  makes the parameter required; and trailing `undefined` arguments, which can
  leave a call short of a required parameter.

## Versioning

The patch is upstream's patch times 100 plus a build number. Upstream 1.83.0
gives 1.83.0, then 1.83.1; an upstream 1.83.1 would give 1.83.100. The major
and minor are upstream's, so `oxlint` ranges keep resolving.

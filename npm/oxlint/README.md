# @block65/oxlint

oxlint with four added type-aware rules and one changed upstream rule. The four
are implemented in @block65/oxlint-tsgolint, and `package.json` declares the
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

## Changed rule

`typescript/no-inferrable-types` reports a primitive annotation it considers
redundant and suggests deleting it. On a `const` holding a literal the
annotation is not redundant: `const a: number = 5` is typed `number`, while
`const a = 5` is typed `5`. Deleting it narrows the type, and on an exported
declaration that narrows what every other file sees.

In this build the rule stops reporting that case for `number`, `string`,
`boolean` and `bigint`. It still reports `const a: 5 = 5`, where the
annotation matches what inference already gives, and `let`, `var`, parameters
and properties, which widen on their own.

## Versioning

The patch is upstream's patch times 100 plus a build number. Upstream 1.83.0
gives 1.83.0, then 1.83.1; an upstream 1.83.1 would give 1.83.100. The major
and minor are upstream's, so `oxlint` ranges keep resolving.

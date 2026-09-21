# @block65/oxlint

oxlint 1.83.0 with four added type-aware rules and one changed upstream rule.
Those four are implemented in @block65/oxlint-tsgolint. Linux x64 and arm64
only.

```yaml
# pnpm-workspace.yaml
overrides:
  oxlint: npm:@block65/oxlint@1.83.0
  oxlint-tsgolint: npm:@block65/oxlint-tsgolint@7.0.200200
```

## Added rules

- `typescript/define-messages-keys`
- `typescript/no-widening-alias`
- `typescript/no-widening-object-keys`
- `typescript/no-widening-return-type`

## Versioning

The patch is upstream's patch times 100 plus a build number. Upstream 1.83.0
gives 1.83.0, then 1.83.1; an upstream 1.83.1 would give 1.83.100. The major
and minor are upstream's, so `oxlint` ranges keep resolving.

## Changed rule

`typescript/no-inferrable-types` reports a primitive annotation it considers
redundant and suggests deleting it. On a `const` holding a literal the
annotation is not redundant: `const a: number = 5` is typed `number`, while
`const a = 5` is typed `5`. Deleting it narrows the type, and on an exported
declaration that narrows what every other file sees.

This build stops reporting that case for `number`, `string`, `boolean` and
`bigint`. Still reported: `const a: 5 = 5`, where the annotation matches what
inference already gives, and `let`, `var`, parameters and properties, which
widen on their own.

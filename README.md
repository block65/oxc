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

A rebase has to carry this forward.

`typescript/no-inferrable-types` no longer reports a widened primitive
annotation on a `const` initialised to a literal (`const a: number = 5`).
Inference gives `a` the type `5`, so removing the annotation narrows it, and
the suggested fix was narrowing exported types. `const a: 5 = 5`, `let`, `var`,
parameters and properties are still reported.

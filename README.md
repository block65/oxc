# @block65/oxlint

oxlint 1.83.0 plus four type-aware rules and one changed upstream rule. The
rules are implemented in @block65/oxlint-tsgolint. Linux x64 and arm64 only.

Internal to Block65, on a public registry only because a pnpm override has to
resolve from one. Not supported for outside use, and the repo takes no issues.

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

Patch is upstream's patch times 100 plus a build number: upstream 1.83.0 gives
1.83.0, then 1.83.1; an upstream 1.83.1 would give 1.83.100. Major and minor
are upstream's, so `oxlint` ranges keep resolving.

## Changed rule

Carry this through a rebase.

`typescript/no-inferrable-types` no longer reports a widened primitive
annotation on a `const` initialised to a literal (`const a: number = 5`).
Inference gives `a` the type `5`, so the annotation is doing work and the
suggested fix narrowed exported types. `const a: 5 = 5`, `let`, `var`,
parameters and properties are still reported.

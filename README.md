# @block65/oxlint

Fork of oxlint 1.82.0 with additional type-aware rules, provided by @block65/oxlint-tsgolint. Block65 internal.

## Use

```yaml
# pnpm-workspace.yaml
overrides:
  oxlint: npm:@block65/oxlint@1.82.0
  oxlint-tsgolint: npm:@block65/oxlint-tsgolint@7.0.2001
```

## Patched rules

Divergences from upstream oxlint, which a rebase has to carry forward:

- `typescript/no-inferrable-types` does not report a widened primitive annotation
  on a `const` initialized to a literal (`const a: number = 5`). Inference would
  give `a` the type `5`, so the annotation is load-bearing and the suggested fix
  silently narrowed exported types. Annotating the literal type itself
  (`const a: 5 = 5`) is still reported, as are `let`, `var`, parameters and
  properties. Upstream oxlint and typescript-eslint both report all of these.

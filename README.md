# @block65/oxlint

Fork of oxlint 1.83.0 with additional type-aware rules, provided by @block65/oxlint-tsgolint. Block65 internal.

## Use

```yaml
# pnpm-workspace.yaml
overrides:
  oxlint: npm:@block65/oxlint@1.83.0
  oxlint-tsgolint: npm:@block65/oxlint-tsgolint@7.0.200200
```

## Versioning

Our patch is upstream's patch times 100, plus our build number (00-99).
Upstream 1.83.0 gives `1.83.0` for build 00 and `1.83.1` for build 01; an
upstream 1.83.1 would give `1.83.100`. Upstream's major and minor are left
alone, so the `oxlint` ranges in consuming repos stay satisfied.

## Patched rules

Divergences from upstream oxlint, which a rebase has to carry forward:

- `typescript/no-inferrable-types` does not report a widened primitive annotation
  on a `const` initialized to a literal (`const a: number = 5`). Inference would
  give `a` the type `5`, so the annotation is load-bearing and the suggested fix
  silently narrowed exported types. Annotating the literal type itself
  (`const a: 5 = 5`) is still reported, as are `let`, `var`, parameters and
  properties. Upstream oxlint and typescript-eslint both report all of these.

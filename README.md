# @block65/oxlint

Block65 build of oxlint 1.82.0 with additional type-aware rules routed to @block65/oxlint-tsgolint. Internal use.

Source: https://github.com/block65/oxc, branch block65/oxlint_v1.82.0. Patch set listed in NOTICE. Upstream LICENSE applies.

## Use

```yaml
# pnpm-workspace.yaml
overrides:
  oxlint: npm:@block65/oxlint@1.82.0
  oxlint-tsgolint: npm:@block65/oxlint-tsgolint@7.0.2001
```

# @block65/oxlint

A Block65 patched build of [oxlint](https://github.com/oxc-project/oxc) 1.82.0.
It is not the oxc project. The source is
[block65/oxc](https://github.com/block65/oxc), branch `block65/oxlint_v1.82.0`,
which is the upstream tag plus the patch set described in `NOTICE`. Upstream's
LICENSE and copyright apply unchanged.

## What it adds

Four type-aware rules under the `typescript/` namespace, routed to the
matching [`@block65/oxlint-tsgolint`](https://github.com/block65/tsgolint)
build, which implements them:

- `typescript/define-messages-keys`
- `typescript/no-widening-alias`
- `typescript/no-widening-object-keys`
- `typescript/no-widening-return-type`

Everything else is upstream oxlint 1.82.0, unchanged. Stock oxlint rejects a
configuration that names these rules, so a project using them fails loudly
rather than linting without them.

## Installation

Keep the plain `oxlint` and `oxlint-tsgolint` dependencies in `package.json`
and route them to the patched builds with a pnpm override, so no command or
config names the fork:

```yaml
# pnpm-workspace.yaml
overrides:
  oxlint: npm:@block65/oxlint@1.82.0
  oxlint-tsgolint: npm:@block65/oxlint-tsgolint@7.0.2001
```

The package keeps upstream's layout and the `oxlint` bin name. Supported
platforms: Linux x64 and Linux arm64 only. Enable the
rules with `"plugins": ["typescript"]`, `"options": { "typeAware": true }` and
the rule names above, then run `oxlint --type-aware`.

## Documentation

Upstream's documentation covers everything but the four rules:
[oxc.rs](https://oxc.rs/docs/guide/usage/linter.html). The four rules are
documented in the [block65 oxlint plugin](https://github.com/block65/oxlint-plugin).

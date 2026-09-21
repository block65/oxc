# block65/oxc

A fork of [oxc](https://github.com/oxc-project/oxc) that builds one package,
`@block65/oxlint`. The rest of the monorepo is upstream's and goes unused.

The working branch is a stack of fork commits rebased onto an upstream release
tag, currently `oxlint_v1.83.0`. Upstream `main` is never merged, so GitHub
reports the branch as behind it by however far `main` has moved past that
release. That is expected.

## Where the changes are

`crates/oxc_linter/src/rules/typescript/` holds the four added rules and the
changed `no-inferrable-types`; `npm/oxlint/README.md` describes their
behaviour. The added rules are stubs, implemented in
[block65/tsgolint](https://github.com/block65/tsgolint).

`.github/workflows/release.yml` is the only workflow. It builds the Linux
bindings and runs `pnpm stage publish`, which uploads for review rather than
going live. `pnpm stage approve` completes a publish.

## Rebasing onto a new upstream release

Cherry-pick the fork commits onto the new `oxlint_v<version>` tag, then update
the version literals and both NOTICE files.

## Releasing

The version is upstream's patch times 100 plus a build number, described in
`npm/oxlint/README.md`. Commit it on its own, tag that commit `v<version>`,
and publish a GitHub release against the tag.

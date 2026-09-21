# block65/oxc

A fork of [oxc](https://github.com/oxc-project/oxc) that builds one package,
`@block65/oxlint`. The rest of the monorepo is upstream's and goes unused.

`main` is a stack of fork commits rebased onto upstream `main`, so it reads as
ahead of upstream and never behind. Upstream is never merged in: each sync
replays the stack and pushes with `--force-with-lease`. `main` carries no fork
version. `npm/oxlint/package.json` holds whatever version upstream `main`
holds, and the package README and description name no version. Releases are
cut from tags, described below.

## Where the changes are

`crates/oxc_linter/src/rules/typescript/` holds the four added rules and the
changed `no-inferrable-types`; `npm/oxlint/README.md` describes their
behaviour. The added rules are stubs, implemented in
[block65/tsgolint](https://github.com/block65/tsgolint).

`.github/workflows/deploy.yml` is the only workflow. It builds the Linux
bindings and runs `pnpm stage publish`, which uploads for review rather than
going live. A maintainer logged in to npm with 2FA completes the publish with
`pnpm stage approve`.

## Syncing with upstream

```sh
git fetch https://github.com/oxc-project/oxc.git main
git rebase FETCH_HEAD
git push --force-with-lease
```

Two kinds of conflict come up. Generated files
(`crates/oxc_linter/src/generated/`, `npm/oxlint/configuration_schema.json`,
`apps/oxlint/src-js/package/config.generated.ts`) take upstream's copy and are
regenerated, never hand-merged:

```sh
cargo run -q -p oxc_linter_codegen
cargo run -q -p website_linter schema-json > schema.json && mv schema.json npm/oxlint/configuration_schema.json
pnpm --filter oxlint-app generate-config-types
```

Files the fork deletes (`AGENTS.md`, `CLAUDE.md`, upstream CI) that upstream
has since edited are resolved with `git rm`.

## Releasing

A release is the stack replayed onto the upstream release tag, plus one commit
that sets the version in `npm/oxlint/package.json`:

```sh
git fetch https://github.com/oxc-project/oxc.git main
base=$(git merge-base main FETCH_HEAD)
git fetch https://github.com/oxc-project/oxc.git tag oxlint_v[upstream]
git checkout --detach main
git rebase --onto oxlint_v[upstream] "$base"
```

The version is upstream's patch times 100 plus a build number, described in
`npm/oxlint/README.md`. The version commit is omitted when the two are equal,
as for build 00 of a `.0` release. Tag the last commit `v[version]` and publish
a GitHub release against the tag, which runs `deploy.yml`. Release tags are not
on `main`'s history, and `main` is not rewritten for a release. When the build
raises the `oxlint-tsgolint` floor in `package.json`, publish tsgolint first.

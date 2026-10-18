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

`crates/oxc_linter/src/rules/typescript/` holds the five added rules and the
changed `no-inferrable-types`. `crates/oxc_linter/src/rules/unicorn/` holds
`prefer-single-call`, `no-typeof-undefined`, `no-useless-undefined`,
`prefer-dom-node-append` and `prefer-query-selector`, whose autofixes are
narrowed, and `catch-error-name`, whose fix keeps shorthand keys. `npm/oxlint/README.md` describes all of them. The added rules are stubs, implemented in
[block65/tsgolint](https://github.com/block65/tsgolint).

`.github/workflows/deploy.yml` is the release workflow. It builds the Linux
bindings and runs `pnpm stage publish`, which uploads for review rather than
going live. A maintainer logged in to npm with 2FA completes the publish with
`pnpm stage approve`. `.github/workflows/lockfile.yml` runs on pull requests
and pushes to `main`, and fails when `pnpm-lock.yaml` is out of date with a
`package.json`, which would otherwise fail the release.

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

`pnpm-lock.yaml` conflicts on its `oxlint-tsgolint` entries whenever upstream
moves that dependency, because `pnpm-workspace.yaml` overrides it to the
block65 build. Take upstream's copy and run `pnpm install --lockfile-only`.

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
raises the `oxlint-tsgolint` floor in `package.json`, publish and approve
tsgolint first, then move the `oxlint-tsgolint` override in
`pnpm-workspace.yaml` to the same floor and run `pnpm install --lockfile-only`,
since the fork's own install resolves the peer and the registry's upstream
package never reaches a block65 build number.

Before pushing the tag, check the lockfile on the release commit. The replayed
stack can leave it behind upstream's `package.json` files, and `deploy.yml`
installs with `--frozen-lockfile`:

```sh
pnpm install --frozen-lockfile --lockfile-only
```

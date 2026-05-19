# CLAUDE.md

Project-specific instructions for Claude Code working in this repository.

## Project

`legba` is a multi-protocol credential bruteforcer / password sprayer / enumerator written in Rust. Binary lives in `src/main.rs`; protocol implementations are under `src/plugins/<name>/`. Release artifacts (Linux/macOS tarballs, Homebrew formula) are published from tags by `.github/workflows/release.yml`.

## Cutting a release

The user invokes the release flow by asking, e.g. "cut a 1.2.1 release" or "release the current branch as 1.2.1". Run through the steps below **in order** and do not skip pre-flight checks even if the user says "go fast" — a bad release tag is much more expensive than 30 seconds of verification.

### 0. Preconditions — verify before touching anything

Bail out and ask the user before proceeding if any of these fail:

```bash
git rev-parse --show-toplevel       # confirm we're in the legba repo root
git rev-parse --abbrev-ref HEAD     # must be 'main' unless user explicitly says otherwise
git status --porcelain              # must be empty; uncommitted changes mean an unclean release
git fetch origin && git status -sb  # local main must not be behind origin/main
```

If the working tree has unrelated modifications, **stop and surface them to the user** — do not stash or discard. The release should commit only the version bump + changelog + lockfile update.

### 1. Quality gates (must pass with zero warnings/failures)

```bash
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

If either fails, stop. Fixing clippy/test failures is a separate task — do not pile them into the release commit.

### 2. Decide the next version

```bash
grep -m1 '^version' Cargo.toml      # current version
```

Ask the user for the next version using `AskUserQuestion` if they haven't already specified it. Follow semver:
- patch (`1.2.0` → `1.2.1`): bugfixes only
- minor (`1.2.0` → `1.3.0`): new features, backwards-compatible
- major (`1.2.0` → `2.0.0`): breaking changes

Remember the chosen version and substitute it directly into the commands below. In the snippets it appears as `<VER>`.

### 3. Inspect changes since the last release

```bash
git log --no-merges --pretty=format:'%h %s' "$(git describe --tags --abbrev=0)..HEAD"
```

Read through the log and keep the commit list handy — it drives both the docs audit (next step) and the changelog (step 5).

### 4. Audit and update the docs

Before drafting the changelog, walk through the commits from step 3 and decide whether any of them require doc updates. Anything below should trigger a doc edit:

- a new plugin / module / subcommand was added → needs a new `docs/plugins/<name>.md` and a link from `docs/index.md`
- new CLI flags or options were added → `docs/usage.md` and any affected `docs/plugins/<plugin>.md`
- existing flags/options changed names, defaults, or semantics → same files as above; flag breaking changes prominently
- behavior changed in a way that's user-visible (output format, success criteria, recipe schema, REST/MCP API surface) → `docs/usage.md`, `docs/recipes.md`, `docs/rest.md`, `docs/mcp.md` as appropriate
- install/build steps changed → `docs/install.md`
- removed/deprecated functionality → strike or mark in every doc that referenced it; do not silently delete

Quick scan to find candidates:

```bash
# CLI surface changes (clap definitions)
git diff "$(git describe --tags --abbrev=0)..HEAD" -- 'src/**/options.rs' 'src/options.rs'

# New or renamed plugin modules
git diff --name-status "$(git describe --tags --abbrev=0)..HEAD" -- 'src/plugins/'

# Doc files that already exist (so you know what to update vs. create)
ls docs/ docs/plugins/
```

For each item that requires a doc change, **make the edit now** in the same release commit. Do not defer doc updates to a separate PR — releasing with stale docs is the failure mode this step exists to prevent.

If you're unsure whether a change is user-visible, surface it to the user via `AskUserQuestion` rather than guessing.

### 5. Generate the changelog

Using the same commit list from step 3, draft a changelog entry matching the format of existing entries in `CHANGELOG.md`:

```markdown
## Version <VER> (<YYYY-MM-DD>)

### 🚀 New Features
- ...

### 🐛 Fixes
- ...

### 📚 Documentation
- ...

### Miscellaneous
- ...
```

Rules:
- Only include sections that have entries; omit empty ones.
- Group commits by the conventional prefix used in this repo (`new:`, `fix:`, `docs:`, `misc:`, `release:`, etc.) — see `git log` for the established style.
- One bullet per user-visible change. Squash trivial commits (typo fixes, formatting) under Miscellaneous or drop them.
- Reference issue/PR numbers in parens when the commit mentions them, e.g. `(#86)`.
- Today's date in `YYYY-MM-DD`.

Prepend the new entry to `CHANGELOG.md` (do not replace existing content).

**Show the drafted entry to the user via AskUserQuestion before writing**, with options to accept, edit, or skip a section. Releases are public artifacts — humans should approve the prose.

### 6. Bump versions in source files

Edit these in lockstep — both must reference the new version:

- `Cargo.toml` — `version = "<VER>"`
- `pkg/brew/legba.rb` — `version '<VER>'` (note: the formula's `sha256` fields are updated **after** the GitHub release publishes binaries; leave them alone for now)

### 7. Sync `Cargo.lock`

```bash
cargo update -p legba              # rewrites the legba package entry only; no transitive bumps
cargo check --locked               # verifies the lockfile is in sync
cargo package --locked --no-verify # confirms the crate is publishable
```

If `--locked` fails, the lockfile is out of sync — re-run `cargo update -p legba` and investigate. Do not pass `--frozen` or `--offline` to paper over a mismatch.

### 8. Final sync check before commit

```bash
# All three must report the new version
grep -m1 '^version' Cargo.toml
grep -A1 '^name = "legba"' Cargo.lock | grep version
grep "version '" pkg/brew/legba.rb

git diff --stat                    # should touch only the expected files
git status                         # nothing unexpected should be modified or untracked
```

The expected diff is: `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, `pkg/brew/legba.rb`, and any `docs/**` files touched in step 4. Anything else is a red flag — stop and ask.

### 9. Commit, tag, push

Confirm with the user before pushing (this is a destructive-blast-radius operation; we don't push on autopilot).

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md pkg/brew/legba.rb docs/
git commit -m "release: <VER>"
git push origin main

git tag -a "v<VER>" -m "releasing v<VER>"
git push origin "v<VER>"
```

Pushing the tag triggers `.github/workflows/release.yml`, which builds the Linux and macOS tarballs and creates the GitHub Release.

### 10. Publish to crates.io

```bash
cargo publish --dry-run --locked
cargo publish --locked
```

Confirm with the user before the non-dry-run publish — crates.io versions are immutable.

### 11. Post-release follow-ups

After the GitHub Actions release workflow finishes and the tarballs are attached to the release:

1. Compute the new SHA256s. Inspect a prior asset URL first to confirm the actual URL pattern the workflow produces (the tag prefix `v` may or may not appear in asset names):
   ```bash
   gh release view "v<VER>" --json assets -q '.assets[].url'
   curl -sL <asset-url> | sha256sum
   ```
2. Update `pkg/brew/legba.rb` with the two `sha256` values.
3. Paste the changelog entry into the GitHub Release notes:
   ```bash
   gh release edit "v<VER>" --notes-file <(awk '/^## Version/{c++} c==1' CHANGELOG.md)
   ```
4. Commit the brew formula update on `main`:
   ```bash
   git add pkg/brew/legba.rb
   git commit -m "misc: update brew formula sha256 for <VER>"
   git push origin main
   ```

### Aborting a release

If something goes wrong **before pushing the tag**, just `git reset --hard origin/main` (after confirming nothing else is dirty) and start over.

If something goes wrong **after pushing the tag**:
- Don't delete the tag from the remote without explicit user instruction — downstream packagers may already have pulled it.
- Cut a new patch release (`1.2.1` → `1.2.2`) with the fix instead.

## Style notes for this repo

- Commit prefixes in use: `new:`, `fix:`, `docs:`, `misc:`, `release:`. Follow these — they drive changelog grouping.
- The release workflow validates that `Cargo.toml`'s `version =` matches the tag (sans `v` prefix). Tag/version mismatch fails CI.
- `pkg/brew/legba.rb` is the canonical Homebrew formula and is mirrored to the tap repo on release.

# Repository maintenance

The project maintainer is [@chai-in](https://github.com/chai-in). Code ownership
is recorded in CODEOWNERS. Contribution and security policies are in the root.
No external upstream project is responsible for this fork's support or releases.

## Contribution rules

`main` requires a pull request, up-to-date `CI` and `Repository checks` results
from GitHub Actions, resolved review conversations, linear history, and no
force pushes or deletion. The project has one maintainer, so a second approval
is not mandatory. Administrator emergency bypass is limited to pull requests.
Squash merge is the only enabled merge strategy, and merged branches are
deleted automatically. GitHub release immutability is enabled, including for
the published 0.2.0 preview: finalize all draft assets before publication.
Binary or source-asset additions require a new release version.
Version tags matching `v*` are protected from update
and deletion, with a repository administrator override for unpublished tags.
GitHub's immutable published-release locks still apply to administrators.

The active rule definitions are versioned under `.github/rulesets/`. Changes
to those files require an administrator to apply them through GitHub's ruleset
API or Settings. They do not synchronize themselves.

## Automation and security

- Workflow tokens default to read-only access. Untrusted pull requests use
  `pull_request`, not privileged `pull_request_target` execution.
- Workflow actions use full commit SHAs. The repository's Actions policy
  permits GitHub-owned actions and requires SHA pins.
- Dependency review blocks newly introduced high/critical vulnerabilities in pull requests.
- Dependabot checks GitHub Actions and the Cargo workspace weekly. Updates
  are reviewed pull requests, not automatically merged dependency changes.
- Dependency alerts, automated security-fix PRs, private vulnerability
  reporting, secret scanning, and secret push protection are enabled.
- CodeQL analyzes C/C++, Rust, Python, and GitHub Actions on relevant changes
  and weekly. These checks supplement archive/recovery tests; they do not
  certify the software as vulnerability-free.
- Docs and project metadata run repository checks. Changes to native/Rust
  code or native build infrastructure run the full cross-platform build.
  Installer changes have real Windows/Linux packaging checks.
- Cargo caches contain downloaded registry/git inputs, not large compiled
  application artifacts. Builds continue to verify the locked dependency tree.

Discussions handle usage questions; issues use structured forms. The wiki and
unused project-board feature are disabled so maintained documentation stays
in the versioned repository. Release steps are in [RELEASING.md](RELEASING.md).

Review security alerts, dependency updates, broken links, release downloads,
and supported platform requirements regularly. Do not auto-close unanswered
bugs as stale or imply a support schedule the maintainer cannot meet.

## Presentation assets

The repository social card and actual desktop screenshot are versioned under
[Hatch/assets](../Hatch/assets). The card is uploaded through repository Settings;
changing its source file does not automatically update GitHub's configured image.

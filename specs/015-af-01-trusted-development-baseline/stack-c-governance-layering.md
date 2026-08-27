# AF-01 Stack C Governance Layering

Status: REVIEW_CANDIDATE

## Problem

The AF-01 workflow trust boundary requires `.github/` changes to receive Code Owner review so an untrusted pull request cannot weaken a required-check workflow while preserving the same GitHub Actions check name and integration identity.

A single personal-repository administrator is also the only valid Code Owner currently available. Requiring that same account to approve its own pull request creates a maintenance deadlock because pull request authors cannot approve their own changes.

Adding an invented or unverified second Code Owner is not acceptable. Removing Code Owner review would reopen the workflow-self-modification risk.

## Layered ruleset design

Stack C therefore separates branch governance into two independently active repository rulesets targeting `refs/heads/main`.

### `commandF main assurance`

Source: `.github/main-ruleset.json`

This ruleset has **no bypass actors** and contains only controls that must remain unbypassable for every actor, including repository administrators:

- branch deletion blocked;
- non-fast-forward updates blocked;
- strict required status checks;
- exact integration-bound required checks `rust`, `assurance-proof`, and `scorecard`.

### `commandF main review governance`

Source: `.github/main-review-ruleset.json`

This ruleset contains only the pull-request review policy:

- merge commits only;
- one approval;
- Code Owner review for `.github/` through `.github/CODEOWNERS`;
- stale approvals dismissed on push;
- latest-push approval required;
- review threads resolved.

The sole escape hatch is:

```json
{
  "actor_id": 5,
  "actor_type": "RepositoryRole",
  "bypass_mode": "pull_request"
}
```

GitHub repository-role actor ID `5` is the repository administrator role. `pull_request` bypass mode still requires the administrator to use a pull request; it does not authorize a direct push.

Because the administrator bypass exists only in the review ruleset, it cannot bypass the separate assurance ruleset. Required checks, deletion protection, and non-fast-forward protection remain subject to a ruleset whose `bypass_actors` list is empty.

## Trust model

For an untrusted contributor changing `.github/`, the base-branch `CODEOWNERS` rule requires `@TheHalfMoon` review. The contributor cannot make its head version of `CODEOWNERS` govern that same pull request.

For a pull request authored by the repository administrator, the administrator is the explicit human governance trust root for this user-owned repository. The PR-only review bypass avoids an impossible self-approval while preserving the audit trail. The administrator still cannot bypass the independent required-check/deletion/non-fast-forward ruleset.

This design does not protect against a fully compromised repository administrator account. Repository administration itself is an out-of-band authority capable of editing repository rules, so claiming protection against total administrator compromise would be false assurance.

## GitHub semantics relied upon

Primary GitHub documentation states that:

- multiple rulesets can target the same branch and their applicable rules are aggregated;
- repository administrators can be granted ruleset bypass;
- `For pull requests only` / `bypass_mode: pull_request` requires the actor to open a pull request instead of pushing directly;
- repository role actor ID `5` represents the administrator role in the Rulesets REST model.

References:

- https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/about-rulesets#about-rule-layering
- https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/creating-rulesets-for-a-repository#granting-bypass-permissions-for-your-branch-or-tag-ruleset
- https://docs.github.com/en/rest/repos/rules

## Regression requirements

The repository tests must fail closed unless:

1. the assurance ruleset has no bypass actors;
2. the assurance ruleset contains deletion, non-fast-forward, and exact required-check rules only;
3. the review ruleset contains the pull-request rule only;
4. the review bypass is exactly repository administrator role `5` in `pull_request` mode;
5. the review ruleset keeps Code Owner review, one approval, stale-review dismissal, latest-push approval, and thread resolution;
6. no required status check, deletion rule, or non-fast-forward rule moves into the bypassable review ruleset.

## Bootstrap and live-enforcement boundary

These checked-in files remain configuration intent until T038 applies both rulesets through an authorized GitHub administrator path. T039 must read both live rulesets back and prove their target, enforcement state, rules, required-check integrations, and bypass separation. T040 must then verify the negative governance properties.

No live enforcement, T038/T039/T040 completion, Stack C merge, or `AF-01=CLOSED_CANONICAL` may be inferred from this document or from the checked-in JSON alone.

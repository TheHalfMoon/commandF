# AF-01 Security Waiver Policy

Status: `T024_POLICY`

AF-01 security findings and dependency/advisory exceptions are fail-closed by default. A waiver is a temporary, reviewable repository artifact; it is never an anonymous scanner ignore, a lowered severity threshold, or a permanent substitute for remediation.

## Required waiver fields

Every waiver MUST record all of the following:

1. **Identity** — exact scanner/finding/advisory identifier and, when applicable, exact package name/version or workflow/action path.
2. **Rationale** — why the finding cannot be remediated immediately and why accepting it is justified for commandF's actual exposure.
3. **Scope** — the smallest affected package/version, file/path, job, action, or rule identity. Family-wide or wildcard scope is prohibited unless a separate canonical plan amendment authorizes it.
4. **Compensating evidence** — exact tests, configuration, reachability evidence, containment, or other controls that reduce the accepted risk. Assertions without inspectable evidence are insufficient.
5. **Revisit/removal condition** — a concrete condition that ends the waiver, such as an upstream fixed release, dependency migration, workflow redesign, or a bounded review date plus an owner decision.

## Evidence binding

A waiver MUST also record:

- the repository task/spec authority that permits the exception;
- the exact tool and policy surface that consumes the waiver;
- the first canonical commit that introduced the waiver;
- links or identifiers for upstream advisories/issues when available;
- whether the finding is security, maintenance, license, source, duplicate, or workflow-authority related.

## Prohibited waiver forms

The following are not valid AF-01 waivers:

- an unannotated advisory ID in an ignore list;
- a broad package-family, source-family, license-family, or workflow-directory wildcard;
- lowering a global severity threshold to make an existing finding disappear;
- disabling a scanner or audit because it reports a valid finding;
- treating a network/tool/reviewer failure as PASS;
- accepting a finding solely because another scanner did not report it;
- a waiver without a removal/revisit condition.

## Review and removal

Waiver introduction, widening, renewal, or removal is a security-relevant repository change and MUST pass the same exact-head AF-01 workflow/security gates and review discipline as the policy it affects. When the removal condition is met, the waiver must be deleted rather than silently retained.

Any future machine-readable waiver representation must preserve these fields and fail closed when one is missing. T024 does not authorize any current waiver; AF-01 Stack B starts with zero advisory/security waivers.

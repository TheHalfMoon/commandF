# Security Policy

## Supported versions

commandF is currently developed from the canonical `main` branch. Until a stable release policy is published, security fixes target `main`; historical commits and experimental branches are not separately supported security lines.

## Reporting a vulnerability

Please do not publish exploit details, credentials, private data, or a working proof of concept in a public issue.

Prefer GitHub's private vulnerability-reporting flow for this repository when available:

https://github.com/TheHalfMoon/commandF/security/advisories/new

Include:

- the affected commandF component and exact commit or release identity;
- the security property that can be violated;
- minimal reproduction conditions and expected versus observed behavior;
- impact and required attacker capabilities;
- any known workaround or containment;
- whether the report contains embargo-sensitive details.

If GitHub does not offer the private reporting control for this repository, open a public issue containing only a request for a private security contact and non-sensitive routing information. Do not include exploit steps or sensitive evidence in that issue.

## Scope

Security reports are especially useful for dependency or workflow supply-chain compromise, unsafe archive or path handling, command execution, credential exposure, source-map or annotation injection, artifact/proof integrity failures, cache or lockfile trust violations, and bypasses of repository assurance gates.

commandF is not a clinical decision system. Reports about clinical interpretation should distinguish a commandF implementation defect from behavior of external HL7/FHIR tooling or source artifacts.

## Disclosure and fixes

Validated reports should receive a bounded remediation plan before public disclosure. Security exceptions must follow the checked-in AF-01 waiver policy; an advisory or scanner finding is not silently ignored and a passing aggregate score is not treated as evidence that a specific vulnerability is resolved.

# commandF Consumer Compatibility Matrix

Status: Draft design
Date: 2026-08-13

## Purpose

Standards conformance alone cannot answer whether a healthcare interoperability change is safe for the systems that actually consume it.

commandF therefore maintains an empirical compatibility matrix inspired by consumer-driven contract testing and environment-aware deployment gates.

## Core question

```text
Can this exact mapping/profile/connector/package version be certified
against every protected consumer version currently deployed or supported
in the target environment?
```

## Matrix dimensions

```text
Producer artifact/version
Consumer system/version
Environment
Contract/profile version
Mapping package version
Terminology package version
Validator/oracle versions
Verification run
Result
Certificate digest
```

## Verification states

```text
PASS
FAIL
PARTIAL
NOT_TESTED
STALE
UNKNOWN
```

## Protected consumers

A protected consumer is an explicitly registered downstream dependency whose compatibility must be evidenced before certification.

Examples:

- hospital EHR interface
- national FHIR gateway
- research OMOP pipeline
- openEHR CDR
- mobile application
- CDS service
- analytics projection
- terminology-dependent workflow
- vendor API client

## Commands

Proposed CLI:

```bash
commandf consumers list
commandf consumers register <asset>
commandf contracts verify
commandf matrix show
commandf can-i-certify --to production
commandf can-i-certify --to research
```

## `can-i-certify`

A candidate is certifiable only when the policy-required matrix edges have successful, non-stale verification evidence.

Example:

```text
Candidate: lab-mapping@4.2.0
Target environment: production

CONSUMER                     VERSION    RESULT
Hospital FHIR gateway        7.1        PASS
OMOP ETL                     12.4       PASS
Research export              3.2        PASS
Legacy CDS client            2.8        FAIL

CAN CERTIFY: NO
Reason: protected consumer Legacy CDS client@2.8 failed verification.
```

## Relationship to semantic compatibility

Consumer verification does not replace semantic verification.

A consumer may accept a response that lost important clinical meaning. Therefore certification requires both:

```text
consumer compatibility evidence
+
semantic/conformance/policy evidence
```

## Contract sources

Contracts may be derived from:

- executable consumer-driven contracts
- OpenAPI expectations
- FHIR CapabilityStatement + empirical behavior
- FHIR profiles/IGs
- recorded query/search behavior
- protected fixtures
- mapping package contracts
- explicitly authored commandF consumer contracts

## API contract diff

Use oasdiff-style rules as a donor/reference for REST/OpenAPI facade changes:

- exact change identifiers
- definite breaking vs warning vs informational
- machine-readable output
- CI failure behavior
- review/approval workflow

FHIR-specific semantic compatibility remains implemented by commandF.

## Environment model

Track which exact consumer/provider/package versions are deployed or supported in each environment:

```text
development
integration
staging
production
research
customer-specific production environments
```

The matrix must support multiple simultaneously supported consumer versions, especially mobile, partner, and externally managed systems.

## Certification Queue integration

Before release:

1. resolve current environment occupants
2. resolve all protected consumers
3. invalidate stale verification edges when relevant dependencies changed
4. execute missing verification edges
5. evaluate semantic/quality/policy gates
6. emit `can-i-certify` decision
7. sign the resulting certificate when permitted
8. record deployment/release if successful

## Provenance

Every matrix result records:

- exact input digests
- exact consumer/provider versions
- environment
- test/contract digest
- commandF version
- oracle/validator versions
- timestamp
- result digest
- certificate reference

## Inspiration

- Pact Broker Matrix / `can-i-deploy`: https://docs.pact.io/pact_broker/can_i_deploy
- Pact consumer-driven contracts: https://docs.pact.io/
- oasdiff breaking changes: https://www.oasdiff.com/docs/breaking-changes

These are design inspirations/donor candidates. commandF's healthcare semantic and certification rules are independent product contracts.

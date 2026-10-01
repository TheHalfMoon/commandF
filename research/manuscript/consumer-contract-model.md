# Consumer Contract Model

Status: RESEARCH_PLANNING. It separates implemented edges from the planned contract. It is not a coverage result. It does not authorize CF-19.

The same artifact change can matter to one consumer and not to another. `research/manuscript/problem-definition.md` states that question. This section states which consumer-facing records exist now.

```text
IMPLEMENTED = check direction, context-graph edges, terminology comparison
PLANNED = commandf.consumer-contract/v1 and its dependency families
NOT_IMPLEMENTED = a versioned consumer contract, and the families that no current command extracts
```

## What the shipped commands record

`CheckDirection` is `Both`, `Producer`, or `Consumer`. It filters findings already labeled `Producer` or `Consumer`. It does not name an application, a deployment, or a protected consumer.

`commandf context` builds one graph from one lock and one cache. It records package nodes, artifact nodes, package-dependency edges with a declared constraint, and canonical-reference edges. The reference relations are `StructureBaseDefinition`, `StructureTypeProfile`, `StructureTypeTargetProfile`, `StructureBindingValueSet`, `ValueSetIncludeSystem`, `ValueSetIncludeValueSet`, `ValueSetExcludeSystem`, `ValueSetExcludeValueSet`, and `CodeSystemSupplements`. Each edge carries a resolution of `Resolved`, `External`, or `Ambiguous`. The extractor schema names `CodeSystem`, `StructureDefinition`, and `ValueSet` as supported sources. Other resource types present in the package are listed under `unsupported_source_resource_types`. They are not rewritten as resolved edges. This graph is not passed into `commandf check`.

`commandf terminology` builds `cf07-terminology-v1` from a before state, an after state, a structural report, and a compatibility report. It compares complete CodeSystems and ValueSet expansions. That is a terminology diff. It is not a consumer contract, and it is not a binding to a named consumer.

## What the contract is planned to be

`docs/COMMAND_F_V3_1_DECISION_ASSURANCE_PLAN.md` section 8 assigns CF-19 the schema `commandf.consumer-contract/v1`. The planned identity includes a contract id and version, a producer package and version scope, a consumer application or service, a deployment or environment label, source-evidence identity, an extraction method, and an explicit protected or unprotected status. The planned dependency families include FHIR packages, profiles, and extensions; canonical resources; element and path dependencies; terminology systems, versions, and bindings; SearchParameters and search interactions; FHIRPath; CQL, ELM, and Library dependencies; SQL-on-FHIR `ViewDefinition` dependencies; CapabilityStatement expectations; REST interactions and operations; SMART scopes and backend-service expectations; Bulk Data expectations; subscriptions; selected TestScript or Inferno references; and declared migration constraints.

Popularity, registry presence, and graph centrality do not stand in for an observed or declared contract. Missing contract information stays missing.

## What is not built

No file in the repository is `commandf.consumer-contract/v1`. SearchParameter dependencies, FHIRPath dependencies, CQL or ELM dependencies, SQL-on-FHIR `ViewDefinition` dependencies, REST interaction dependencies, and authorization or protocol expectations are not extracted by the current context graph or by the current check. Profile and extension edges exist only as the canonical relations listed above, on one package graph, not as a protected-consumer contract. Package dependency exists as a declared constraint on that same graph, not as a consumer-application dependency set.

## What this section does not claim

It does not claim a consumer was protected, or that an artifact-level change was measured as harmless. It does not run CF-19.

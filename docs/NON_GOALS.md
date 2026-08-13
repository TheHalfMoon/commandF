# commandF Non-Goals

These constraints keep the first product focused.

For the first product horizon, commandF will not implement a FHIR server, terminology server, patient-matching engine, new authoring language, new executable mapping language, new universal query language, competing FHIR package registry, hospital integration runtime, or universal semantic IR required by V1.

commandF will use established validators and package infrastructure instead of replacing them. R6 production guarantees are also outside the first horizon while the specification is still changing.

The core review path should operate on conformance artifacts without requiring patient data. Any later instance-data profiler must have a separate on-premises trust boundary.

## Scope rule

No new crate is added unless a shipped command or immediate executable test uses it.

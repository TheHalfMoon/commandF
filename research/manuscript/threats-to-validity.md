# Threats to Validity

Status: RESEARCH_PLANNING. These are threats to a study that has not been run. Residual empirical threats stay `RESULT_PENDING`.

`research/THREATS_TO_VALIDITY.md` already records five threats that stand before measurement: ambiguous ground truth stays ambiguous, tuning on the final held-out set would leak evaluation, a reconstructed artifact that misses a recorded digest is not historical recovery, missing evidence labeled as verification is a construct-validity failure, and an external tool compared without a pinned version is an unfair baseline. This section adds the threats that follow from the current records. It does not retire those five.

## Construct

A structural finding is not a consumer break, and a check pass is not `PROVEN_COMPATIBLE`. Using `CheckDecision.passed` as the unsafe-auto-allow indicator would measure a different question from the one sap-1 names. The mapping to `ALLOW`, `DENY`, and `ABSTAIN` is not bound. Healthcare interoperability has package, profile, terminology, search, and protocol surfaces. The shipped commands cover a subset of those surfaces. A study that treated the subset as the whole problem would overclaim the construct.

## Corpus and labels

The candidate corpus has three members and one source family, `hapifhir/hapi-fhir`. `research/CASE_CLASS_MATRIX.md` leaves the other classes blocked or low. Three items from one family do not represent the protocol classes the benchmark outline names. The three labels are external. No adjudication has been run. A later item that depended on adjudication would inherit the disagreement rule, and that rule has not been exercised.

Rights for `FHIR/fhir-test-cases` are unresolved. Protocol and authorization candidates in the matrix are unresolved on rights. An item admitted without those rights would not be a valid member. This section does not resolve them.

## Oracles, baselines, and maturity

`commandf oracle` needs a caller-supplied adapter. The validator coordinate is not a jar digest, and the Java runtime is not pinned. B1 and B5 are `NOT_PINNED`. B3, B4, B6, and B7 are `NOT_IMPLEMENTED`. No baseline has been executed. A comparison that filled those gaps with a different tool would be a different study.

The decision envelope, the consumer contract, automatic escalation, and the acquisition equality `bytes_consumed == bytes_digest_verified` are specified or planned and not implemented. A result that assumed they ran would describe a system this repository does not build today.

## Leakage and mitigation status

`sp-1` refuses assignment while the corpus has one leakage group, so no held-out manifest exists to leak into. That refusal is a recorded control. It is not a completed split. A later assignment that ignored the group rule would be leakage. This text does not apply the role rule to close that threat.

None of these mitigations is a completed empirical control. They are records that block a premature measurement. They do not estimate the remaining bias.

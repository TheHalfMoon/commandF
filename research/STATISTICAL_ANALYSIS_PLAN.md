# Statistical Analysis Plan

Status: RESEARCH_PLANNING. Written before headline results. Version sap-1 specifies the calculations. It does not run them.

No interval, p-value, rate, or score exists.

```text
STATISTICAL_PLAN_VERSION = sap-1
STATISTICAL_PLAN = SPECIFIED
PRIMARY_RATE = RESULT_PENDING
CONFIDENCE_LEVEL = 0.95
BOOTSTRAP_REPLICATES = 10000
BOOTSTRAP_UNIT = leakage group from research/SPLIT_POLICY.md version sp-1
RESULT = RESULT_PENDING
```

A decision or an external label is bound when a later record names the item digest, the decision class or the label statement, and the record that produced it. No such binding exists. The output mapping from a commandF document to `ALLOW`, `DENY`, and `ABSTAIN` is not bound. `research/BASELINE_IDENTITIES.md` says a `check` process status is not a benchmark label. The external labels in the candidate corpus are not yet mapped to acceptable or not acceptable. Until those bindings exist, an applicable formula stays `RESULT_PENDING`. A formula whose inputs cannot exist stays `NOT_APPLICABLE`. A zero denominator stays `UNDEFINED`. Those three states are not interchangeable.

## Primary endpoint

One confirmatory endpoint: the unsafe auto-allow rate, reported together with automation coverage.

On the held-out manifest, an auto-allow is a bound `ALLOW`. It is unsafe when the bound external label says that decision is not acceptable. The rate is the unsafe auto-allow count divided by the auto-allow count. There is no held-out manifest, so the rate stays `RESULT_PENDING`.

Automation coverage is the count of held-out items with a bound `ALLOW` or `DENY`, divided by the count of items in the held-out manifest. A missing execution and an `ABSTAIN` both sit in that denominator and outside the numerator, so both lower coverage. Coverage is reported beside the primary rate. It is not a second confirmatory test.

## Secondary endpoints

These are descriptive. They are not a second confirmatory family. The only predeclared paired comparison is below, and it is also descriptive. No multiplicity adjustment is applied. A later confirmatory family, if one is added, must be named in a new plan version while results are still `RESULT_PENDING`.

- Automation coverage, defined above.
- Brier score and expected calibration error. When a probability in the closed unit interval is emitted, that probability is the system's forecast that the external label says the item is not acceptable. The binary event is that label outcome. B7 is `NOT_IMPLEMENTED` and emits no probability, so both scores are `NOT_APPLICABLE` until such a forecast exists. The error uses ten equal-width bins on `[0, 1]`. An empty bin is omitted from the weighted sum and the omission count is reported. The estimator is the frequency of the event in the bin, not a fitted recalibration.
- A risk-coverage curve at the fixed coverage points `0.50`, `0.80`, `0.90`, and `0.95`. A point the system does not reach is `NOT_REACHED`. The point is not moved after seeing which point looks safer.

## Intervals

The interval level is `0.95`. It was not chosen from data.

The bootstrap resamples leakage groups with replacement, `10000` times. The statistic is recomputed on each resample. The reported interval is the `0.025` and `0.975` percentiles of those statistics. The execution record must name the generator and the seed. This plan does not choose a seed.

If the leakage-group count is below two, the interval is `NOT_APPLICABLE`. Version sp-1 currently finds one group, so this condition already holds and no interval is produced. When the group count is at least two and the freeze has happened, an interval that has not yet been calculated stays `RESULT_PENDING`.

## Paired comparison

One descriptive comparison is predeclared: B0 against B2, and only after both have been executed on the same held-out items. No other pair is part of this version.

The paired quantity is not the primary rate. The primary rate is conditional on an auto-allow. The paired quantity is the unconditional indicator that the item's decision was an unsafe auto-allow. McNemar's exact two-sided binomial test is applied to discordant item pairs of that indicator. The pair key is the item digest. The null is equal marginal probability of the indicator. A p-value from that test is descriptive. If the discordant-pair count is zero, the test is `UNDEFINED`. It is not given a p-value. B0 and B2 are both `NOT_EXECUTED`, so the test stays `RESULT_PENDING`.

## Zero denominators

A zero auto-allow count makes the primary rate `UNDEFINED`, not zero. A zero held-out manifest makes coverage `UNDEFINED`, not zero. A zero discordant-pair count makes the paired test `UNDEFINED`. None of these is filled with a favorable number.

## Missing runs and abstention

`ABSTAIN` is a system decision. It is not an allow and not a deny. A missing execution is not that decision. Both are inside the held-out coverage denominator and outside the auto-allow count, and the missing count is reported separately from the abstention count. An item is not removed from the manifest because its result would be unfavorable.

An adjudication state of `DISAGREEMENT` is not converted into one acceptable bit. The item stays out of the primary rate until a rule written before results says otherwise. Adjudication runs are still zero.

## Negative results and stopping

A higher unsafe rate, a null paired difference, and a failed hypothesis are kept. They are not deleted to improve a later summary.

No headline number is computed before the protocol freeze binds this plan version, the corpus digest, the held-out manifest, and the baseline identities that will actually run. An interval does not authorize adding an item, removing an item, changing version sp-1, or changing version sap-1.

## What this record does not do

It does not compute a rate. It does not assign a split. It does not freeze the protocol. It does not run a baseline. It does not authorize CF-17, AF-02, AF-03, AF-04, or CF-18.

# Statistical Analysis Plan

Status: RESEARCH_PLANNING. Written before headline results. Version sap-1 specifies the calculations. It does not run them.

No interval, p-value, rate, or score exists. Every numeric outcome below stays `NOT_COMPUTED`.

```text
STATISTICAL_PLAN_VERSION = sap-1
STATISTICAL_PLAN = SPECIFIED
PRIMARY_RATE = NOT_COMPUTED
CONFIDENCE_LEVEL = 0.95
BOOTSTRAP_REPLICATES = 10000
BOOTSTRAP_UNIT = leakage group from research/SPLIT_POLICY.md version sp-1
RESULT = RESULT_PENDING
```

The output mapping from a commandF document to the decision classes `ALLOW`, `DENY`, and `ABSTAIN` is not bound. `research/BASELINE_IDENTITIES.md` says a `check` process status is not a benchmark label. Until a later record binds that mapping, a formula that needs it stays `NOT_COMPUTED`. The external labels in the candidate corpus are not yet mapped to acceptable or not acceptable. That mapping is also unbound, so the primary rate stays `NOT_COMPUTED`.

## Primary endpoint

One confirmatory endpoint: the unsafe auto-allow rate, reported together with automation coverage.

An auto-allow is a bound `ALLOW` decision. It is unsafe when the bound external label says that decision is not acceptable. The rate is the unsafe auto-allow count divided by the auto-allow count, on the held-out manifest only. There is no held-out manifest, so the rate is not calculated.

Automation coverage is the count of items with a bound `ALLOW` or `DENY`, divided by the count of items that have a bound `ALLOW`, `DENY`, or `ABSTAIN`. Coverage is reported beside the primary rate. It is not a second confirmatory test.

## Secondary endpoints

These are descriptive. They are not a second confirmatory family, and no multiplicity adjustment is applied to them. A later confirmatory family, if one is added, must be named in a new plan version while results are still `RESULT_PENDING`.

- Automation coverage at the operating point the system actually used.
- Brier score, only when the system emits a probability in the closed unit interval. B7 is `NOT_IMPLEMENTED`, so this score is `NOT_COMPUTED`.
- Expected calibration error for that probability, using ten equal-width bins on `[0, 1]`. An empty bin is omitted from the weighted sum and the omission count is reported. The estimator is the frequency of the event in the bin, not a fitted recalibration. It is `NOT_COMPUTED`.
- A risk-coverage curve at the fixed coverage points `0.50`, `0.80`, `0.90`, and `0.95`. A point the system does not reach is `NOT_REACHED`. The point is not moved after seeing which point looks safer.

## Intervals

The interval level is `0.95`. It was not chosen from data.

The bootstrap resamples leakage groups with replacement, `10000` times. The statistic is recomputed on each resample. The reported interval is the `0.025` and `0.975` percentiles of those statistics. The execution record must name the generator and the seed. This plan does not choose a seed.

If the leakage-group count is below two, the interval is `NOT_COMPUTED`. Version sp-1 currently finds one group, so this condition already holds and no interval is produced.

## Paired comparisons

A paired comparison uses McNemar's exact two-sided binomial test on discordant item pairs. The pair key is the item digest. Both families must have been executed on that item. The binary outcome is whether the decision was an unsafe auto-allow. The null is equal marginal probability of that outcome. Families marked `NOT_IMPLEMENTED`, `NOT_PINNED`, or `NOT_EXECUTED` are not entered. If the discordant-pair count is zero, the test is `UNDEFINED`. It is not given a p-value.

## Zero denominators

A zero auto-allow count makes the primary rate `UNDEFINED`, not zero. A zero count of allow, deny, and abstain makes coverage `UNDEFINED`, not zero. A zero discordant-pair count makes the paired test `UNDEFINED`. None of these is filled with a favorable number.

## Missing runs and abstention

`ABSTAIN` is not an allow and not a deny. It lowers coverage. It is outside both the numerator and the denominator of the unsafe auto-allow rate.

A missing execution is not an abstention. The item is excluded from both rates, and the exclusion count is reported. An item is not dropped because its result would be unfavorable.

An adjudication state of `DISAGREEMENT` is not converted into one acceptable bit. The item stays out of the primary rate until a rule written before results says otherwise. Adjudication runs are still zero.

## Negative results and stopping

A higher unsafe rate, a null paired difference, and a failed hypothesis are kept. They are not deleted to improve a later summary.

No headline number is computed before the protocol freeze binds this plan version, the corpus digest, the held-out manifest, and the baseline identities that will actually run. An interval does not authorize adding an item, removing an item, changing version sp-1, or changing version sap-1.

## What this record does not do

It does not compute a rate. It does not assign a split. It does not freeze the protocol. It does not run a baseline. It does not authorize CF-17, AF-02, AF-03, AF-04, or CF-18.

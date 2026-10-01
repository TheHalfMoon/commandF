# Split Policy

Status: RESEARCH_PLANNING. Version sp-1 is specified. It does not assign an item. It does not bind a held-out manifest. It does not freeze the benchmark protocol. It does not authorize an experiment.

The candidate corpus digest exists, so `research/BENCHMARK_ADMISSION_LIFECYCLE.md` permits this function. The same digest has one source family, so the function refuses to assign. Observed against main `8293f944a8223642f5ae4f2bf4dba1837622d75b`.

```text
SPLIT_POLICY_VERSION = sp-1
SPLIT_POLICY = SPECIFIED
SPLIT_ASSIGNMENT = NOT_ASSIGNED
HELD_OUT_MANIFEST = NOT_BOUND
PROTOCOL_FREEZE = NOT_FROZEN
ITEM_COUNT = 3
CORPUS_DIGEST = e92d734ab2981691322074f9d963bcf6ffc9058dace3875fdea64d2558b417aa
LEAKAGE_GROUP_COUNT = 1
REFUSAL = LEAKAGE_GROUP_COUNT_BELOW_MINIMUM
```

## What the function may read

An item digest is the SHA-256 of the UTF-8 canonical item line defined in `research/CANDIDATE_CORPUS.md`. The function reads only:

- this policy version, the literal `sp-1`;
- those item lines;
- `source_repository`;
- `input_sha256`;
- `pair_id` when the line contains it;
- `variant_family` when the line contains it;
- `near_duplicate_group` when the line contains it.

It does not read commandF output, a score, a coverage value, a pass or fail bit, or an evaluator log. An item is not deleted because a score is bad. An item is not moved from a held-out role because a score is bad. No held-out manifest exists, so no role can be moved.

A field that is absent does not create an edge. The three current lines do not contain `pair_id`, `variant_family`, or `near_duplicate_group`.

## Leakage groups

Two items are in the same group when any of these is true:

- `source_repository` is the same string;
- `input_sha256` is the same string;
- both lines contain the same non-empty `pair_id`;
- both lines contain the same non-empty `variant_family`;
- both lines contain the same non-empty `near_duplicate_group`.

The group is the connected component of those edges. Items from one source repository stay together, including different commits of that repository. A before/after pair that shares one `pair_id` stays together even when the repositories differ. A related variant that shares one `variant_family` stays together. A near-duplicate that shares one `near_duplicate_group`, or that repeats an `input_sha256`, stays together. No component is split across a development role and a held-out role.

The group id is the lexicographically smallest item digest in the component, compared as lowercase hexadecimal.

## When assignment is refused

Assignment is not computed unless every one of these is true:

- the named corpus digest is the digest of the lines being assigned;
- there are at least two leakage groups;
- headline results for that digest are still `RESULT_PENDING`;
- after the role rule below, both roles are non-empty.

If any condition fails, the result is `SPLIT_ASSIGNMENT = NOT_ASSIGNED` and `HELD_OUT_MANIFEST = NOT_BOUND`. The modulus is not changed to force a role. Items are not dropped to force a role.

The current three lines all have `source_repository` `hapifhir/hapi-fhir`. Their item digests are:

```text
648e748c3dd5d3f71724bb543a99eee934ccf241fa45eddda3a24840cfdd7143
955211e7e09cf3749d184513976c235f37499967e965fc0f614b7a5278deef0f
bd4b8037f3bd1636f9eaa8f3cbaf5baecaa54faa24e0058efd9fba17b725482c
```

Those three digests are one group. The group id is `648e748c3dd5d3f71724bb543a99eee934ccf241fa45eddda3a24840cfdd7143`. Two groups are required. The role rule is not applied to this digest.

## Role rule, not applied here

When the conditions above hold, each group is assigned once. The input is the UTF-8 bytes of the literal `sp-1`, one byte `0x0A`, and the lowercase hexadecimal group id, with no byte after the group id. Let `n` be the integer formed by the first eight bytes of the SHA-256 of that input, big-endian. The role is `held_out` when `n` modulo 5 is 0. Otherwise the role is `development`.

Every item in a group receives that group's role. One item receives one role. The same digest and the same policy version produce the same roles. The modulus 5 is a constant of version sp-1. It was not fit to a score and it was not fit to this three-item digest.

A later version of the function, if one is required, must be written while results for the affected digest are `RESULT_PENDING` and before a held-out manifest for that digest is bound. A bound manifest is not recomputed.

The held-out manifest is a later bind. It would name the policy version, the corpus digest, each item digest, and the role. This file does not bind it. `research/PROTOCOL_FREEZE.md` remains `NOT_FROZEN`.

## Current refusal

```text
SOURCE_FAMILY = hapifhir/hapi-fhir
LEAKAGE_GROUP_COUNT = 1
MINIMUM_LEAKAGE_GROUPS = 2
ROLE_RULE = NOT_APPLIED
SPLIT_ASSIGNMENT = NOT_ASSIGNED
HELD_OUT_MANIFEST = NOT_BOUND
```

## What this record does not do

It does not add a corpus member. It does not change `CORPUS_DIGEST`. It does not freeze the benchmark protocol. It does not pin a baseline. It does not compute a statistic. It does not authorize CF-17, AF-02, AF-03, AF-04, or CF-18. It does not reopen a rejected source.

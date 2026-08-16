#!/usr/bin/env python3
from __future__ import annotations

import argparse
import importlib.util
import json
from pathlib import Path
import sys


QUALIFY_PATH = Path(__file__).parents[1] / "cf10-oracle-qualification" / "qualify_cf10_oracle.py"
SPEC = importlib.util.spec_from_file_location("cf10_qualify", QUALIFY_PATH)
if SPEC is None or SPEC.loader is None:
    raise SystemExit(f"unable to load {QUALIFY_PATH}")
Q = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = Q
SPEC.loader.exec_module(Q)

CANONICALS = {
    "C001": "http://hl7.org/fhir/us/core/StructureDefinition/us-core-observation-lab",
    "C002": "http://hl7.org/fhir/uv/ips/StructureDefinition/Composition-uv-ips",
}


def build_pair(case_id, before, after):
    canonical = CANONICALS[case_id]
    before_profile = Q.find_profile(before, canonical)
    after_profile = Q.find_profile(after, canonical)
    pair = Q.MatchedProfilePair(
        resource_key=canonical,
        canonical_url=canonical,
        lookup_version=None,
        before=before_profile,
        after=after_profile,
    )
    return pair, before_profile, after_profile


def context_for(mode, state):
    if mode == "DIRECT":
        return Q.direct_context_packages(state)
    if mode == "FULL_CLOSURE":
        return Q.full_context_packages(state)
    raise AssertionError(mode)


def one_probe(*, java, probe_classes, oracle_jar, work_root, pair, label, mode,
              left, right, left_profile, right_profile):
    left_context = context_for(mode, left)
    right_context = context_for(mode, right)
    invocation = Q.invocation_evidence(
        label,
        mode,
        pair,
        left,
        right,
        left_profile,
        right_profile,
        left_context,
        right_context,
    )
    argv = Q.probe_argv(
        java,
        probe_classes,
        oracle_jar,
        pair,
        left,
        right,
        left_profile,
        right_profile,
        left_context,
        right_context,
    )
    return Q.run_probe(work_root, invocation, argv)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--variant", required=True, choices=("inherited", "content-reference", "combined"))
    parser.add_argument("--java", required=True, type=Path)
    parser.add_argument("--probe-classes", required=True, type=Path)
    parser.add_argument("--oracle-jar", required=True, type=Path)
    parser.add_argument("--work-root", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()

    states = {
        (spec.case_id, spec.side): Q.verify_state(args.work_root, spec)
        for spec in Q.STATE_SPECS
    }

    probes = []
    for case_id in ("C001", "C002"):
        before = states[(case_id, "before")]
        after = states[(case_id, "after")]
        pair, before_profile, after_profile = build_pair(case_id, before, after)
        for mode in ("DIRECT", "FULL_CLOSURE"):
            probes.append(one_probe(
                java=args.java,
                probe_classes=args.probe_classes,
                oracle_jar=args.oracle_jar,
                work_root=args.work_root,
                pair=pair,
                label="self_before",
                mode=mode,
                left=before,
                right=before,
                left_profile=before_profile,
                right_profile=before_profile,
            ))
            probes.append(one_probe(
                java=args.java,
                probe_classes=args.probe_classes,
                oracle_jar=args.oracle_jar,
                work_root=args.work_root,
                pair=pair,
                label="self_after",
                mode=mode,
                left=after,
                right=after,
                left_profile=after_profile,
                right_profile=after_profile,
            ))
            probes.append(one_probe(
                java=args.java,
                probe_classes=args.probe_classes,
                oracle_jar=args.oracle_jar,
                work_root=args.work_root,
                pair=pair,
                label="cross",
                mode=mode,
                left=before,
                right=after,
                left_profile=before_profile,
                right_profile=after_profile,
            ))

    required_complete = {
        "inherited": {"C001"},
        "content-reference": {"C002"},
        "combined": {"C001", "C002"},
    }[args.variant]

    statuses = {
        case_id: [probe["result"]["status"] for probe in probes if probe["case_id"] == case_id]
        for case_id in ("C001", "C002")
    }
    for case_id in required_complete:
        if statuses[case_id] != ["completed"] * 6:
            raise SystemExit(
                f"{args.variant}: {case_id} did not complete all six probes: {statuses[case_id]}"
            )

    report = {
        "schema": 1,
        "candidate_variant": args.variant,
        "upstream": {
            "project": "hapifhir/org.hl7.fhir.core",
            "release": "6.10.2",
            "source_commit": "d06577dbc5c62c74a2a8823fbc4830a3024d5b0b",
        },
        "required_complete": sorted(required_complete),
        "case_statuses": statuses,
        "probes": probes,
    }
    args.output.write_bytes(Q.canonical_json_bytes(report))
    print(json.dumps({"variant": args.variant, "case_statuses": statuses}, sort_keys=True))


if __name__ == "__main__":
    main()

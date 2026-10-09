# CF11 registry identity probe (investigation only)

This Python-stdlib and curl (platform TLS), read-only helper compares the exact same package
version against the two official FHIR registry archive endpoints configured in
the CommandF V2 resolver. It is evidence gathering, **not** a release or AF-02
qualification oracle, and never replaces the cf11-multi-version-proof assertion.

## Example

    python3 scripts/cf11_registry_identity_probe.py \
      --lock /path/to/commandf.lock \
      --name hl7.fhir.uv.ips --version 2.0.1 > /tmp/cf11-observation.json

Use --all instead of --name and --version to compare every package in the lock.
This can download up to 128 MiB per endpoint per package. The probe downloads each bounded archive to a private temporary file and deletes it immediately after hashing; it never extracts or executes it.

## Classification

| State | Exit | Interpretation |
|---|---:|---|
| SOURCE_BYTES_IDENTICAL | 0 | Both endpoint hashes match each other **and** the supplied lock digest |
| SOURCE_IDENTITY_CONFLICT | 2 | Both endpoints are reachable, but raw archive hashes differ |
| LOCK_DIGEST_DISAGREEMENT | 2 | Any available archive hash disagrees with the supplied lock |
| INCOMPLETE_SOURCE_OBSERVATION | 3 | At least one endpoint unavailable, with no observed digest disagreement |
| INVALID_INPUT | 4 | Invalid or unreadable lock or exact package selector |

The lockfile is only an expected input; it is **not independent trust evidence**.
Same hashes do not prove the source's legal provenance, manifest integrity or
FHIR conformance. A future historical CI discrepancy still requires capture
of the actual failed attempts' source/lock identities.

Properties: direct fixed HTTPS hosts, curl platform TLS verification, disabled user curlrc
and environment proxy use, no redirects or authentication,
no retries, per-request 30-second timeout, 128-MiB streaming limit,
16-MiB lockfile read limit, and no secrets or local paths in JSON output.
Non-200, malformed gzip, oversized content and transport errors never
become success. Response bytes themselves are not retained.

## Offline tests

    PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover \
      -s scripts/tests -p 'test_cf11_registry_identity_probe.py' -v

This candidate does not alter V2 resolver, protected AF-02 files or
GitHub workflow enforcement. Review, signed DCO, explicit governance approvals
where applicable and exact-head CI are separately required before adoption.

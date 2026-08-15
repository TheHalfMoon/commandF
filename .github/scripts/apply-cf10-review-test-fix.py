from pathlib import Path

path = Path('crates/commandf-pkg/tests/corpus_evaluator.rs')
text = path.read_text()

old = '''    let summary = summarize_corpus_case(&fixture.case, &reports, &oracle).unwrap();'''
new = '''    let summary = summarize_corpus_case(
        &fixture.case,
        &reports,
        &oracle,
        &fixture.before_lock,
        &fixture.after_lock,
    )
    .unwrap();'''
if text.count(old) != 1:
    raise SystemExit(f'expected one complete summary call, found {text.count(old)}')
text = text.replace(old, new, 1)

old = '''    assert_eq!(summary.oracle.as_ref().unwrap().compared, 0);
    assert_eq!(summary.oracle.as_ref().unwrap().uncomparable, 1);
}'''
new = '''    assert_eq!(summary.oracle.as_ref().unwrap().compared, 0);
    assert_eq!(summary.oracle.as_ref().unwrap().uncomparable, 1);
    let before_closure = summary.before.closure.as_ref().expect("before closure evidence");
    assert_eq!(before_closure.len(), fixture.before_lock.packages.len());
    assert_eq!(before_closure[0].name, "example.pkg");
    assert_eq!(
        before_closure[0].sha256,
        fixture.case.before.archive_sha256
    );
    assert_eq!(summary.before.closure_sha256.as_deref().map(str::len), Some(64));

    let mut transport_changed = fixture.before_lock.clone();
    transport_changed.packages[0].source = "https://fallback.example.org/example.pkg/1.0.0".to_owned();
    let transport_summary = summarize_corpus_case(
        &fixture.case,
        &reports,
        &oracle,
        &transport_changed,
        &fixture.after_lock,
    )
    .unwrap();
    assert_eq!(
        transport_summary.before.closure_sha256,
        summary.before.closure_sha256
    );

    let mut dependency_digest_changed = fixture.before_lock.clone();
    dependency_digest_changed.packages.push(locked(
        "example.dep",
        "1.0.0",
        &"c".repeat(64),
    ));
    dependency_digest_changed.packages.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.version.cmp(&right.version))
    });
    let changed_summary = summarize_corpus_case(
        &fixture.case,
        &reports,
        &oracle,
        &dependency_digest_changed,
        &fixture.after_lock,
    )
    .unwrap();
    assert_ne!(changed_summary.before.closure_sha256, summary.before.closure_sha256);
}'''
if text.count(old) != 1:
    raise SystemExit('summary closure assertion marker mismatch')
text = text.replace(old, new, 1)

old = '''        summarize_corpus_case(&fixture.case, &reports, &oracle),'''
new = '''        summarize_corpus_case(
            &fixture.case,
            &reports,
            &oracle,
            &fixture.before_lock,
            &fixture.after_lock,
        ),'''
if text.count(old) != 1:
    raise SystemExit(f'expected one rejection summary call, found {text.count(old)}')
text = text.replace(old, new, 1)

old = '''    assert!(failed.oracle.is_none());

    let report = CorpusRunSummary {'''
new = '''    assert!(failed.oracle.is_none());
    assert!(failed.before.closure.is_none());
    assert!(failed.after.closure.is_none());

    let report = CorpusRunSummary {'''
if text.count(old) != 1:
    raise SystemExit('failed closure assertion marker mismatch')
text = text.replace(old, new, 1)

path.write_text(text)
print('CF-10 reviewer regression tests staged')

from pathlib import Path

path = Path("crates/commandf-pkg/src/terminology_index.rs")
text = path.read_text()

old = '''        let target_core = root_core_family(root)?;
        Self::load_scoped(lockfile, cache, Some(target_core))
'''
new = '''        let target_core = root_core_family(root)?;
        Self::load_scoped(lockfile, cache, target_core)
'''
if text.count(old) != 1:
    raise SystemExit(f"load_for_root fallback: expected 1 match, found {text.count(old)}")
text = text.replace(old, new, 1)

old = '''fn root_core_family(root: &LockedPackage) -> Result<&str, TerminologyError> {
    let cores = root
        .dependencies
        .keys()
        .filter(|name| is_fhir_core_package(name))
        .map(String::as_str)
        .collect::<Vec<_>>();
    match cores.as_slice() {
        [single] => Ok(*single),
        [] => Err(lock_graph_error(
            root,
            "root package does not declare a FHIR core dependency",
        )),
        _ => Err(lock_graph_error(
            root,
            "root package declares more than one FHIR core dependency",
        )),
    }
}
'''
new = '''fn root_core_family(root: &LockedPackage) -> Result<Option<&str>, TerminologyError> {
    let cores = root
        .dependencies
        .keys()
        .filter(|name| is_fhir_core_package(name))
        .map(String::as_str)
        .collect::<Vec<_>>();
    match cores.as_slice() {
        [single] => Ok(Some(*single)),
        [] => Ok(None),
        _ => Err(lock_graph_error(
            root,
            "root package declares more than one FHIR core dependency",
        )),
    }
}
'''
if text.count(old) != 1:
    raise SystemExit(f"root_core_family fallback: expected 1 match, found {text.count(old)}")
text = text.replace(old, new, 1)

old = '''        assert_eq!(root_core_family(&root).unwrap(), "hl7.fhir.r4.core");
'''
new = '''        assert_eq!(
            root_core_family(&root).unwrap(),
            Some("hl7.fhir.r4.core")
        );
        let synthetic = locked("example.synthetic", "1.0.0", &[]);
        assert_eq!(root_core_family(&synthetic).unwrap(), None);
'''
if text.count(old) != 1:
    raise SystemExit(f"root_core_family test: expected 1 match, found {text.count(old)}")
text = text.replace(old, new, 1)

path.write_text(text)
print("no-core fallback preserved")

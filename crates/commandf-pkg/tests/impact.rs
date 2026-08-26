use commandf_pkg::{
    build_impact_report, CanonicalReferenceRelation, CanonicalResolutionStatus,
    ContextArtifactIdentity, ContextArtifactNode, ContextCanonicalReferenceEdge, ContextCoverage,
    ContextGraphReport, ContextPackageDependencyEdge, ContextPackageIdentity, ContextPackageNode,
    ImpactSeedKind, ImpactSide, Lockfile, PackageEvidence, ResourceKey, ResourceKeyKind,
    StructuralChange, StructuralChangeKind, StructuralDiffReport,
};

#[test]
fn reports_direct_transitive_cycle_and_unresolved_artifact_exposure() {
    let before_subject = package("acme.changed", "1.0.0", "subject-before");
    let after_subject = package("acme.changed", "2.0.0", "subject-after");
    let dependent = package("acme.dep", "1.0.0", "dep");

    let before_seed = artifact(&before_subject, "seed.json", "seed-before", Some("https://example.org/B"));
    let after_seed = artifact(&after_subject, "seed.json", "seed-after", Some("https://example.org/B"));
    let a = artifact(&dependent, "a.json", "a", Some("https://example.org/A"));
    let x = artifact(&dependent, "x.json", "x", Some("https://example.org/X"));

    let before = graph(
        vec![before_subject.clone(), dependent.clone()],
        vec![before_seed.clone(), a.clone(), x.clone()],
        vec![dependency(&dependent, &before_subject, "1.0.0")],
        vec![
            resolved(&a, &before_seed, "https://example.org/B"),
            resolved(&x, &a, "https://example.org/A"),
            resolved(&a, &x, "https://example.org/X"),
            external(&a, "https://external.example/ValueSet/missing"),
            ambiguous(&x, "https://example.org/ambiguous", vec![a.identity.clone(), before_seed.identity.clone()]),
        ],
    );
    let after = graph(
        vec![after_subject.clone(), dependent.clone()],
        vec![after_seed.clone(), a.clone(), x.clone()],
        vec![dependency(&dependent, &after_subject, "2.0.0")],
        vec![
            resolved(&a, &after_seed, "https://example.org/B"),
            resolved(&x, &a, "https://example.org/A"),
            resolved(&a, &x, "https://example.org/X"),
            external(&a, "https://external.example/ValueSet/missing"),
            ambiguous(&x, "https://example.org/ambiguous", vec![a.identity.clone(), after_seed.identity.clone()]),
        ],
    );
    let diff = modified_diff(&before_subject, &after_subject, "https://example.org/B", "seed.json");

    let report = build_impact_report(&diff, &before, &after).unwrap();

    assert_eq!(report.seeds.len(), 1);
    assert_eq!(report.seeds[0].kind, ImpactSeedKind::Modified);
    assert_eq!(report.artifact_impacts.len(), 4);
    assert!(report.artifact_impacts.iter().any(|impact| {
        impact.impacted == a.identity && impact.side == ImpactSide::Before && impact.path.len() == 1
    }));
    assert!(report.artifact_impacts.iter().any(|impact| {
        impact.impacted == x.identity && impact.side == ImpactSide::After && impact.path.len() == 2
    }));
    assert_eq!(report.package_impacts.len(), 2);
    assert_eq!(report.unresolved_boundaries.len(), 4);
    assert!(report
        .unresolved_boundaries
        .iter()
        .all(|boundary| boundary.resolution != CanonicalResolutionStatus::Resolved));

    let first = report.to_json_bytes().unwrap();
    let second = build_impact_report(&diff, &before, &after)
        .unwrap()
        .to_json_bytes()
        .unwrap();
    assert_eq!(first, second);
}

#[test]
fn chooses_lexicographically_first_equal_length_shortest_path() {
    let subject = package("acme.changed", "1.0.0", "subject");
    let dep = package("acme.dep", "1.0.0", "dep");
    let seed = artifact(&subject, "seed.json", "seed", Some("https://example.org/Seed"));
    let a = artifact(&dep, "a.json", "a", Some("https://example.org/A"));
    let c = artifact(&dep, "c.json", "c", Some("https://example.org/C"));
    let d = artifact(&dep, "d.json", "d", Some("https://example.org/D"));
    let graph = graph(
        vec![subject.clone(), dep],
        vec![seed.clone(), a.clone(), c.clone(), d.clone()],
        Vec::new(),
        vec![
            resolved(&a, &seed, "https://example.org/Seed"),
            resolved(&c, &seed, "https://example.org/Seed"),
            resolved(&d, &a, "https://example.org/A"),
            resolved(&d, &c, "https://example.org/C"),
        ],
    );
    let diff = modified_diff(&subject, &subject, "https://example.org/Seed", "seed.json");

    let report = build_impact_report(&diff, &graph, &graph).unwrap();
    let d_impact = report
        .artifact_impacts
        .iter()
        .find(|impact| impact.impacted == d.identity)
        .unwrap();

    assert_eq!(d_impact.side, ImpactSide::Both);
    assert_eq!(d_impact.path.len(), 2);
    assert_eq!(d_impact.path[0].target, a.identity);
}

#[test]
fn preserves_added_and_removed_canonical_seeds_on_their_evidence_side() {
    let before_subject = package("acme.changed", "1.0.0", "before");
    let after_subject = package("acme.changed", "2.0.0", "after");
    let removed = artifact(&before_subject, "removed.json", "removed", Some("https://example.org/Removed"));
    let added = artifact(&after_subject, "added.json", "added", Some("https://example.org/Added"));
    let before = graph(vec![before_subject.clone()], vec![removed.clone()], Vec::new(), Vec::new());
    let after = graph(vec![after_subject.clone()], vec![added.clone()], Vec::new(), Vec::new());
    let diff = StructuralDiffReport {
        schema: StructuralDiffReport::SCHEMA_V1,
        package_name: "acme.changed".to_owned(),
        before: PackageEvidence {
            version: before_subject.version.clone(),
            archive_sha256: before_subject.sha256.clone(),
        },
        after: PackageEvidence {
            version: after_subject.version.clone(),
            archive_sha256: after_subject.sha256.clone(),
        },
        changes: vec![
            change(
                StructuralChangeKind::ResourceRemoved,
                "https://example.org/Removed",
                Some("removed.json"),
                None,
            ),
            change(
                StructuralChangeKind::ResourceAdded,
                "https://example.org/Added",
                None,
                Some("added.json"),
            ),
        ],
    };

    let report = build_impact_report(&diff, &before, &after).unwrap();
    assert_eq!(report.seeds.len(), 2);
    let added_seed = report
        .seeds
        .iter()
        .find(|seed| seed.kind == ImpactSeedKind::Added)
        .unwrap();
    assert!(added_seed.before.is_none());
    assert_eq!(added_seed.after.as_ref(), Some(&added.identity));
    let removed_seed = report
        .seeds
        .iter()
        .find(|seed| seed.kind == ImpactSeedKind::Removed)
        .unwrap();
    assert_eq!(removed_seed.before.as_ref(), Some(&removed.identity));
    assert!(removed_seed.after.is_none());
}

#[test]
fn canonical_url_change_becomes_removed_and_added_seed() {
    let before_subject = package("acme.changed", "1.0.0", "before");
    let after_subject = package("acme.changed", "2.0.0", "after");
    let before_artifact = artifact(&before_subject, "profile.json", "old", Some("https://example.org/Old"));
    let after_artifact = artifact(&after_subject, "profile.json", "new", Some("https://example.org/New"));
    let before = graph(vec![before_subject.clone()], vec![before_artifact], Vec::new(), Vec::new());
    let after = graph(vec![after_subject.clone()], vec![after_artifact], Vec::new(), Vec::new());
    let diff = StructuralDiffReport {
        schema: 1,
        package_name: "acme.changed".to_owned(),
        before: PackageEvidence {
            version: before_subject.version.clone(),
            archive_sha256: before_subject.sha256.clone(),
        },
        after: PackageEvidence {
            version: after_subject.version.clone(),
            archive_sha256: after_subject.sha256.clone(),
        },
        changes: vec![StructuralChange {
            kind: StructuralChangeKind::ResourceBytesChanged,
            resource: ResourceKey {
                kind: ResourceKeyKind::ResourceId,
                value: "profile".to_owned(),
            },
            before_filename: Some("profile.json".to_owned()),
            after_filename: Some("profile.json".to_owned()),
            view: None,
            element_id: None,
            field: None,
            before: None,
            after: None,
        }],
    };

    let report = build_impact_report(&diff, &before, &after).unwrap();
    assert_eq!(report.seeds.len(), 2);
    assert_eq!(report.seeds[0].kind, ImpactSeedKind::Added);
    assert_eq!(report.seeds[0].canonical, "https://example.org/New");
    assert_eq!(report.seeds[1].kind, ImpactSeedKind::Removed);
    assert_eq!(report.seeds[1].canonical, "https://example.org/Old");
}

#[test]
fn package_exposure_keeps_same_name_versions_distinct_and_terminates_cycles() {
    let subject = package("acme.changed", "2.0.0", "subject");
    let dep_v1 = package("acme.dep", "1.0.0", "dep-v1");
    let dep_v2 = package("acme.dep", "2.0.0", "dep-v2");
    let root = package("acme.root", "1.0.0", "root");
    let seed = artifact(&subject, "seed.json", "seed", Some("https://example.org/Seed"));
    let graph = graph(
        vec![subject.clone(), dep_v1.clone(), dep_v2.clone(), root.clone()],
        vec![seed],
        vec![
            dependency(&dep_v1, &subject, "2.0.0"),
            dependency(&dep_v2, &subject, "2.0.0"),
            dependency(&root, &dep_v1, "1.0.0"),
            dependency(&root, &dep_v2, "2.0.0"),
            dependency(&dep_v1, &root, "1.0.0"),
        ],
        Vec::new(),
    );
    let diff = modified_diff(&subject, &subject, "https://example.org/Seed", "seed.json");

    let report = build_impact_report(&diff, &graph, &graph).unwrap();
    let identities = report
        .package_impacts
        .iter()
        .map(|impact| impact.impacted.clone())
        .collect::<std::collections::BTreeSet<_>>();
    assert!(identities.contains(&dep_v1));
    assert!(identities.contains(&dep_v2));
    assert!(identities.contains(&root));
    assert_eq!(identities.len(), 3);
    assert!(report
        .package_impacts
        .iter()
        .all(|impact| impact.side == ImpactSide::Both));
}

#[test]
fn does_not_traverse_ambiguous_or_external_reference_edges() {
    let subject = package("acme.changed", "1.0.0", "subject");
    let dep = package("acme.dep", "1.0.0", "dep");
    let seed = artifact(&subject, "seed.json", "seed", Some("https://example.org/Seed"));
    let source = artifact(&dep, "source.json", "source", Some("https://example.org/Source"));
    let graph = graph(
        vec![subject.clone(), dep],
        vec![seed.clone(), source.clone()],
        Vec::new(),
        vec![ContextCanonicalReferenceEdge {
            source: source.identity.clone(),
            relation: CanonicalReferenceRelation::StructureBaseDefinition,
            source_path: "baseDefinition".to_owned(),
            source_element_id: None,
            canonical: "https://example.org/Seed".to_owned(),
            resolution: CanonicalResolutionStatus::Ambiguous,
            candidates: vec![seed.identity.clone(), source.identity.clone()],
        }],
    );
    let diff = modified_diff(&subject, &subject, "https://example.org/Seed", "seed.json");

    let report = build_impact_report(&diff, &graph, &graph).unwrap();
    assert!(report.artifact_impacts.is_empty());
    assert!(report.unresolved_boundaries.is_empty());
}

fn modified_diff(
    before: &ContextPackageIdentity,
    after: &ContextPackageIdentity,
    canonical: &str,
    filename: &str,
) -> StructuralDiffReport {
    StructuralDiffReport {
        schema: StructuralDiffReport::SCHEMA_V1,
        package_name: before.name.clone(),
        before: PackageEvidence {
            version: before.version.clone(),
            archive_sha256: before.sha256.clone(),
        },
        after: PackageEvidence {
            version: after.version.clone(),
            archive_sha256: after.sha256.clone(),
        },
        changes: vec![change(
            StructuralChangeKind::ResourceBytesChanged,
            canonical,
            Some(filename),
            Some(filename),
        )],
    }
}

fn change(
    kind: StructuralChangeKind,
    canonical: &str,
    before_filename: Option<&str>,
    after_filename: Option<&str>,
) -> StructuralChange {
    StructuralChange {
        kind,
        resource: ResourceKey {
            kind: ResourceKeyKind::Canonical,
            value: canonical.to_owned(),
        },
        before_filename: before_filename.map(str::to_owned),
        after_filename: after_filename.map(str::to_owned),
        view: None,
        element_id: None,
        field: None,
        before: None,
        after: None,
    }
}

fn graph(
    packages: Vec<ContextPackageIdentity>,
    artifacts: Vec<ContextArtifactNode>,
    package_dependency_edges: Vec<ContextPackageDependencyEdge>,
    canonical_reference_edges: Vec<ContextCanonicalReferenceEdge>,
) -> ContextGraphReport {
    ContextGraphReport {
        schema: ContextGraphReport::SCHEMA_V1,
        lock_schema: Lockfile::SCHEMA_V2,
        root_requests: Vec::new(),
        packages: packages
            .into_iter()
            .map(|identity| ContextPackageNode {
                identity,
                source: "https://packages.example/archive.tgz".to_owned(),
            })
            .collect(),
        artifacts,
        package_dependency_edges,
        canonical_reference_edges,
        coverage: ContextCoverage {
            extractor_schema: 1,
            supported_source_resource_types: vec!["StructureDefinition".to_owned()],
            unsupported_source_resource_types: Vec::new(),
        },
    }
}

fn package(name: &str, version: &str, sha256: &str) -> ContextPackageIdentity {
    ContextPackageIdentity {
        name: name.to_owned(),
        version: version.to_owned(),
        sha256: sha256.to_owned(),
    }
}

fn artifact(
    package: &ContextPackageIdentity,
    filename: &str,
    sha256: &str,
    canonical_url: Option<&str>,
) -> ContextArtifactNode {
    ContextArtifactNode {
        identity: ContextArtifactIdentity {
            package: package.clone(),
            filename: filename.to_owned(),
            sha256: sha256.to_owned(),
        },
        resource_type: "StructureDefinition".to_owned(),
        id: Some(filename.trim_end_matches(".json").to_owned()),
        canonical_url: canonical_url.map(str::to_owned),
        canonical_version: Some("1.0.0".to_owned()),
    }
}

fn dependency(
    from: &ContextPackageIdentity,
    to: &ContextPackageIdentity,
    constraint: &str,
) -> ContextPackageDependencyEdge {
    ContextPackageDependencyEdge {
        from: from.clone(),
        to: to.clone(),
        declared_constraint: constraint.to_owned(),
    }
}

fn resolved(
    source: &ContextArtifactNode,
    target: &ContextArtifactNode,
    canonical: &str,
) -> ContextCanonicalReferenceEdge {
    ContextCanonicalReferenceEdge {
        source: source.identity.clone(),
        relation: CanonicalReferenceRelation::StructureBaseDefinition,
        source_path: "baseDefinition".to_owned(),
        source_element_id: None,
        canonical: canonical.to_owned(),
        resolution: CanonicalResolutionStatus::Resolved,
        candidates: vec![target.identity.clone()],
    }
}

fn external(source: &ContextArtifactNode, canonical: &str) -> ContextCanonicalReferenceEdge {
    ContextCanonicalReferenceEdge {
        source: source.identity.clone(),
        relation: CanonicalReferenceRelation::StructureBindingValueSet,
        source_path: "differential.element[0].binding.valueSet".to_owned(),
        source_element_id: Some("Observation.value".to_owned()),
        canonical: canonical.to_owned(),
        resolution: CanonicalResolutionStatus::External,
        candidates: Vec::new(),
    }
}

fn ambiguous(
    source: &ContextArtifactNode,
    canonical: &str,
    mut candidates: Vec<ContextArtifactIdentity>,
) -> ContextCanonicalReferenceEdge {
    candidates.sort();
    ContextCanonicalReferenceEdge {
        source: source.identity.clone(),
        relation: CanonicalReferenceRelation::StructureTypeProfile,
        source_path: "differential.element[0].type[0].profile[0]".to_owned(),
        source_element_id: Some("Observation.value".to_owned()),
        canonical: canonical.to_owned(),
        resolution: CanonicalResolutionStatus::Ambiguous,
        candidates,
    }
}

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    CanonicalResolutionStatus, ContextArtifactIdentity, ContextArtifactNode,
    ContextCanonicalReferenceEdge, ContextGraphReport, ContextPackageDependencyEdge,
    ContextPackageIdentity, ContextPackageNode, ImpactArtifactPathStep, ImpactArtifactRelation,
    ImpactCoverage, ImpactError, ImpactGraphEvidence, ImpactPackagePathStep, ImpactPackageRelation,
    ImpactReport, ImpactSeed, ImpactSeedKind, ImpactSide, ImpactSubject, ImpactUnresolvedBoundary,
    Lockfile, ResourceKey, ResourceKeyKind, StructuralChangeKind, StructuralDiffReport,
};

#[derive(Default)]
struct SeedAccumulator {
    before_filename: Option<String>,
    after_filename: Option<String>,
    added: bool,
    removed: bool,
    modified: bool,
}

pub fn build_impact_report(
    diff: &StructuralDiffReport,
    before_graph: &ContextGraphReport,
    after_graph: &ContextGraphReport,
) -> Result<ImpactReport, ImpactError> {
    validate_inputs(diff, before_graph, after_graph)?;

    let before_subject = subject_node(before_graph, diff, ImpactSide::Before)?;
    let after_subject = subject_node(after_graph, diff, ImpactSide::After)?;
    let seeds = build_seeds(
        diff,
        before_graph,
        after_graph,
        &before_subject,
        &after_subject,
    )?;

    let mut artifact_impacts = Vec::new();
    let mut unresolved_boundaries = Vec::new();
    for seed in &seeds {
        if let Some(before_seed) = &seed.before {
            collect_artifact_side(
                before_graph,
                before_seed,
                ImpactSide::Before,
                &mut artifact_impacts,
                &mut unresolved_boundaries,
            )?;
        }
        if let Some(after_seed) = &seed.after {
            collect_artifact_side(
                after_graph,
                after_seed,
                ImpactSide::After,
                &mut artifact_impacts,
                &mut unresolved_boundaries,
            )?;
        }
    }

    let mut package_impacts = Vec::new();
    collect_package_side(
        before_graph,
        &before_subject.identity,
        ImpactSide::Before,
        &mut package_impacts,
    );
    collect_package_side(
        after_graph,
        &after_subject.identity,
        ImpactSide::After,
        &mut package_impacts,
    );

    let artifact_impacts = normalize_artifact_sides(artifact_impacts);
    let package_impacts = normalize_package_sides(package_impacts);
    let unresolved_boundaries = normalize_boundary_sides(unresolved_boundaries);

    Ok(ImpactReport {
        schema: ImpactReport::SCHEMA_V1,
        subject: ImpactSubject {
            package_name: diff.package_name.clone(),
            before: before_subject.identity.clone(),
            after: after_subject.identity.clone(),
        },
        before_evidence: graph_evidence(before_graph, before_subject),
        after_evidence: graph_evidence(after_graph, after_subject),
        seeds,
        artifact_impacts,
        package_impacts,
        unresolved_boundaries,
        coverage: ImpactCoverage {
            before: before_graph.coverage.clone(),
            after: after_graph.coverage.clone(),
        },
    })
}

fn validate_inputs(
    diff: &StructuralDiffReport,
    before_graph: &ContextGraphReport,
    after_graph: &ContextGraphReport,
) -> Result<(), ImpactError> {
    if diff.schema != StructuralDiffReport::SCHEMA_V1 {
        return Err(ImpactError::UnsupportedDiffSchema { found: diff.schema });
    }
    for (side, graph) in [("before", before_graph), ("after", after_graph)] {
        if graph.schema != ContextGraphReport::SCHEMA_V1 {
            return Err(ImpactError::UnsupportedContextSchema {
                side,
                found: graph.schema,
            });
        }
        if graph.lock_schema != Lockfile::SCHEMA_V2 {
            return Err(ImpactError::UnsupportedLockSchema {
                side,
                found: graph.lock_schema,
            });
        }
    }
    Ok(())
}

fn subject_node(
    graph: &ContextGraphReport,
    diff: &StructuralDiffReport,
    side: ImpactSide,
) -> Result<ContextPackageNode, ImpactError> {
    let (version, sha256, label) = match side {
        ImpactSide::Before => (
            diff.before.version.as_str(),
            diff.before.archive_sha256.as_str(),
            "before",
        ),
        ImpactSide::After => (
            diff.after.version.as_str(),
            diff.after.archive_sha256.as_str(),
            "after",
        ),
        ImpactSide::Both => unreachable!("subject lookup is side-specific"),
    };
    let expected = ContextPackageIdentity {
        name: diff.package_name.clone(),
        version: version.to_owned(),
        sha256: sha256.to_owned(),
    };
    let matches = graph
        .packages
        .iter()
        .filter(|package| package.identity == expected)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [package] => Ok((*package).clone()),
        [] => Err(ImpactError::SubjectPackageMissing {
            side: label,
            identity: format!("{}@{}#{}", expected.name, expected.version, expected.sha256),
        }),
        _ => Err(ImpactError::SubjectPackageAmbiguous {
            side: label,
            identity: format!("{}@{}#{}", expected.name, expected.version, expected.sha256),
        }),
    }
}

fn graph_evidence(graph: &ContextGraphReport, subject: ContextPackageNode) -> ImpactGraphEvidence {
    ImpactGraphEvidence {
        graph_schema: graph.schema,
        lock_schema: graph.lock_schema,
        root_requests: graph.root_requests.clone(),
        subject,
    }
}

fn build_seeds(
    diff: &StructuralDiffReport,
    before_graph: &ContextGraphReport,
    after_graph: &ContextGraphReport,
    before_subject: &ContextPackageNode,
    after_subject: &ContextPackageNode,
) -> Result<Vec<ImpactSeed>, ImpactError> {
    let mut grouped = BTreeMap::<ResourceKey, SeedAccumulator>::new();
    for change in &diff.changes {
        let entry = grouped.entry(change.resource.clone()).or_default();
        merge_filename(
            &mut entry.before_filename,
            change.before_filename.as_deref(),
            &change.resource,
            "before",
        )?;
        merge_filename(
            &mut entry.after_filename,
            change.after_filename.as_deref(),
            &change.resource,
            "after",
        )?;
        match change.kind {
            StructuralChangeKind::ResourceAdded => entry.added = true,
            StructuralChangeKind::ResourceRemoved => entry.removed = true,
            _ => entry.modified = true,
        }
    }

    let mut seeds = Vec::new();
    for (resource, state) in grouped {
        let before = artifact_for_filename(
            before_graph,
            &before_subject.identity,
            state.before_filename.as_deref(),
            "before",
        )?;
        let after = artifact_for_filename(
            after_graph,
            &after_subject.identity,
            state.after_filename.as_deref(),
            "after",
        )?;
        let before_canonical = before.as_ref().and_then(|node| node.canonical_url.clone());
        let after_canonical = after.as_ref().and_then(|node| node.canonical_url.clone());

        match (before_canonical, after_canonical) {
            (None, None) => {
                if resource.kind == ResourceKeyKind::Canonical {
                    push_seed_from_key(&mut seeds, &resource, &state, before, after);
                }
            }
            (Some(before_url), Some(after_url)) if before_url == after_url => {
                seeds.push(ImpactSeed {
                    kind: seed_kind(&state),
                    canonical: before_url,
                    before: before.map(|node| node.identity),
                    after: after.map(|node| node.identity),
                });
            }
            (Some(before_url), Some(after_url)) => {
                seeds.push(ImpactSeed {
                    kind: ImpactSeedKind::Removed,
                    canonical: before_url,
                    before: before.map(|node| node.identity),
                    after: None,
                });
                seeds.push(ImpactSeed {
                    kind: ImpactSeedKind::Added,
                    canonical: after_url,
                    before: None,
                    after: after.map(|node| node.identity),
                });
            }
            (Some(before_url), None) => seeds.push(ImpactSeed {
                kind: ImpactSeedKind::Removed,
                canonical: before_url,
                before: before.map(|node| node.identity),
                after: None,
            }),
            (None, Some(after_url)) => seeds.push(ImpactSeed {
                kind: ImpactSeedKind::Added,
                canonical: after_url,
                before: None,
                after: after.map(|node| node.identity),
            }),
        }
    }
    seeds.sort();
    seeds.dedup();
    Ok(seeds)
}

fn push_seed_from_key(
    seeds: &mut Vec<ImpactSeed>,
    resource: &ResourceKey,
    state: &SeedAccumulator,
    before: Option<ContextArtifactNode>,
    after: Option<ContextArtifactNode>,
) {
    seeds.push(ImpactSeed {
        kind: seed_kind(state),
        canonical: resource.value.clone(),
        before: before.map(|node| node.identity),
        after: after.map(|node| node.identity),
    });
}

fn seed_kind(state: &SeedAccumulator) -> ImpactSeedKind {
    if state.added && !state.removed && !state.modified {
        ImpactSeedKind::Added
    } else if state.removed && !state.added && !state.modified {
        ImpactSeedKind::Removed
    } else {
        ImpactSeedKind::Modified
    }
}

fn merge_filename(
    slot: &mut Option<String>,
    candidate: Option<&str>,
    resource: &ResourceKey,
    side: &'static str,
) -> Result<(), ImpactError> {
    let Some(candidate) = candidate else {
        return Ok(());
    };
    match slot {
        Some(existing) if existing != candidate => Err(ImpactError::ConflictingResourceFilename {
            resource: format!("{:?}:{}", resource.kind, resource.value),
            side,
        }),
        Some(_) => Ok(()),
        None => {
            *slot = Some(candidate.to_owned());
            Ok(())
        }
    }
}

fn artifact_for_filename(
    graph: &ContextGraphReport,
    package: &ContextPackageIdentity,
    filename: Option<&str>,
    side: &'static str,
) -> Result<Option<ContextArtifactNode>, ImpactError> {
    let Some(filename) = filename else {
        return Ok(None);
    };
    let matches = graph
        .artifacts
        .iter()
        .filter(|artifact| {
            artifact.identity.package == *package && artifact.identity.filename == filename
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [artifact] => Ok(Some((*artifact).clone())),
        [] => Err(ImpactError::ArtifactMissing {
            side,
            file: filename.to_owned(),
        }),
        _ => Err(ImpactError::ArtifactAmbiguous {
            side,
            file: filename.to_owned(),
        }),
    }
}

fn collect_artifact_side(
    graph: &ContextGraphReport,
    seed: &ContextArtifactIdentity,
    side: ImpactSide,
    impacts: &mut Vec<ImpactArtifactRelation>,
    boundaries: &mut Vec<ImpactUnresolvedBoundary>,
) -> Result<(), ImpactError> {
    let best = artifact_paths(graph, seed, side)?;
    let mut sources = BTreeSet::from([seed.clone()]);
    for (impacted, path) in &best {
        sources.insert(impacted.clone());
        impacts.push(ImpactArtifactRelation {
            impacted: impacted.clone(),
            seed: seed.clone(),
            side,
            path: path.clone(),
        });
    }

    for edge in &graph.canonical_reference_edges {
        if !sources.contains(&edge.source) || edge.resolution == CanonicalResolutionStatus::Resolved
        {
            continue;
        }
        boundaries.push(ImpactUnresolvedBoundary {
            source: edge.source.clone(),
            seed: seed.clone(),
            side,
            relation: edge.relation,
            source_path: edge.source_path.clone(),
            source_element_id: edge.source_element_id.clone(),
            canonical: edge.canonical.clone(),
            resolution: edge.resolution,
            candidates: edge.candidates.clone(),
        });
    }
    Ok(())
}

fn artifact_paths(
    graph: &ContextGraphReport,
    seed: &ContextArtifactIdentity,
    side: ImpactSide,
) -> Result<BTreeMap<ContextArtifactIdentity, Vec<ImpactArtifactPathStep>>, ImpactError> {
    let label = side_label(side);
    let mut reverse =
        BTreeMap::<ContextArtifactIdentity, Vec<ContextCanonicalReferenceEdge>>::new();
    for edge in &graph.canonical_reference_edges {
        if edge.resolution != CanonicalResolutionStatus::Resolved {
            continue;
        }
        if edge.candidates.len() != 1 {
            return Err(ImpactError::InconsistentResolvedReference {
                side: label,
                canonical: edge.canonical.clone(),
                candidates: edge.candidates.len(),
            });
        }
        reverse
            .entry(edge.candidates[0].clone())
            .or_default()
            .push(edge.clone());
    }
    for edges in reverse.values_mut() {
        edges.sort();
        edges.dedup();
    }

    let mut best = BTreeMap::<ContextArtifactIdentity, Vec<ImpactArtifactPathStep>>::new();
    best.insert(seed.clone(), Vec::new());
    let mut frontier = BTreeSet::new();
    frontier.insert((0usize, Vec::<ImpactArtifactPathStep>::new(), seed.clone()));

    while let Some((_, path, current)) = frontier.pop_first() {
        if best.get(&current) != Some(&path) {
            continue;
        }
        for edge in reverse.get(&current).into_iter().flatten() {
            let step = ImpactArtifactPathStep {
                source: edge.source.clone(),
                target: current.clone(),
                relation: edge.relation,
                source_path: edge.source_path.clone(),
                source_element_id: edge.source_element_id.clone(),
                canonical: edge.canonical.clone(),
            };
            let mut candidate = Vec::with_capacity(path.len() + 1);
            candidate.push(step);
            candidate.extend(path.iter().cloned());
            let source = edge.source.clone();
            if is_better_path(best.get(&source), &candidate) {
                best.insert(source.clone(), candidate.clone());
                frontier.insert((candidate.len(), candidate, source));
            }
        }
    }
    best.remove(seed);
    Ok(best)
}

fn collect_package_side(
    graph: &ContextGraphReport,
    subject: &ContextPackageIdentity,
    side: ImpactSide,
    impacts: &mut Vec<ImpactPackageRelation>,
) {
    for (impacted, path) in package_paths(graph, subject) {
        impacts.push(ImpactPackageRelation {
            impacted,
            subject: subject.clone(),
            side,
            path,
        });
    }
}

fn package_paths(
    graph: &ContextGraphReport,
    subject: &ContextPackageIdentity,
) -> BTreeMap<ContextPackageIdentity, Vec<ImpactPackagePathStep>> {
    let mut reverse = BTreeMap::<ContextPackageIdentity, Vec<ContextPackageDependencyEdge>>::new();
    for edge in &graph.package_dependency_edges {
        reverse
            .entry(edge.to.clone())
            .or_default()
            .push(edge.clone());
    }
    for edges in reverse.values_mut() {
        edges.sort();
        edges.dedup();
    }

    let mut best = BTreeMap::<ContextPackageIdentity, Vec<ImpactPackagePathStep>>::new();
    best.insert(subject.clone(), Vec::new());
    let mut frontier = BTreeSet::new();
    frontier.insert((0usize, Vec::<ImpactPackagePathStep>::new(), subject.clone()));

    while let Some((_, path, current)) = frontier.pop_first() {
        if best.get(&current) != Some(&path) {
            continue;
        }
        for edge in reverse.get(&current).into_iter().flatten() {
            let step = ImpactPackagePathStep {
                source: edge.from.clone(),
                target: current.clone(),
                declared_constraint: edge.declared_constraint.clone(),
            };
            let mut candidate = Vec::with_capacity(path.len() + 1);
            candidate.push(step);
            candidate.extend(path.iter().cloned());
            let source = edge.from.clone();
            if is_better_path(best.get(&source), &candidate) {
                best.insert(source.clone(), candidate.clone());
                frontier.insert((candidate.len(), candidate, source));
            }
        }
    }
    best.remove(subject);
    best
}

fn is_better_path<T: Ord>(existing: Option<&Vec<T>>, candidate: &Vec<T>) -> bool {
    match existing {
        None => true,
        Some(existing) => (candidate.len(), candidate) < (existing.len(), existing),
    }
}

fn normalize_artifact_sides(input: Vec<ImpactArtifactRelation>) -> Vec<ImpactArtifactRelation> {
    let mut grouped = BTreeMap::<
        (
            ContextArtifactIdentity,
            ContextArtifactIdentity,
            Vec<ImpactArtifactPathStep>,
        ),
        BTreeSet<ImpactSide>,
    >::new();
    for relation in input {
        grouped
            .entry((relation.impacted, relation.seed, relation.path))
            .or_default()
            .insert(relation.side);
    }
    let mut output = grouped
        .into_iter()
        .map(|((impacted, seed, path), sides)| ImpactArtifactRelation {
            impacted,
            seed,
            side: normalized_side(&sides),
            path,
        })
        .collect::<Vec<_>>();
    output.sort();
    output
}

fn normalize_package_sides(input: Vec<ImpactPackageRelation>) -> Vec<ImpactPackageRelation> {
    let mut grouped = BTreeMap::<
        (
            ContextPackageIdentity,
            ContextPackageIdentity,
            Vec<ImpactPackagePathStep>,
        ),
        BTreeSet<ImpactSide>,
    >::new();
    for relation in input {
        grouped
            .entry((relation.impacted, relation.subject, relation.path))
            .or_default()
            .insert(relation.side);
    }
    let mut output = grouped
        .into_iter()
        .map(|((impacted, subject, path), sides)| ImpactPackageRelation {
            impacted,
            subject,
            side: normalized_side(&sides),
            path,
        })
        .collect::<Vec<_>>();
    output.sort();
    output
}

fn normalize_boundary_sides(input: Vec<ImpactUnresolvedBoundary>) -> Vec<ImpactUnresolvedBoundary> {
    type BoundaryKey = (
        ContextArtifactIdentity,
        ContextArtifactIdentity,
        crate::CanonicalReferenceRelation,
        String,
        Option<String>,
        String,
        CanonicalResolutionStatus,
        Vec<ContextArtifactIdentity>,
    );
    let mut grouped = BTreeMap::<BoundaryKey, BTreeSet<ImpactSide>>::new();
    for boundary in input {
        grouped
            .entry((
                boundary.source,
                boundary.seed,
                boundary.relation,
                boundary.source_path,
                boundary.source_element_id,
                boundary.canonical,
                boundary.resolution,
                boundary.candidates,
            ))
            .or_default()
            .insert(boundary.side);
    }
    let mut output = grouped
        .into_iter()
        .map(
            |(
                (
                    source,
                    seed,
                    relation,
                    source_path,
                    source_element_id,
                    canonical,
                    resolution,
                    candidates,
                ),
                sides,
            )| ImpactUnresolvedBoundary {
                source,
                seed,
                side: normalized_side(&sides),
                relation,
                source_path,
                source_element_id,
                canonical,
                resolution,
                candidates,
            },
        )
        .collect::<Vec<_>>();
    output.sort();
    output
}

fn normalized_side(sides: &BTreeSet<ImpactSide>) -> ImpactSide {
    if sides.contains(&ImpactSide::Before) && sides.contains(&ImpactSide::After) {
        ImpactSide::Both
    } else if sides.contains(&ImpactSide::Before) {
        ImpactSide::Before
    } else {
        ImpactSide::After
    }
}

fn side_label(side: ImpactSide) -> &'static str {
    match side {
        ImpactSide::Before => "before",
        ImpactSide::After => "after",
        ImpactSide::Both => "both",
    }
}

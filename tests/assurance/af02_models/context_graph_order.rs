pub const INVALIDITY_CLASSES: &[&str] = &[
    "PACKAGE_ORDER_PERMUTATION",
    "ARTIFACT_ORDER_PERMUTATION",
    "DEPENDENCY_EDGE_ORDER_PERMUTATION",
    "REFERENCE_ORDER_PERMUTATION",
    "DUPLICATE_ARTIFACT",
    "DUPLICATE_EDGE",
    "DUPLICATE_REFERENCE",
];

use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PackageIdentity {
    pub name: String,
    pub version: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PackageNode {
    pub identity: PackageIdentity,
    pub source: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ArtifactNode {
    pub package: PackageIdentity,
    pub filename: String,
    pub sha256: String,
    pub resource_type: String,
    pub id: Option<String>,
    pub canonical_url: Option<String>,
    pub canonical_version: Option<String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DependencyEdge {
    pub from: PackageIdentity,
    pub to: PackageIdentity,
    pub declared_constraint: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ReferenceEdge {
    pub source: String,
    pub relation: String,
    pub source_path: String,
    pub canonical: String,
    pub candidates: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Graph {
    pub root_requests: Vec<String>,
    pub packages: Vec<PackageNode>,
    pub artifacts: Vec<ArtifactNode>,
    pub dependency_edges: Vec<DependencyEdge>,
    pub reference_edges: Vec<ReferenceEdge>,
    pub unsupported_resource_types: Vec<String>,
}

pub fn canonicalize(
    root_requests: Vec<String>,
    mut packages: Vec<PackageNode>,
    mut artifacts: Vec<ArtifactNode>,
    mut dependency_edges: Vec<DependencyEdge>,
    mut reference_edges: Vec<ReferenceEdge>,
) -> Graph {
    packages.sort();

    artifacts.sort();
    artifacts.dedup();

    dependency_edges.sort();
    dependency_edges.dedup();

    for edge in &mut reference_edges {
        edge.candidates.sort();
        edge.candidates.dedup();
    }
    reference_edges.sort();
    reference_edges.dedup();

    let supported = ["CodeSystem", "StructureDefinition", "ValueSet"]
        .into_iter()
        .collect::<BTreeSet<_>>();
    let unsupported_resource_types = artifacts
        .iter()
        .map(|artifact| artifact.resource_type.as_str())
        .filter(|resource_type| !supported.contains(resource_type))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    Graph {
        root_requests,
        packages,
        artifacts,
        dependency_edges,
        reference_edges,
        unsupported_resource_types,
    }
}

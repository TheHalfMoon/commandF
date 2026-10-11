use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelLockfile {
    pub roots: Vec<String>,
    pub packages: Vec<ModelPackage>,
    pub resolved_dependencies: Vec<ModelEdge>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelPackage {
    pub name: String,
    pub version: String,
    pub dependencies: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ModelEdge {
    pub from_name: String,
    pub from_version: String,
    pub to_name: String,
    pub to_version: String,
    pub declared_constraint: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Invalidity {
    UnsortedRoots,
    DuplicateRoot,
    UnsortedPackages,
    DuplicatePackageIdentity,
    UnsortedEdges,
    DuplicateEdge,
    MissingSourcePackage,
    MissingTargetPackage,
    EmptyConstraint,
    UndeclaredDependency,
    ConstraintMismatch,
    UnsatisfiedTargetVersion,
    MultipleTargetsForDependency,
    MissingResolvedEdge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Verdict {
    ValidCanonical,
    Invalid(Invalidity),
}

pub fn canonical_case() -> ModelLockfile {
    let mut dependencies = BTreeMap::new();
    dependencies.insert("b.dep".to_owned(), "1.2.x".to_owned());
    dependencies.insert("c.dep".to_owned(), "2.0.0".to_owned());

    ModelLockfile {
        roots: vec!["a.pkg@1.0.0".to_owned()],
        packages: vec![
            ModelPackage {
                name: "a.pkg".to_owned(),
                version: "1.0.0".to_owned(),
                dependencies,
            },
            ModelPackage {
                name: "b.dep".to_owned(),
                version: "1.2.3".to_owned(),
                dependencies: BTreeMap::new(),
            },
            ModelPackage {
                name: "c.dep".to_owned(),
                version: "2.0.0".to_owned(),
                dependencies: BTreeMap::new(),
            },
        ],
        resolved_dependencies: vec![
            ModelEdge {
                from_name: "a.pkg".to_owned(),
                from_version: "1.0.0".to_owned(),
                to_name: "b.dep".to_owned(),
                to_version: "1.2.3".to_owned(),
                declared_constraint: "1.2.x".to_owned(),
            },
            ModelEdge {
                from_name: "a.pkg".to_owned(),
                from_version: "1.0.0".to_owned(),
                to_name: "c.dep".to_owned(),
                to_version: "2.0.0".to_owned(),
                declared_constraint: "2.0.0".to_owned(),
            },
        ],
    }
}

pub fn generated_valid_case(major: u8, minor: u8, patch: u8, wildcard: bool) -> ModelLockfile {
    let target_version = format!("{major}.{minor}.{patch}");
    let constraint = if wildcard {
        format!("{major}.{minor}.x")
    } else {
        target_version.clone()
    };
    let mut dependencies = BTreeMap::new();
    dependencies.insert("b.dep".to_owned(), constraint.clone());

    ModelLockfile {
        roots: vec!["a.pkg@1.0.0".to_owned()],
        packages: vec![
            ModelPackage {
                name: "a.pkg".to_owned(),
                version: "1.0.0".to_owned(),
                dependencies,
            },
            ModelPackage {
                name: "b.dep".to_owned(),
                version: target_version.clone(),
                dependencies: BTreeMap::new(),
            },
        ],
        resolved_dependencies: vec![ModelEdge {
            from_name: "a.pkg".to_owned(),
            from_version: "1.0.0".to_owned(),
            to_name: "b.dep".to_owned(),
            to_version: target_version,
            declared_constraint: constraint,
        }],
    }
}

pub fn invalid_case(kind: Invalidity) -> ModelLockfile {
    let mut case = canonical_case();
    match kind {
        Invalidity::UnsortedRoots => {
            case.roots = vec!["z.pkg@1.0.0".to_owned(), "a.pkg@1.0.0".to_owned()];
        }
        Invalidity::DuplicateRoot => {
            case.roots.push(case.roots[0].clone());
        }
        Invalidity::UnsortedPackages => {
            case.packages.swap(0, 1);
        }
        Invalidity::DuplicatePackageIdentity => {
            case.packages.push(case.packages[0].clone());
        }
        Invalidity::UnsortedEdges => {
            case.resolved_dependencies.swap(0, 1);
        }
        Invalidity::DuplicateEdge => {
            case.resolved_dependencies.push(case.resolved_dependencies[0].clone());
        }
        Invalidity::MissingSourcePackage => {
            case.resolved_dependencies[0].from_name = "missing.pkg".to_owned();
            case.resolved_dependencies.sort();
        }
        Invalidity::MissingTargetPackage => {
            case.resolved_dependencies[0].to_name = "missing.dep".to_owned();
            case.resolved_dependencies.sort();
        }
        Invalidity::EmptyConstraint => {
            case.resolved_dependencies[0].declared_constraint.clear();
        }
        Invalidity::UndeclaredDependency => {
            case.packages[0].dependencies.remove("b.dep");
        }
        Invalidity::ConstraintMismatch => {
            case.resolved_dependencies[0].declared_constraint = "1.2.3".to_owned();
        }
        Invalidity::UnsatisfiedTargetVersion => {
            case.packages[0]
                .dependencies
                .insert("b.dep".to_owned(), "1.3.x".to_owned());
            case.resolved_dependencies[0].declared_constraint = "1.3.x".to_owned();
        }
        Invalidity::MultipleTargetsForDependency => {
            case.packages.push(ModelPackage {
                name: "b.dep".to_owned(),
                version: "1.2.4".to_owned(),
                dependencies: BTreeMap::new(),
            });
            case.packages.sort_by(|left, right| {
                left.name
                    .cmp(&right.name)
                    .then_with(|| left.version.cmp(&right.version))
            });
            case.resolved_dependencies.push(ModelEdge {
                from_name: "a.pkg".to_owned(),
                from_version: "1.0.0".to_owned(),
                to_name: "b.dep".to_owned(),
                to_version: "1.2.4".to_owned(),
                declared_constraint: "1.2.x".to_owned(),
            });
            case.resolved_dependencies.sort();
        }
        Invalidity::MissingResolvedEdge => {
            case.resolved_dependencies.remove(0);
        }
    }
    case
}

pub fn validate(case: &ModelLockfile) -> Verdict {
    if has_duplicate(&case.roots) {
        return Verdict::Invalid(Invalidity::DuplicateRoot);
    }
    if !is_sorted(&case.roots) {
        return Verdict::Invalid(Invalidity::UnsortedRoots);
    }

    let package_ids = case
        .packages
        .iter()
        .map(|package| (package.name.as_str(), package.version.as_str()))
        .collect::<Vec<_>>();
    if package_ids.iter().copied().collect::<BTreeSet<_>>().len() != package_ids.len() {
        return Verdict::Invalid(Invalidity::DuplicatePackageIdentity);
    }
    if !is_sorted(&package_ids) {
        return Verdict::Invalid(Invalidity::UnsortedPackages);
    }

    if case
        .resolved_dependencies
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .len()
        != case.resolved_dependencies.len()
    {
        return Verdict::Invalid(Invalidity::DuplicateEdge);
    }
    if !is_sorted(&case.resolved_dependencies) {
        return Verdict::Invalid(Invalidity::UnsortedEdges);
    }

    let packages = case
        .packages
        .iter()
        .map(|package| ((package.name.as_str(), package.version.as_str()), package))
        .collect::<BTreeMap<_, _>>();
    let mut covered = BTreeSet::new();

    for edge in &case.resolved_dependencies {
        let Some(parent) = packages.get(&(edge.from_name.as_str(), edge.from_version.as_str())) else {
            return Verdict::Invalid(Invalidity::MissingSourcePackage);
        };
        if !packages.contains_key(&(edge.to_name.as_str(), edge.to_version.as_str())) {
            return Verdict::Invalid(Invalidity::MissingTargetPackage);
        }
        if edge.declared_constraint.is_empty() {
            return Verdict::Invalid(Invalidity::EmptyConstraint);
        }
        let Some(manifest_constraint) = parent.dependencies.get(&edge.to_name) else {
            return Verdict::Invalid(Invalidity::UndeclaredDependency);
        };
        if manifest_constraint != &edge.declared_constraint {
            return Verdict::Invalid(Invalidity::ConstraintMismatch);
        }
        if !constraint_matches(&edge.declared_constraint, &edge.to_version) {
            return Verdict::Invalid(Invalidity::UnsatisfiedTargetVersion);
        }
        if !covered.insert((
            edge.from_name.as_str(),
            edge.from_version.as_str(),
            edge.to_name.as_str(),
        )) {
            return Verdict::Invalid(Invalidity::MultipleTargetsForDependency);
        }
    }

    for package in &case.packages {
        for dependency in package.dependencies.keys() {
            if !covered.contains(&(
                package.name.as_str(),
                package.version.as_str(),
                dependency.as_str(),
            )) {
                return Verdict::Invalid(Invalidity::MissingResolvedEdge);
            }
        }
    }

    Verdict::ValidCanonical
}

fn has_duplicate<T: Ord>(values: &[T]) -> bool {
    values.iter().collect::<BTreeSet<_>>().len() != values.len()
}

fn is_sorted<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] <= pair[1])
}

fn constraint_matches(constraint: &str, version: &str) -> bool {
    let Some((major, minor, patch)) = parse_version(version) else {
        return false;
    };
    if let Some(prefix) = constraint.strip_suffix(".x") {
        let mut pieces = prefix.split('.');
        let expected_major = pieces.next().and_then(|value| value.parse::<u64>().ok());
        let expected_minor = pieces.next().and_then(|value| value.parse::<u64>().ok());
        return pieces.next().is_none()
            && expected_major == Some(major)
            && expected_minor == Some(minor);
    }
    parse_version(constraint) == Some((major, minor, patch))
}

fn parse_version(value: &str) -> Option<(u64, u64, u64)> {
    let mut pieces = value.split('.');
    let major = pieces.next()?.parse().ok()?;
    let minor = pieces.next()?.parse().ok()?;
    let patch = pieces.next()?.parse().ok()?;
    if pieces.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

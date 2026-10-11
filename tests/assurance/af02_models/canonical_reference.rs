pub const INVALIDITY_CLASSES: &[&str] = &[
    "EMPTY_TARGET",
    "EMPTY_TARGET_BEFORE_VERSION",
    "EMPTY_EXPLICIT_VERSION",
    "UNKNOWN_URL",
    "VERSION_WITHOUT_MATCH",
    "DUPLICATE_CANDIDATE_IDENTITY",
];

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Candidate {
    pub identity: String,
    pub url: String,
    pub version: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Resolution {
    EmptyTarget,
    EmptyVersion,
    External,
    Resolved(String),
    Ambiguous(Vec<String>),
}

pub fn resolve(canonical: &str, candidates: &[Candidate]) -> Resolution {
    let target = canonical
        .split_once('#')
        .map(|(target, _)| target)
        .unwrap_or(canonical);
    if target.is_empty() {
        return Resolution::EmptyTarget;
    }

    let (url, explicit_version) = if let Some((url, version)) = target.rsplit_once('|') {
        if url.is_empty() {
            return Resolution::EmptyTarget;
        }
        if version.is_empty() {
            return Resolution::EmptyVersion;
        }
        (url, Some(version))
    } else {
        (target, None)
    };

    let mut matches = candidates
        .iter()
        .filter(|candidate| {
            candidate.url == url
                && explicit_version
                    .map(|version| candidate.version.as_deref() == Some(version))
                    .unwrap_or(true)
        })
        .map(|candidate| candidate.identity.clone())
        .collect::<Vec<_>>();
    matches.sort();
    matches.dedup();

    match matches.len() {
        0 => Resolution::External,
        1 => Resolution::Resolved(matches.remove(0)),
        _ => Resolution::Ambiguous(matches),
    }
}

pub fn invalid_cases() -> [(&'static str, Resolution); 3] {
    [
        ("", Resolution::EmptyTarget),
        ("#fragment", Resolution::EmptyTarget),
        ("https://example.org/ValueSet/x|", Resolution::EmptyVersion),
    ]
}

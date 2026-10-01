pub const INVALIDITY_CLASSES: &[&str] = &[
    "EMPTY",
    "LEADING_SLASH",
    "UNC_PREFIX",
    "DRIVE_PREFIX",
    "EMPTY_COMPONENT",
    "DOT_COMPONENT",
    "PARENT_COMPONENT",
    "DISALLOWED_SINGLE_DOT",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Invalidity {
    Empty,
    LeadingSlash,
    UncPrefix,
    DrivePrefix,
    EmptyComponent,
    DotComponent,
    ParentComponent,
    DisallowedSingleDot,
}

pub fn normalize(value: &str, allow_dot: bool) -> Result<String, Invalidity> {
    if value.is_empty() {
        return Err(Invalidity::Empty);
    }
    if value == "." {
        return if allow_dot {
            Ok(String::new())
        } else {
            Err(Invalidity::DisallowedSingleDot)
        };
    }

    let normalized = value.replace('\\', "/");
    if normalized.starts_with("//") {
        return Err(Invalidity::UncPrefix);
    }
    if normalized.starts_with('/') {
        return Err(Invalidity::LeadingSlash);
    }
    if normalized.as_bytes().get(1) == Some(&b':') {
        return Err(Invalidity::DrivePrefix);
    }

    let mut output = Vec::new();
    for component in normalized.split('/') {
        if component.is_empty() {
            return Err(Invalidity::EmptyComponent);
        }
        if component == "." {
            return Err(Invalidity::DotComponent);
        }
        if component == ".." {
            return Err(Invalidity::ParentComponent);
        }
        output.push(component);
    }
    Ok(output.join("/"))
}

pub fn invalid_case(kind: Invalidity) -> (&'static str, bool) {
    match kind {
        Invalidity::Empty => ("", false),
        Invalidity::LeadingSlash => ("/absolute", false),
        Invalidity::UncPrefix => ("\\\\server\\share", false),
        Invalidity::DrivePrefix => ("C:\\escape", false),
        Invalidity::EmptyComponent => ("alpha//beta", false),
        Invalidity::DotComponent => ("alpha/./beta", false),
        Invalidity::ParentComponent => ("alpha/../beta", false),
        Invalidity::DisallowedSingleDot => (".", false),
    }
}

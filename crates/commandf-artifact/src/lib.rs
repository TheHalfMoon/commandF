mod error;
mod inspect;
mod model;

pub use error::ArtifactError;
pub use inspect::inspect_package;
pub use model::{ElementAddress, ElementView, PackageInspection, ResourceArtifact};

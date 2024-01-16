use strong_xml::XmlRead;

use crate::maven::pom::{ArtifactId, GroupId, Version};

/// The Maven variant to parse poms
/// These structs is directly modelled after the XML because that is what strong-xml plugin requires
#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "metadata")]
pub struct Metadata {
    #[xml(child = "groupId")]
    pub group_id: GroupId,
    #[xml(child = "artifactId")]
    pub artifact_id: ArtifactId,
    #[xml(child = "version")]
    pub version: Version,
    #[xml(child = "versioning")]
    pub versioning: Versioning,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "versioning")]
pub struct Versioning {
    #[xml(child = "snapshot")]
    pub snapshot: Snapshot,
    #[xml(child = "lastUpdated")]
    pub last_updated: LastUpdated,
    #[xml(child = "snapshotVersions")]
    pub snapshot_versions: SnapshotVersions,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "snapshot")]
pub struct Snapshot {
    #[xml(child = "timestamp")]
    pub timestamp: Timestamp,
    #[xml(child = "buildNumber")]
    pub build_number: BuildNumber,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "snapshotVersions")]
pub struct SnapshotVersions {
    #[xml(child = "snapshotVersion")]
    pub snapshot_versions: Vec<SnapshotVersion>,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "snapshotVersion")]
pub struct SnapshotVersion {
    #[xml(child = "classifier")]
    pub classifier: Option<Classifier>,
    #[xml(child = "extension")]
    pub extension: Extension,
    #[xml(child = "value")]
    pub value: Value,
    #[xml(child = "updated")]
    pub updated: Updated,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "timestamp")]
pub struct Timestamp {
    #[xml(text)]
    pub value: String,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "buildNumber")]
pub struct BuildNumber {
    #[xml(text)]
    pub value: String,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "lastUpdated")]
pub struct LastUpdated {
    #[xml(text)]
    pub value: String,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "updated")]
pub struct Updated {
    #[xml(text)]
    pub value: String,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "extension")]
pub struct Extension {
    #[xml(text)]
    pub value: String,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "classifier")]
pub struct Classifier {
    #[xml(text)]
    pub value: String,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "value")]
pub struct Value {
    #[xml(text)]
    pub value: String,
}

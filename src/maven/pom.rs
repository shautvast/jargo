use strong_xml::XmlRead;

/// The Maven variant to parse poms
/// These structs is directly modelled after the XML because that is what strong-xml plugin requires
#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "project")]
pub struct Pom {
    #[xml(child = "modelVersion")]
    pub(crate) model_version: ModelVersion,
    #[xml(child = "parent")]
    pub(crate) parent: Option<Parent>,
    #[xml(child = "groupId")]
    pub(crate) group_id: Option<GroupId>,
    #[xml(child = "artifactId")]
    pub(crate) artifact_id: ArtifactId,
    #[xml(child = "version")]
    pub(crate) version: Option<Version>,
    #[xml(child = "name")]
    pub(crate) name: Name,
    #[xml(child = "packaging")]
    pub(crate) packaging: Option<Packaging>,
    #[xml(child = "url")]
    pub(crate) url: Option<Url>,
    #[xml(child = "description")]
    pub(crate) description: Description,
    #[xml(child = "licenses")]
    pub(crate) licences: Option<Licenses>,
    #[xml(child = "scm")]
    pub(crate) scm: Option<Scm>,
    #[xml(child = "developers")]
    pub(crate) developers: Option<Developers>,
    #[xml(child = "dependencies")]
    pub(crate) dependencies: Option<Dependencies>,
    #[xml(child = "dependencyManagement")]
    pub(crate) dependency_management: Option<DependencyManagement>,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "modelVersion")]
pub struct ModelVersion {
    #[xml(text)]
    pub value: String,
}

#[derive(XmlRead, PartialEq, Debug, Clone)]
#[xml(tag = "groupId")]
pub struct GroupId {
    #[xml(text)]
    pub(crate) value: String,
}

#[derive(XmlRead, PartialEq, Debug, Clone)]
#[xml(tag = "artifactId")]
pub struct ArtifactId {
    #[xml(text)]
    pub(crate) value: String,
}

#[derive(XmlRead, PartialEq, Debug, Clone)]
#[xml(tag = "version")]
pub struct Version {
    #[xml(text)]
    pub(crate) value: String,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "name")]
pub struct Name {
    #[xml(text)]
    pub(crate) value: String,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "id")]
pub struct Id {
    #[xml(text)]
    pub(crate) value: String,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "packaging")]
pub struct Packaging {
    #[xml(text)]
    pub(crate) value: String,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "url")]
pub struct Url {
    #[xml(text)]
    pub(crate) value: String,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "description")]
pub struct Description {
    #[xml(text)]
    pub(crate) value: String,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "licenses")]
pub struct Licenses {
    #[xml(child = "license")]
    pub(crate) licenses: Vec<License>,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "distribution")]
pub struct Distribution {
    #[xml(text)]
    pub(crate) value: String,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "license")]
pub struct License {
    #[xml(child = "name")]
    pub(crate) name: Name,
    #[xml(child = "url")]
    pub(crate) url: Url,
    #[xml(child = "distribution")]
    pub(crate) distribution: Option<Distribution>,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "parent")]
pub struct Parent {
    #[xml(child = "groupId")]
    pub(crate) group_id: GroupId,
    #[xml(child = "artifactId")]
    pub(crate) artifact_id: ArtifactId,
    #[xml(child = "version")]
    pub(crate) version: Version,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "scm")]
pub struct Scm {
    #[xml(child = "url")]
    pub(crate) url: Url,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "developers")]
pub struct Developers {
    #[xml(child = "developer")]
    pub(crate) developers: Vec<Developer>,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "developer")]
struct Developer {
    #[xml(child = "id")]
    pub(crate) id: Option<Id>,
    #[xml(child = "name")]
    pub(crate) name: Name,
}

#[derive(XmlRead, PartialEq, Debug, Clone)]
#[xml(tag = "dependencies")]
pub struct Dependencies {
    #[xml(child = "dependency")]
    pub(crate) value: Vec<Dependency>,
}

#[derive(XmlRead, PartialEq, Debug, Clone)]
#[xml(tag = "dependencyManagement")]
pub struct DependencyManagement {
    #[xml(child = "dependencies")]
    pub(crate) value: Dependencies,
}

#[derive(XmlRead, PartialEq, Debug, Clone)]
#[xml(tag = "dependency")]
pub struct Dependency {
    #[xml(child = "groupId")]
    pub(crate) group_id: GroupId,
    #[xml(child = "artifactId")]
    pub(crate) artifact_id: ArtifactId,
    #[xml(child = "version")]
    pub(crate) version: Option<Version>,
}

#[cfg(test)]
mod test {
    use strong_xml::XmlRead;

    use crate::maven::pom::Pom;

    #[test]
    fn parse_should_not_fail() {
        Pom::from_str(r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 http://maven.apache.org/maven-v4_0_0.xsd ">
    <modelVersion>4.0.0</modelVersion>
    <groupId>org.mockito</groupId>
    <artifactId>mockito-core</artifactId>
    <version>1.9.5</version>
    <name>Mockito</name>
    <packaging>jar</packaging>
    <url>http://www.mockito.org</url>
    <description>Mock objects library for java</description>
    <licenses>
        <license>
            <name>The MIT License</name>
            <url>http://code.google.com/p/mockito/wiki/License</url>
            <distribution>repo</distribution>
        </license>
    </licenses>
    <scm>
        <url>http://code.google.com/p/mockito/source/browse/</url>
    </scm>
    <developers>
        <developer>
            <id>szczepiq</id>
            <name>Szczepan Faber</name>
        </developer>
    </developers>
    <dependencies>
        <dependency>
            <groupId>org.hamcrest</groupId>
            <artifactId>hamcrest-core</artifactId>
            <version>1.1</version>
        </dependency>
        <dependency>
            <groupId>org.objenesis</groupId>
            <artifactId>objenesis</artifactId>
            <version>1.0</version>
        </dependency>
    </dependencies>
</project>
"#).unwrap();
    }
}

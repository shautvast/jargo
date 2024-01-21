use anyhow::Error;
use strong_xml::XmlRead;

use crate::deploader;
use crate::maven::pom::{Dependency, Parent, Pom};
use crate::project::{Artifact, Project};

/// offers a (non-mutable) view on the pom-as-xml-representation
/// the main use of this is that it resolves the parent information when needed
pub struct PomView<'a> {
    pom: Pom,
    project: &'a Project,
    parent: Option<Box<PomView<'a>>>,
}

impl<'a> PomView<'a> {
    pub fn new(pom: Pom, project: &'a Project) -> Result<Self, Error> {
        // recursively lookup the parents
        if let Some(parent) = &pom.parent {
            let parent_artifact = Artifact::new(
                &parent.group_id.value,
                &parent.artifact_id.value,
                &parent.version.value,
            );

            let parent_pom = Pom::from_str(
                &deploader::lookup_verified_pom(project, &parent_artifact)?
                    .pom_xml,
            )?;

            Ok(Self {
                pom,
                project,
                parent: Some(Box::new(PomView::new(parent_pom, project)?)),
            })
        } else {
            Ok(Self {
                pom,
                project,
                parent: None,
            })
        }
    }
    pub fn model_version(&self) -> String {
        self.pom.model_version.value.clone()
    }
    pub fn parent(&self) -> Option<ParentView> {
        self.pom.parent.as_ref().map(|p| ParentView { parent: &p })
    }
    pub fn group_id(&self) -> Option<String> {
        //TODO get value from parent
        self.pom.group_id.as_ref().map(|g| g.value.clone())
    }
    pub fn artifact_id(&self) -> &String {
        &self.pom.artifact_id.value
    }
    pub fn version(&self) -> String {
        let mut version = &self.pom.version;
        while version.is_none() {
            if let Some(parent) = &self.parent {
                version = &parent.pom.version;
            }
        }
        version.as_ref().map(|v| v.value.clone()).unwrap()
        // unwrap? This is assuming there is always a version,
        // which sounds plausible but is unproven
        // TODO resolve properties
    }
    pub fn name(&self) -> &String {
        &self.pom.name.value
    }
    pub fn packaging(&self) -> Option<String> {
        self.pom.packaging.as_ref().map(|v| v.value.clone())
    }
    pub fn url(&self) -> Option<String> {
        self.pom.url.as_ref().map(|v| v.value.clone())
    }
    pub fn description(&self) -> &String {
        &self.pom.description.value
    }

    pub fn dependency_management(&self) -> DependencyManagementView {
        DependencyManagementView {
            dependencies: self
                .pom
                .dependency_management
                .as_ref()
                .map(|d| d.value.clone())
                .unwrap_or(vec![]),
        }
    }

    pub fn dependencies(&self) -> Vec<DependencyView> {
        let mut resolved_deps = vec![];
        if let Some(deps) = &self.pom.dependencies {
            for dep in &deps.value {
                let version = if let Some(version) = &dep.version {
                    Some(version.value.clone())
                } else {
                    if let Some(parent) = &self.parent {
                        search_version(dep, parent.dependency_management())
                    } else {
                        None
                    }
                };
                resolved_deps.push(DependencyView {
                    group_id: &dep.group_id.value,
                    artifact_id: &dep.artifact_id.value,
                    version: version.expect(&format!(
                        "Could not find version for {}:{}",
                        dep.group_id.value, dep.artifact_id.value
                    )),
                })
            }
        }
        resolved_deps
    }
}

fn search_version(dep: &Dependency, depman: DependencyManagementView) -> Option<String> {
    for mandep in depman.dependencies {
        if mandep.group_id == dep.group_id && mandep.artifact_id == dep.artifact_id {
            return mandep.version.map(|v| v.value.clone());
        }
    }
    None
}

struct ParentView<'a> {
    parent: &'a Parent,
}

impl<'a> ParentView<'a> {
    pub fn group_id(&self) -> &'a String {
        &self.parent.group_id.value
    }

    pub fn artifact_id(&self) -> &'a String {
        &self.parent.artifact_id.value
    }
    pub fn version(&self) -> &'a String {
        &self.parent.version.value
    }
}

// a copied view
pub(crate) struct DependencyView<'a> {
    artifact_id: &'a String,
    group_id: &'a String,
    version: String,
}

struct DependencyManagementView {
    dependencies: Vec<Dependency>,
}

impl<'a> From<DependencyView<'a>> for Artifact {
    fn from(value: DependencyView) -> Self {
        Artifact::new(
            value.group_id,
            value.artifact_id,
            &value.version,
        )
    }
}
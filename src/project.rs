use std::fs;
use std::path::Path;

use crate::maven::pom::Dependency;
use anyhow::{anyhow, Error};
use toml::{Table, Value};

/// Top struct for jargo project data
#[derive(Debug)]
pub struct Project {
    pub group: String,
    pub name: String,
    pub version: String,
    pub main_dependencies: Vec<Artifact>,
    pub test_dependencies: Vec<Artifact>,
    pub project_root: String,
    pub repositories: Vec<String>,
}

/// The identifier for any released bundle (jar, war etc) like in maven
#[derive(Debug)]
pub struct Artifact {
    pub group: String,
    pub name: String,
    pub version: String,
    pub path: String,
}

impl Artifact {
    pub fn new(group: &str, name: &str, version: &str) -> Self {
        Self {
            group: group.into(),
            name: name.into(),
            version: version.into(),
            path: format!("{}/{}/{}", group.replace(".", "/"), name, version),
        }
    }

    pub fn is_snapshot(&self) -> bool {
        self.version.ends_with("-SNAPSHOT")
    }
}

/// Convert from XML view
impl From<Dependency> for Artifact {
    fn from(value: Dependency) -> Self {
        Self::new(
            &value.group_id.value,
            &value.artifact_id.value,
            &value.version.unwrap().value,
        )
    }
}

impl Artifact {
    /// Convert from TOML view
    pub fn from_table_entry(name_group: &str, version: String) -> Result<Self, Error> {
        let name_group_split: Vec<&str> = name_group.split(":").collect();
        if 2 != name_group_split.len() {
            return Err(anyhow!("dependency {} not well formatted", name_group));
        }
        let group = name_group_split[0].into();
        let name = name_group_split[1].into();

        Ok(Self::new(
            group,
            name,
            version[1..version.len() - 1].to_owned().as_str(),
        ))
    }
}

/// loads the project from the TOML file
pub fn load_project(jargo_file: Option<&str>) -> Result<Project, Error> {
    let jargo = Path::new(jargo_file.unwrap_or("./Jargo.toml"));

    let project_table = fs::read_to_string(jargo)?.parse::<Table>()?;
    let package = project_table.get("package").expect("package info missing");

    let repositories = repositories(project_table.get("repositories"))?;
    let main_dependencies = dependencies(project_table.get("dependencies"))?;
    let test_dependencies = dependencies(project_table.get("test-dependencies"))?;

    Ok(Project {
        group: strip_first_last(package.get("group").unwrap().to_string()),
        name: strip_first_last(package.get("name").unwrap().to_string()),
        version: strip_first_last(package.get("version").unwrap().to_string()),
        repositories,
        main_dependencies,
        test_dependencies,
        project_root: jargo
            .parent()
            .map(Path::to_str)
            .unwrap()
            .expect(&format!("projectroot {:?} not usable", jargo))
            .into(),
    })
}

fn repositories(table: Option<&Value>) -> Result<Vec<String>, Error> {
    let mut repositories = vec!["https://repo.maven.apache.org/maven2".to_owned()];
        if let Some(Some(table)) = table.map(|t|t.as_table()) {
            for repo in table {
                let repo_details = repo.1.clone();
                if let Value::Table(repo_details) = repo_details {
                    if let Some(Value::String(url)) = repo_details.get("url") {
                        repositories.push(url.into());
                    }
                }
        }
    }
    Ok(repositories)
}

/// convert dependencies from the TOML view
fn dependencies(table: Option<&Value>) -> Result<Vec<Artifact>, Error> {
    let mut dependencies = vec![];
    if let Some(table) = table {
        let table = table.as_table();
        if let Some(table) = table {
            for dep in table {
                dependencies.push(Artifact::from_table_entry(dep.0, dep.1.to_string())?);
            }
        }
    }
    Ok(dependencies)
}

/// Because strings in the toml are surrounded by double quotes
fn strip_first_last(text: String) -> String {
    text[1..text.len() - 1].into()
}

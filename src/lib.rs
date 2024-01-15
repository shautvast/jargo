use std::fs;
use std::path::Path;

use anyhow::{anyhow, Error};
use toml::{Table, Value};

pub mod pom;
mod config;
pub mod deploader;
pub mod compile;

#[derive(Debug)]
pub struct Project {
    pub group: String,
    pub name: String,
    pub version: String,
    pub dependencies: Vec<Artifact>,
    pub test_dependencies: Vec<Artifact>,
    pub project_root: String,
}

#[derive(Debug)]
pub struct Artifact {
    pub group: String,
    pub name: String,
    pub version: String,
}

impl From<pom::model::Dependency> for Artifact {
    fn from(value: pom::model::Dependency) -> Self {
        Artifact {
            group: value.group_id.value,
            name: value.artifact_id.value,
            version: value.version.value,
        }
    }
}

impl Artifact {
    pub fn from_table_entry(name_group: &str, version: String) -> Result<Self, Error>{
        let name_group_split: Vec<&str> = name_group.split(":").collect();
        if 2 != name_group_split.len(){
            return Err(anyhow!("dependency {} not well formatted", name_group));
        }
        let group = name_group_split[0].into();
        let name = name_group_split[1].into();

        Ok(Self{
            group,
            name,
            version: version[1..version.len()-1].to_owned(),
        })
    }
}

pub fn load_project(jargo_file: Option<&str>) -> Result<Project, Error> {
    let jargo = Path::new(jargo_file.unwrap_or("./Jargo.toml"));

    let project_table = fs::read_to_string(jargo)?.parse::<Table>()?;
    let package = project_table.get("package").expect("package info missing");

    let dependencies = get_dependencies(project_table.get("dependencies"))?;

    let test_dependencies = get_dependencies(project_table.get("test-dependencies"))?;


    Ok(Project {
        group: strip_first_last(package.get("group").unwrap().to_string()),
        name: strip_first_last(package.get("name").unwrap().to_string()),
        version: strip_first_last(package.get("version").unwrap().to_string()),
        dependencies,
        test_dependencies,
        project_root: jargo.parent().map(Path::to_str).unwrap().expect(&format!("projectroot {:?} not usable", jargo)).into(),
    })
}

fn get_dependencies(table: Option<&Value>) -> Result<Vec<Artifact>, Error> {
    let mut dependencies = vec![];
    if let Some(table) = table {
        let table = table.as_table();
        if let Some(table) = table {
            for dep in table {
                dependencies.push(
                    Artifact::from_table_entry(dep.0, dep.1.to_string())?
                )
            }
        }
    }
    Ok(dependencies)
}

fn strip_first_last(text: String) -> String{
    text[1..text.len()-1].into()
}
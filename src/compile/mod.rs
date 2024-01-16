use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Error;
use colored::Colorize;
use crate::compile::PathNode::*;

use crate::Project;

const SOURCES: &str = "src/main/java";
const TESTSOURCES: &str = "src/test/java";
const RESOURCES: &str = "src/main/resources";
const TESTRESOURCES: &str = "src/main/java";

const TARGET_MAIN: &str = "target/classes";
const TARGET_TEST: &str = "target/test-classes";


/// internal view of the src filesystem
#[derive(Debug)]
enum PathNode {
    DirNode(PathBuf, Vec<PathNode>, Vec<PathNode>),
    FileNode(PathBuf),
}

/// runs the compile stage
pub fn run(project: &Project) -> Result<(), Error> {
    println!("{} {}.{}-{}", "Compiling".green(), project.group, project.name, project.version);

    let root = PathBuf::from(&project.project_root).join(SOURCES);

    let mut src_tree = DirNode(root.clone(), Vec::new(), Vec::new());
    determine_src_tree(root, &mut src_tree)?;
    println!("{:?}", src_tree);

    compile_sourcedir(project, &mut src_tree)?;

    Ok(())
}

/// walks the source tree and compiles any java files
fn compile_sourcedir(project: &Project, src_tree: &mut PathNode) -> Result<(), Error> {
    if let DirNode(dir_name, subdirs, contents) = src_tree {
        if !contents.is_empty() {
            let mut javac = if cfg!(target_os = "windows") {
                vec!["/C"]
            } else {
                vec![]
            };
            let classes = PathBuf::from(&project.project_root).join(TARGET_MAIN);
            let classes = classes.to_str().unwrap();

            javac.append(&mut vec!["javac", "-d", classes, "-sourcepath"]);
            javac.push(dir_name.to_str().unwrap().into());
            for source in contents {
                if let FileNode(source_name) = source {
                    let name = source_name.to_str().unwrap();
                    javac.push(name);
                }
            }

            let output = if cfg!(target_os = "windows") {
                Command::new("cmd")
                    .args(javac)
                    .output()
                    .expect("failed to execute process")
            } else {
                Command::new("sh")
                    .arg("-c")
                    .arg(javac.join(" "))
                    .output()
                    .expect("failed to execute process")
            };
            if !output.stderr.is_empty() {
                println!("{}", String::from_utf8(output.stderr)?.red());
            }
            println!("{}", String::from_utf8(output.stdout)?);
        }
        for subdir in subdirs {
            compile_sourcedir(project, subdir)?;
        }
    }
    Ok(())
}

/// the source tree on disk is first read into memory
fn determine_src_tree(parent: PathBuf, parent_node: &mut PathNode) -> Result<(), Error> {
    let paths = fs::read_dir(&parent)?;

    for path in paths {
        let path = path?;

        if path.metadata()?.is_dir() {
            let mut subdir = DirNode(path.path(), Vec::new(), Vec::new());
            determine_src_tree(path.path(), &mut subdir)?;
            if let DirNode(_, subdirs, _) = parent_node {
                subdirs.push(subdir);
            }
        } else {
            let name = path.file_name();
            let name = name.to_str().unwrap().to_owned();
            if name.ends_with(".java") {
                if let DirNode(_, _, contents) = parent_node {
                    contents.push(FileNode(path.path()));
                }
            }
        }
    }
    Ok(())
}
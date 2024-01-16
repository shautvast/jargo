use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::Write;
use std::path::Path;

use anyhow::{anyhow, Error};
use bytes::Bytes;
use sha1::{Digest, Sha1};
use strong_xml::XmlRead;

use crate::config::config;
use crate::maven;
use crate::maven::metadata::Metadata;
use crate::maven::pom::Pom;
use crate::project::{Artifact, Project};
use colored::Colorize;
use reqwest::StatusCode;

/// Loads a list of artifacts from remote repo or local cache
///
/// 1. check if calculated location (directory) exists on local disk
/// 2. if not create it
/// 3. checks if jar exists on local disk
/// 4. if not downloads it from a repo (now mavencentral only)
/// 5. downloads/reads from disk the SHA1 (hex) file and compares it with the calculated one for the jar (this is done always)
/// 6. checks if pom exists on local disk
/// 7. if not downloads it from a repo (now mavencentral only)
/// 8. verifies the SHA1 as for the jar
/// 9. extracts the transitive dependencies from the pom and recurses to (1) for the list of dependencies
pub fn load(project: &Project) -> Result<(), Error> {
    load_artifacts(project, &project.main_dependencies)?;
    load_artifacts(project, &project.test_dependencies)?;
    Ok(())
}

fn load_artifacts(project: &Project, artifacts: &Vec<Artifact>) -> Result<(), Error> {
    for art in artifacts {
        load_artifact(project, art)?;
    }
    Ok(())
}

/// loads the artifact (all data and metadata for 1 artifact)
/// 1. create dir in local cache if necessary
/// 2. look up the pom
/// 3. look up the jar
fn load_artifact(project: &Project, artifact: &Artifact) -> Result<(), Error> {
    // check/create artifact directory
    let local_artifact_loc = format!("{}/{}", config().cache_location, artifact.path);
    if !exists(&local_artifact_loc) {
        fs::create_dir_all(&local_artifact_loc)?;
    }

    // download remote pom if not in cache
    let pom_lookup = lookup_verified_pom(project, artifact, &local_artifact_loc)?;

    // download remote jar if not in cache and check its SHA-1 checksum
    let local_artifact_jar_path = format!(
        "{}/{}-{}.jar",
        local_artifact_loc, artifact.name, artifact.version
    );
    if !exists(&local_artifact_jar_path) {
        lookup_verified_jar(
            project,
            artifact,
            &local_artifact_jar_path,
            &pom_lookup.resolved_repo.unwrap(),
            &pom_lookup.resolved_version.unwrap(),
        )?;
    }

    println!("{}", pom_lookup.pom_xml);
    // parse pom file
    let pom = Pom::from_str(&pom_lookup.pom_xml).unwrap();

    //TODO exclusions
    if let Some(dependencies) = pom.dependencies {
        let artifacts = dependencies.value.into_iter().map(|d| d.into()).collect();
        load_artifacts(project, &artifacts)?;
    }
    Ok(())
}

/// main function to download and verify the pom xml.
/// 1. check if file is locally cached and load if it is
/// 2. or find a suitable repo (deferred to find_pom)
/// 3. this function returns the downloaded pom, together with the repo it was found in and the "resolved version'
/// this is only applicable to SNAPSHOT's where x-SNAPSHOT is resolved to x-<timestamp>-<build_nr>
/// 4. download the SHA1 file from the same location
/// 5. validate if the checksum equals the checksum calculated from the pom
///
/// The result from find_pom is passed on to the caller so that the information can be used
/// for subsequent requests.
fn lookup_verified_pom(
    project: &Project,
    artifact: &Artifact,
    local_artifact_loc: &str,
) -> Result<PomLookupResult, Error> {
    let local_artifact_pom_path = &format!(
        "{}/{}-{}.pom",
        local_artifact_loc, artifact.name, artifact.version
    );
    let result = if exists(local_artifact_pom_path) {
        read_file_to_string(local_artifact_pom_path)?
    } else {
        find_pom(project, artifact, local_artifact_pom_path)?
    };
    if result.is_none() {
        panic!("Could not find pom for {}", artifact.path)
    } else {
        let result = result.unwrap();
        let repo_with_pom = result.resolved_repo.as_ref(); //TODO replace tuple with struct
        let pom_xml = &result.pom_xml;

        let local_artifact_pom_sha1_path = format!("{}.sha1", local_artifact_pom_path);

        // verify jarfile with SHA1 checksum (which is hex encoded)
        let checksum = if !exists(&local_artifact_pom_sha1_path) {
            download_checksum(repo_with_pom, &local_artifact_pom_sha1_path)?
        } else {
            read_file_to_bytes(local_artifact_pom_sha1_path)?
        };
        if let Some(checksum) = checksum {
            let validated = validate_checksum_text(&pom_xml, hex::decode(checksum)?);
            return if !validated {
                Err(anyhow!("SHA1 checksum for {} is not valid", artifact.path))
            } else {
                Ok(result.clone()) // SHA1 ok
            };
        } else {
            // no SHA1 found
            Ok(result.clone())
        }
    }
}

#[derive(Debug, Clone)]
struct PomLookupResult {
    pom_xml: String,
    resolved_repo: Option<String>,
    resolved_version: Option<String>,
}

fn find_pom(
    project: &Project,
    artifact: &Artifact,
    local_artifact_pom_path: &str,
) -> Result<Option<PomLookupResult>, Error> {
    for repo in &project.repositories {
        let resolved_version = resolve_version(&artifact, repo)?;
        let r = download_pom(artifact, &resolved_version, local_artifact_pom_path, &repo)?;
        if r.is_some() {
            return Ok(Some(PomLookupResult {
                pom_xml: r.unwrap(),
                resolved_repo: Some(repo.clone()),
                resolved_version: Some(resolved_version),
            }));
        }
    }
    Ok(None)
}

/// returns the pom and the repo where it was found
fn download_pom(
    artifact: &Artifact,
    resolved_version: &str,
    local_artifact_pom_path: &str,
    repo: &str,
) -> Result<Option<String>, Error> {
    let remote_artifact_pom_url = format!(
        "{}/{}/{}-{}.pom",
        repo, artifact.path, artifact.name, resolved_version
    );

    println!("{} {}", "Downloading".green(), remote_artifact_pom_url);
    let response = reqwest::blocking::get(&remote_artifact_pom_url)?;
    if response.status().is_success() {
        let body = response.text().unwrap();
        println!("{} {}", "Downloaded".green(), remote_artifact_pom_url);
        write_text(local_artifact_pom_path, &body)?;
        Ok(Some((body)))
    } else {
        Ok(None)
    }
}

/// Download jar from remote repo and check its signature
/// For now it's a blocking call, because async and recursion add unwanted complexity/I don't understand that
/// TODO add progress bar
fn lookup_verified_jar(
    project: &Project,
    artifact: &Artifact,
    local_artifact_jar_path: &str,
    resolved_repo: &str,
    resolved_version: &str,
) -> Result<(), Error> {
    let remote_artifact_jar_url = format!(
        "{}/{}/{}-{}.jar",
        resolved_repo, artifact.path, artifact.name, resolved_version
    );

    println!("{} {}", "Downloading".green(), remote_artifact_jar_url);
    let response = reqwest::blocking::get(&remote_artifact_jar_url)?;
    if response.status().is_success() {
        let jar = response.bytes().unwrap();
        println!("{} {}", "Downloaded".green(), remote_artifact_jar_url);
        write_bytes_to_file(&local_artifact_jar_path, &jar)?;

        let local_artifact_jar_sha1_path = format!("{}.sha1", local_artifact_jar_path);

        // verify jarfile with SHA1 checksum (which is hex encoded)
        let checksum = if !exists(&local_artifact_jar_sha1_path) {
            download_checksum(
                Some(&remote_artifact_jar_url),
                &local_artifact_jar_sha1_path,
            )?
        } else {
            read_file_to_bytes(local_artifact_jar_sha1_path)?
        };
        return if let Some(checksum) = checksum {
            let validated = validate_checksum_bytes(&jar, hex::decode(checksum)?);
            if !validated {
                Err(anyhow!(
                    "SHA1 checksum for {} is not valid",
                    remote_artifact_jar_url
                ))
            } else {
                Ok(()) // checksum found and ok
            }
        } else {
            Ok(()) // no checksum found
        };
    }
    panic!(
        "Artifact {} not found in remote repository {}",
        artifact.path, resolved_repo
    );
}

fn resolve_version(artifact: &Artifact, repo: &String) -> Result<String, Error> {
    Ok(if artifact.is_snapshot() {
        let build_nr = load_snapshot_build_nr(&artifact.path, repo)?;
        if let Some(build_nr) = build_nr {
            artifact.version.replace("SNAPSHOT", &build_nr)
        } else {
            artifact.version.clone()
        }
    } else {
        artifact.version.clone()
    })
}

/// Snapshots in a maven repo can be in the form
/// 'spring-boot-starter-web-3.0.0-20221124.170206-1099.jar'
/// while we ask for
/// 'spring-boot-starter-web-3.0.0-SNAPSHOT.jar'
/// the metadata xml contains the info on what snapshot to download
/// so we download and parse it
fn load_snapshot_build_nr(artifact_path: &str, repo: &String) -> Result<Option<String>, Error> {
    let metadata_url = format!("{}/{}/maven-metadata.xml", repo, artifact_path);
    let response = reqwest::blocking::get(&metadata_url)?;
    if response.status().is_success() {
        let body = response.text().unwrap();
        write_text(
            format!(
                "{}/{}/maven-metadata.xml",
                config().cache_location,
                artifact_path
            )
            .as_str(),
            &body,
        )?;
        let metadata = Metadata::from_str(&body)?;
        Ok(Some(format!(
            "{}-{}",
            metadata.versioning.snapshot.timestamp.value,
            metadata.versioning.snapshot.build_number.value
        )))
    } else {
        Ok(None)
    }
}

fn download_checksum(
    remote_artifact_url: Option<&String>,
    local_artifact_jar_sha1_path: &str,
) -> Result<Option<Vec<u8>>, Error> {
    if let Some(remote_artifact_url) = remote_artifact_url {
        let remote_artifact_jar_sha1_url = format!("{}.sha1", remote_artifact_url);
        let response = reqwest::blocking::get(&remote_artifact_jar_sha1_url)?;
        if response.status() == StatusCode::OK {
            let jar_checksum = response.bytes().unwrap();
            write_bytes_to_file(&local_artifact_jar_sha1_path, &jar_checksum)?;
            Ok(Some(jar_checksum.to_vec()))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

fn validate_checksum_bytes(jar: &Bytes, checksum: Vec<u8>) -> bool {
    let mut hasher = Sha1::new();
    hasher.update(&jar.to_vec());
    let result = hasher.finalize();
    result[..] == checksum
}

fn validate_checksum_text(text: &str, checksum: Vec<u8>) -> bool {
    let mut hasher = Sha1::new();
    hasher.update(text.as_bytes());
    let result = hasher.finalize();
    result[..] == checksum
}

fn exists(path: &str) -> bool {
    Path::new(path).exists()
}

fn write_bytes_to_file(jar_path: &str, bytes: &Bytes) -> Result<(), Error> {
    let mut file = File::create(jar_path)?;
    file.write_all(bytes)?;
    Ok(())
}

fn write_text(path: &str, contents: &String) -> Result<(), Error> {
    let mut file = File::create(path)?;
    file.write(contents.as_bytes())?;
    Ok(())
}

fn read_file_to_string(local_artifact_pom_path: &str) -> Result<Option<PomLookupResult>, Error> {
    let mut file = File::open(local_artifact_pom_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(Some(PomLookupResult {
        pom_xml: contents,
        resolved_repo: None,
        resolved_version: None,
    }))
}

fn read_file_to_bytes(local_artifact_jar_sha1_path: String) -> Result<Option<Vec<u8>>, Error> {
    let mut file = File::open(local_artifact_jar_sha1_path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(Some(contents))
}

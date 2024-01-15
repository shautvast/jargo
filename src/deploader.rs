use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::Write;
use std::path::Path;

use anyhow::{anyhow, Error};
use bytes::Bytes;
use sha1::{Digest, Sha1};
use strong_xml::XmlRead;

use crate::Artifact;
use crate::config::config;
use crate::pom::model::Pom;
use colored::Colorize;

pub fn load_artifacts(artifacts: &[Artifact]) -> Result<(), Error> {
    for art in artifacts {
        load_artifact(art)?;
    }
    Ok(())
}

pub fn load_artifact(art: &Artifact) -> Result<(), Error> {
    let artifact_path = format!("{}/{}/{}", art.group.replace(".", "/"), art.name, art.version);

    // check/create artifact directory
    let local_artifact_loc = format!("{}/{}", config().cache_location, artifact_path);
    if !exists(&local_artifact_loc) {
        fs::create_dir_all(&local_artifact_loc)?;
    }

    // download remote jar if not in cache and check its SHA-1 checksum
    let local_artifact_jar_path = format!("{}/{}-{}.jar", local_artifact_loc, art.name, art.version);
    if !exists(&local_artifact_jar_path) {
        download_verify_artifact_jar(art, &artifact_path, &local_artifact_jar_path)?;
    }

    // download remote pom if not in cache //TODO check its SHA-1 checksum
    let pom_xml = download_verify_pom(art, artifact_path, local_artifact_loc)?;

    // parse pom file
    let pom = Pom::from_str(&pom_xml).unwrap();

    let dependencies: Vec<Artifact> = pom.dependencies.value.into_iter().map(|d| d.into()).collect();
    load_artifacts(&dependencies)?;
    Ok(())
}

fn download_verify_pom(art: &Artifact, artifact_path: String, local_artifact_loc: String) -> Result<String, Error> {
    let local_artifact_pom_path = &format!("{}/{}-{}.pom", local_artifact_loc, art.name, art.version);
    let remote_artifact_pom_url = format!("{}/{}/{}-{}.pom", config().maven_central, artifact_path, art.name, art.version);
    let pom_xml = if !exists(local_artifact_pom_path) {
        println!("{} {}", "Downloading".green(), remote_artifact_pom_url);
        let body = reqwest::blocking::get(&remote_artifact_pom_url)?.text().unwrap();
        println!("{} {}", "Downloaded".green(), remote_artifact_pom_url);
        write_text(local_artifact_pom_path, &body)?;
        body
    } else {
        read_file_to_string(local_artifact_pom_path)?
    };

    let local_artifact_pom_sha1_path = format!("{}.sha1", local_artifact_pom_path);

    // verify jarfile with SHA1 checksum (which is hex encoded)
    let checksum = hex::decode(
        if !exists(&local_artifact_pom_sha1_path) {
            download_checksum(&remote_artifact_pom_url, &local_artifact_pom_sha1_path)?
        } else {
            read_file_to_bytes(local_artifact_pom_sha1_path)?
        })?;
    let validated = validate_checksum_text(&pom_xml, checksum);
    if !validated {
        Err(anyhow!("SHA1 checksum for {} is not valid", remote_artifact_pom_url))
    } else { Ok(pom_xml) }
}


/// Download jar from remote repo and check its signature
/// For now it's a blocking call, because async and recursion add unwanted complexity/I don't understand that
/// TODO add progress bar
fn download_verify_artifact_jar(art: &Artifact, artifact_path: &str, local_artifact_jar_path: &str) -> Result<(), Error> {
    let remote_artifact_jar_url = format!("{}/{}/{}-{}.jar", config().maven_central, artifact_path, art.name, art.version);

    println!("{} {}", "Downloading".green(), remote_artifact_jar_url);
    let jar = reqwest::blocking::get(&remote_artifact_jar_url)?.bytes().unwrap();
    println!("{} {}", "Downloaded".green(), remote_artifact_jar_url);
    write_bytes_to_file(&local_artifact_jar_path, &jar)?;

    let local_artifact_jar_sha1_path = format!("{}.sha1", local_artifact_jar_path);

    // verify jarfile with SHA1 checksum (which is hex encoded)
    let checksum = hex::decode(
        if !exists(&local_artifact_jar_sha1_path) {
            download_checksum(&remote_artifact_jar_url, &local_artifact_jar_sha1_path)?
        } else {
            read_file_to_bytes(local_artifact_jar_sha1_path)?
        })?;
    let validated = validate_checksum_bytes(&jar, checksum);
    if !validated {
        Err(anyhow!("SHA1 checksum for {} is not valid", remote_artifact_jar_url))
    } else { Ok(()) }
}


fn download_checksum(remote_artifact_url: &String, local_artifact_jar_sha1_path: &String) -> Result<Vec<u8>, Error> {
    let remote_artifact_jar_sha1_url = format!("{}.sha1", remote_artifact_url);
    let jar_checksum = reqwest::blocking::get(&remote_artifact_jar_sha1_url)?.bytes().unwrap();
    write_bytes_to_file(&local_artifact_jar_sha1_path, &jar_checksum)?;
    Ok(jar_checksum.to_vec())
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

fn read_file_to_string(local_artifact_pom_path: &str) -> Result<String, Error> {
    let mut file = File::open(local_artifact_pom_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn read_file_to_bytes(local_artifact_jar_sha1_path: String) -> Result<Vec<u8>, Error> {
    let mut file = File::open(local_artifact_jar_sha1_path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}
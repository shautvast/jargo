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
use crate::config::get_config;
use crate::pom::model::Pom;

pub fn load_artifacts(artifacts: &[Artifact]) -> Result<(), Error> {
    for art in artifacts {
        load_artifact(art)?;
    }
    Ok(())
}

pub fn load_artifact(art: &Artifact) -> Result<(), Error> {
    let artifact_path = format!("{}/{}/{}",
                                art.group.replace(".", "/"),
                                art.name,
                                art.version,
    );
    let local_artifact_loc = format!("{}/{}", get_config().cache_location, artifact_path);
    if !exists(&local_artifact_loc) {
        fs::create_dir_all(&local_artifact_loc)?;
    }

    let local_artifact_jar_path = format!("{}/{}-{}.jar", local_artifact_loc, art.name, art.version);
    if !exists(&local_artifact_jar_path) {
        load_verify_artifact_jar(art, &artifact_path, &local_artifact_jar_path)?;
    }

    let local_artifact_pom_path = format!("{}/{}-{}.pom", local_artifact_loc, art.name, art.version);
    let pom_xml = if !exists(&local_artifact_pom_path) {
        // download pom
        let remote_artifact_pom_url = format!("{}/{}/{}-{}.pom", get_config().maven_central, artifact_path, art.name, art.version);
        println!("Downloading {}", remote_artifact_pom_url);
        let body = reqwest::blocking::get(&remote_artifact_pom_url)?.text().unwrap();
        println!("Downloaded {}", remote_artifact_pom_url);
        write_text(&local_artifact_pom_path, &body)?;
        body
    } else {
        // read local pom file
        let mut file = File::open(local_artifact_pom_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        contents
    };

    // parse pom file
    let pom = Pom::from_str(&pom_xml).unwrap();

    let dependencies: Vec<Artifact> = pom.dependencies.value.into_iter().map(|d| d.into()).collect();
    load_artifacts(&dependencies)?;
    Ok(())
}

fn load_verify_artifact_jar(art: &Artifact, artifact_path: &str, local_artifact_jar_path: &str) -> Result<(), Error> {
    let remote_artifact_jar_url = format!("{}/{}/{}-{}.jar", get_config().maven_central, artifact_path, art.name, art.version);

    println!("Downloading {}", remote_artifact_jar_url);
    let jar = reqwest::blocking::get(&remote_artifact_jar_url)?.bytes().unwrap();
    println!("Downloaded {}", remote_artifact_jar_url);
    write_bytes(&local_artifact_jar_path, jar.clone())?;

    let local_artifact_jar_sha1_path = format!("{}.sha1", local_artifact_jar_path);


    // verify jarfile with SHA1 checksum
    let checksum = hex::decode(
        if !exists(&local_artifact_jar_sha1_path) {
            let remote_artifact_jar_sha1_url = format!("{}.sha1", remote_artifact_jar_url);
            println!("{}", remote_artifact_jar_sha1_url);
            let jar_checksum = reqwest::blocking::get(&remote_artifact_jar_sha1_url)?.bytes().unwrap();
            write_bytes(&local_artifact_jar_sha1_path, jar_checksum.clone())?;
            jar_checksum.to_vec()
        } else {
            let mut file = File::open(local_artifact_jar_sha1_path)?;
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)?;
            contents
        })?;

    let mut hasher = Sha1::new();
    hasher.update(&jar.to_vec());
    let result = hasher.finalize();
    let validated = result[..] == checksum;

    if !validated {
        Err(anyhow!("SHA1 checksum for {} is not valid", remote_artifact_jar_url))
    } else { Ok(()) }
}

fn exists(path: &str) -> bool {
    Path::new(path).exists()
}


fn write_bytes(jar_path: &str, bytes: Bytes) -> Result<(), Error> {
    let mut file = File::create(jar_path)?;
    file.write_all(&bytes)?;
    Ok(())
}

fn write_text(path: &str, contents: &String) -> Result<(), Error> {
    let mut file = File::create(path)?;
    file.write(contents.as_bytes())?;
    Ok(())
}
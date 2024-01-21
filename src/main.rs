use anyhow::Error;

use jargo::config::config;

/// sample main that will be replaced by a generic one later
fn main() -> anyhow::Result<(), Error> {
    let repo  = format!("{}/jargo/repo",config().user_home);
    std::fs::remove_dir_all(&repo)?;
    std::fs::create_dir(repo)?;

    let project = jargo::project::load_project(Some("tests/sample_project/Jargo.toml"))?;
    println!("{:?}", project);
    jargo::deploader::load(&project)?;
    jargo::compile::run(&project)?;
    Ok(())
}

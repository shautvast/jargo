use anyhow::Error;

/// sample main that will be replaced by a generic one later
fn main() -> anyhow::Result<(), Error> {
    let project = jargo::load_project(Some("tests/sample_project/Jargo.toml"))?;
    jargo::deploader::load_artifacts(&project.test_dependencies)?;
    jargo::compile::run(&project)?;
    Ok(())
}

use anyhow::Error;

fn main() -> anyhow::Result<(), Error> {
    let project = jargo::load_project(Some("tests/sample_project/Jargo.toml"))?;
    jargo::deploader::load_artifacts(&project.test_dependencies)?;
    Ok(())
}

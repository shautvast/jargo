use anyhow::Error;

/// sample main that will be replaced by a generic one later
fn main() -> anyhow::Result<(), Error> {
    let project = jargo::project::load_project(Some("tests/sample_project/Jargo.toml"))?;
    println!("{:?}", project);
    jargo::deploader::load(&project)?;
    jargo::compile::run(&project)?;
    Ok(())
}

mod cmd;

fn main() -> anyhow::Result<()> {
    cmd::Command::run()?;
    Ok(())
}

mod cmd;
mod git;

fn main() -> anyhow::Result<()> {
    cmd::Command::run()?;
    Ok(())
}

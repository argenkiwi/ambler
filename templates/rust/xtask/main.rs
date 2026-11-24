//! Runs the Ambler CLI that this project depends on, so that `cargo ambler <verb>` works without
//! installing anything: see `.cargo/config.toml`.

#[tokio::main]
async fn main() -> std::process::ExitCode {
    ambler::cli::run(std::env::args().skip(1).collect()).await
}

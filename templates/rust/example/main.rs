//! Runs the `counter` walk; see `specs/counter.md`.

#[tokio::main]
async fn main() -> Result<(), {{crate}}::walks::counter::Error> {
    {{crate}}::walks::counter::run().await
}

mod behavior;
mod ext;
mod introduce;
pub mod manager;
mod test_node;

pub use ext::*;
pub use introduce::*;
pub use test_node::*;
use tracing_subscriber::EnvFilter;

pub fn setup_tracing(dirs: &str, more: bool) {
    let filter = EnvFilter::try_new(dirs).unwrap();
    tracing_subscriber::fmt::fmt()
        .with_target(more)
        .with_file(more)
        .with_line_number(more)
        .with_env_filter(filter)
        .init();
}

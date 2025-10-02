mod introduce;
mod test_node;

pub use introduce::*;
pub use test_node::*;
use tracing_subscriber::EnvFilter;

pub fn setup_tracing(dirs: &str) {
    let filter = EnvFilter::try_new(dirs).unwrap();
    tracing_subscriber::fmt::fmt()
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(filter)
        .init();
}

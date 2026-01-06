mod behavior;
mod introduce;
mod test_node;

pub use introduce::*;
pub use test_node::*;
use tracing_subscriber::EnvFilter;

pub fn setup_tracing(dirs: &str, more: bool) {
    let filter = EnvFilter::try_new(dirs).unwrap();
    tracing_subscriber::fmt::fmt()
        .with_thread_names(false)
        .with_target(more)
        .with_file(more)
        .with_line_number(more)
        .with_env_filter(filter)
        .init();
}

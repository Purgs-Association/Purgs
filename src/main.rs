use purgs::parse;
use tracing_subscriber::EnvFilter;

fn main() {
    // debug logging
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    println!("{:#?}", parse(include_str!("double_top.pug")));
}

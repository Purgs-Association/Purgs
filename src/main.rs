use purgs::parse;
use tracing_subscriber::EnvFilter;

fn main() {
    // debug logging
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let tags = parse(include_str!("tag.pug"));

    println!("{:#?}", tags);
    println!(
        "{}",
        tags.unwrap()
            .into_iter()
            .map(|tag| tag.htmlify())
            .collect::<Vec<_>>()
            .join("")
    );
}

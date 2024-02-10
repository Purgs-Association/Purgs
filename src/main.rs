use purgs::parse;
use tracing::*;
use tracing_subscriber::EnvFilter;

fn main() {
    // debug logging
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_env_filter(
            EnvFilter::builder()
            // default level = info
                .with_default_directive(Level::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let tags = parse(include_str!("tag.pug")).unwrap_or_else(|e| {
        error!("{e}");
        panic!()
    });

    trace!("{:#?}", tags);
    info!(
        "{}",
        tags
            .iter()
            .map(|tag| tag.htmlify())
            .collect::<Vec<_>>()
            .join(""),
    );
}

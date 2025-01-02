#[cfg(not(feature = "tracy"))]
pub fn enable_tracy() {}

#[cfg(feature = "tracy")]
pub fn enable_tracy() {
    use tracing_subscriber::layer::SubscriberExt;

    tracing::subscriber::set_global_default(
        tracing_subscriber::registry().with(tracing_tracy::TracyLayer::default()),
    )
    .ok();
}

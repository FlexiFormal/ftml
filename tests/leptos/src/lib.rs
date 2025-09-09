#![recursion_limit = "256"]

pub mod app;

pub fn filter() -> tracing_subscriber::filter::Targets {
    tracing_subscriber::filter::Targets::new()
        .with_target("ftml_dom", tracing::Level::INFO)
        .with_target("ftml_components", tracing::Level::WARN)
        .with_target("ftml_parser", tracing::Level::INFO)
        .with_target("ftml_backend", tracing::Level::WARN)
        .with_target("ssr_example", tracing::Level::WARN)
        .with_target(
            "leptos_posthoc",
            tracing_subscriber::filter::LevelFilter::ERROR,
        )
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    //let mut config = tracing_wasm::WASMLayerConfigBuilder::new();
    //config.set_max_level(tracing::Level::TRACE);
    //tracing_wasm::set_as_global_default_with_config(config.build());
    //
    use tracing_subscriber::prelude::*;
    let filter = filter();
    tracing_subscriber::registry()
        .with(tracing_wasm::WASMLayer::default())
        .with(filter)
        .init();

    //tracing_wasm::set_as_global_default();

    leptos::mount::hydrate_body(App);
}

#![allow(unexpected_cfgs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
/*!
 * ## Feature flags
 */
#![cfg_attr(doc,doc = document_features::document_features!())]

pub mod backend;
pub mod config;

#[cfg(not(feature = "typescript"))]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn run() {
    use tracing_subscriber::prelude::*;
    fn filter(lvl: config::LogLevel) -> tracing_subscriber::filter::Targets {
        let lvl: tracing::Level = lvl.into();
        tracing_subscriber::filter::Targets::new()
            .with_target("ftml_dom", lvl)
            .with_target("ftml_leptos", lvl)
            .with_target("ftml_core", lvl)
            .with_target("ftml_backend", lvl)
            .with_target("ssr_example", lvl)
            .with_target(
                "leptos_posthoc",
                tracing_subscriber::filter::LevelFilter::ERROR,
            )
    }

    console_error_panic_hook::set_once();

    let meta = ftml_dom::DocumentMeta::get();
    let (mut cfg, errors) = config::parse_config();

    tracing_subscriber::registry()
        .with(tracing_wasm::WASMLayer::default())
        .with(filter(cfg.log_level))
        .init();

    for e in errors {
        tracing::error!("{e}");
    }

    if cfg.inner.document_uri.is_none() {
        cfg.inner.document_uri = meta.uri;
    }

    iterate_body(cfg);
}

/// Activate FTML viewer on the entire body of the page
pub fn iterate_body(cfg: config::FtmlViewerConfig) {
    use ftml_dom::FtmlViews;
    leptos_posthoc::hydrate_body(move |orig| {
        use ftml_uris::DocumentUri;

        let uri = cfg.apply().unwrap_or_else(|| DocumentUri::no_doc().clone());
        ftml_leptos::Views::<backend::GlobalBackend>::top(|| {
            ftml_dom::setup_document(uri, || {
                ftml_leptos::Views::<backend::GlobalBackend>::document(|| {
                    ftml_leptos::Views::<backend::GlobalBackend>::cont(orig, false)
                })
            })
        })
    });
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn print_cache() {
    use ftml_backend::GlobalBackend;
    let uris = ftml_uris::get_memory_state();
    let local_cache = ftml_dom::utils::local_cache::cache_size();
    let remote_cache = backend::GlobalBackend::get().cache_size();
    let total = uris.total_bytes() + local_cache.total_bytes() + remote_cache.total_bytes();
    leptos::logging::log!(
        "Uris: {uris}\nLocal Cache: {local_cache}\nRemote Cache: {remote_cache}\
        \n---------------------\nTotal: {}",
        bytesize::ByteSize::b(total as u64).display().iec_short()
    );
}

/*

#[cfg(feature = "deepsize")]
{
    let uris = ftml_uris::get_memory_state();
    let cache = crate::utils::local_cache::cache_size();
    let total = uris.total_bytes() + cache.total_bytes();
    tracing::warn!(
        "Uris: {uris}\nCache: {cache}\n---------------------\nTotal: {}",
        bytesize::ByteSize::b(total as u64).display().iec_short()
    );
}
 */

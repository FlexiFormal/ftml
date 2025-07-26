use ftml_dom::utils::local_cache::{LocalCache, SendBackend};
use ftml_uris::{DocumentElementUri, LeafUri};
use leptos::prelude::*;

use crate::{config::FtmlConfigState, utils::LocalCacheExt};

pub fn has_notation<
    B: SendBackend,
    V: IntoView + 'static,
    F: FnOnce() -> V + Clone + Send + 'static,
>(
    uri: LeafUri,
    children: F,
) -> impl IntoView + use<V, B, F> + 'static {
    use leptos::either::Either::{Left, Right};
    let notation = FtmlConfigState::notation_preference(&uri);
    move || {
        notation.get().map_or_else(
            || Left((children.clone())()),
            |notation| {
                Right(with_notation::<B, _, _>(
                    uri.clone(),
                    notation,
                    children.clone(),
                ))
            },
        )
    }
}

pub fn with_notation<
    B: SendBackend,
    V: IntoView + 'static,
    F: FnOnce() -> V + Clone + Send + 'static,
>(
    head: LeafUri,
    notation: DocumentElementUri,
    children: F,
) -> impl IntoView {
    LocalCache::with_or_toast::<B, _, _, _, _>(
        |c| c.get_notation(head, notation),
        |n| {
            ftml_core::TODO!();
            ""
        },
        children,
    )
}

#![allow(unexpected_cfgs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
/*!
 * ## Feature flags
 */
#![cfg_attr(doc,doc = document_features::document_features!())]

#[cfg(feature = "callbacks")]
pub mod callbacks;
pub mod components;
pub mod config;
pub mod utils;

use ftml_dom::utils::local_cache::SendBackend;
use std::marker::PhantomData;

pub struct Views<B: SendBackend>(PhantomData<B>);

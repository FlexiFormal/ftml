#![allow(unexpected_cfgs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
/*!
 * ## Feature flags
 */
#![cfg_attr(doc,doc = document_features::document_features!())]

mod keys;
pub use keys::{FtmlKey, NUM_KEYS, PREFIX};
pub mod extraction;
//pub mod keys2;

#[macro_export]
macro_rules! TODO {
    ($($t:tt)*) => {
        todo!($($t)*)
    }; //() => { TODO };
}

pub type NodePath = smallvec::SmallVec<u32, 4>;

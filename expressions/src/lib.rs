#![allow(unexpected_cfgs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
/*!
 * ## Feature flags
 */
#![cfg_attr(doc,doc = document_features::document_features!())]

mod expr;
pub use expr::Expr;
mod variables;
pub use variables::Variable;
mod arguments;
pub use arguments::{Argument, ArgumentMode};

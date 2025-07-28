mod term;
pub use term::Term;
mod variables;
pub use variables::Variable;
mod arguments;
pub mod opaque;
pub use arguments::{Argument, ArgumentMode, BoundArgument};
#[cfg(feature = "openmath")]
pub mod om;

//mod syn;

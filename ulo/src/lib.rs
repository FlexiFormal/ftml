//#![allow(unexpected_cfgs)]
//#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
#![recursion_limit = "512"]
#![doc = include_str!("../README.md")]

/// Reexports for convenience
pub mod rdf_types {
    pub use oxrdf::{
        BlankNode, GraphName, GraphNameRef, Literal, LiteralRef, NamedNode, NamedNodeRef,
        NamedOrBlankNode as Subject, NamedOrBlankNodeRef as SubjectRef, Quad, QuadRef,
        Term as RDFTerm, TermRef as RDFTermRef, Triple, TripleRef, Variable,
    };
}
mod ontologies;
pub use ontologies::{dc, owl, rdf, rdfs, ulo, xsd};

#[macro_export]
macro_rules! triple {
    (<($sub:expr)> $($tt:tt)*) => {
        $crate::triple!(@PRED $crate::rdf_types::Subject::NamedNode($sub); $($tt)*)
    };

    (($sub:expr)! $($tt:tt)*) => {
        $crate::triple!(@PRED $crate::rdf_types::Subject::BlankNode($sub); $($tt)*)
    };

    (@PRED $sub:expr; : $($tt:tt)*) => {
        $crate::triple!(@OBJ $sub;$crate::rdf::TYPE.into_owned(); $($tt)*)
    };
    (@PRED $sub:expr;ulo:$pred:ident $($tt:tt)*) => {
        $crate::triple!(@OBJ $sub;$crate::ulo::$pred.into_owned(); $($tt)*)
    };
    (@PRED $sub:expr;dc:$pred:ident $($tt:tt)*) => {
        $crate::triple!(@OBJ $sub;$crate::dc::$pred.into_owned(); $($tt)*)
    };
    (@PRED $sub:expr;rdfs:$pred:ident $($tt:tt)*) => {
        $crate::triple!(@OBJ $sub;$crate::rdfs::$pred.into_owned(); $($tt)*)
    };

    (@OBJ $sub:expr;$pred:expr; = ($obj:expr) $($tt:tt)*) => {
        $crate::triple!(@MAYBEQUAD $sub;$pred;$crate::rdf_types::RDFTerm::Literal(
            $crate::rdf_types::Literal::new_simple_literal($obj)
        ); $($tt)*)
    };
    (@OBJ $sub:expr;$pred:expr; ulo:$obj:ident $($tt:tt)*) => {
        $crate::triple!(@MAYBEQUAD $sub;$pred;$crate::rdf_types::RDFTerm::NamedNode($crate::ulo::$obj.into_owned()); $($tt)*)
    };
    (@OBJ $sub:expr;$pred:expr; <($obj:expr)> $($tt:tt)*) => {
        $crate::triple!(@MAYBEQUAD $sub;$pred;$crate::rdf_types::RDFTerm::NamedNode($obj); $($tt)*)
    };
    (@OBJ $sub:expr;$pred:expr; ($obj:expr)! $($tt:tt)*) => {
        $crate::triple!(@MAYBEQUAD $sub;$pred;$crate::rdf_types::RDFTerm::BlankNode($obj); $($tt)*)
    };

    (@MAYBEQUAD $sub:expr;$pred:expr;$obj:expr;) => {
        $crate::rdf_types::Triple {
            subject: $sub,
            predicate: $pred,
            object: $obj
        }
    }
}

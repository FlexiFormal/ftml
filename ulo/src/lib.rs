//#![allow(unexpected_cfgs)]
//#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
#![recursion_limit = "512"]
#![doc = include_str!("../README.md")]

/// Reexports for convenience
pub mod rdf_types {
    pub use oxrdf::{
        BlankNode, GraphName, GraphNameRef, Literal, LiteralRef, NamedNode, NamedNodeRef, Quad,
        QuadRef, Subject, SubjectRef, Term as RDFTerm, TermRef as RDFTermRef, Triple, TripleRef,
        Variable,
    };
}
mod ontologies;
pub use ontologies::{dc, owl, rdf, rdfs, ulo, xsd};

#[macro_export]
macro_rules! triple {
    (<($sub:expr)> $($tt:tt)*) => {
        triple!(@PRED $crate::rdf::Subject::NamedNode($sub); $($tt)*)
    };

    (($sub:expr)! $($tt:tt)*) => {
        triple!(@PRED $crate::rdf::Subject::BlankNode($sub); $($tt)*)
    };

    (@PRED $sub:expr; : $($tt:tt)*) => {
        triple!(@OBJ $sub;$crate::rdf::ontologies::rdf::TYPE.into_owned(); $($tt)*)
    };
    (@PRED $sub:expr;ulo:$pred:ident $($tt:tt)*) => {
        triple!(@OBJ $sub;$crate::rdf::ontologies::ulo2::$pred.into_owned(); $($tt)*)
    };
    (@PRED $sub:expr;dc:$pred:ident $($tt:tt)*) => {
        triple!(@OBJ $sub;$crate::rdf::ontologies::dc::$pred.into_owned(); $($tt)*)
    };
    (@PRED $sub:expr;rdfs:$pred:ident $($tt:tt)*) => {
        triple!(@OBJ $sub;$crate::rdf::ontologies::rdfs::$pred.into_owned(); $($tt)*)
    };

    (@OBJ $sub:expr;$pred:expr; = ($obj:expr) $($tt:tt)*) => {
        triple!(@MAYBEQUAD $sub;$pred;$crate::rdf::RDFTerm::Literal(
            $crate::rdf::Literal::new_simple_literal($obj)
        ); $($tt)*)
    };
    (@OBJ $sub:expr;$pred:expr; ulo:$obj:ident $($tt:tt)*) => {
        triple!(@MAYBEQUAD $sub;$pred;$crate::rdf::RDFTerm::NamedNode($crate::rdf::ontologies::ulo2::$obj.into_owned()); $($tt)*)
    };
    (@OBJ $sub:expr;$pred:expr; <($obj:expr)> $($tt:tt)*) => {
        triple!(@MAYBEQUAD $sub;$pred;$crate::rdf::RDFTerm::NamedNode($obj); $($tt)*)
    };
    (@OBJ $sub:expr;$pred:expr; ($obj:expr)! $($tt:tt)*) => {
        triple!(@MAYBEQUAD $sub;$pred;$crate::rdf::RDFTerm::BlankNode($obj); $($tt)*)
    };

    (@MAYBEQUAD $sub:expr;$pred:expr;$obj:expr;) => {
        $crate::rdf::Triple {
            subject: $sub,
            predicate: $pred,
            object: $obj
        }
    }
}

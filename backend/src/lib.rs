#![allow(unexpected_cfgs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
/*!
 * ## Feature flags
 */
#![cfg_attr(doc,doc = document_features::document_features!())]

#[cfg(feature = "cached")]
mod cache;
#[cfg(feature = "wasm")]
mod utils;
#[cfg(feature = "cached")]
pub use cache::*;

#[cfg(any(feature = "wasm", feature = "reqwest"))]
mod remote;
use either::Either;
#[cfg(any(feature = "wasm", feature = "reqwest"))]
pub use remote::*;

pub mod errors;
pub use errors::*;

use ftml_ontology::{
    domain::{
        SharedDeclaration,
        declarations::{
            structures::{MathStructure, StructureExtension},
            symbols::Symbol,
        },
        modules::Module,
    },
    narrative::{
        SharedDocumentElement,
        documents::Document,
        elements::{DocumentTerm, Notation, VariableDeclaration, problems::CognitiveDimension},
    },
    utils::Css,
};
use ftml_uris::{
    DocumentElementUri, DocumentUri, LeafUri, ModuleUri, NarrativeUri, SymbolUri, Uri,
};
use futures_util::{FutureExt, TryFutureExt};

pub const DEFAULT_SERVER_URL: &str = "https://mathhub.info";

#[macro_export]
macro_rules! new_global {
    ($name:ident = $($rest:tt)*) => {
        struct $name;
        impl $crate::GlobalBackend for $name {
            type Error = <$crate::new_global!(@TYPE $($rest)*) as $crate::FtmlBackend>::Error;
            type Backend = $crate::new_global!(@TYPE $($rest)*);
            #[inline]
            fn get() -> &'static Self::Backend {
                static BACKEND: ::std::sync::LazyLock<$crate::new_global!(@TYPE $($rest)*)>
                    = ::std::sync::LazyLock::new(|| $crate::new_global!(@NEW $($rest)*) );
                &BACKEND
            }
        }
    };

    (@TYPE RemoteFlams($val:expr;$tp:ty) [$($rkey:literal = $rval:literal),+;$num:literal] ) => {
        $crate::RemoteFlamsBackend<$tp,[(::ftml_uris::DocumentUri,&'static str);$num]>
    };
    (@NEW RemoteFlams($val:expr;$tp:ty) [$($rkey:literal = $rval:literal),+;$num:literal] ) => {
        $crate::RemoteFlamsBackend::new_with_redirects($val,[$(
            (std::str::FromStr::from_str($rkey).expect("invalid DocumentUri"),$rval)
        ),*])
    };

    (@TYPE RemoteFlams [$($rkey:expr => $rval:expr),+;$num:literal] ) => {
        $crate::RemoteFlamsBackend<&'static str,[(::ftml_uris::DocumentUri,&'static str);$num]>
    };
    (@NEW RemoteFlams [$($rkey:expr => $rval:expr),+;$num:literal] ) => {
        $crate::RemoteFlamsBackend::new_with_redirects($crate::DEFAULT_SERVER_URL,[$(
            (std::str::FromStr::from_str($rkey).expect("invalid DocumentUri"),$rval)
        ),*])
    };

    (@TYPE RemoteFlams($val:expr;$tp:ty) ) => { $crate::RemoteFlamsBackend<$tp,$crate::NoRedirects> };
    (@NEW RemoteFlams($val:expr;$tp:ty) ) => { $crate::RemoteFlamsBackend::new($val) };
    (@TYPE RemoteFlams) => { $crate::RemoteFlamsBackend<&'static str,$crate::NoRedirects> };
    (@NEW RemoteFlams) => { $crate::RemoteFlamsBackend::new($crate::DEFAULT_SERVER_URL) };

    (@TYPE Cached($($rest:tt)*) ) => { $crate::CachedBackend< $crate::new_global!(@TYPE $($rest)*) > };
    (@NEW Cached($($rest:tt)*) ) => { $crate::FtmlBackend::cached($crate::new_global!(@NEW $($rest)*)) };
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(tag = "type")]
pub enum ParagraphOrProblemKind {
    Definition,
    Example,
    Problem(CognitiveDimension),
    SubProblem(CognitiveDimension),
}

pub trait GlobalBackend: 'static {
    type Error: std::fmt::Display + std::fmt::Debug;
    type Backend: FtmlBackend<Error = Self::Error>;
    fn get() -> &'static Self::Backend;
}

pub trait FtmlBackend {
    type Error: std::fmt::Display + std::fmt::Debug;

    #[cfg(feature = "cached")]
    #[inline]
    fn cached(self) -> cache::CachedBackend<Self>
    where
        Self: Sized,
        Self::Error: Clone + Send + Sync,
    {
        cache::CachedBackend::new(self)
    }

    fn document_link_url(&self, uri: &DocumentUri) -> String;

    fn get_fragment(
        &self,
        uri: Uri,
        context: Option<NarrativeUri>,
    ) -> impl Future<Output = Result<(String, Vec<Css>), BackendError<Self::Error>>> + Send;

    fn get_logical_paragraphs(
        &self,
        uri: SymbolUri,
        problems: bool,
    ) -> impl Future<
        Output = Result<
            Vec<(DocumentElementUri, ParagraphOrProblemKind)>,
            BackendError<Self::Error>,
        >,
    > + Send;

    fn get_module(
        &self,
        uri: ModuleUri,
    ) -> impl Future<Output = Result<Module, BackendError<Self::Error>>> + Send;

    fn get_document(
        &self,
        uri: DocumentUri,
    ) -> impl Future<Output = Result<Document, BackendError<Self::Error>>> + Send;

    fn get_symbol(
        &self,
        uri: SymbolUri,
    ) -> impl Future<
        Output = Result<Either<Symbol, SharedDeclaration<Symbol>>, BackendError<Self::Error>>,
    > + Send {
        let uri = uri.simple_module();
        let name = uri.name;
        self.get_module(uri.module).map(move |r| {
            let m = r?;
            m.get_as(&name).map_or_else(
                || Err(BackendError::NotFound(ftml_uris::UriKind::Symbol)),
                |d| Ok(Either::Right(d)),
            )
        })
    }

    #[allow(clippy::type_complexity)]
    fn get_structure(
        &self,
        uri: SymbolUri,
    ) -> impl Future<
        Output = Result<
            Either<SharedDeclaration<MathStructure>, SharedDeclaration<StructureExtension>>,
            BackendError<Self::Error>,
        >,
    > + Send {
        let uri = uri.simple_module();
        let name = uri.name;
        self.get_module(uri.module).map(move |r| {
            let m = r?;
            m.get_as(&name).map_or_else(
                || {
                    m.get_as(&name).map_or_else(
                        || Err(BackendError::NotFound(ftml_uris::UriKind::Symbol)),
                        |d| Ok(Either::Right(d)),
                    )
                },
                |d| Ok(Either::Left(d)),
            )
        })
    }

    fn get_variable(
        &self,
        uri: DocumentElementUri,
    ) -> impl Future<
        Output = Result<
            Either<VariableDeclaration, SharedDocumentElement<VariableDeclaration>>,
            BackendError<Self::Error>,
        >,
    > + Send {
        let name = uri.name;
        self.get_document(uri.document).map(move |r| {
            let m = r?;
            m.get_as(&name).map_or_else(
                || Err(BackendError::NotFound(ftml_uris::UriKind::DocumentElement)),
                |d| Ok(Either::Right(d)),
            )
        })
    }

    fn get_document_term(
        &self,
        uri: DocumentElementUri,
    ) -> impl Future<
        Output = Result<
            Either<DocumentTerm, SharedDocumentElement<DocumentTerm>>,
            BackendError<Self::Error>,
        >,
    > + Send {
        let name = uri.name;
        self.get_document(uri.document).map(move |r| {
            let m = r?;
            m.get_as(&name).map_or_else(
                || Err(BackendError::NotFound(ftml_uris::UriKind::DocumentElement)),
                |d| Ok(Either::Right(d)),
            )
        })
    }

    #[inline]
    fn get_definition(
        &self,
        uri: SymbolUri,
        context: Option<NarrativeUri>,
    ) -> impl Future<Output = Result<(String, Vec<Css>), BackendError<Self::Error>>> + Send {
        self.get_fragment(uri.into(), context)
    }

    #[inline]
    fn get_document_html(
        &self,
        uri: DocumentUri,
        context: Option<NarrativeUri>,
    ) -> impl Future<Output = Result<(String, Vec<Css>), BackendError<Self::Error>>> + Send {
        self.get_fragment(uri.into(), context)
    }

    fn get_notations(
        &self,
        uri: LeafUri,
    ) -> impl Future<Output = Result<Vec<(DocumentElementUri, Notation)>, BackendError<Self::Error>>>
    + Send;

    fn get_notation(
        &self,
        symbol: LeafUri,
        uri: DocumentElementUri,
    ) -> impl Future<Output = Result<Notation, BackendError<Self::Error>>> + Send {
        self.get_notations(symbol).map_ok_or_else(Err, move |r| {
            r.into_iter()
                .find(|(u, _)| *u == uri)
                .map(|(_, n)| n)
                .ok_or(BackendError::NotFound(ftml_uris::UriKind::DocumentElement))
        })
    }
}

#[cfg(feature = "server_fn")]
pub trait FlamsBackend {
    fn document_link_url(&self, uri: &DocumentUri) -> String;

    /// `/content/fragment`
    #[allow(clippy::too_many_arguments)]
    fn get_fragment(
        &self,
        uri: Option<Uri>,
        rp: Option<String>,
        a: Option<ftml_uris::ArchiveId>,
        p: Option<String>,
        d: Option<String>,
        m: Option<String>,
        l: Option<ftml_uris::Language>,
        e: Option<String>,
        s: Option<String>,
        context: Option<NarrativeUri>,
    ) -> impl Future<
        Output = Result<(Uri, Vec<Css>, String), BackendError<server_fn::error::ServerFnErrorErr>>,
    > + Send;

    /// `/content/module`
    #[allow(clippy::too_many_arguments)]
    fn get_module(
        &self,
        uri: Option<ModuleUri>,
        a: Option<ftml_uris::ArchiveId>,
        p: Option<String>,
        m: Option<String>,
    ) -> impl Future<Output = Result<Module, BackendError<server_fn::error::ServerFnErrorErr>>> + Send;

    /// `/content/document`
    #[allow(clippy::too_many_arguments)]
    fn get_document(
        &self,
        uri: Option<DocumentUri>,
        rp: Option<String>,
        a: Option<ftml_uris::ArchiveId>,
        p: Option<String>,
        d: Option<String>,
        l: Option<ftml_uris::Language>,
    ) -> impl Future<Output = Result<Document, BackendError<server_fn::error::ServerFnErrorErr>>> + Send;

    /// `/content/notations`
    #[allow(clippy::too_many_arguments)]
    fn get_notations(
        &self,
        uri: Option<Uri>,
        rp: Option<String>,
        a: Option<ftml_uris::ArchiveId>,
        p: Option<String>,
        d: Option<String>,
        m: Option<String>,
        l: Option<ftml_uris::Language>,
        e: Option<String>,
        s: Option<String>,
    ) -> impl Future<
        Output = Result<
            Vec<(DocumentElementUri, Notation)>,
            BackendError<server_fn::error::ServerFnErrorErr>,
        >,
    > + Send;

    /// `/content/los`
    #[allow(clippy::too_many_arguments)]
    fn get_logical_paragraphs(
        &self,
        uri: Option<SymbolUri>,
        a: Option<ftml_uris::ArchiveId>,
        p: Option<String>,
        m: Option<String>,
        s: Option<String>,
        problems: bool,
    ) -> impl Future<
        Output = Result<
            Vec<(DocumentElementUri, ParagraphOrProblemKind)>,
            BackendError<server_fn::error::ServerFnErrorErr>,
        >,
    > + Send;
}

#[cfg(feature = "server_fn")]
impl<FB> FtmlBackend for FB
where
    FB: FlamsBackend,
{
    type Error = server_fn::error::ServerFnErrorErr;

    #[inline]
    fn document_link_url(&self, uri: &DocumentUri) -> String {
        <Self as FlamsBackend>::document_link_url(self, uri)
    }

    fn get_fragment(
        &self,
        uri: Uri,
        context: Option<NarrativeUri>,
    ) -> impl Future<Output = Result<(String, Vec<Css>), BackendError<Self::Error>>> + Send {
        <Self as FlamsBackend>::get_fragment(
            self,
            Some(uri),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            context,
        )
        .map(|r| r.map(|(_, css, s)| (s, css)))
    }

    #[inline]
    fn get_module(
        &self,
        uri: ModuleUri,
    ) -> impl Future<Output = Result<Module, BackendError<Self::Error>>> + Send {
        <Self as FlamsBackend>::get_module(self, Some(uri), None, None, None)
    }

    #[inline]
    fn get_document(
        &self,
        uri: DocumentUri,
    ) -> impl Future<Output = Result<Document, BackendError<Self::Error>>> + Send {
        <Self as FlamsBackend>::get_document(self, Some(uri), None, None, None, None, None)
    }

    #[inline]
    fn get_notations(
        &self,
        uri: LeafUri,
    ) -> impl Future<Output = Result<Vec<(DocumentElementUri, Notation)>, BackendError<Self::Error>>>
    + Send {
        <Self as FlamsBackend>::get_notations(
            self,
            Some(uri.into()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
    }

    #[inline]
    fn get_logical_paragraphs(
        &self,
        uri: SymbolUri,
        problems: bool,
    ) -> impl Future<
        Output = Result<
            Vec<(DocumentElementUri, ParagraphOrProblemKind)>,
            BackendError<Self::Error>,
        >,
    > + Send {
        <Self as FlamsBackend>::get_logical_paragraphs(
            self,
            Some(uri),
            None,
            None,
            None,
            None,
            problems,
        )
    }
}

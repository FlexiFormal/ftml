#![allow(unexpected_cfgs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_cfg))]
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
            morphisms::Morphism,
            structures::{MathStructure, StructureExtension},
            symbols::Symbol,
        },
        modules::Module,
    },
    narrative::{
        SharedDocumentElement,
        documents::{Document, TocElem},
        elements::{
            DocumentTerm, Notation, ParagraphOrProblemKind, VariableDeclaration,
            problems::Solutions,
        },
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
    (@TYPE RemoteFlams [$($rkey:expr => $rval:expr),+;$num:literal] ) => {
        $crate::RemoteFlamsBackend<&'static str,[(::ftml_uris::DocumentUri,&'static str);$num]>
    };
    (@TYPE RemoteFlams($val:expr;$tp:ty) ) => { $crate::RemoteFlamsBackend<$tp,$crate::NoRedirects> };
    (@TYPE RemoteFlams) => { $crate::RemoteFlamsBackend<&'static str,$crate::NoRedirects> };

    (@NEW RemoteFlams($val:expr;$tp:ty) [$($rkey:literal = $rval:literal),+;$num:literal] ) => {
        $crate::RemoteFlamsBackend::new_with_redirects($val,[$(
            (std::str::FromStr::from_str($rkey).expect("invalid DocumentUri"),$rval)
        ),*],true)
    };
    (@NEW RemoteFlams($val:expr;$tp:ty) ) => { $crate::RemoteFlamsBackend::new($val,true) };
    (@NEW RemoteFlams) => { $crate::RemoteFlamsBackend::new($crate::DEFAULT_SERVER_URL,true) };
    (@NEW RemoteFlams [$($rkey:expr => $rval:expr),+;$num:literal] ) => {
        $crate::RemoteFlamsBackend::new_with_redirects($crate::DEFAULT_SERVER_URL,[$(
            (std::str::FromStr::from_str($rkey).expect("invalid DocumentUri"),$rval)
        ),*],true)
    };

    (@TYPE RemoteFlamsLike($val:expr;$tp:ty) [$($rkey:literal = $rval:literal),+;$num:literal] ) => {
        $crate::RemoteFlamsBackend<$tp,[(::ftml_uris::DocumentUri,&'static str);$num]>
    };
    (@TYPE RemoteFlamsLike [$($rkey:expr => $rval:expr),+;$num:literal] ) => {
        $crate::RemoteFlamsBackend<&'static str,[(::ftml_uris::DocumentUri,&'static str);$num]>
    };
    (@TYPE RemoteFlamsLike($val:expr;$tp:ty) ) => { $crate::RemoteFlamsBackend<$tp,$crate::NoRedirects> };
    (@TYPE RemoteFlamsLike) => { $crate::RemoteFlamsBackend<&'static str,$crate::NoRedirects> };

    (@NEW RemoteFlamsLike($val:expr;$tp:ty) ) => { $crate::RemoteFlamsBackend::new($val,false) };
    (@NEW RemoteFlamsLike [$($rkey:expr => $rval:expr),+;$num:literal] ) => {
        $crate::RemoteFlamsBackend::new_with_redirects($crate::DEFAULT_SERVER_URL,[$(
            (std::str::FromStr::from_str($rkey).expect("invalid DocumentUri"),$rval)
        ),*],false)
    };
    (@NEW RemoteFlamsLike($val:expr;$tp:ty) [$($rkey:literal = $rval:literal),+;$num:literal] ) => {
        $crate::RemoteFlamsBackend::new_with_redirects($val,[$(
            (std::str::FromStr::from_str($rkey).expect("invalid DocumentUri"),$rval)
        ),*],false)
    };
    (@NEW RemoteFlamsLike) => { $crate::RemoteFlamsBackend::new($crate::DEFAULT_SERVER_URL,false) };




    (@TYPE Cached($($rest:tt)*) ) => { $crate::CachedBackend< $crate::new_global!(@TYPE $($rest)*) > };
    (@NEW Cached($($rest:tt)*) ) => { $crate::FtmlBackend::cached($crate::new_global!(@NEW $($rest)*)) };
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
    fn resource_link_url(&self, uri: &DocumentUri, kind: &'static str) -> Option<String>;

    fn get_fragment(
        &self,
        uri: Uri,
        context: Option<NarrativeUri>,
    ) -> impl Future<Output = Result<(Box<str>, Box<[Css]>, bool), BackendError<Self::Error>>> + Send;

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

    fn get_toc(
        &self,
        uri: DocumentUri,
    ) -> impl Future<Output = Result<(Box<[Css]>, Box<[TocElem]>), BackendError<Self::Error>>> + Send;

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

    fn get_morphism(
        &self,
        uri: SymbolUri,
    ) -> impl Future<
        Output = Result<Either<Morphism, SharedDeclaration<Morphism>>, BackendError<Self::Error>>,
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
    #[allow(async_fn_in_trait)]
    async fn get_definition(
        &self,
        uri: SymbolUri,
        context: Option<NarrativeUri>,
    ) -> Result<(Box<str>, Box<[Css]>), BackendError<Self::Error>> {
        let (a, b, _) = self.get_fragment(uri.into(), context).await?;
        Ok((a, b))
    }

    fn get_document_html(
        &self,
        uri: DocumentUri,
        context: Option<NarrativeUri>,
    ) -> impl Future<Output = Result<(Box<str>, Box<[Css]>, bool), BackendError<Self::Error>>> + Send;

    fn get_solutions(
        &self,
        uri: DocumentElementUri,
    ) -> impl Future<Output = Result<Solutions, BackendError<Self::Error>>> + Send;

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
    fn resource_link_url(&self, uri: &DocumentUri, kind: &'static str) -> Option<String>;
    fn stripped(&self) -> bool;

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
        Output = Result<
            (Uri, Box<[Css]>, Box<str>),
            BackendError<server_fn::error::ServerFnErrorErr>,
        >,
    > + Send;

    /// `/content/document`
    #[allow(clippy::too_many_arguments)]
    fn get_document_html(
        &self,
        uri: Option<DocumentUri>,
        rp: Option<String>,
        a: Option<ftml_uris::ArchiveId>,
        p: Option<String>,
        d: Option<String>,
        l: Option<ftml_uris::Language>,
    ) -> impl Future<
        Output = Result<
            (DocumentUri, Box<[Css]>, Box<str>),
            BackendError<server_fn::error::ServerFnErrorErr>,
        >,
    > + Send;

    fn get_toc(
        &self,
        uri: Option<DocumentUri>,
        rp: Option<String>,
        a: Option<ftml_uris::ArchiveId>,
        p: Option<String>,
        d: Option<String>,
        l: Option<ftml_uris::Language>,
    ) -> impl Future<
        Output = Result<
            (Box<[Css]>, Box<[TocElem]>),
            BackendError<server_fn::error::ServerFnErrorErr>,
        >,
    > + Send;

    /// `/domain/module`
    #[allow(clippy::too_many_arguments)]
    fn get_module(
        &self,
        uri: Option<ModuleUri>,
        a: Option<ftml_uris::ArchiveId>,
        p: Option<String>,
        m: Option<String>,
    ) -> impl Future<Output = Result<Module, BackendError<server_fn::error::ServerFnErrorErr>>> + Send;

    /// `/domain/document`
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

    /// `/content/solution`
    fn get_solutions(
        &self,
        uri: DocumentElementUri,
    ) -> impl Future<Output = Result<Solutions, BackendError<server_fn::error::ServerFnErrorErr>>> + Send;

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
    #[inline]
    fn resource_link_url(&self, uri: &DocumentUri, kind: &'static str) -> Option<String> {
        <Self as FlamsBackend>::resource_link_url(self, uri, kind)
    }

    fn get_fragment(
        &self,
        uri: Uri,
        context: Option<NarrativeUri>,
    ) -> impl Future<Output = Result<(Box<str>, Box<[Css]>, bool), BackendError<Self::Error>>> + Send
    {
        let stripped = self.stripped();
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
        .map(move |r| r.map(|(_, css, s)| (s, css, stripped)))
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
    fn get_toc(
        &self,
        uri: DocumentUri,
    ) -> impl Future<Output = Result<(Box<[Css]>, Box<[TocElem]>), BackendError<Self::Error>>> + Send
    {
        <Self as FlamsBackend>::get_toc(self, Some(uri), None, None, None, None, None)
    }

    #[inline]
    fn get_document_html(
        &self,
        uri: DocumentUri,
        _context: Option<NarrativeUri>,
    ) -> impl Future<Output = Result<(Box<str>, Box<[Css]>, bool), BackendError<Self::Error>>> + Send
    {
        let stripped = self.stripped();
        let fut = <Self as FlamsBackend>::get_document_html(
            self,
            Some(uri),
            None,
            None,
            None,
            None,
            None,
        );
        async move {
            let r = fut.await?;
            Ok((r.2, r.1, stripped))
        }
    }

    fn get_solutions(
        &self,
        uri: DocumentElementUri,
    ) -> impl Future<Output = Result<Solutions, BackendError<Self::Error>>> + Send {
        <Self as FlamsBackend>::get_solutions(self, uri)
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

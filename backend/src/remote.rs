use std::marker::PhantomData;
use std::str::FromStr;

use ftml_ontology::{narrative::elements::SectionLevel, utils::Css};
use ftml_uris::{DocumentUri, FtmlUri, LeafUri, ModuleUri, NarrativeUri, SymbolUri, Uri};

use crate::BackendError;

pub trait Redirects {
    #[inline]
    fn for_fragment<'s>(&'s self, _uri: &DocumentUri) -> Option<impl std::fmt::Display + 's> {
        None::<&str>
    }
    #[inline]
    fn for_document_html<'s>(&'s self, _uri: &DocumentUri) -> Option<impl std::fmt::Display + 's> {
        None::<&str>
    }
    #[inline]
    fn for_notations<'s>(&'s self, _uri: &LeafUri) -> Option<impl std::fmt::Display + 's> {
        None::<&str>
    }
    #[inline]
    fn for_paragraphs<'s>(
        &'s self,
        _uri: &SymbolUri,
        _problems: bool,
    ) -> Option<impl std::fmt::Display + 's> {
        None::<&str>
    }
    fn for_modules<'s>(&'s self, _uri: &ModuleUri) -> Option<impl std::fmt::Display + 's> {
        None::<&str>
    }
    fn for_documents<'s>(&'s self, _uri: &DocumentUri) -> Option<impl std::fmt::Display + 's> {
        None::<&str>
    }
}
pub struct NoRedirects;
impl Redirects for NoRedirects {}
impl<const LEN: usize, D: std::fmt::Display> Redirects for [(DocumentUri, D); LEN] {
    fn for_fragment<'s>(&'s self, uri: &DocumentUri) -> Option<impl std::fmt::Display + 's> {
        self.iter()
            .find_map(|(u, d)| if *u == *uri { Some(d) } else { None })
    }
}

pub struct RemoteBackend<
    E: std::fmt::Display + std::fmt::Debug,
    Url: std::fmt::Display = &'static str,
    Re: Redirects = NoRedirects,
> {
    pub check_url: Url,
    pub fragment_url: Url,
    pub document_html_url: Url,
    pub solutions_url: Url,
    pub notations_url: Url,
    pub paragraphs_url: Url,
    pub modules_url: Url,
    pub documents_url: Url,
    pub toc_url: Url,
    pub resources_url: Option<Url>,
    pub redirects: Re,
    __phantom: PhantomData<E>,
}

#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("error during request: {0}")]
    Request(String),
    #[error("error deserializing response: {0}")]
    Deserialization(String),
}

#[cfg(feature = "serde-lite")]
impl From<serde_lite::Error> for RequestError {
    fn from(value: serde_lite::Error) -> Self {
        Self::Deserialization(value.to_string())
    }
}

impl<Url, E, Re: Redirects> RemoteBackend<E, Url, Re>
where
    Url: std::fmt::Display,
    E: std::fmt::Display + std::fmt::Debug + From<RequestError>,
{
    pub const fn new_with_redirects(
        fragment_url: Url,
        document_html_url: Url,
        notations_url: Url,
        solutions_url: Url,
        paragraphs_url: Url,
        modules_url: Url,
        documents_url: Url,
        toc_url: Url,
        check_url: Url,
        redirects: Re,
    ) -> Self {
        Self {
            check_url,
            fragment_url,
            document_html_url,
            notations_url,
            paragraphs_url,
            modules_url,
            solutions_url,
            documents_url,
            toc_url,
            resources_url: None,
            redirects,
            __phantom: PhantomData,
        }
    }
}

impl<Url, E> RemoteBackend<E, Url>
where
    Url: std::fmt::Display,
    E: std::fmt::Display + std::fmt::Debug + From<RequestError>,
{
    pub const fn new(
        fragment_url: Url,
        document_html_url: Url,
        notations_url: Url,
        solutions_url: Url,
        paragraphs_url: Url,
        modules_url: Url,
        documents_url: Url,
        toc_url: Url,
        check_url: Url,
    ) -> Self {
        Self {
            check_url,
            fragment_url,
            document_html_url,
            notations_url,
            paragraphs_url,
            solutions_url,
            modules_url,
            documents_url,
            toc_url,
            resources_url: None,
            redirects: NoRedirects,
            __phantom: PhantomData,
        }
    }
}

impl<Url, E, Re: Redirects> RemoteBackend<E, Url, Re>
where
    Url: std::fmt::Display,
    E: std::fmt::Display + std::fmt::Debug + From<RequestError> + std::str::FromStr,
    E::Err: Into<BackendError<E>>,
{
    fn make_url<D: std::fmt::Display>(
        base: D,
        uri: &Uri,
        context: Option<&NarrativeUri>,
    ) -> String {
        context.map_or_else(
            || format!("{}?uri={}", base, uri.url_encoded()),
            |ctx| {
                format!(
                    "{}?uri={}&context={}",
                    base,
                    uri.url_encoded(),
                    ctx.url_encoded()
                )
            },
        )
    }
}

impl<Url, E, Re: Redirects> super::FtmlBackend for RemoteBackend<E, Url, Re>
where
    Url: std::fmt::Display,
    E: std::fmt::Display + std::fmt::Debug + From<RequestError> + std::str::FromStr + Send,
    E::Err: Into<BackendError<E>>,
{
    type Error = E;

    fn check_term(
        &self,
        global_context: &[ModuleUri],
        term: &ftml_ontology::terms::Term,
        in_path: &ftml_ontology::terms::termpaths::TermPath,
    ) -> impl Future<Output = Result<crate::BackendCheckResult, BackendError<Self::Error>>>
    + Send
    + use<Url, E, Re> {
        fn body(
            global_context: &[ModuleUri],
            term: &ftml_ontology::terms::Term,
            in_path: &ftml_ontology::terms::termpaths::TermPath,
        ) -> Result<String, serde_json::Error> {
            Ok(format!(
                "{{\"global_context\":{},\"term\":{},\"in_path\":{}}}",
                serde_json::to_string(global_context)?,
                serde_json::to_string(term)?,
                serde_json::to_string(in_path)?
            ))
        }
        let url = self.check_url.to_string();
        let body = match body(global_context, term, in_path) {
            Ok(body) => body,
            Err(e) => {
                return futures_util::future::Either::Right(std::future::ready(Err(
                    BackendError::ToDo(e.to_string()),
                )));
            }
        };
        futures_util::future::Either::Left(post(url, body))
    }

    fn document_link_url(&self, uri: &DocumentUri) -> String {
        self.redirects.for_documents(uri).map_or_else(
            || format!("{}?uri={}", self.documents_url, uri.url_encoded()),
            |r| r.to_string(),
        )
    }
    fn resource_link_url(&self, uri: &DocumentUri, kind: &'static str) -> Option<String> {
        self.resources_url
            .as_ref()
            .map(|s| format!("{s}?uri={}&format={kind}", uri.url_encoded()))
    }

    #[allow(clippy::similar_names)]
    fn get_fragment(
        &self,
        uri: ftml_uris::Uri,
        context: Option<NarrativeUri>,
    ) -> impl Future<Output = Result<(Box<str>, Box<[Css]>, bool), BackendError<E>>> {
        let url = if let Uri::Document(d) = &uri {
            self.redirects.for_fragment(d).map_or_else(
                || Self::make_url(&self.fragment_url, &uri, context.as_ref()),
                |r| r.to_string(),
            )
        } else {
            Self::make_url(&self.fragment_url, &uri, context.as_ref())
        };
        call(url)
    }

    #[allow(clippy::similar_names)]
    fn get_document_html(
        &self,
        uri: DocumentUri,
        context: Option<NarrativeUri>,
    ) -> impl Future<Output = Result<(Box<str>, Box<[Css]>, bool), BackendError<Self::Error>>> + Send
    {
        let url = self.redirects.for_document_html(&uri).map_or_else(
            || Self::make_url(&self.document_html_url, &uri.into(), context.as_ref()),
            |r| r.to_string(),
        );
        call(url)
    }

    #[allow(clippy::similar_names)]
    fn get_toc(
        &self,
        uri: DocumentUri,
    ) -> impl Future<
        Output = Result<
            (
                Box<[Css]>,
                SectionLevel,
                Box<[ftml_ontology::narrative::documents::TocElem]>,
            ),
            BackendError<Self::Error>,
        >,
    > + Send {
        let url = Self::make_url(&self.toc_url, &uri.into(), None);
        call(url)
    }

    #[allow(clippy::similar_names)]
    fn get_solutions(
        &self,
        uri: ftml_uris::DocumentElementUri,
    ) -> impl Future<
        Output = Result<
            ftml_ontology::narrative::elements::problems::Solutions,
            BackendError<Self::Error>,
        >,
    > + Send {
        let url = Self::make_url(&self.solutions_url, &uri.into(), None);
        call(url)
    }

    #[allow(clippy::similar_names)]
    fn get_module(
        &self,
        uri: ModuleUri,
    ) -> impl Future<
        Output = Result<ftml_ontology::domain::modules::ModuleLike, BackendError<Self::Error>>,
    > {
        let url = self.redirects.for_modules(&uri).map_or_else(
            || Self::make_url(&self.modules_url, &uri.into(), None),
            |r| r.to_string(),
        );
        call(url)
    }

    #[allow(clippy::similar_names)]
    fn get_document(
        &self,
        uri: DocumentUri,
    ) -> impl Future<
        Output = Result<ftml_ontology::narrative::documents::Document, BackendError<Self::Error>>,
    > {
        let url = self.redirects.for_documents(&uri).map_or_else(
            || Self::make_url(&self.documents_url, &uri.into(), None),
            |r| r.to_string(),
        );
        call(url)
    }

    #[allow(clippy::similar_names)]
    fn get_notations(
        &self,
        uri: ftml_uris::LeafUri,
    ) -> impl Future<
        Output = Result<
            Vec<(
                ftml_uris::DocumentElementUri,
                ftml_ontology::narrative::elements::Notation,
            )>,
            BackendError<Self::Error>,
        >,
    > {
        let url = self.redirects.for_notations(&uri).map_or_else(
            || Self::make_url(&self.notations_url, &uri.into(), None),
            |r| r.to_string(),
        );
        call(url)
    }

    #[allow(clippy::similar_names)]
    fn get_logical_paragraphs(
        &self,
        uri: SymbolUri,
        problems: bool,
    ) -> impl Future<
        Output = Result<
            Vec<(ftml_uris::DocumentElementUri, crate::ParagraphOrProblemKind)>,
            BackendError<Self::Error>,
        >,
    > {
        let url = self.redirects.for_paragraphs(&uri, problems).map_or_else(
            || Self::make_url(&self.paragraphs_url, &uri.into(), None),
            |r| r.to_string(),
        );
        call(url)
    }
}

#[cfg(feature = "server_fn")]
pub struct RemoteFlamsBackend<Url: std::fmt::Display, Re: Redirects = NoRedirects> {
    pub url: Url,
    pub redirects: Re,
    pub stripped: bool,
}

#[cfg(feature = "server_fn")]
impl<Url, Re: Redirects> RemoteFlamsBackend<Url, Re>
where
    Url: std::fmt::Display,
{
    pub const fn new_with_redirects(url: Url, redirects: Re, stripped: bool) -> Self {
        Self {
            url,
            redirects,
            stripped,
        }
    }
}

#[cfg(feature = "server_fn")]
impl<Url: std::fmt::Display> RemoteFlamsBackend<Url> {
    pub const fn new(url: Url, stripped: bool) -> Self {
        Self {
            url,
            redirects: NoRedirects,
            stripped,
        }
    }
}

#[cfg(feature = "server_fn")]
mod server_fn {
    use crate::{
        BackendError, FlamsBackend, ParagraphOrProblemKind, Redirects, RemoteFlamsBackend,
    };
    use ::server_fn::error::ServerFnErrorErr;
    use ftml_ontology::{
        narrative::elements::{SectionLevel, problems::Solutions},
        utils::Css,
    };
    use ftml_uris::{
        DocumentElementUri, DocumentUri, FtmlUri, LeafUri, ModuleUri, NarrativeUri, Uri,
        components::UriComponentTuple,
    };
    use futures_util::TryFutureExt;

    impl<Url: std::fmt::Display, Re: Redirects> FlamsBackend for RemoteFlamsBackend<Url, Re> {
        #[inline]
        fn stripped(&self) -> bool {
            self.stripped
        }
        fn document_link_url(&self, uri: &DocumentUri) -> String {
            self.redirects.for_documents(uri).map_or_else(
                || format!("{}?uri={}", self.url, uri.url_encoded()),
                |r| r.to_string(),
            )
        }
        fn resource_link_url(&self, uri: &DocumentUri, kind: &'static str) -> Option<String> {
            Some(format!(
                "{}/doc?uri={}&format={kind}",
                self.url,
                uri.url_encoded()
            ))
        }

        fn check_term(
            &self,
            global_context: &[ftml_uris::ModuleUri],
            term: &ftml_ontology::terms::Term,
            in_path: &ftml_ontology::terms::termpaths::TermPath,
        ) -> impl Future<
            Output = Result<crate::BackendCheckResult, BackendError<ServerFnErrorErr>>,
        > + Send
        + use<Url, Re> {
            fn body(
                global_context: &[ModuleUri],
                term: &ftml_ontology::terms::Term,
                in_path: &ftml_ontology::terms::termpaths::TermPath,
            ) -> Result<String, serde_json::Error> {
                Ok(format!(
                    "{{\"global_context\":{},\"term\":{},\"in_path\":{}}}",
                    serde_json::to_string(global_context)?,
                    serde_json::to_string(term)?,
                    serde_json::to_string(in_path)?
                ))
            }
            let url = format!("{}/content/check_term", self.url);
            let body = match body(global_context, term, in_path) {
                Ok(body) => body,
                Err(e) => {
                    return futures_util::future::Either::Right(std::future::ready(Err(
                        BackendError::ToDo(e.to_string()),
                    )));
                }
            };
            futures_util::future::Either::Left(
                super::post::<_, SFnE>(url, body).map_err(BackendError::from_other),
            )
        }

        ftml_uris::compfun! {!!
            #[allow(clippy::similar_names)]
            fn get_fragment(&self,uri:Uri,context:Option<NarrativeUri>) -> impl Future<Output=Result<(Uri, Box<[Css]>,Box<str>), BackendError<ServerFnErrorErr>>> {
                fn make_url<D: std::fmt::Display>(
                    base: D,
                    uri: &UriComponentTuple,
                    context: Option<&NarrativeUri>,
                ) -> String {
                    context.map_or_else(
                        || format!("{}/content/fragment{}", base, uri.as_query()),
                        |ctx| {
                            format!(
                                "{}/content/fragment{}&context={}",
                                base,
                                uri.as_query(),
                                ctx.url_encoded()
                            )
                        },
                    )
                }

                let url = if let Some(Uri::Document(d)) = uri.uri.as_ref() {
                    self.redirects.for_fragment(d).map_or_else(
                        || make_url(&self.url, &uri, context.as_ref()),
                        |r| r.to_string(),
                    )
                } else {
                    make_url(&self.url, &uri, context.as_ref())
                };
                super::call::<_,SFnE>(url).map_err(BackendError::from_other)
            }
        }

        #[allow(clippy::similar_names)]
        fn get_solutions(
            &self,
            uri: DocumentElementUri,
        ) -> impl Future<
            Output = Result<
                ftml_ontology::narrative::elements::problems::Solutions,
                BackendError<ServerFnErrorErr>,
            >,
        > + Send {
            let url = format!("{}/content/solution?uri={}", &self.url, uri.url_encoded());
            async move {
                let s = super::call::<String, SFnE>(url)
                    .await
                    .map_err(BackendError::from_other)?;
                //tracing::error!("Solution string: {s}");
                let r = Solutions::from_jstring(&s)
                    .ok_or_else(|| BackendError::ToDo("illegal solution string".to_string()));
                //tracing::error!("Result: {r:#?}");
                r
            }
        }

        #[allow(clippy::similar_names)]
        #[allow(clippy::many_single_char_names)]
        #[allow(clippy::useless_let_if_seq)]
        fn get_document_html(
            &self,
            uri: Option<DocumentUri>,
            rp: Option<String>,
            a: Option<ftml_uris::ArchiveId>,
            p: Option<String>,
            d: Option<String>,
            l: Option<ftml_uris::Language>,
        ) -> impl Future<
            Output = Result<(DocumentUri, Box<[Css]>, Box<str>), BackendError<ServerFnErrorErr>>,
        > {
            use std::fmt::Write;
            if let Some(uri) = &uri
                && let Some(url) = self.redirects.for_document_html(uri)
            {
                return super::call::<_, SFnE>(url.to_string()).map_err(BackendError::from_other);
            }
            let url = {
                let mut s = String::with_capacity(64);
                let _ = write!(&mut s, "{}/content/document", &self.url);
                let mut sep = '?';
                if let Some(uri) = uri {
                    let _ = write!(&mut s, "?uri={}", uri.url_encoded());
                    sep = '&';
                }
                if let Some(rp) = rp {
                    let _ = write!(&mut s, "{sep}rp={rp}");
                    sep = '&';
                }
                if let Some(a) = a {
                    let _ = write!(&mut s, "{sep}a={a}");
                    sep = '&';
                }
                if let Some(p) = p {
                    let _ = write!(&mut s, "{sep}p={p}");
                    sep = '&';
                }
                if let Some(d) = d {
                    let _ = write!(&mut s, "{sep}d={d}");
                }
                if let Some(l) = l {
                    let _ = write!(&mut s, "{sep}l={l}");
                }
                s
            };
            super::call::<_, SFnE>(url).map_err(BackendError::from_other)
        }

        #[allow(clippy::similar_names)]
        #[allow(clippy::many_single_char_names)]
        #[allow(clippy::useless_let_if_seq)]
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
                (
                    Box<[Css]>,
                    SectionLevel,
                    Box<[ftml_ontology::narrative::documents::TocElem]>,
                ),
                BackendError<server_fn::error::ServerFnErrorErr>,
            >,
        > + Send {
            use std::fmt::Write;
            if let Some(uri) = &uri
                && let Some(url) = self.redirects.for_document_html(uri)
            {
                return super::call::<_, SFnE>(url.to_string()).map_err(BackendError::from_other);
            }
            let url = {
                let mut s = String::with_capacity(64);
                let _ = write!(&mut s, "{}/content/toc", &self.url);
                let mut sep = '?';
                if let Some(uri) = uri {
                    let _ = write!(&mut s, "?uri={}", uri.url_encoded());
                    sep = '&';
                }
                if let Some(rp) = rp {
                    let _ = write!(&mut s, "{sep}rp={rp}");
                    sep = '&';
                }
                if let Some(a) = a {
                    let _ = write!(&mut s, "{sep}a={a}");
                    sep = '&';
                }
                if let Some(p) = p {
                    let _ = write!(&mut s, "{sep}p={p}");
                    sep = '&';
                }
                if let Some(d) = d {
                    let _ = write!(&mut s, "{sep}d={d}");
                }
                if let Some(l) = l {
                    let _ = write!(&mut s, "{sep}l={l}");
                }
                s
            };
            super::call::<_, SFnE>(url).map_err(BackendError::from_other)
        }

        #[allow(clippy::similar_names)]
        #[allow(clippy::useless_let_if_seq)]
        fn get_module(
            &self,
            uri: Option<ftml_uris::ModuleUri>,
            a: Option<ftml_uris::ArchiveId>,
            p: Option<String>,
            m: Option<String>,
        ) -> impl Future<
            Output = Result<
                ftml_ontology::domain::modules::ModuleLike,
                BackendError<server_fn::error::ServerFnErrorErr>,
            >,
        > {
            use std::fmt::Write;
            if let Some(uri) = &uri
                && let Some(url) = self.redirects.for_modules(uri)
            {
                return super::call::<_, SFnE>(url.to_string()).map_err(BackendError::from_other);
            }
            let url = {
                let mut s = String::with_capacity(64);
                let _ = write!(&mut s, "{}/domain/module", &self.url);
                let mut sep = '?';
                if let Some(uri) = uri {
                    let _ = write!(&mut s, "?uri={}", uri.url_encoded());
                    sep = '&';
                }
                if let Some(a) = a {
                    let _ = write!(&mut s, "{sep}a={a}");
                    sep = '&';
                }
                if let Some(p) = p {
                    let _ = write!(&mut s, "{sep}p={p}");
                    sep = '&';
                }
                if let Some(m) = m {
                    let _ = write!(&mut s, "{sep}m={m}");
                }
                s
            };
            super::call::<_, SFnE>(url).map_err(BackendError::from_other)
        }

        #[allow(clippy::similar_names)]
        #[allow(clippy::many_single_char_names)]
        #[allow(clippy::useless_let_if_seq)]
        fn get_document(
            &self,
            uri: Option<ftml_uris::DocumentUri>,
            rp: Option<String>,
            a: Option<ftml_uris::ArchiveId>,
            p: Option<String>,
            d: Option<String>,
            l: Option<ftml_uris::Language>,
        ) -> impl Future<
            Output = Result<
                ftml_ontology::narrative::documents::Document,
                BackendError<server_fn::error::ServerFnErrorErr>,
            >,
        > {
            use std::fmt::Write;
            if let Some(uri) = &uri
                && let Some(url) = self.redirects.for_documents(uri)
            {
                return super::call::<_, SFnE>(url.to_string()).map_err(BackendError::from_other);
            }
            let url = {
                let mut s = String::with_capacity(64);
                let _ = write!(&mut s, "{}/domain/document", &self.url);
                let mut sep = '?';
                if let Some(uri) = uri {
                    let _ = write!(&mut s, "?uri={}", uri.url_encoded());
                    sep = '&';
                }
                if let Some(rp) = rp {
                    let _ = write!(&mut s, "{sep}rp={rp}");
                    sep = '&';
                }
                if let Some(a) = a {
                    let _ = write!(&mut s, "{sep}a={a}");
                    sep = '&';
                }
                if let Some(p) = p {
                    let _ = write!(&mut s, "{sep}p={p}");
                    sep = '&';
                }
                if let Some(d) = d {
                    let _ = write!(&mut s, "{sep}d={d}");
                }
                if let Some(l) = l {
                    let _ = write!(&mut s, "{sep}l={l}");
                }
                s
            };
            super::call::<_, SFnE>(url).map_err(BackendError::from_other)
        }

        ftml_uris::compfun! {!!
            #[allow(clippy::similar_names)]
            fn get_notations(&self,uri:Uri) -> impl Future<Output=Result<Vec<(
                ftml_uris::DocumentElementUri,
                ftml_ontology::narrative::elements::Notation,
            )>, BackendError<ServerFnErrorErr>>> {
                fn leaf(base:impl std::fmt::Display,re:&impl Redirects,uri:&LeafUri) -> String {
                    re.for_notations(uri).map_or_else(
                        || format!(
                            "{}/content/notations?uri={}",
                            base,
                            uri.url_encoded(),
                        ),
                        |r| r.to_string()
                    )
                }
                let url = match uri.uri {
                    Some(Uri::Symbol(s)) => {
                        let uri = s.into();
                        leaf(&self.url,&self.redirects,&uri)
                    },
                    Some(Uri::DocumentElement(e)) => {
                        let uri = e.into();
                        leaf(&self.url,&self.redirects,&uri)
                    },
                    _ => format!(
                        "{}/content/notations{}",
                        self.url,
                        uri.as_query(),
                    )
                };
                super::call::<_,SFnE>(url).map_err(BackendError::from_other)
            }
        }

        ftml_uris::compfun! {!!
            fn get_logical_paragraphs(
                &self,
                uri: SymbolUri,
                problems: bool
            ) -> impl Future<
                Output = Result<
                    Vec<(DocumentElementUri, ParagraphOrProblemKind)>,
                    BackendError<server_fn::error::ServerFnErrorErr>,
                >,
            > {
                let url = uri.uri.as_ref().map_or_else(
                    || format!(
                        "{}/content/los{}&problems={problems}",
                        self.url,
                        uri.as_query(),
                    ),
                    |s| self.redirects.for_paragraphs(s, problems).map_or_else(|| {
                        format!(
                            "{}/content/los{}&problems={problems}",
                            self.url,
                            uri.as_query(),
                        )
                    },|s| s.to_string())
                );
                super::call::<_,SFnE>(url).map_err(BackendError::from_other)
            }
        }
    }

    #[derive(Debug)]
    struct SFnE(ServerFnErrorErr);
    impl std::fmt::Display for SFnE {
        #[inline]
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.fmt(f)
        }
    }
    impl From<SFnE> for ServerFnErrorErr {
        #[inline]
        fn from(value: SFnE) -> Self {
            value.0
        }
    }
    impl From<super::RequestError> for SFnE {
        fn from(value: super::RequestError) -> Self {
            Self(match value {
                super::RequestError::Request(r) => ServerFnErrorErr::Request(r),
                super::RequestError::Deserialization(r) => ServerFnErrorErr::Deserialization(r),
            })
        }
    }
    impl std::str::FromStr for SFnE {
        type Err = BackendError<Self>;
        fn from_str(string: &str) -> Result<Self, Self::Err> {
            let Some(j) = string.find('|') else {
                return Err(BackendError::Connection(Self(
                    ServerFnErrorErr::Deserialization(format!(
                        "Invalid format: missing delimiter in {string:?}"
                    )),
                )));
            };
            if j == 0 {
                return Err(BackendError::Connection(Self(
                    ServerFnErrorErr::Deserialization(format!(
                        "Invalid format: missing delimiter in {string:?}"
                    )),
                )));
            }
            let data = string[j + 1..].to_string();
            let prefix = &string[..string.len() - 1];
            crate::server_fn_impl::decode_server_fn(prefix, data)
                .map(Self)
                .map_err(|e| BackendError::Connection(Self(ServerFnErrorErr::Deserialization(e))))
        }
    }
}

trait JsWrap {
    #[cfg(not(feature = "serde-lite"))]
    async fn get<
        T: serde::de::DeserializeOwned,
        E: From<RequestError> + std::fmt::Display + std::fmt::Debug + std::str::FromStr,
    >(
        self,
    ) -> Result<T, BackendError<E>>;
    #[cfg(feature = "serde-lite")]
    async fn get<
        T: serde_lite::Deserialize,
        E: From<RequestError> + std::fmt::Display + std::fmt::Debug + std::str::FromStr,
    >(
        self,
    ) -> Result<T, BackendError<E>>;
}

#[cfg(feature = "wasm")]
impl JsWrap for reqwasm::http::Response {
    #[cfg(not(feature = "serde-lite"))]
    #[allow(clippy::future_not_send)]
    async fn get<
        T: serde::de::DeserializeOwned,
        E: From<RequestError> + std::fmt::Display + std::fmt::Debug + std::str::FromStr,
    >(
        self,
    ) -> Result<T, BackendError<E>> {
        self.json()
            .await
            .map_err(|e| BackendError::Connection(E::from(e.into())))
    }
    #[cfg(feature = "serde-lite")]
    #[allow(clippy::future_not_send)]
    async fn get<
        T: serde_lite::Deserialize,
        E: From<RequestError> + std::fmt::Display + std::fmt::Debug + std::str::FromStr,
    >(
        self,
    ) -> Result<T, BackendError<E>> {
        let im = self
            .json()
            .await
            .map_err(|e| BackendError::Connection(E::from(e.into())))?;
        serde_lite::Deserialize::deserialize(&im)
            .map_err(|e| BackendError::Connection(E::from(e.into())))
    }
}

#[cfg(feature = "reqwest")]
impl JsWrap for ::reqwest::Response {
    #[cfg(not(feature = "serde-lite"))]
    async fn get<
        T: serde::de::DeserializeOwned,
        E: From<RequestError> + std::fmt::Display + std::fmt::Debug + std::str::FromStr,
    >(
        self,
    ) -> Result<T, BackendError<E>> {
        self.json()
            .await
            .map_err(|e| BackendError::Connection(E::from(e.into())))
    }
    #[cfg(feature = "serde-lite")]
    async fn get<
        T: serde_lite::Deserialize,
        E: From<RequestError> + std::fmt::Display + std::fmt::Debug + std::str::FromStr,
    >(
        self,
    ) -> Result<T, BackendError<E>> {
        let im = self
            .json()
            .await
            .map_err(|e| BackendError::Connection(E::from(e.into())))?;
        serde_lite::Deserialize::deserialize(&im)
            .map_err(|e| BackendError::Connection(E::from(e.into())))
    }
}

#[cfg(all(feature = "wasm", feature = "serde-lite"))]
fn post<R, E>(url: String, body: String) -> impl Future<Output = Result<R, BackendError<E>>>
where
    R: serde_lite::Deserialize,
    E: From<RequestError> + std::fmt::Display + std::fmt::Debug + std::str::FromStr,
    E::Err: Into<BackendError<E>>,
{
    let fut = async move {
        let res = reqwasm::http::Request::post(&url)
            .body(body)
            .send()
            .await
            .map_err(|e| BackendError::Connection(E::from(e.into())))?;
        let status = res.status();
        if (400..=599).contains(&status) {
            let str = res
                .text()
                .await
                .map_err(|e| BackendError::Connection(E::from(e.into())))?;
            return Err(BackendError::<E>::from_str(&str).map_err(Into::into)?);
        }

        res.get().await
    };
    crate::utils::FutWrap::new(fut)
}

#[cfg(all(feature = "wasm", feature = "serde-lite"))]
fn call<R, E>(url: String) -> impl Future<Output = Result<R, BackendError<E>>>
where
    R: serde_lite::Deserialize,
    E: From<RequestError> + std::fmt::Display + std::fmt::Debug + std::str::FromStr,
    E::Err: Into<BackendError<E>>,
{
    #[allow(clippy::future_not_send)]
    async fn call_i<R, E>(url: String) -> Result<R, BackendError<E>>
    where
        R: serde_lite::Deserialize,
        E: From<RequestError> + std::fmt::Display + std::fmt::Debug + std::str::FromStr,
        E::Err: Into<BackendError<E>>,
    {
        let res = reqwasm::http::Request::get(&url)
            .send()
            .await
            .map_err(|e| BackendError::Connection(E::from(e.into())))?;

        let status = res.status();
        if (400..=599).contains(&status) {
            let str = res
                .text()
                .await
                .map_err(|e| BackendError::Connection(E::from(e.into())))?;
            return Err(BackendError::<E>::from_str(&str).map_err(Into::into)?);
        }

        res.get().await
    }

    crate::utils::FutWrap::new(call_i(url))
}

#[cfg(all(feature = "wasm", not(feature = "serde-lite")))]
fn call<R, E>(url: String) -> impl Future<Output = Result<R, BackendError<E>>>
where
    R: serde::de::DeserializeOwned,
    E: From<RequestError> + std::fmt::Display + std::fmt::Debug + std::str::FromStr,
    E::Err: Into<BackendError<E>>,
{
    #[allow(clippy::future_not_send)]
    async fn call_i<R, E>(url: String) -> Result<R, BackendError<E>>
    where
        R: serde::de::DeserializeOwned,
        E: From<RequestError> + std::fmt::Display + std::fmt::Debug + std::str::FromStr,
        E::Err: Into<BackendError<E>>,
    {
        let res = reqwasm::http::Request::get(&url)
            .send()
            .await
            .map_err(|e| BackendError::Connection(E::from(e.into())))?;

        let status = res.status();
        if (400..=599).contains(&status) {
            let str = res
                .text()
                .await
                .map_err(|e| BackendError::Connection(E::from(e.into())))?;
            return Err(BackendError::<E>::from_str(&str).map_err(Into::into)?);
        }

        res.get().await
    }

    crate::utils::FutWrap::new(call_i(url))
}

#[cfg(all(feature = "wasm", not(feature = "serde-lite")))]
fn post<R, E>(url: String, body: String) -> impl Future<Output = Result<R, BackendError<E>>>
where
    R: serde::de::DeserializeOwned,
    E: From<RequestError> + std::fmt::Display + std::fmt::Debug + std::str::FromStr,
    E::Err: Into<BackendError<E>>,
{
    let fut = async move {
        let res = reqwasm::http::Request::post(&url)
            .body(body)
            .send()
            .await
            .map_err(|e| BackendError::Connection(E::from(e.into())))?;
        let status = res.status();
        if (400..=599).contains(&status) {
            let str = res
                .text()
                .await
                .map_err(|e| BackendError::Connection(E::from(e.into())))?;
            return Err(BackendError::<E>::from_str(&str).map_err(Into::into)?);
        }

        res.get().await
    };
    crate::utils::FutWrap::new(fut)
}

#[cfg(all(not(feature = "serde-lite"), not(feature = "wasm")))]
async fn call<R, E>(url: String) -> Result<R, BackendError<E>>
where
    R: serde::de::DeserializeOwned,
    E: From<RequestError> + std::fmt::Debug + std::fmt::Display + std::str::FromStr,
    E::Err: Into<BackendError<E>>,
{
    let res = ::reqwest::get(&url)
        .await
        .map_err(|e| BackendError::Connection(E::from(e.into())))?;

    let status = res.status().as_u16();
    if (400..=599).contains(&status) {
        let str = res
            .text()
            .await
            .map_err(|e| BackendError::Connection(E::from(e.into())))?;
        return Err(BackendError::<E>::from_str(&str).map_err(Into::into)?);
    }

    res.get().await
}

#[cfg(all(not(feature = "wasm"), not(feature = "serde-lite")))]
async fn post<R, E>(url: String, body: String) -> Result<R, BackendError<E>>
where
    R: serde::de::DeserializeOwned,
    E: From<RequestError> + std::fmt::Debug + std::fmt::Display + std::str::FromStr,
    E::Err: Into<BackendError<E>>,
{
    let res = ::reqwest::Client::new()
        .post(&url)
        .body(body)
        .send()
        .await
        .map_err(|e| BackendError::Connection(E::from(e.into())))?;

    let status = res.status().as_u16();
    if (400..=599).contains(&status) {
        let str = res
            .text()
            .await
            .map_err(|e| BackendError::Connection(E::from(e.into())))?;
        return Err(BackendError::<E>::from_str(&str).map_err(Into::into)?);
    }

    res.get().await
}

#[cfg(all(feature = "serde-lite", not(feature = "wasm")))]
async fn call<R, E>(url: String) -> Result<R, BackendError<E>>
where
    R: serde_lite::Deserialize,
    E: From<RequestError> + std::fmt::Debug + std::fmt::Display + std::str::FromStr,
    E::Err: Into<BackendError<E>>,
{
    let res = ::reqwest::get(&url)
        .await
        .map_err(|e| BackendError::Connection(E::from(e.into())))?;

    let status = res.status().as_u16();
    if (400..=599).contains(&status) {
        let str = res
            .text()
            .await
            .map_err(|e| BackendError::Connection(E::from(e.into())))?;
        return Err(BackendError::<E>::from_str(&str).map_err(Into::into)?);
    }

    res.get().await
}

#[cfg(all(not(feature = "wasm"), feature = "serde-lite"))]
async fn post<R, E>(url: String, body: String) -> Result<R, BackendError<E>>
where
    R: serde_lite::Deserialize,
    E: From<RequestError> + std::fmt::Debug + std::fmt::Display + std::str::FromStr,
    E::Err: Into<BackendError<E>>,
{
    let res = ::reqwest::Client::new()
        .post(&url)
        .body(body)
        .send()
        .await
        .map_err(|e| BackendError::Connection(E::from(e.into())))?;

    let status = res.status().as_u16();
    if (400..=599).contains(&status) {
        let str = res
            .text()
            .await
            .map_err(|e| BackendError::Connection(E::from(e.into())))?;
        return Err(BackendError::<E>::from_str(&str).map_err(Into::into)?);
    }

    res.get().await
}

#[cfg(feature = "wasm")]
impl From<reqwasm::Error> for RequestError {
    fn from(value: reqwasm::Error) -> Self {
        match value {
            reqwasm::Error::JsError(j) => Self::Request(j.to_string()),
            reqwasm::Error::SerdeError(e) => Self::Deserialization(e.to_string()),
        }
    }
}

#[cfg(feature = "reqwest")]
impl From<::reqwest::Error> for RequestError {
    fn from(value: ::reqwest::Error) -> Self {
        let value = value.without_url();
        if value.is_body() || value.is_decode() {
            Self::Deserialization(value.to_string())
        } else {
            Self::Request(value.to_string())
        }
    }
}

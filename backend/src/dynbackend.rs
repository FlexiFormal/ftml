use either::Either;
use ftml_ontology::{
    domain::{
        SharedDeclaration,
        declarations::{
            morphisms::Morphism,
            structures::{MathStructure, StructureExtension},
            symbols::Symbol,
        },
        modules::ModuleLike,
    },
    narrative::{
        SharedDocumentElement,
        documents::{Document, TocElem},
        elements::{
            DocumentTerm, Notation, ParagraphOrProblemKind, SectionLevel, VariableDeclaration,
            problems::Solutions,
        },
    },
    terms::{Term, termpaths::TermPath},
    utils::Css,
};
use ftml_uris::{
    DocumentElementUri, DocumentUri, LeafUri, ModuleUri, NarrativeUri, SymbolUri, Uri,
};

use crate::{BackendCheckResult, BackendError, FtmlBackend};

pub type Fut<T> = std::pin::Pin<Box<dyn Future<Output = Result<T, BackendError<String>>> + Send>>;

pub trait DynBackend: Send + Sync {
    fn document_link_url(&self, uri: &DocumentUri) -> String;
    fn resource_link_url(&self, uri: &DocumentUri, kind: &'static str) -> Option<String>;

    fn check_term(
        &self,
        global_context: &[ModuleUri],
        term: &Term,
        in_path: &TermPath,
    ) -> Fut<BackendCheckResult>;

    fn get_fragment(
        &self,
        uri: Uri,
        context: Option<NarrativeUri>,
    ) -> Fut<(Box<str>, Box<[Css]>, bool)>;

    fn get_logical_paragraphs(
        &self,
        uri: SymbolUri,
        problems: bool,
    ) -> Fut<Vec<(DocumentElementUri, ParagraphOrProblemKind)>>;

    fn get_module(&self, uri: ModuleUri) -> Fut<ModuleLike>;

    fn get_document(&self, uri: DocumentUri) -> Fut<Document>;

    fn get_toc(&self, uri: DocumentUri) -> Fut<(Box<[Css]>, SectionLevel, Box<[TocElem]>)>;

    fn get_symbol(&self, uri: SymbolUri) -> Fut<Either<Symbol, SharedDeclaration<Symbol>>>;

    fn get_morphism(&self, uri: SymbolUri) -> Fut<Either<Morphism, SharedDeclaration<Morphism>>>;

    #[allow(clippy::type_complexity)]
    fn get_structure(
        &self,
        uri: SymbolUri,
    ) -> Fut<Either<SharedDeclaration<MathStructure>, SharedDeclaration<StructureExtension>>>;

    fn get_variable(
        &self,
        uri: DocumentElementUri,
    ) -> Fut<Either<VariableDeclaration, SharedDocumentElement<VariableDeclaration>>>;

    fn get_document_term(
        &self,
        uri: DocumentElementUri,
    ) -> Fut<Either<DocumentTerm, SharedDocumentElement<DocumentTerm>>>;

    fn get_definition(
        &self,
        uri: SymbolUri,
        context: Option<NarrativeUri>,
    ) -> Fut<(Box<str>, Box<[Css]>)>;

    fn get_document_html(
        &self,
        uri: DocumentUri,
        context: Option<NarrativeUri>,
    ) -> Fut<(Box<str>, Box<[Css]>, bool)>;

    fn get_solutions(&self, uri: DocumentElementUri) -> Fut<Solutions>;

    fn get_notations(&self, uri: LeafUri) -> Fut<Vec<(DocumentElementUri, Notation)>>;

    fn get_notation(&self, symbol: LeafUri, uri: DocumentElementUri) -> Fut<Notation>;
}

fn wrap<R, E: std::fmt::Debug + std::fmt::Display>(
    f: impl Future<Output = Result<R, BackendError<E>>> + Send + 'static,
) -> Fut<R> {
    Box::pin(async move {
        f.await.map_err(|e| match e {
            BackendError::HtmlNotFound => BackendError::HtmlNotFound,
            BackendError::NoDefinition => BackendError::NoDefinition,
            BackendError::NoFragment => BackendError::NoFragment,
            BackendError::InvalidUriComponent(u) => BackendError::InvalidUriComponent(u),
            BackendError::InvalidArgument(s) => BackendError::InvalidArgument(s),
            BackendError::NotFound(n) => BackendError::NotFound(n),
            BackendError::ToDo(t) => BackendError::ToDo(t),
            BackendError::Connection(c) => BackendError::Connection(c.to_string()),
        })
    })
}

impl FtmlBackend for dyn DynBackend {
    type Error = String;
    #[inline]
    fn document_link_url(&self, uri: &DocumentUri) -> String {
        <Self as DynBackend>::document_link_url(self, uri)
    }
    #[inline]
    fn resource_link_url(&self, uri: &DocumentUri, kind: &'static str) -> Option<String> {
        <Self as DynBackend>::resource_link_url(self, uri, kind)
    }
    #[inline]
    fn check_term(
        &self,
        global_context: &[ModuleUri],
        term: &Term,
        in_path: &TermPath,
    ) -> impl Future<Output = Result<BackendCheckResult, BackendError<Self::Error>>>
    + Send
    + use<>
    + 'static {
        <Self as DynBackend>::check_term(self, global_context, term, in_path)
    }
    #[inline]
    fn get_fragment(
        &self,
        uri: Uri,
        context: Option<NarrativeUri>,
    ) -> impl Future<Output = Result<(Box<str>, Box<[Css]>, bool), BackendError<Self::Error>>>
    + Send
    + 'static {
        <Self as DynBackend>::get_fragment(self, uri, context)
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
    > + Send
    + 'static {
        <Self as DynBackend>::get_logical_paragraphs(self, uri, problems)
    }
    #[inline]
    fn get_module(
        &self,
        uri: ModuleUri,
    ) -> impl Future<Output = Result<ModuleLike, BackendError<Self::Error>>> + Send + 'static {
        <Self as DynBackend>::get_module(self, uri)
    }
    #[inline]
    fn get_document(
        &self,
        uri: DocumentUri,
    ) -> impl Future<Output = Result<Document, BackendError<Self::Error>>> + Send + 'static {
        <Self as DynBackend>::get_document(self, uri)
    }
    #[inline]
    fn get_toc(
        &self,
        uri: DocumentUri,
    ) -> impl Future<
        Output = Result<(Box<[Css]>, SectionLevel, Box<[TocElem]>), BackendError<Self::Error>>,
    > + Send
    + 'static {
        <Self as DynBackend>::get_toc(self, uri)
    }
    #[inline]
    fn get_symbol(
        &self,
        uri: SymbolUri,
    ) -> impl Future<
        Output = Result<Either<Symbol, SharedDeclaration<Symbol>>, BackendError<Self::Error>>,
    > + Send
    + 'static {
        <Self as DynBackend>::get_symbol(self, uri)
    }
    #[inline]
    fn get_morphism(
        &self,
        uri: SymbolUri,
    ) -> impl Future<
        Output = Result<Either<Morphism, SharedDeclaration<Morphism>>, BackendError<Self::Error>>,
    > + Send
    + 'static {
        <Self as DynBackend>::get_morphism(self, uri)
    }
    #[inline]
    fn get_structure(
        &self,
        uri: SymbolUri,
    ) -> impl Future<
        Output = Result<
            Either<SharedDeclaration<MathStructure>, SharedDeclaration<StructureExtension>>,
            BackendError<Self::Error>,
        >,
    > + Send
    + 'static {
        <Self as DynBackend>::get_structure(self, uri)
    }
    #[inline]
    fn get_variable(
        &self,
        uri: DocumentElementUri,
    ) -> impl Future<
        Output = Result<
            Either<VariableDeclaration, SharedDocumentElement<VariableDeclaration>>,
            BackendError<Self::Error>,
        >,
    > + Send
    + 'static {
        <Self as DynBackend>::get_variable(self, uri)
    }
    #[inline]
    fn get_document_term(
        &self,
        uri: DocumentElementUri,
    ) -> impl Future<
        Output = Result<
            Either<DocumentTerm, SharedDocumentElement<DocumentTerm>>,
            BackendError<Self::Error>,
        >,
    > + Send
    + 'static {
        <Self as DynBackend>::get_document_term(self, uri)
    }
    #[inline]
    fn get_definition(
        &self,
        uri: SymbolUri,
        context: Option<NarrativeUri>,
    ) -> impl Future<Output = Result<(Box<str>, Box<[Css]>), BackendError<Self::Error>>> + Send + 'static
    {
        <Self as DynBackend>::get_definition(self, uri, context)
    }
    #[inline]
    fn get_document_html(
        &self,
        uri: DocumentUri,
        context: Option<NarrativeUri>,
    ) -> impl Future<Output = Result<(Box<str>, Box<[Css]>, bool), BackendError<Self::Error>>>
    + Send
    + 'static {
        <Self as DynBackend>::get_document_html(self, uri, context)
    }
    #[inline]
    fn get_solutions(
        &self,
        uri: DocumentElementUri,
    ) -> impl Future<Output = Result<Solutions, BackendError<Self::Error>>> + Send + 'static {
        <Self as DynBackend>::get_solutions(self, uri)
    }
    #[inline]
    fn get_notations(
        &self,
        uri: LeafUri,
    ) -> impl Future<Output = Result<Vec<(DocumentElementUri, Notation)>, BackendError<Self::Error>>>
    + Send
    + 'static {
        <Self as DynBackend>::get_notations(self, uri)
    }
    #[inline]
    fn get_notation(
        &self,
        symbol: LeafUri,
        uri: DocumentElementUri,
    ) -> impl Future<Output = Result<Notation, BackendError<Self::Error>>> + Send + 'static {
        <Self as DynBackend>::get_notation(self, symbol, uri)
    }
}

impl<B: FtmlBackend + Send + Sync> DynBackend for B {
    #[inline]
    fn document_link_url(&self, uri: &DocumentUri) -> String {
        <Self as FtmlBackend>::document_link_url(self, uri)
    }
    #[inline]
    fn resource_link_url(&self, uri: &DocumentUri, kind: &'static str) -> Option<String> {
        <Self as FtmlBackend>::resource_link_url(self, uri, kind)
    }
    #[inline]
    fn check_term(
        &self,
        global_context: &[ModuleUri],
        term: &Term,
        in_path: &TermPath,
    ) -> Fut<BackendCheckResult> {
        wrap(<Self as FtmlBackend>::check_term(
            self,
            global_context,
            term,
            in_path,
        ))
    }
    #[inline]
    fn get_fragment(
        &self,
        uri: Uri,
        context: Option<NarrativeUri>,
    ) -> Fut<(Box<str>, Box<[Css]>, bool)> {
        wrap(<Self as FtmlBackend>::get_fragment(self, uri, context))
    }
    #[inline]
    fn get_logical_paragraphs(
        &self,
        uri: SymbolUri,
        problems: bool,
    ) -> Fut<Vec<(DocumentElementUri, ParagraphOrProblemKind)>> {
        wrap(<Self as FtmlBackend>::get_logical_paragraphs(
            self, uri, problems,
        ))
    }
    #[inline]
    fn get_module(&self, uri: ModuleUri) -> Fut<ModuleLike> {
        wrap(<Self as FtmlBackend>::get_module(self, uri))
    }
    #[inline]
    fn get_document(&self, uri: DocumentUri) -> Fut<Document> {
        wrap(<Self as FtmlBackend>::get_document(self, uri))
    }
    #[inline]
    fn get_toc(&self, uri: DocumentUri) -> Fut<(Box<[Css]>, SectionLevel, Box<[TocElem]>)> {
        wrap(<Self as FtmlBackend>::get_toc(self, uri))
    }
    #[inline]
    fn get_symbol(&self, uri: SymbolUri) -> Fut<Either<Symbol, SharedDeclaration<Symbol>>> {
        wrap(<Self as FtmlBackend>::get_symbol(self, uri))
    }
    #[inline]
    fn get_morphism(&self, uri: SymbolUri) -> Fut<Either<Morphism, SharedDeclaration<Morphism>>> {
        wrap(<Self as FtmlBackend>::get_morphism(self, uri))
    }
    #[inline]
    fn get_structure(
        &self,
        uri: SymbolUri,
    ) -> Fut<Either<SharedDeclaration<MathStructure>, SharedDeclaration<StructureExtension>>> {
        wrap(<Self as FtmlBackend>::get_structure(self, uri))
    }
    #[inline]
    fn get_variable(
        &self,
        uri: DocumentElementUri,
    ) -> Fut<Either<VariableDeclaration, SharedDocumentElement<VariableDeclaration>>> {
        wrap(<Self as FtmlBackend>::get_variable(self, uri))
    }
    #[inline]
    fn get_document_term(
        &self,
        uri: DocumentElementUri,
    ) -> Fut<Either<DocumentTerm, SharedDocumentElement<DocumentTerm>>> {
        wrap(<Self as FtmlBackend>::get_document_term(self, uri))
    }
    #[inline]
    fn get_definition(
        &self,
        uri: SymbolUri,
        context: Option<NarrativeUri>,
    ) -> Fut<(Box<str>, Box<[Css]>)> {
        wrap(<Self as FtmlBackend>::get_definition(self, uri, context))
    }
    #[inline]
    fn get_document_html(
        &self,
        uri: DocumentUri,
        context: Option<NarrativeUri>,
    ) -> Fut<(Box<str>, Box<[Css]>, bool)> {
        wrap(<Self as FtmlBackend>::get_document_html(self, uri, context))
    }
    #[inline]
    fn get_solutions(&self, uri: DocumentElementUri) -> Fut<Solutions> {
        wrap(<Self as FtmlBackend>::get_solutions(self, uri))
    }
    #[inline]
    fn get_notations(&self, uri: LeafUri) -> Fut<Vec<(DocumentElementUri, Notation)>> {
        wrap(<Self as FtmlBackend>::get_notations(self, uri))
    }
    #[inline]
    fn get_notation(&self, symbol: LeafUri, uri: DocumentElementUri) -> Fut<Notation> {
        wrap(<Self as FtmlBackend>::get_notation(self, symbol, uri))
    }
}

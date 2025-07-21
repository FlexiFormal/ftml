#![allow(unexpected_cfgs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]

/*! # FTML URIs
 *
 * [FTML](https://mathhub.info/?a=Papers%2F25-CICM-FLAMS&d=paper&l=en) URIs are intended to serve as hierarchical,
 * globally unique identifiers, are used as keys for retrieving identified content elements, and occur in large
 * numbers in FTML documents. As such, it is important that they are fast to parse, clone, equality-check,
 * (de)serialize, and ideally are parsimonious with respect to memory usage.
 *
 * Naturally, these desiderata are contradictory. Hence, as a tradeoff, we
 * - intern [Uri]s and Uri *components* for deduplication,
 * - use [strumbra](strumbra::SharedString) strings to keep allocations infrequent,
 * - use [Arc](triomphe::Arc)s where heap is unavoidable
 * - use pointer-equality (thanks to interning) for fast equality checks
 *
 * ## Grammar
 *
 * | Type  |     | Cases/Def | Trait |
 * |----------- |---- | -----|-------|
 * | [`Uri`]      | ::= | [`BaseUri`]⏐[`ArchiveUri`]⏐[`PathUri`]⏐[`ModuleUri`]⏐[`SymbolUri`]⏐[`DocumentUri`]⏐[`DocumentElementUri`] | [`FtmlUri`] |
 * | [`BaseUri`]  | ::= | (URL with no query/fragment) | - |
 * | [`ArchiveUri`] | ::= | <code>[BaseUri]?a=[ArchiveId]</code> | [`UriWithArchive`] |
 * | [`PathUri`]  | ::= | <code>[ArchiveUri][&p=[UriPath]]</code> | [`UriWithPath`] |
 * | [`DomainUri`] | ::= | [`ModuleUri`]⏐[`SymbolUri`]   | [`IsDomainUri`] |
 * | [`ModuleUri`] | ::= | <code>[PathUri]&m=[UriName]&l=[Language]</code> | - |
 * | [`SymbolUri`] | ::= | <code>[ModuleUri]&s=[UriName]</code> | - |
 * | [`NarrativeUri`] | ::= | [`DocumentUri`]⏐[`DocumentElementUri`] | [`IsNarrativeUri`] |
 * | [`DocumentUri`] | ::= | <code>[PathUri]&d=[SimpleUriName]&l=[Language]</code> | - |
 * | [`DocumentElementUri`] | ::= | <code>[DocumentUri]&e=[UriName]</code> | - |
 *
 * ## Feature flags
 **/
#![cfg_attr(doc,doc = document_features::document_features!())]

mod uris {
    pub mod archive;
    pub mod base;
    pub mod doc_element;
    pub mod document;
    pub mod module;
    pub mod paths;
    pub mod symbol;
}
mod aux;
mod language;
#[allow(clippy::wildcard_imports)]
pub(crate) use uris::*;

/// parsing and related errors
pub mod errors {
    pub use crate::aux::errors::*;
}
#[cfg(feature = "components")]
pub mod components;
pub mod metatheory;
use std::str::FromStr;
mod traits;

pub(crate) use aux::macros::{debugdisplay, tests, ts};

/// exports all Uri types and associated traits
pub mod prelude {
    pub use super::archive::{ArchiveId, ArchiveUri};
    pub use super::base::BaseUri;
    pub use super::language::Language;
    pub use super::paths::{PathUri, UriPath};
    pub use super::symbol::SymbolUri;
    pub use super::{DomainUri, NarrativeUri, Uri};
    pub use crate::aux::Id;
    pub use crate::doc_element::DocumentElementUri;
    pub use crate::document::{DocumentUri, SimpleUriName};
    pub use crate::module::{ModuleUri, UriName};
    pub use crate::traits::{
        FtmlUri, IsDomainUri, IsNarrativeUri, NamedUri, UriWithArchive, UriWithPath,
    };
}
use const_format::concatcp;
use either::Either::{Left, Right};
pub use prelude::*;

use crate::errors::UriParseError;

pub(crate) mod sealed {
    pub trait Sealed {}
}

/// Enum representing any type of FTML URI.
///
/// This enum provides a unified type that can hold any FTML URI variant,
/// from simple base URIs to complex module URIs. It implements the core
/// [`FtmlUri`] trait, allowing uniform access to the base URI component.
///
/// # Examples
///
/// ```
/// # use ftml_uris::prelude::*;
/// # use std::str::FromStr;
/// let base_uri = BaseUri::from_str("http://example.com").unwrap();
/// let archive_uri = ArchiveUri::from_str("http://example.com?a=archive").unwrap();
///
/// let uris: Vec<Uri> = vec![
///     Uri::Base(base_uri),
///     Uri::Archive(archive_uri),
/// ];
///
/// for uri in &uris {
///     println!("Base: {}", uri.base());
/// }
/// ```
#[derive(Clone, PartialEq, Eq, Hash, strum::EnumDiscriminants)]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(name(UriKind))]
#[strum_discriminants(derive(strum::Display))]
#[cfg_attr(
    feature = "serde",
    strum_discriminants(derive(serde::Serialize, serde::Deserialize))
)]
#[cfg_attr(
    feature = "serde",
    derive(serde_with::DeserializeFromStr, serde_with::SerializeDisplay)
)]
pub enum Uri {
    /// A base URI with no additional components.
    Base(BaseUri),
    /// An archive URI identifying a specific archive.
    Archive(ArchiveUri),
    /// A path URI identifying a location within an archive.
    Path(PathUri),
    /// A module URI identifying a specific module.
    Module(ModuleUri),
    /// A symbol URI identifying a specific concept.
    Symbol(SymbolUri),
    /// A document URI identifying a document in some archive.
    Document(DocumentUri),
    /// A document element URI identifying a named part in a document (section, paragraph, etc.).
    DocumentElement(DocumentElementUri),
}
impl crate::sealed::Sealed for Uri {}
crate::ts!(Uri);
crate::debugdisplay!(Uri);

/// Like [Uri], but on references rather than owned values
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum UriRef<'u> {
    /// A base URI with no additional components.
    Base(&'u BaseUri),
    /// An archive URI identifying a specific archive.
    Archive(&'u ArchiveUri),
    /// A path URI identifying a location within an archive.
    Path(&'u PathUri),
    /// A module URI identifying a specific module.
    Module(&'u ModuleUri),
    /// A symbol URI identifying a specific concept.
    Symbol(&'u SymbolUri),
    /// A document URI identifying a document in some archive.
    Document(&'u DocumentUri),
    /// A document element URI identifying a named part in a document (section, paragraph, etc.).
    DocumentElement(&'u DocumentElementUri),
}
impl UriRef<'_> {
    /// convert this reference into an owned [`Uri`]
    #[must_use]
    pub fn owned(self) -> Uri {
        match self {
            Self::Base(s) => Uri::Base(s.clone()),
            Self::Archive(s) => Uri::Archive(s.clone()),
            Self::Path(s) => Uri::Path(s.clone()),
            Self::Module(s) => Uri::Module(s.clone()),
            Self::Symbol(s) => Uri::Symbol(s.clone()),
            Self::Document(s) => Uri::Document(s.clone()),
            Self::DocumentElement(s) => Uri::DocumentElement(s.clone()),
        }
    }
}

/// Enum ranging over all url parameters occurring in [`Uri`]s; used for error messaging etc.
#[derive(
    Copy, Clone, PartialEq, Eq, Debug, strum::Display, strum::IntoStaticStr, strum::EnumString,
)]
#[cfg_attr(
    feature = "serde",
    derive(serde_with::DeserializeFromStr, serde_with::SerializeDisplay)
)]
#[allow(non_camel_case_types)]
pub enum UriComponentKind {
    /// full URI
    uri,
    /// relative path; requires [a](UriComponentKind::a)
    rp,
    /// an archive ID
    a,
    /// a path; requires [a](UriComponentKind::a)
    p,
    /// a module name; requires [a](UriComponentKind::a)
    m,
    /// a document name; [a](UriComponentKind::a)
    d,
    /// a language; requires [a](UriComponentKind::a) and [d](UriComponentKind::d)
    l,
    /// a symbol name; requires [a](UriComponentKind::a) and [m](UriComponentKind::a)
    s,
    /// a document element name; requires [a](UriComponentKind::a), [d](UriComponentKind::d)
    /// and [l](UriComponentKind::l)
    e,
}

/// Enum ranging over all [`IsDomainUri`] types ([`ModuleUri`] and [`SymbolUri`]).
///
/// # Examples
///
/// ```
/// # use ftml_uris::prelude::*;
/// # use std::str::FromStr;
/// let module_uri = ModuleUri::from_str("http://example.com?a=archive&m=module").unwrap();
/// let domain_uri: DomainUri = module_uri.into();
///
/// match domain_uri {
///     DomainUri::Module(m) => {
///         assert_eq!(m.name.to_string(), "module");
///     }
///     DomainUri::Symbol(m) => unreachable!()
/// }
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde_with::DeserializeFromStr, serde_with::SerializeDisplay)
)]
pub enum DomainUri {
    /// A module URI identifying a specific module within an archive.
    Module(ModuleUri),
    /// A symbol URI identifying a specific concept.
    Symbol(SymbolUri),
}
crate::ts!(DomainUri);
crate::debugdisplay!(DomainUri);
impl crate::sealed::Sealed for DomainUri {}

/// Like [`DomainUri`] but wrapping around references
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde_with::SerializeDisplay))]
pub enum DomainUriRef<'u> {
    /// A module URI identifying a specific module within an archive.
    Module(&'u ModuleUri),
    /// A symbol URI identifying a specific concept.
    Symbol(&'u SymbolUri),
}
impl crate::sealed::Sealed for DomainUriRef<'_> {}

/// Like [`NarrativeUri`] but wrapping around references
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde_with::SerializeDisplay))]
pub enum NarrativeUriRef<'u> {
    /// A document URI identifying a specific document within an archive.
    Document(&'u DocumentUri),
    /// A document element URI identifying a named part of a document.
    Element(&'u DocumentElementUri),
}

/// Enum ranging over all [`IsNarrativeUri`] types ([`DocumentUri`] and [`DocumentElementUri`]).
///
/// # Examples
///
/// ```
/// # use ftml_uris::prelude::*;
/// # use std::str::FromStr;
/// let document_uri = DocumentUri::from_str("http://example.com?a=archive&d=document&l=en").unwrap();
/// let narrative_uri: NarrativeUri = document_uri.into();
///
/// match narrative_uri {
///     NarrativeUri::Document(d) => {
///         assert_eq!(d.document_name().as_ref(), "document");
///     }
///     NarrativeUri::Element(e) => unreachable!()
/// }
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde_with::DeserializeFromStr, serde_with::SerializeDisplay)
)]
pub enum NarrativeUri {
    /// A document URI identifying a specific document within an archive.
    Document(DocumentUri),
    /// A document element URI identifying a named part of a document.
    Element(DocumentElementUri),
}
crate::ts!(NarrativeUri);
crate::debugdisplay!(NarrativeUri);
impl crate::sealed::Sealed for NarrativeUri {}

// parsing -----------------------------------------------------------------------------------

fn parse_domain(
    module: &str,
    path: impl FnOnce() -> Result<PathUri, UriParseError>,
    mut split: std::str::Split<char>,
) -> Result<DomainUri, UriParseError> {
    let name = move || module.parse();
    let module = move || {
        Ok::<_, UriParseError>(ModuleUri {
            path: path()?,
            name: name()?,
        })
    };
    let Some(next) = split.next() else {
        return Ok(DomainUri::Module(module()?));
    };
    next.strip_prefix(concatcp!(SymbolUri::SEPARATOR, "="))
        .map_or_else(
            || Err(UriParseError::UnknownParameter),
            |symbol| {
                if split.next().is_some() {
                    Err(UriParseError::TooManyPartsFor {
                        uri_kind: UriKind::Symbol,
                    })
                } else {
                    Ok(DomainUri::Symbol(SymbolUri {
                        module: module()?,
                        name: symbol.parse()?,
                    }))
                }
            },
        )
}

fn parse_narrative(
    document: &str,
    (language, next): (Language, Option<&str>),
    path: impl FnOnce() -> Result<PathUri, UriParseError>,
    mut split: std::str::Split<char>,
) -> Result<NarrativeUri, UriParseError> {
    let name = move || document.parse();
    let document = move || {
        Ok::<_, UriParseError>(DocumentUri {
            path: path()?,
            name: name()?,
            language,
        })
    };
    let Some(next) = next else {
        return Ok(NarrativeUri::Document(document()?));
    };
    next.strip_prefix(concatcp!(DocumentElementUri::SEPARATOR, "="))
        .map_or_else(
            || Err(UriParseError::UnknownParameter),
            |element| {
                if split.next().is_some() {
                    Err(UriParseError::TooManyPartsFor {
                        uri_kind: UriKind::DocumentElement,
                    })
                } else {
                    Ok(NarrativeUri::Element(DocumentElementUri {
                        document: document()?,
                        name: element.parse()?,
                    }))
                }
            },
        )
}

impl FromStr for Uri {
    type Err = UriParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (base, mut split) = match BaseUri::pre_parse(s)? {
            Left(base) => return Ok(Self::Base(base)),
            Right(c) => c,
        };
        let Some(next) = split.next() else {
            unreachable!()
        };
        next.strip_prefix(concatcp!(ArchiveUri::SEPARATOR, "="))
            .map_or_else(
                || Err(UriParseError::UnknownParameter),
                |archive| {
                    let archive = move || -> Result<_, UriParseError> {
                        Ok(ArchiveUri {
                            base,
                            id: archive.parse()?,
                        })
                    };
                    let Some(next) = split.next() else {
                        return Ok(Self::Archive(archive()?));
                    };
                    let (path, next) =
                        if let Some(path) = next.strip_prefix(concatcp!(PathUri::SEPARATOR, "=")) {
                            (
                                Left(|| {
                                    Ok(PathUri {
                                        archive: archive()?,
                                        path: Some(path.parse()?),
                                    })
                                }),
                                split.next(),
                            )
                        } else {
                            (
                                Right(|| -> Result<_, UriParseError> {
                                    Ok(PathUri {
                                        archive: archive()?,
                                        path: None,
                                    })
                                }),
                                Some(next),
                            )
                        };
                    let path = move || match path {
                        Left(p) => p(),
                        Right(p) => Ok(p()?),
                    };
                    let Some(next) = next else {
                        return Ok(Self::Path(path()?));
                    };
                    let mut language = || {
                        split.next().map_or_else(
                            || Ok((Language::default(), None)),
                            |n| {
                                n.strip_prefix(concatcp!(Language::SEPARATOR, "="))
                                    .map_or_else(
                                        || Ok((Language::default(), Some(n))),
                                        |l| {
                                            l.parse()
                                                .map_err(|_| UriParseError::InvalidLanguage)
                                                .map(|l| (l, split.next()))
                                        },
                                    )
                            },
                        )
                    };
                    if let Some(module) = next.strip_prefix(concatcp!(ModuleUri::SEPARATOR, "=")) {
                        Ok(parse_domain(module, path, split)?.into())
                    } else if let Some(document) =
                        next.strip_prefix(concatcp!(DocumentUri::SEPARATOR, "="))
                    {
                        Ok(parse_narrative(document, language()?, path, split)?.into())
                    } else {
                        Err(UriParseError::UnknownParameter)
                    }
                },
            )
    }
}

impl FromStr for DomainUri {
    type Err = errors::UriParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ModuleUri::pre_parse(s, UriKind::Module, |module, mut split| {
            let Some(c) = split.next() else {
                return Ok(Self::Module(module));
            };
            c.strip_prefix(concatcp!(SymbolUri::SEPARATOR, "="))
                .map_or_else(
                    || {
                        Err(UriParseError::TooManyPartsFor {
                            uri_kind: UriKind::Symbol,
                        })
                    },
                    |name| {
                        Ok(Self::Symbol(SymbolUri {
                            module,
                            name: name.parse()?,
                        }))
                    },
                )
        })
    }
}

impl FromStr for NarrativeUri {
    type Err = UriParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        DocumentUri::pre_parse(s, UriKind::Document, |document, mut split| {
            let Some(c) = split.next() else {
                return Ok(Self::Document(document));
            };
            c.strip_prefix(concatcp!(DocumentElementUri::SEPARATOR, "="))
                .map_or_else(
                    || {
                        Err(UriParseError::TooManyPartsFor {
                            uri_kind: UriKind::DocumentElement,
                        })
                    },
                    |name| {
                        Ok(Self::Element(DocumentElementUri {
                            document,
                            name: name.parse()?,
                        }))
                    },
                )
        })
    }
}

// impls -------------------------------------------------------------------------------------------

impl std::fmt::Display for Uri {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Base(b) => b.fmt(f),
            Self::Archive(a) => a.fmt(f),
            Self::Path(p) => p.fmt(f),
            Self::Module(m) => m.fmt(f),
            Self::Symbol(s) => s.fmt(f),
            Self::Document(d) => d.fmt(f),
            Self::DocumentElement(e) => e.fmt(f),
        }
    }
}
impl FtmlUri for Uri {
    fn base(&self) -> &BaseUri {
        match self {
            Self::Base(b) => b,
            Self::Archive(a) => a.base(),
            Self::Path(p) => p.base(),
            Self::Module(m) => m.base(),
            Self::Symbol(s) => s.base(),
            Self::Document(d) => d.base(),
            Self::DocumentElement(e) => e.base(),
        }
    }

    #[inline]
    fn as_uri(&self) -> UriRef<'_> {
        match self {
            Self::Base(b) => UriRef::Base(b),
            Self::Archive(a) => UriRef::Archive(a),
            Self::Path(p) => UriRef::Path(p),
            Self::Module(m) => UriRef::Module(m),
            Self::Symbol(s) => UriRef::Symbol(s),
            Self::Document(d) => UriRef::Document(d),
            Self::DocumentElement(e) => UriRef::DocumentElement(e),
        }
    }

    fn could_be(maybe_uri: &str) -> bool {
        if !maybe_uri.contains("?a") {
            return BaseUri::could_be(maybe_uri);
        }
        if maybe_uri.contains("?d") {
            NarrativeUri::could_be(maybe_uri)
        } else if maybe_uri.contains("?m") {
            DomainUri::could_be(maybe_uri)
        } else {
            PathUri::could_be(maybe_uri)
        }
    }
}
impl PartialEq<str> for Uri {
    fn eq(&self, other: &str) -> bool {
        match self {
            Self::Base(b) => *b == *other,
            Self::Archive(a) => *a == *other,
            Self::Path(p) => *p == *other,
            Self::Module(m) => *m == *other,
            Self::Symbol(s) => *s == *other,
            Self::Document(d) => *d == *other,
            Self::DocumentElement(e) => *e == *other,
        }
    }
}
impl From<Uri> for BaseUri {
    #[inline]
    fn from(value: Uri) -> Self {
        match value {
            Uri::Base(b) => b,
            Uri::Archive(a) => a.into(),
            Uri::Path(p) => p.into(),
            Uri::Module(m) => m.into(),
            Uri::Symbol(s) => s.into(),
            Uri::Document(d) => d.into(),
            Uri::DocumentElement(e) => e.into(),
        }
    }
}
impl From<BaseUri> for Uri {
    #[inline]
    fn from(value: BaseUri) -> Self {
        Self::Base(value)
    }
}
impl From<ArchiveUri> for Uri {
    #[inline]
    fn from(value: ArchiveUri) -> Self {
        Self::Archive(value)
    }
}
impl From<PathUri> for Uri {
    #[inline]
    fn from(value: PathUri) -> Self {
        Self::Path(value)
    }
}
impl From<ModuleUri> for Uri {
    #[inline]
    fn from(value: ModuleUri) -> Self {
        Self::Module(value)
    }
}
impl From<SymbolUri> for Uri {
    #[inline]
    fn from(value: SymbolUri) -> Self {
        Self::Symbol(value)
    }
}
impl From<DocumentUri> for Uri {
    #[inline]
    fn from(value: DocumentUri) -> Self {
        Self::Document(value)
    }
}
impl From<DocumentElementUri> for Uri {
    #[inline]
    fn from(value: DocumentElementUri) -> Self {
        Self::DocumentElement(value)
    }
}
impl From<DomainUri> for Uri {
    #[inline]
    fn from(value: DomainUri) -> Self {
        match value {
            DomainUri::Module(m) => Self::Module(m),
            DomainUri::Symbol(s) => Self::Symbol(s),
        }
    }
}
impl From<NarrativeUri> for Uri {
    #[inline]
    fn from(value: NarrativeUri) -> Self {
        match value {
            NarrativeUri::Document(d) => Self::Document(d),
            NarrativeUri::Element(e) => Self::DocumentElement(e),
        }
    }
}

impl std::fmt::Display for DomainUri {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Module(m) => m.fmt(f),
            Self::Symbol(s) => s.fmt(f),
        }
    }
}
impl FtmlUri for DomainUri {
    #[inline]
    fn base(&self) -> &BaseUri {
        match self {
            Self::Module(m) => m.base(),
            Self::Symbol(s) => s.base(),
        }
    }

    fn as_uri(&self) -> UriRef<'_> {
        match self {
            Self::Module(m) => UriRef::Module(m),
            Self::Symbol(s) => UriRef::Symbol(s),
        }
    }

    fn could_be(maybe_uri: &str) -> bool {
        if maybe_uri.contains("&s") {
            SymbolUri::could_be(maybe_uri)
        } else {
            ModuleUri::could_be(maybe_uri)
        }
    }
}

impl PartialEq<str> for DomainUri {
    fn eq(&self, other: &str) -> bool {
        match self {
            Self::Module(m) => *m == *other,
            Self::Symbol(s) => *s == *other,
        }
    }
}
impl IsDomainUri for DomainUri {
    #[inline]
    fn module_uri(&self) -> &ModuleUri {
        match self {
            Self::Module(m) => m,
            Self::Symbol(s) => s.module_uri(),
        }
    }
}
impl From<DomainUri> for BaseUri {
    #[inline]
    fn from(value: DomainUri) -> Self {
        match value {
            DomainUri::Module(m) => m.into(),
            DomainUri::Symbol(s) => s.into(),
        }
    }
}
impl UriWithArchive for DomainUri {
    #[inline]
    fn archive_uri(&self) -> &ArchiveUri {
        match self {
            Self::Module(m) => m.archive_uri(),
            Self::Symbol(s) => s.archive_uri(),
        }
    }
}
impl From<DomainUri> for ArchiveUri {
    #[inline]
    fn from(value: DomainUri) -> Self {
        match value {
            DomainUri::Module(m) => m.into(),
            DomainUri::Symbol(s) => s.into(),
        }
    }
}
impl UriWithPath for DomainUri {
    #[inline]
    fn path_uri(&self) -> &PathUri {
        match self {
            Self::Module(m) => m.path_uri(),
            Self::Symbol(s) => s.path_uri(),
        }
    }
}
impl From<DomainUri> for PathUri {
    #[inline]
    fn from(value: DomainUri) -> Self {
        match value {
            DomainUri::Module(m) => m.into(),
            DomainUri::Symbol(s) => s.into(),
        }
    }
}
impl From<DomainUri> for ModuleUri {
    #[inline]
    fn from(value: DomainUri) -> Self {
        match value {
            DomainUri::Module(m) => m,
            DomainUri::Symbol(s) => s.into(),
        }
    }
}
impl From<ModuleUri> for DomainUri {
    #[inline]
    fn from(value: ModuleUri) -> Self {
        Self::Module(value)
    }
}
impl From<SymbolUri> for DomainUri {
    #[inline]
    fn from(value: SymbolUri) -> Self {
        Self::Symbol(value)
    }
}
impl NamedUri for DomainUri {
    #[inline]
    fn name(&self) -> &UriName {
        match self {
            Self::Module(m) => m.name(),
            Self::Symbol(s) => s.name(),
        }
    }
}

impl std::fmt::Display for DomainUriRef<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Module(m) => m.fmt(f),
            Self::Symbol(s) => s.fmt(f),
        }
    }
}
impl std::fmt::Debug for DomainUriRef<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Display>::fmt(self, f)
    }
}

impl<'u> DomainUriRef<'u> {
    #[inline]
    #[must_use]
    pub fn base(self) -> &'u BaseUri {
        match self {
            Self::Module(m) => m.base(),
            Self::Symbol(s) => s.base(),
        }
    }

    #[inline]
    #[must_use]
    pub const fn as_uri(self) -> UriRef<'u> {
        match self {
            Self::Module(m) => UriRef::Module(m),
            Self::Symbol(s) => UriRef::Symbol(s),
        }
    }

    #[inline]
    #[must_use]
    pub fn could_be(maybe_uri: &str) -> bool {
        if maybe_uri.contains("&s") {
            SymbolUri::could_be(maybe_uri)
        } else {
            ModuleUri::could_be(maybe_uri)
        }
    }

    #[cfg(feature = "rdf")]
    #[inline]
    #[must_use]
    /// Returns this URI as an RDF-IRI; possibly escaping invalid characters.
    pub fn to_iri(self) -> oxrdf::NamedNode {
        match self {
            Self::Module(m) => m.to_iri(),
            Self::Symbol(s) => s.to_iri(),
        }
    }
}
impl PartialEq<str> for DomainUriRef<'_> {
    fn eq(&self, other: &str) -> bool {
        match self {
            Self::Module(m) => **m == *other,
            Self::Symbol(s) => **s == *other,
        }
    }
}
/*
impl<'a> FtmlUri for DomainUriRef<'a> {
    fn base(&self) -> &'a BaseUri {
        match self {
            Self::Module(m) => m.base(),
            Self::Symbol(s) => s.base(),
        }
    }
}
impl<'a> UriWithArchive for DomainUriRef<'a> {
    fn archive_uri(&self) -> &'a ArchiveUri {
        match self {
            Self::Module(m) => m.archive_uri(),
            Self::Symbol(s) => s.archive_uri(),
        }
    }
}

impl<'a> UriWithPath for DomainUriRef<'a> {
    #[inline]
    fn path_uri(&self) -> &'a PathUri {
        match self {
            Self::Module(m) => m.path_uri(),
            Self::Symbol(s) => s.path_uri(),
        }
    }
}
impl<'a> NamedUri for DomainUriRef<'a> {
    #[inline]
    fn name(&self) -> &'a UriName {
        match self {
            Self::Module(m) => m.name(),
            Self::Symbol(s) => s.name(),
        }
    }
}
 */

impl<'u> NarrativeUriRef<'u> {
    #[inline]
    #[must_use]
    pub fn base(self) -> &'u BaseUri {
        match self {
            Self::Document(m) => m.base(),
            Self::Element(s) => s.base(),
        }
    }

    #[inline]
    #[must_use]
    pub const fn as_uri(self) -> UriRef<'u> {
        match self {
            Self::Document(m) => UriRef::Document(m),
            Self::Element(s) => UriRef::DocumentElement(s),
        }
    }

    #[inline]
    #[must_use]
    pub fn could_be(maybe_uri: &str) -> bool {
        if maybe_uri.contains("&s") {
            DocumentElementUri::could_be(maybe_uri)
        } else {
            DocumentUri::could_be(maybe_uri)
        }
    }

    #[cfg(feature = "rdf")]
    #[inline]
    #[must_use]
    /// Returns this URI as an RDF-IRI; possibly escaping invalid characters.
    pub fn to_iri(self) -> oxrdf::NamedNode {
        match self {
            Self::Document(m) => m.to_iri(),
            Self::Element(s) => s.to_iri(),
        }
    }
}

impl std::fmt::Display for NarrativeUriRef<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Document(m) => m.fmt(f),
            Self::Element(s) => s.fmt(f),
        }
    }
}
impl std::fmt::Debug for NarrativeUriRef<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Display>::fmt(self, f)
    }
}
/*
impl<'a> FtmlUri for NarrativeUriRef<'a> {
    fn base(&self) -> &'a BaseUri {
        match self {
            Self::Document(m) => m.base(),
            Self::Element(s) => s.base(),
        }
    }
}
impl<'a> UriWithArchive for NarrativeUriRef<'a> {
    fn archive_uri(&self) -> &'a ArchiveUri {
        match self {
            Self::Document(m) => m.archive_uri(),
            Self::Element(s) => s.archive_uri(),
        }
    }
}

impl<'a> UriWithPath for NarrativeUriRef<'a> {
    #[inline]
    fn path_uri(&self) -> &'a PathUri {
        match self {
            Self::Document(m) => m.path_uri(),
            Self::Element(s) => s.path_uri(),
        }
    }
}
impl<'a> NamedUri for NarrativeUriRef<'a> {
    #[inline]
    fn name(&self) -> &'a UriName {
        match self {
            Self::Document(m) => m.name(),
            Self::Element(s) => s.name(),
        }
    }
}
 */

impl std::fmt::Display for NarrativeUri {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Document(d) => d.fmt(f),
            Self::Element(e) => e.fmt(f),
        }
    }
}
impl FtmlUri for NarrativeUri {
    #[inline]
    fn base(&self) -> &BaseUri {
        match self {
            Self::Document(d) => d.base(),
            Self::Element(e) => e.base(),
        }
    }

    fn as_uri(&self) -> UriRef<'_> {
        match self {
            Self::Document(d) => UriRef::Document(d),
            Self::Element(e) => UriRef::DocumentElement(e),
        }
    }

    fn could_be(maybe_uri: &str) -> bool {
        if maybe_uri.contains("&e") {
            DocumentElementUri::could_be(maybe_uri)
        } else {
            DocumentUri::could_be(maybe_uri)
        }
    }
}
impl PartialEq<str> for NarrativeUri {
    fn eq(&self, other: &str) -> bool {
        match self {
            Self::Document(m) => *m == *other,
            Self::Element(s) => *s == *other,
        }
    }
}
impl IsNarrativeUri for NarrativeUri {
    #[inline]
    fn document_uri(&self) -> &DocumentUri {
        match self {
            Self::Document(d) => d,
            Self::Element(e) => e.document_uri(),
        }
    }
}
impl From<NarrativeUri> for BaseUri {
    #[inline]
    fn from(value: NarrativeUri) -> Self {
        match value {
            NarrativeUri::Document(d) => d.into(),
            NarrativeUri::Element(e) => e.into(),
        }
    }
}
impl UriWithArchive for NarrativeUri {
    #[inline]
    fn archive_uri(&self) -> &ArchiveUri {
        match self {
            Self::Document(d) => d.archive_uri(),
            Self::Element(e) => e.archive_uri(),
        }
    }
}
impl From<NarrativeUri> for ArchiveUri {
    #[inline]
    fn from(value: NarrativeUri) -> Self {
        match value {
            NarrativeUri::Document(d) => d.into(),
            NarrativeUri::Element(e) => e.into(),
        }
    }
}
impl UriWithPath for NarrativeUri {
    #[inline]
    fn path_uri(&self) -> &PathUri {
        match self {
            Self::Document(d) => d.path_uri(),
            Self::Element(e) => e.path_uri(),
        }
    }
}
impl From<NarrativeUri> for PathUri {
    #[inline]
    fn from(value: NarrativeUri) -> Self {
        match value {
            NarrativeUri::Document(d) => d.into(),
            NarrativeUri::Element(e) => e.into(),
        }
    }
}
impl From<NarrativeUri> for DocumentUri {
    #[inline]
    fn from(value: NarrativeUri) -> Self {
        match value {
            NarrativeUri::Document(d) => d,
            NarrativeUri::Element(e) => e.into(),
        }
    }
}
impl From<DocumentUri> for NarrativeUri {
    #[inline]
    fn from(value: DocumentUri) -> Self {
        Self::Document(value)
    }
}
impl From<DocumentElementUri> for NarrativeUri {
    #[inline]
    fn from(value: DocumentElementUri) -> Self {
        Self::Element(value)
    }
}
impl NamedUri for NarrativeUri {
    fn name(&self) -> &UriName {
        match self {
            Self::Document(d) => d.name(),
            Self::Element(e) => e.name(),
        }
    }
}

// TESTS -------------------------------------------------------------------------------------------

#[cfg(test)]
#[rstest::fixture]
fn trace() {
    let _ = tracing_subscriber::fmt().try_init();
}

crate::tests! {
    uri_enum {
        use std::str::FromStr;

        let Uri::Base(base_uri) = Uri::from_str("http://example.com").expect("works") else { panic!("Didn't work!")};
        let Uri::Archive(archive_uri) = Uri::from_str("http://example.com?a=archive").expect("works") else { panic!("Didn't work!")};
        let Uri::Path(path_uri) = Uri::from_str("http://example.com?a=archive&p=path").expect("works") else { panic!("Didn't work!")};
        let Uri::Module(module_uri) = Uri::from_str("http://example.com?a=archive&m=module").expect("works") else { panic!("Didn't work!")};
        let Uri::Symbol(symbol_uri) = Uri::from_str("http://example.com?a=archive&m=module&s=symbol").expect("works") else { panic!("Didn't work!")};
        let Uri::Document(document_uri) = Uri::from_str("http://example.com?a=archive&d=document&l=en").expect("works") else { panic!("Didn't work!")};
        let Uri::DocumentElement(element_uri) = Uri::from_str("http://example.com?a=archive&d=document&l=fr&e=foo/bar/baz").expect("works") else { panic!("Didn't work!")};

        // Test URI enum conversions
        let uri_base: Uri = base_uri.clone().into();
        let uri_archive: Uri = archive_uri.into();
        let uri_path: Uri = path_uri.into();
        let uri_module: Uri = module_uri.into();
        let uri_symbol: Uri = symbol_uri.into();
        let uri_document: Uri = document_uri.into();
        let uri_element: Uri = element_uri.into();

        // Test FtmlUri implementation
        assert_eq!(uri_base.base().as_str(), "http://example.com");
        assert_eq!(uri_archive.base().as_str(), "http://example.com");
        assert_eq!(uri_path.base().as_str(), "http://example.com");
        assert_eq!(uri_module.base().as_str(), "http://example.com");
        assert_eq!(uri_symbol.base().as_str(), "http://example.com");

        // Test Display implementation
        assert_eq!(uri_base.to_string(), "http://example.com");
        assert_eq!(uri_archive.to_string(), "http://example.com?a=archive");
        assert_eq!(uri_path.to_string(), "http://example.com?a=archive&p=path");
        assert_eq!(uri_module.to_string(), "http://example.com?a=archive&m=module");
        assert_eq!(uri_symbol.to_string(), "http://example.com?a=archive&m=module&s=symbol");

        // Test conversion back to BaseUri
        let base_from_uri: BaseUri = uri_base.into();
        assert_eq!(base_from_uri, base_uri);
    };
    domain_uri {
        use std::str::FromStr;

        let DomainUri::Module(module_uri) = DomainUri::from_str("http://example.com?a=archive&p=path&m=module").expect("works") else {
            panic!("Didn't work!")
        };
        let DomainUri::Symbol(symbol_uri) = DomainUri::from_str("http://example.com?a=archive&p=path&m=module&s=symbol").expect("works") else {
            panic!("Didn't work!")
        };
        let domain_uri: DomainUri = module_uri.clone().into();
        let domain_uri2: DomainUri = symbol_uri.into();

        // Test ContentUri traits
        assert_eq!(domain_uri.base().as_str(), "http://example.com");
        assert_eq!(domain_uri.archive_id().to_string(), "archive");
        assert_eq!(domain_uri.path().expect("works").to_string(), "path");
        assert_eq!(domain_uri.module_name().to_string(), "module");
        assert_eq!(domain_uri2.base().as_str(), "http://example.com");
        assert_eq!(domain_uri2.archive_id().to_string(), "archive");
        assert_eq!(domain_uri2.path().expect("works").to_string(), "path");
        assert_eq!(domain_uri2.module_name().to_string(), "module");

        // Test Display
        assert_eq!(domain_uri.to_string(), "http://example.com?a=archive&p=path&m=module");
        assert_eq!(domain_uri2.to_string(), "http://example.com?a=archive&p=path&m=module&s=symbol");

        // Test conversions
        let base_from_domain: BaseUri = domain_uri.clone().into();
        let archive_from_domain: ArchiveUri = domain_uri.clone().into();
        let path_from_domain: PathUri = domain_uri.clone().into();
        let module_from_domain: ModuleUri = domain_uri.into();

        assert_eq!(base_from_domain.as_str(), "http://example.com");
        assert_eq!(archive_from_domain.id.to_string(), "archive");
        assert_eq!(path_from_domain.path().expect("works").to_string(), "path");
        assert_eq!(module_from_domain.name.to_string(), "module");

        // Test Uri conversion
        let uri_from_content: Uri = DomainUri::Module(module_uri).into();
        assert_eq!(uri_from_content.to_string(), "http://example.com?a=archive&p=path&m=module");
    };
    narrative_uri {
        use std::str::FromStr;

        let NarrativeUri::Document(document_uri) = NarrativeUri::from_str("http://example.com?a=archive&p=path&d=document&l=de").expect("works") else {
            panic!("Didn't work!")
        };
        let NarrativeUri::Element(element_uri) = NarrativeUri::from_str("http://example.com?a=archive&p=path&d=doc&l=de&e=elem").expect("works") else {
            panic!("Didn't work!")
        };
        let narr_uri: NarrativeUri = document_uri.clone().into();
        let narr_uri2: NarrativeUri = element_uri.into();

        // Test NarrativeUri traits
        assert_eq!(narr_uri.base().as_str(), "http://example.com");
        assert_eq!(narr_uri.archive_id().to_string(), "archive");
        assert_eq!(narr_uri.path().expect("works").to_string(), "path");
        assert_eq!(narr_uri.document_name().to_string(), "document");
        assert_eq!(narr_uri.language(), Language::German);
        assert_eq!(narr_uri2.base().as_str(), "http://example.com");
        assert_eq!(narr_uri2.archive_id().to_string(), "archive");
        assert_eq!(narr_uri2.path().expect("works").to_string(), "path");
        assert_eq!(narr_uri2.document_name().to_string(), "doc");
        assert_eq!(narr_uri2.language(), Language::German);

        // Test Display
        assert_eq!(narr_uri.to_string(), "http://example.com?a=archive&p=path&d=document&l=de");
        assert_eq!(narr_uri2.to_string(), "http://example.com?a=archive&p=path&d=doc&l=de&e=elem");

        // Test conversions
        let base_from_domain: BaseUri = narr_uri.clone().into();
        let archive_from_domain: ArchiveUri = narr_uri.clone().into();
        let path_from_domain: PathUri = narr_uri.clone().into();
        let document_from_domain: DocumentUri = narr_uri.into();

        assert_eq!(base_from_domain.as_str(), "http://example.com");
        assert_eq!(archive_from_domain.id.to_string(), "archive");
        assert_eq!(path_from_domain.path().expect("works").to_string(), "path");
        assert_eq!(document_from_domain.name.to_string(), "document");

        // Test Uri conversion
        let uri_from_narrative: Uri = NarrativeUri::Document(document_uri).into();
        assert_eq!(uri_from_narrative.to_string(),  "http://example.com?a=archive&p=path&d=document&l=de");
    };
    trait_implementations {
        use std::str::FromStr;

        let module_uri = ModuleUri::from_str("http://example.com?a=archive&p=path&m=math/algebra").expect("works");

        // Test all trait implementations
        assert_eq!(module_uri.base().as_str(), "http://example.com");
        assert_eq!(module_uri.archive_id().to_string(), "archive");
        assert_eq!(module_uri.path().expect("works").to_string(), "path");
        assert_eq!(module_uri.module_name().to_string(), "math/algebra");

        // Test trait method access
        assert_eq!(module_uri.base().as_str(), "http://example.com");
        assert_eq!(module_uri.archive_id().to_string(), "archive");
        assert_eq!(module_uri.path().expect("works").to_string(), "path");
        assert_eq!(module_uri.module_name().to_string(), "math/algebra");
    };
    uri_sizes {
        tracing::info!("Size of Uri: {}", std::mem::size_of::<Uri>());
        tracing::info!("Size of DomainUri: {}", std::mem::size_of::<DomainUri>());
        tracing::info!("Size of Option<Uri>: {}", std::mem::size_of::<Option<Uri>>());
        tracing::info!("Size of Option<DomainUri>: {}", std::mem::size_of::<Option<DomainUri>>());
    }
}

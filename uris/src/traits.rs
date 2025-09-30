use std::str::FromStr;

use crate::{
    ArchiveId, ArchiveUri, BaseUri, DocumentUri, DomainUri, Language, ModuleUri, NarrativeUri,
    PathUri, SimpleUriName, Uri, UriName, UriPath, UriRef,
};

/// Core trait for all FTML URI types.
///
/// This trait provides the fundamental interface that all implement. They only
/// common component of all URI types is that they have (or are) a [`BaseUri`].
///
/// # Examples
///
/// ```
/// # use ftml_uris::prelude::*;
/// # use std::str::FromStr;
/// let base = BaseUri::from_str("http://example.com").unwrap();
/// let archive_uri = ArchiveUri::from_str("http://example.com?a=my/archive").unwrap();
///
/// // Both types implement FtmlUri
/// assert_eq!(base.base().as_str(), "http://example.com");
/// assert_eq!(archive_uri.base().as_str(), "http://example.com");
/// ```
pub trait FtmlUri:
    Into<BaseUri>
    + Into<Uri>
    + PartialEq<str>
    + FromStr<Err = crate::errors::UriParseError>
    + std::fmt::Debug
    + std::fmt::Display
    + std::hash::Hash
    + crate::sealed::Sealed
{
    /// Iterate over all ancestors of this uri (including self)
    fn ancestors(self) -> impl Iterator<Item = Uri>;

    /// Returns a reference to the [`BaseUri`] component.
    fn base(&self) -> &BaseUri;
    /// whether the given string slice *might* represent this kind of Uri
    fn could_be(maybe_uri: &str) -> bool;
    fn as_uri(&self) -> UriRef<'_>;
    #[cfg(feature = "rdf")]
    /// Returns this URI as an RDF-IRI; possibly escaping invalid characters.
    fn to_iri(&self) -> oxrdf::NamedNode {
        struct Writer(String);
        impl std::fmt::Write for Writer {
            fn write_str(&mut self, s: &str) -> std::fmt::Result {
                for c in s.chars() {
                    self.write_char(c)?;
                }
                Ok(())
            }
            fn write_char(&mut self, c: char) -> std::fmt::Result {
                self.0.push_str(match c {
                    ' ' => "%20",
                    '\\' => "%5C",
                    '^' => "%5E",
                    '[' => "%5B",
                    ']' => "%5D",
                    '|' => "%7C",
                    c => {
                        self.0.push(c);
                        return Ok(());
                    }
                });
                Ok(())
            }
        }
        let mut s = Writer(String::with_capacity(64));
        let _ = std::fmt::Write::write_fmt(&mut s, format_args!("{self}"));
        oxrdf::NamedNode::new(s.0).expect("All illegal characters are replaced")
    }

    #[cfg(feature = "rdf")]
    /// Parses this URI from an RDF-IRI; possibly unescaping characters.
    ///
    /// # Errors
    /// if the iri is not a valid `Self`.
    fn from_iri(iri: oxrdf::NamedNodeRef) -> Result<Self, crate::errors::UriParseError> {
        let mut s = iri.as_str();
        if !s.contains('%') {
            return s.parse();
        }
        let mut out = String::with_capacity(64);
        while !s.is_empty() {
            if s.len() > 3 {
                match &s[..3] {
                    "%20" => out.push(' '),
                    "%5C" => out.push('\\'),
                    "%5E" => out.push('^'),
                    "%5B" => out.push('['),
                    "%5D" => out.push(']'),
                    "%7C" => out.push('|'),
                    _ => {
                        // SAFETY: !is_empty() + len > 3 even
                        let next = unsafe { s.chars().next().unwrap_unchecked() };
                        let len = next.len_utf8();
                        out.push(next);
                        s = &s[len..];
                        continue;
                    }
                }
                s = &s[3..];
            } else {
                out.push_str(s);
                break;
            }
        }
        out.parse()
    }

    /// Display as this Uri url-encoded
    fn url_encoded(&self) -> impl std::fmt::Display;
}

/// URIs that have a name component ([`DocumentUri`], [`DocumentElementUri`], [`ModuleUri`], [`SymbolUri`])
pub trait NamedUri: UriWithPath {
    fn name(&self) -> &UriName;
}

/// Trait for URI types that have an associated archive; i.e. have an [`ArchiveUri`] component.
///
/// # Examples
///
/// ```
/// # use ftml_uris::prelude::*;
/// # use std::str::FromStr;
/// let path_uri = PathUri::from_str("http://example.com?a=my/archive&p=some/path").unwrap();
///
/// assert_eq!(path_uri.archive_id().to_string(), "my/archive");
/// assert_eq!(path_uri.archive_uri().to_string(), "http://example.com?a=my/archive");
/// ```
pub trait UriWithArchive: Into<ArchiveUri> + FtmlUri {
    /// Returns a reference to the [`ArchiveUri`] component.
    fn archive_uri(&self) -> &ArchiveUri;

    /// Returns a reference to the [`ArchiveId`] of this URI.
    #[inline]
    fn archive_id(&self) -> &ArchiveId {
        &self.archive_uri().id
    }
}

/// Trait for URI types that have an associated path within an archive.
///
/// # Examples
///
/// ```
/// # use ftml_uris::prelude::*;
/// # use std::str::FromStr;
/// let path_uri = PathUri::from_str("http://example.com?a=archive&p=folder/file").unwrap();
/// let archive_uri = ArchiveUri::from_str("http://example.com?a=archive").unwrap();
/// let path_uri_no_path: PathUri = archive_uri.into();
///
/// assert_eq!(path_uri.path().unwrap().to_string(), "folder/file");
/// assert!(path_uri_no_path.path().is_none());
/// ```
pub trait UriWithPath: UriWithArchive + Into<PathUri> + FtmlUri {
    /// Returns a reference to the [`PathUri`] component.
    fn path_uri(&self) -> &PathUri;

    /// Returns the [`UriPath`] component, if present.
    #[inline]
    fn path(&self) -> Option<&UriPath> {
        self.path_uri().path.as_ref()
    }
}

/// Trait for URI types that represent domain knowledge.
///
/// This trait is implemented by URI types that identify specific domain content,
/// i.e. modules or symbols. All domain URIs have (or are) an associated [`ModuleUri`].
///
/// # Examples
///
/// ```
/// # use ftml_uris::prelude::*;
/// # use std::str::FromStr;
/// let module_uri = ModuleUri::from_str("http://example.com?a=archive&m=my/module").unwrap();
///
/// assert_eq!(module_uri.module_name().to_string(), "my/module");
/// ```
pub trait IsDomainUri: NamedUri + Into<ModuleUri> + Into<DomainUri> {
    /// Returns a reference to the [`ModuleUri`] component.
    fn module_uri(&self) -> &ModuleUri;

    /// Returns the module name.
    #[inline]
    fn module_name(&self) -> &UriName {
        &self.module_uri().name
    }
}

/// Trait for URI types that represent narration.
///
/// This trait is implemented by URI types that identify narraitve, human-oriented content,
/// i.e. documents, paragraphs, etc. All narrative URIs have (or are) an associated
/// [`DocumentUri`] and [`Language`].
///
/// # Examples
///
/// ```
/// # use ftml_uris::prelude::*;
/// # use std::str::FromStr;
/// let document_uri = DocumentUri::from_str("http://example.com?a=archive&d=document&l=en").unwrap();
///
/// assert_eq!(document_uri.document_name().as_ref(), "document");
/// assert_eq!(document_uri.language(), Language::English);
/// ```
pub trait IsNarrativeUri: NamedUri + Into<DocumentUri> + Into<NarrativeUri> {
    /// Returns a reference to the [`DocumentUri`] component.
    fn document_uri(&self) -> &DocumentUri;

    /// Returns the document's name.
    #[inline]
    fn document_name(&self) -> &SimpleUriName {
        &self.document_uri().name
    }

    /// Returns the language of the (containing) document.
    #[inline]
    fn language(&self) -> Language {
        self.document_uri().language
    }
}

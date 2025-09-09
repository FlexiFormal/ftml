use crate::{
    BaseUri, FtmlUri, UriKind, UriWithArchive,
    aux::NonEmptyStr,
    errors::{SegmentParseError, UriParseError},
};
use const_format::concatcp;
use either::Either::Right;
use std::{fmt::Write, str::FromStr};

crate::aux::macros::intern!(
    IDS = IdStore:NonEmptyStr @ 256
);

static NO_ARCHIVE_URI: std::sync::LazyLock<ArchiveUri> = std::sync::LazyLock::new(||
    // SAFETY: known to be valid ArchiveUri
    unsafe {
        ArchiveUri::from_str("http://unknown.source?a=no/archive").unwrap_unchecked()
    });

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(
        serde_with::DeserializeFromStr,
        serde_with::SerializeDisplay,
        bincode::Decode,
        bincode::Encode
    )
)]
/// A hierarchical identifier for an FTML archive.
///
/// [`ArchiveId`] represents a path-like identifier that can contain forward slashes
/// as separators, such as "org/project/archive" or "math/algebra". Archive IDs
/// are interned for efficient storage and fast equality comparisons.
///
/// [`ArchiveId`]s cannot be empty and cannot contain empty segments (no leading,
/// trailing, or consecutive forward slashes).
///
/// [`ArchiveId`]s whose last segment is (case-insensitively equal to) "meta-inf"
/// are special in that they are assumed to provide meta data for all other archives
/// in the same identifier space.
///
/// # Examples
///
/// ```
/// # use ftml_uris::prelude::*;
/// # use std::str::FromStr;
/// let archive_id = ArchiveId::from_str("org/project/archive").unwrap();
///
/// assert_eq!(archive_id.first(), "org");
/// assert_eq!(archive_id.last(), "archive");
/// assert_eq!(archive_id.steps().collect::<Vec<_>>(), vec!["org", "project", "archive"]);
///
/// // Test for META-INF detection
/// let meta_archive = ArchiveId::from_str("some/path/meta-inf").unwrap();
/// assert!(meta_archive.is_meta());
/// ```
pub struct ArchiveId(NonEmptyStr<IdStore>);
crate::ts!(ArchiveId);
impl ArchiveId {
    /// Returns a reference to the default "no archive" [`ArchiveId`].
    #[inline]
    #[must_use]
    pub fn no_archive() -> &'static Self {
        NO_ARCHIVE_URI.archive_id()
    }

    /// Returns the last segment of the hierarchical [`ArchiveId`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ftml_uris::prelude::*;
    /// # use std::str::FromStr;
    /// let archive_id = ArchiveId::from_str("org/project/archive").unwrap();
    /// assert_eq!(archive_id.last(), "archive");
    ///
    /// let simple = ArchiveId::from_str("simple").unwrap();
    /// assert_eq!(simple.last(), "simple");
    /// ```
    #[inline]
    #[must_use]
    pub fn last(&self) -> &str {
        self.0.last_of::<'/'>()
    }

    /// Returns the first segment of the hierarchical [`ArchiveId`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ftml_uris::prelude::*;
    /// # use std::str::FromStr;
    /// let archive_id = ArchiveId::from_str("org/project/archive").unwrap();
    /// assert_eq!(archive_id.first(), "org");
    ///
    /// let simple = ArchiveId::from_str("simple").unwrap();
    /// assert_eq!(simple.first(), "simple");
    /// ```
    #[inline]
    #[must_use]
    pub fn first(&self) -> &str {
        self.0.first_of::<'/'>()
    }

    /// Returns an iterator over all segments in the hierarchical [`ArchiveId`].
    ///
    /// The iterator supports both forward and backward iteration.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ftml_uris::prelude::*;
    /// # use std::str::FromStr;
    /// let archive_id = ArchiveId::from_str("org/project/archive").unwrap();
    /// let segments: Vec<&str> = archive_id.steps().collect();
    /// assert_eq!(segments, vec!["org", "project", "archive"]);
    ///
    /// let reversed: Vec<&str> = archive_id.steps().rev().collect();
    /// assert_eq!(reversed, vec!["archive", "project", "org"]);
    /// ```
    #[inline]
    #[must_use]
    pub fn steps(&self) -> impl DoubleEndedIterator<Item = &str> {
        self.0.segmented::<'/'>()
    }

    /// Returns `true` if this [`ArchiveId`] represents a META-INF archive.
    ///
    /// META-INF archives are special archives that contain metadata
    /// for all archives in the same identifier space.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ftml_uris::prelude::*;
    /// # use std::str::FromStr;
    /// assert!(ArchiveId::from_str("some/path/meta-inf").unwrap().is_meta());
    /// assert!(ArchiveId::from_str("some/path/META-INF").unwrap().is_meta());
    /// assert!(ArchiveId::from_str("some/path/Meta-Inf").unwrap().is_meta());
    /// assert!(!ArchiveId::from_str("some/path/metadata").unwrap().is_meta());
    /// assert!(!ArchiveId::from_str("meta-inf/subpath").unwrap().is_meta());
    /// ```
    #[must_use]
    pub fn is_meta(&self) -> bool {
        self.last().eq_ignore_ascii_case("meta-inf")
    }

    /// Creates a new [`ArchiveId`] from a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the string:
    /// - Is empty
    /// - Contains empty segments (consecutive forward slashes)
    /// - Contains illegal characters (backslash, curly braces)
    /// - Exceeds the maximum length supported by the interning system (`u32::MAX`)
    ///
    /// # Examples
    ///
    /// ```
    /// # use ftml_uris::prelude::*;
    /// let archive_id = ArchiveId::new("org/project/archive").unwrap();
    /// assert_eq!(archive_id.to_string(), "org/project/archive");
    ///
    /// // These will fail
    /// assert!(ArchiveId::new("").is_err());
    /// assert!(ArchiveId::new("a//b").is_err());
    /// assert!(ArchiveId::new("a/b\\c").is_err());
    /// ```
    #[inline]
    pub fn new(s: &str) -> Result<Self, SegmentParseError> {
        Ok(Self(NonEmptyStr::new_with_sep::<'/'>(s)?))
    }
}
impl FromStr for ArchiveId {
    type Err = SegmentParseError;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}
impl std::fmt::Display for ArchiveId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl AsRef<str> for ArchiveId {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
crate::debugdisplay!(ArchiveId);

// ----------------------------------------------------------------------------------------

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serde",
    derive(
        serde_with::DeserializeFromStr,
        serde_with::SerializeDisplay,
        bincode::Decode,
        bincode::Encode
    )
)]
/// A URI that identifies a specific archive within an FTML system.
///
/// An [`ArchiveUri`] combines a [`BaseUri`] with an [`ArchiveId`] to create a complete
/// reference to an archive. Archive URIs have the form:
/// `http://example.com?a=archive/id`
///
/// # Examples
///
/// ```
/// # use ftml_uris::prelude::*;
/// # use std::str::FromStr;
/// let archive_uri = ArchiveUri::from_str("http://example.com?a=org/project").unwrap();
///
/// assert_eq!(archive_uri.id.to_string(), "org/project");
/// assert_eq!(archive_uri.base.as_str(), "http://example.com");
/// assert_eq!(archive_uri.to_string(), "http://example.com?a=org/project");
///
/// // Archive URIs can be used with the bitwise AND operator to create them
/// let base = BaseUri::from_str("http://example.com").unwrap();
/// let archive_id = ArchiveId::from_str("my/archive").unwrap();
/// let archive_uri2 = base & archive_id;
/// ```
pub struct ArchiveUri {
    /// The base URI component.
    pub base: BaseUri,
    /// The archive identifier component.
    pub id: ArchiveId,
}
crate::ts!(ArchiveUri = TS2);
crate::debugdisplay!(ArchiveUri);
impl crate::sealed::Sealed for ArchiveUri {}

impl ArchiveUri {
    pub(crate) const SEPARATOR: char = 'a';
    /// Returns a default "no archive" URI.
    ///
    #[must_use]
    pub fn no_archive() -> Self {
        NO_ARCHIVE_URI.clone()
    }

    /// Internal parsing method used by URI parsing infrastructure.
    ///
    /// This method handles the common parsing logic for archive URIs and
    /// URI types that extend archive URIs (like path URIs and module URIs).
    pub(super) fn pre_parse<R>(
        s: &str,
        uri_kind: UriKind,
        f: impl FnOnce(Self, std::str::Split<char>) -> Result<R, UriParseError>,
    ) -> Result<R, UriParseError> {
        let Right((base, mut split)) = BaseUri::pre_parse(s)? else {
            return Err(UriParseError::MissingPartFor {
                uri_kind,
                part: crate::UriComponentKind::a,
            });
        };
        let Some(archive) = split.next() else {
            unreachable!()
        };
        if !archive.starts_with(concatcp!(ArchiveUri::SEPARATOR, "=")) {
            return Err(UriParseError::MissingPartFor {
                uri_kind,
                part: crate::UriComponentKind::a,
            });
        }
        let archive = Self {
            base,
            id: ArchiveId::new(&archive[2..])?,
        };
        f(archive, split)
    }
}
impl From<ArchiveUri> for BaseUri {
    #[inline]
    fn from(value: ArchiveUri) -> Self {
        value.base
    }
}
impl FtmlUri for ArchiveUri {
    fn url_encoded(&self) -> impl std::fmt::Display {
        struct Enc<'a>(&'a ArchiveUri);
        impl std::fmt::Display for Enc<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.base.url_encoded().fmt(f)?;
                f.write_str("%3F")?;
                f.write_char(ArchiveUri::SEPARATOR)?;
                f.write_str("%3D")?;
                urlencoding::Encoded(self.0.id.as_ref()).fmt(f)
            }
        }
        Enc(self)
    }
    #[inline]
    fn base(&self) -> &BaseUri {
        &self.base
    }

    #[inline]
    fn as_uri(&self) -> crate::UriRef<'_> {
        crate::UriRef::Archive(self)
    }

    fn could_be(maybe_uri: &str) -> bool {
        let Some((start, e)) = maybe_uri.split_once('?') else {
            return false;
        };
        BaseUri::could_be(start) && e.starts_with("a=") && !e.contains(['&', '?', '\\'])
    }
}
impl PartialEq<str> for ArchiveUri {
    fn eq(&self, other: &str) -> bool {
        let Some((p, r)) = other.split_once("?a=") else {
            return false;
        };
        self.base == *p && *self.id.as_ref() == *r
    }
}
impl UriWithArchive for ArchiveUri {
    #[inline]
    fn archive_uri(&self) -> &ArchiveUri {
        self
    }
}

impl std::fmt::Display for ArchiveUri {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}?{}={}", self.base, Self::SEPARATOR, self.id)
    }
}
impl FromStr for ArchiveUri {
    type Err = UriParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::pre_parse(s, UriKind::Archive, |a, mut split| {
            if split.next().is_some() {
                return Err(UriParseError::TooManyPartsFor {
                    uri_kind: UriKind::Archive,
                });
            }
            Ok(a)
        })
    }
}

// -----------------------------------------------------------------------------------------

crate::tests! {
    archives {
        tracing::info!("Size of ArchiveId: {}",std::mem::size_of::<ArchiveId>());
        tracing::info!("Size of ArchiveUri: {}",std::mem::size_of::<ArchiveUri>());
    };
    archive_uri_parsing {
        // Valid archive URIs
        let uri = ArchiveUri::from_str("http://example.com?a=archive/id").expect("works");
        assert_eq!(uri.base.to_string(), "http://example.com");
        assert_eq!(uri.id.to_string(), "archive/id");

        // Invalid archive URIs
        assert!(ArchiveUri::from_str("http://example.com").is_err());
        assert!(ArchiveUri::from_str("http://example.com?b=wrong").is_err());
        assert!(ArchiveUri::from_str("http://example.com?a=").is_err());
    };
    archive_steps {
        let archive = ArchiveId::from_str("org/example/project").expect("works");

        let segments: Vec<&str> = archive.steps().collect();
        assert_eq!(segments, vec!["org", "example", "project"]);

        assert_eq!(archive.first(), "org");
        assert_eq!(archive.last(), "project");

        // Test reverse iteration
        let rev_segments: Vec<&str> = archive.steps().rev().collect();
        assert_eq!(rev_segments, vec!["project", "example", "org"]);
    };
    meta_inf {
        assert!(ArchiveId::from_str("some/path/meta-inf").expect("works").is_meta());
        assert!(ArchiveId::from_str("some/path/META-INF").expect("works").is_meta());
        assert!(ArchiveId::from_str("some/path/Meta-Inf").expect("works").is_meta());
        assert!(!ArchiveId::from_str("some/path/metadata").expect("works").is_meta());
        assert!(!ArchiveId::from_str("meta-inf/subpath").expect("works").is_meta());
    }
}

use crate::extraction::{
    ArgumentPosition, FtmlExtractionError, FtmlExtractor, OpenDomainElement, OpenNarrativeElement,
    attributes::Attributes,
};
use ftml_ontology::{
    domain::declarations::symbols::{ArgumentSpec, AssocType, SymbolData},
    narrative::{
        documents::{DocumentCounter, DocumentStyle},
        elements::{
            SectionLevel,
            paragraphs::{ParagraphFormatting, ParagraphKind},
            variables::VariableData,
        },
    },
    terms::{ArgumentMode, Term, VarOrSym, Variable},
};
use ftml_uris::{
    DocumentElementUri, DocumentUri, Id, IsNarrativeUri, Language, ModuleUri, SymbolUri, UriName,
    errors::SegmentParseError,
};
use std::{borrow::Cow, num::NonZeroU8, str::FromStr};

#[cfg(doc)]
use ftml_ontology::narrative::{documents::Document, elements::DocumentElement};

macro_rules! ftml {
    () => {
        "data-ftml-"
    };
    ($l:literal) => {
        concat!(ftml!(), $l)
    };
}
pub const PREFIX: &str = "data-ftml-";
pub const NUM_KEYS: u8 = 119;
/*
pub struct FtmlRuleSet<E: crate::extraction::FtmlExtractor>(
    pub(crate)  [fn(
        &mut E,
        &mut E::Attributes<'_>,
        &mut KeyList,
        &E::Node,
    ) -> Result<
        (E::Return, Option<crate::extraction::CloseFtmlElement>),
        crate::extraction::FtmlExtractionError,
    >; NUM_KEYS as usize],
);

pub struct KeyList(pub(crate) smallvec::SmallVec<FtmlKey, 4>);
 */
use crate::extraction::{FtmlRuleSet, KeyList};

#[allow(clippy::unnecessary_wraps)]
fn todo<E: crate::extraction::FtmlExtractor>(
    key: FtmlKey,
    ext: &mut E,
    _: &mut E::Attributes<'_>,
    _: &mut KeyList,
    node: &E::Node,
) -> Result<
    (E::Return, Option<crate::extraction::CloseFtmlElement>),
    crate::extraction::FtmlExtractionError,
> {
    tracing::warn!("Not yet implemented: {key}");
    Ok((
        ext.add_element(crate::extraction::OpenFtmlElement::None, node)?,
        None,
    ))
}

#[allow(clippy::unnecessary_wraps)]
fn no_op<E: crate::extraction::FtmlExtractor>(
    key: FtmlKey,
    ext: &mut E,
    _: &mut E::Attributes<'_>,
    _: &mut KeyList,
    node: &E::Node,
) -> Result<
    (E::Return, Option<crate::extraction::CloseFtmlElement>),
    crate::extraction::FtmlExtractionError,
> {
    tracing::warn!("auxilliary key {key} missing its main attribute");
    Ok((
        ext.add_element(crate::extraction::OpenFtmlElement::None, node)?,
        None,
    ))
}

macro_rules! opt {
    ($e:expr) => {
        match $e {
            Ok(r) => Some(r),
            Err(FtmlExtractionError::MissingKey(_)) => None,
            Err(e) => return Err(e),
        }
    };
}

macro_rules! ret {
    ($ext:ident,$node:ident) => {Ok(($ext.add_element(OpenFtmlElement::None,$node)?,None))};
    (@I $ext:ident,$node:ident <- $id:ident{$($b:tt)*} + $r:expr) => {
        Ok(($ext.add_element(OpenFtmlElement::$id{$($b)*},$node)?,$r))
    };
    (@I $ext:ident,$node:ident <- $id:ident($($a:expr),*) + $r:expr) => {
        Ok(($ext.add_element(crate::extraction::OpenFtmlElement::$id( $($a),* ),$node)?,$r))
    };
    (@I $ext:ident,$node:ident <- $id:ident + $r:expr) => {
        Ok(($ext.add_element(OpenFtmlElement::$id,$node)?,$r))
    };
    ($ext:ident,$node:ident <- $id:ident{$($b:tt)*} + $r:ident) => {
        ret!(@I $ext,$node <- $id{$($b)*} + Some(crate::extraction::CloseFtmlElement::$r))
    };
    ($ext:ident,$node:ident <- $id:ident( $($a:expr),* ) + $r:ident) => {
        ret!(@I $ext,$node <- $id( $($a),*) + Some(crate::extraction::CloseFtmlElement::$r))
    };
    ($ext:ident,$node:ident <- $id:ident + $r:ident) => {
        ret!(@I $ext,$node <- $id + Some(crate::extraction::CloseFtmlElement::$r))
    };
    ($ext:ident,$node:ident <- $id:ident{$($b:tt)*}) => {
        ret!(@I $ext,$node <- $id{$($b)*} + None)
    };
    ($ext:ident,$node:ident <- $id:ident( $($a:expr),* )) => {
        ret!(@I $ext,$node <- $id( $($a),*) + None)
    };
    ($ext:ident,$node:ident <- $id:ident) => {
        ret!(@I $ext,$node <- $id + None)
    };
}
macro_rules! del {
    ($keys:ident - $($k:ident),* $(,)?) => {
        $keys.0.retain(|e| !matches!(e,$(FtmlKey::$k)|*))
    }
}

macro_rules! do_keys {
    (@LDOC) => {""};
    (@RDOC) => {""};
    (@ADOC) => {""};

    (@LDOC -!$not:literal $($rest:tt)*) => {do_keys!(@LDOC $($rest)*)};
    (@RDOC -!$not:literal $($rest:tt)*) => {do_keys!(@RDOC $($rest)*)};
    (@ADOC -!$not:literal $($rest:tt)*) => {
        concat!(
            "\n\n<div class=\"warning\">\n\n*Not allowed ",$not,"*\n\n</div>\n\n",
            do_keys!(@ADOC $($rest)*)
        )
    };

    (@LDOC !$not:literal $($rest:tt)*) => {do_keys!(@LDOC $($rest)*)};
    (@RDOC !$not:literal $($rest:tt)*) => {do_keys!(@RDOC $($rest)*)};
    (@ADOC !$not:literal $($rest:tt)*) => {
        concat!(
            "\n\n<div class=\"warning\">\n\n*Only allowed ",$not,"*\n\n</div>\n\n",
            do_keys!(@ADOC $($rest)*)
        )
    };

    (@LDOC -( $($req:ident),+ ) $($rest:tt)*) => {do_keys!(@LDOC $($rest)*)};
    (@RDOC -( $($req:ident),+ ) $($rest:tt)*) => {
        concat!(
            "Attribute of: " $(,
                "[" ,stringify!($req),"](FtmlKey::",stringify!($req), "), "
            )*,
            do_keys!(@RDOC $($rest)*)
        )
    };
    (@ADOC -( $($req:ident),+ ) $($rest:tt)*) => {do_keys!(@ADOC $($rest)*)};

    (@LDOC &( $($req:ident),+ ) $($rest:tt)*) => {do_keys!(@LDOC $($rest)*)};
    (@RDOC &( $($req:ident),+ ) $($rest:tt)*) => {
        concat!(
            "Children: " $(,
                "[" ,stringify!($req),"](FtmlKey::",stringify!($req), "), "
            )*,"<br/>",
            do_keys!(@RDOC $($rest)*)
        )
    };
    (@ADOC &( $($req:ident),+ ) $($rest:tt)*) => {do_keys!(@ADOC $($rest)*)};

    (@LDOC +( $($other:ident),* ) $($rest:tt)*) => {do_keys!(@LDOC $($rest)*)};
    (@RDOC +( $($other:ident),* ) $($rest:tt)*) => {
        concat!(
            "Additional attributes: "
            $(, "[",stringify!($other),"](FtmlKey::",stringify!($other), "), " )*,"<br/>",
            do_keys!(@RDOC $($rest)*)
        )
    };
    (@ADOC +( $($other:ident),* ) $($rest:tt)*) => {do_keys!(@ADOC $($rest)*)};

    (@LDOC <=( $($other:ident),* ) $($rest:tt)*) => {do_keys!(@LDOC $($rest)*)};
    (@RDOC <=( $($other:ident),* ) $($rest:tt)*) => {
        concat!(
            "Only allowed in: "
            $(, "[",stringify!($other),"](FtmlKey::",stringify!($other), "), " )*,"<br/>",
            do_keys!(@RDOC $($rest)*)
        )
    };
    (@ADOC <=( $($other:ident),* ) $($rest:tt)*) => {do_keys!(@ADOC $($rest)*)};

    (@LDOC -?($tp:ty) $($rest:tt)*) => {
        concat!(
            "`[`=\"`<[",stringify!($tp),"]>`\"`]`",
            do_keys!(@LDOC $($rest)*)
        )
    };
    (@RDOC -?($tp:ty) $($rest:tt)*) => {do_keys!(@RDOC $($rest)*)};
    (@ADOC -?($tp:ty) $($rest:tt)*) => {do_keys!(@ADOC $($rest)*)};

    (@LDOC =$tp:literal $($rest:tt)*) => {
        concat!(
            "<code>=\"&lt;",$tp,"&gt;\"</code>",
            do_keys!(@LDOC $($rest)*)
        )
    };
    (@RDOC =$tp:literal $($rest:tt)*) => {do_keys!(@RDOC $($rest)*)};
    (@ADOC =$tp:literal $($rest:tt)*) => {do_keys!(@ADOC $($rest)*)};

    (@DOC $tag:ident = $key:literal $($rest:tt)*) => {
        concat!(
            "<div class=\"ftml-syntax\">\n\n","`",ftml!($key),"`",
            do_keys!(@LDOC $($rest)*),
            "\n\n",
            do_keys!(@RDOC $($rest)*),
            "\n\n</div>",
            do_keys!(@ADOC $($rest)*),"\n\n"
        )
    };
    (@ENUM $(
        $(#[$meta:meta])*
        $tag:ident = $key:literal
        $({$($rest:tt)*})? :=
            $($todo:ident)?
            $(
                ($ext:ident,$attrs:ident,$keys:ident,$node:ident) => {$($impl:tt)+}
                $(=> $open:ident $({$($f:ident:$ft:ty),*$(,)?})? $( ($($tn:ident:$t:ty),*) )? )?
                $(+ $close:ident => $closeb:block   )?
            )?
    ),* $(,)? ) => {
        #[allow(clippy::unsafe_derive_deserialize)]
        #[derive(Copy,Clone,PartialEq, Eq,Hash,serde::Serialize, serde::Deserialize)]
        #[cfg_attr(feature = "typescript", wasm_bindgen::prelude::wasm_bindgen)]
        #[repr(u8)]
        pub enum FtmlKey {
            $(
                #[doc = do_keys!(@DOC $tag = $key $( $($rest)* )? )]
                $(#[$meta])* $tag
            ),*
        }

        impl FtmlKey {
            #[must_use]#[inline]
            pub const fn as_str(self) -> &'static str {
                static AS_STRS: [&str;NUM_KEYS as usize] = [$($key),*];
                AS_STRS[(self as u8) as usize]
            }
            #[inline]#[must_use]
            pub const fn as_u8(self) -> u8 {
                self as _
            }
            #[must_use]
            pub const fn from_u8(b:u8) -> Option<Self> {
                $(
                    if b == Self::$tag as u8 { return Some(Self::$tag);}
                )*
                None
            }
            #[must_use]#[inline]
            pub const fn attr_name(self) -> &'static str {
                static ATTR_STRS: [&str;NUM_KEYS as usize] = [$(ftml!($key)),*];
                ATTR_STRS[(self as u8) as usize]
            }
            #[must_use]
            pub fn from_attr(s:&str) -> Option<Self> {
                match s {
                    $( ftml!($key) => Some(Self::$tag) ),*,
                    _ => None
                }
            }

            #[must_use]
            pub const fn all_rules<E:crate::extraction::FtmlExtractor>() -> FtmlRuleSet<E> {
                FtmlRuleSet([$(
                    do_keys!(@fun $tag $($todo)? $(
                        ($ext,$attrs,$keys,$node) => {$($impl)*}
                        //$(=> $open $($b)? $( ($($t),*) )?     )?
                    )? )
                ),*])
            }
        }
        pub enum OpenFtmlElement {
            $($($(
                $open $({$($f:$ft),*})? $( ( $($t),*) )? ,
            )?)?)*
            SymbolReference {
                uri: SymbolUri,
                notation: Option<Id>,
            },
            VariableReference {
                var: Variable,
                notation: Option<Id>,
            },
            OMA {
                head: VarOrSym,
                notation: Option<Id>,
                uri: Option<DocumentElementUri>,
            },
            OMBIND {
                head: VarOrSym,
                notation: Option<Id>,
                uri: Option<DocumentElementUri>,
            },
            ComplexTerm {
                head: VarOrSym,
                notation: Option<Id>,
                uri: Option<DocumentElementUri>,
            },
            OML {
                name: UriName,
            },
            SectionTitle,
            ParagraphTitle,
            None,
        }
        impl<N: crate::extraction::nodes::FtmlNode + std::fmt::Debug> crate::extraction::state::ExtractorState<N> {
            /*pub fn add2(&mut self,e:OpenFtmlElement,node:&N) {
                match e {
                    $($($(
                        OpenFtmlElement::$open $({$($f),*})? $( ( $($tn),*) )? => todo!() ,
                    )?)?)*
                    _ => ()
                }
            }*/
        }
    };

    (@fun $self:ident todo) => { |e,a,k,n| todo(Self::$self,e,a,k,n) };
    (@fun $self:ident noop) => { |e,a,k,n| no_op(Self::$self,e,a,k,n) };
    (@fun $self:ident ($ext:ident,$attrs:ident,$keys:ident,$node:ident) => {$($impl:tt)+} ) => {
        |$ext,$attrs,$keys,$node| { $($impl)*}
    };

    ( $(
        $(#[$meta:meta])*
        $tag:ident = $key:literal
        $({ $($rest:tt)* })?
        :=
            $($todo:ident)?
            $(
                ($ext:ident,$attrs:ident,$keys:ident,$node:ident) => {$($impl:tt)+}
                $(=> $open:ident $({$($f:ident:$ft:ty),*$(,)?})? $( ($($tn:ident:$t:ty),*) )? )?
                $(+ $close:ident => $closeb:block   )?
            )?
    ),* $(,)? ) => {
        do_keys!{@ENUM $( $(#[$meta])* $tag = $key $({ $($rest)*})? :=
            $($todo)?
            $(
                ($ext,$attrs,$keys,$node) => {$($impl)*}
                $(=> $open $({$($f:$ft),*})? $( ( $($tn:$t),*) )?     )?
                $(+ $close => $closeb   )?
            )?
        ),*}
    };
}

impl std::fmt::Display for FtmlKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::fmt::Debug for FtmlKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.attr_name())
    }
}

do_keys! {
    /// Denotes the title of the current document (if any). Should occur at most once.
    DocTitle = "doctitle"
        := (ext,_attrs,_keys,node) => {
            ret!(ext, node <- None + DocTitle)
        },

    /// Declares a new CSS style for a section or logical paragraph. May reference a counter
    /// used for numbering the paragraphs/sections of this style.
    Style = "style"
        { ="[DocumentStyle]" +(Counter) }
        := (ext,attrs,keys,node) => {
            let mut style = attrs.get_typed(FtmlKey::Style, |s| {
                DocumentStyle::from_str(s).map_err(|_| ())
            })?;
            if let Some(count) = opt!(attrs.get_typed(FtmlKey::Counter, Id::from_str)) {
                style.counter = Some(count);
            }
            del!(keys - Counter);
            ret!(ext,node <- Style(style))
        } => Style(style:DocumentStyle),

    /// Declares the referenced counter to have this one as a parent; meaning, whenever
    /// the parent is increased, the counter is reset to 0.
    CounterParent = "counter-parent"
        {="[Id]" +(Counter)}
        := (ext,attrs,keys,node) => {
            let name = attrs.get_typed(FtmlKey::Counter, Id::from_str)?;
            let parent: SectionLevel = {
                let lvl = attrs.get_typed(FtmlKey::CounterParent, |s| {
                    u8::from_str(s).map_err(|_| ())
                })?;
                lvl.try_into()
                    .map_err(|_| FtmlExtractionError::InvalidValue(FtmlKey::CounterParent))?
            };
            del!(keys - Counter,CounterParent);
            ret!(ext,node <- Counter(DocumentCounter { name, parent:Some(parent) }))
        } => Counter(counter:DocumentCounter),

    /// Declares a new counter with an optional [`CounterParent`](FtmlKey::CounterParent)
    Counter = "counter"
        {="[Id]" +(CounterParent)}
        := (ext,attrs,keys,node) => {
            let name = attrs.get_typed(FtmlKey::Counter, Id::from_str)?;
            let parent: Option<SectionLevel> = {
                let lvl = opt!(attrs.get_typed(FtmlKey::CounterParent, |s| {
                    u8::from_str(s).map_err(|_| ())
                }));
                lvl.map(|lvl| {
                    lvl.try_into()
                        .map_err(|_| FtmlExtractionError::InvalidValue(FtmlKey::CounterParent))
                })
                .transpose()?
            };
            del!(keys - Counter,CounterParent);
            ret!(ext,node <- Counter(DocumentCounter { name, parent }))
        },

    /// A [`DocumentReference`](DocumentElement::DocumentReference); Inserts the referenced [`Document`] here; loosely analogous to an iframe, but the referenced
    /// [`Document`] is adapted to the current document in various ways (e.g. wrt [`SectionLevel`]s).
    InputRef = "inputref"
        { ="[DocumentUri]" +(Id) }
        := (ext,attrs,_keys,node) => {
            let target = attrs.get_document_uri(FtmlKey::InputRef)?;
            let uri = attrs.get_elem_uri_from_id(ext, Cow::Owned(target.document_name().to_string()))?;
            ret!(ext,node <- InputRef{uri,target})
        } => InputRef{uri:DocumentElementUri,target:DocumentUri},

    /// If `true`, shows the node iff the current document is being rendered as an
    /// [`InputRef`](FtmlKey::InputRef) in some other (top-level) document;
    /// conversely, if `false`, shows the node only iff the current document *is* the top-level document.
    ///
    /// Useful to wrap e.g. lists of references, titles etc., so they are not shown when "inputreffed".
    IfInputref = "ifinputref"
        { ="[bool]" }
        := (ext,attrs,_keys,node) => {
            let value = attrs.get_bool(FtmlKey::IfInputref);
            ret!(ext,node <- IfInputref(value))
        } => IfInputref(value:bool),

    /// Marks that this document / section / block makes use of symbols rom the referenced module.
    /// Useful for some bookkeeping, but not strictly necessary.
    UseModule = "usemodule"
        { ="[ModuleUri]" }
        := (ext,attrs,_keys,node) => {
            let uri = attrs.take_symbol_or_module_uri(FtmlKey::UseModule)?;
            ret!(ext,node <- UseModule(uri))
        } => UseModule(uri:ModuleUri),

    /// Denotes a new [`Section`]. The [`SectionLevel`] is determined by its nested occurrence
    /// within a [`Document`].
    Section = "section"
        { +(Id) -!"in [`LogicalParagraph`]s, [`Problem`]s or [`Slide`]s" &(Title) }
        := (ext,attrs,keys,node) => {
            let uri = attrs.get_elem_uri_from_id(ext, "section")?;
            del!(keys - Id);
            ret!(ext,node <- Section(uri) + Section)
        } => Section(uri:DocumentElementUri),

    /// Behaves internally like a [`Section`] without producing any output; in particular,
    /// increases the [`SectionLevel`] by 1 in its scope. Useful to e.g. make an introductory
    /// section 0.1 before the first actual chapter 1.
    SkipSection = "skipsection"
        := (ext,_attrs,_keys,node) => {
            ret!(ext,node <- SkipSection + SkipSection)
        } => SkipSection,

    /// Sets the top-most [`SectionLevel`] to use in this document. Should occur at most once and *before*
    /// any section.
    SetSectionLevel = "sectionlevel"
        { ="[SectionLevel] as [u8]" }
        := (ext,attrs,_keys,node) => {
            let lvl = attrs.get_typed(FtmlKey::SetSectionLevel, |s| {
                u8::from_str(s).map_err(|_| ())
            })?;
            let lvl: SectionLevel = lvl
                .try_into()
                .map_err(|_| FtmlExtractionError::InvalidValue(FtmlKey::SetSectionLevel))?;
            ret!(ext,node <- SetSectionLevel(lvl))
        } => SetSectionLevel(lvl:SectionLevel),

    /// Gets dynamically replaced by the current [`SectionLevel`]. If [`Capitalize`](FtmlKey::Capitalize)
    /// is `true`, the first letter gets capitalized ("Chapter"), otherwise not ("chapter").
    /// Outside of any section, yields "document".
    CurrentSectionLevel = "currentsectionlevel"
        { +(Capitalize) }
        := (ext,attrs,keys,node) => {
            let cap = attrs.get_bool(FtmlKey::Capitalize);
            del!(keys - Capitalize);
            ret!(ext,node <- CurrentSectionLevel(cap))
        } => CurrentSectionLevel(cap:bool),

    Capitalize = "capitalize"
        {-(CurrentSectionLevel)}
        := todo,

    // ------------------------------------------------------------------------------------

    /// Denotes a new [`LogicalParagraph`] of [`ParagraphKind::Definition`]
    /// for the given [Symbol]s using the given styles.
    Definition = "definition"
        { +(Id,Inline,Fors,Styles) &(Title,Definiens, Definiendum) }
        := (ext,attrs,keys,node) => {
            do_paragraph(ext, attrs, keys, node, ParagraphKind::Definition)
        },

    /// Denotes a new [`LogicalParagraph`] of [`ParagraphKind::Assertion`] (Theorems, Lemmata,
    /// Axioms, etc.) for the given [Symbol]s using the given styles.
    Assertion = "assertion"
        { +(Id,Inline,Fors,Styles) &(Title) }
        := (ext,attrs,keys,node) => {
            do_paragraph(ext, attrs, keys, node, ParagraphKind::Assertion)
        },

    /// Denotes a new [`LogicalParagraph`] of [`ParagraphKind::Example`] (this includes counterexamples)
    /// for the given [Symbol]s using the given styles.
    Example = "example"
        {+(Id,Inline,Fors,Styles) &(Title) }
        := (ext,attrs,keys,node) => {
            do_paragraph(ext, attrs, keys, node, ParagraphKind::Example)
        },

    /// Denotes a new [`LogicalParagraph`] of [`ParagraphKind::Proof`]
    /// for the given [Symbol]s using the given styles.
    Proof = "proof"
        {+(Id,Inline,Fors,Styles,ProofHide)}
        := (ext,attrs,keys,node) => {
            do_paragraph(ext, attrs, keys, node, ParagraphKind::Proof)
        },

    /// Denotes a new [`LogicalParagraph`] of [`ParagraphKind::SubProof`]
    /// for the given [Symbol]s using the given styles.
    SubProof = "subproof"
        {+(Id,Inline,Fors,Styles,ProofHide)}
        := (ext,attrs,keys,node) => {
            do_paragraph(ext, attrs, keys, node, ParagraphKind::SubProof)
        },

    /// Denotes a new [`LogicalParagraph`] of [`ParagraphKind::Paragraph`]
    /// for the given [`Symbol`]s using the given styles.
    Paragraph = "paragraph"
        {+(Id,Inline,Fors,Styles) &(Title) }
        := (ext,attrs,keys,node) => {
            do_paragraph(ext, attrs, keys, node, ParagraphKind::Paragraph)
        } => Paragraph{
            kind:ParagraphKind,
            formatting:ParagraphFormatting,
            styles:Box<[Id]>,
            uri:DocumentElementUri,
            fors:Vec<(SymbolUri,Option<Term>)>
        },

    /// This [`LogicalParagraph`] is *inline*; i.e. not a separate paragraph
    Inline = "inline"
        {-(Definition, Paragraph, Assertion, Example) }
        := noop,

    /// The comma-separated list of [`Symbol`]s this paragraph is concerned with
    /// ("example for", "defines", etc.)
    Fors = "fors"
        {="[SymbolUri]*" -(Definition, Paragraph, Assertion, Example, Proof, SubProof)}
        := noop,

    /// The CSS styles to use to format this paragraph (in order of priority, if available)
    Styles = "styles"
        {-(Definition, Paragraph, Assertion, Example, Problem, SubProblem, Proof, SubProof) }
        := noop,

    ProofHide = "proofhide"
        {-(Proof,SubProof) }
        := todo,

    ProofTitle = "prooftitle"
        := todo,

    SubproofTitle = "subprooftitle"
        := todo,

    ProofMethod = "proofmethod"
        := todo,

    ProofSketch = "proofsketch"
        := todo,

    ProofTerm = "proofterm"
        := todo,

    ProofBody = "proofbody"
        := todo,

    ProofAssumption = "spfassumption"
        := todo,

    ProofStep = "spfstep"
        := todo,

    ProofStepName = "stepname"
        := todo,

    ProofEqStep = "spfeqstep"
        := todo,

    ProofPremise = "premise"
        := todo,

    ProofConclusion = "spfconclusion"
        := todo,

    // ------------------------------------------------------------------------------------

    /// Denotes a new [`Problem`] with <code>[sub_problem](Problem::sub_problem)=false</code>
    Problem = "problem"
        {+(Id,Styles,Autogradable,ProblemPoints) }
        := todo,

    /// Denotes a new [`Problem`] with <code>[sub_problem](Problem::sub_problem)=true</code>
    SubProblem = "subproblem"
        {+(Id,Styles,Autogradable,ProblemPoints) }
        := todo,

    PreconditionDimension = "preconditiondimension"
        := todo,

    PreconditionSymbol = "preconditionsymbol"
        := todo,

    ObjectiveDimension = "objectivedimension"
        := todo,

    ObjectiveSymbol = "objectivesymbol"
        := todo,

    AnswerClass = "answerclass"
        := todo,

    AnswerClassPts = "answerclass-pts"
        := todo,

    AnswerclassFeedback = "answerclass-feedback"
        := todo,

    ProblemMinutes = "problemminutes"
        := todo,

    ProblemMultipleChoiceBlock = "multiple-choice-block"
        := todo,

    ProblemSingleChoiceBlock = "single-choice-block"
        := todo,

    ProblemChoice = "problem-choice"
        := todo,

    ProblemChoiceVerdict = "problem-choice-verdict"
        := todo,

    ProblemChoiceFeedback = "problem-choice-feedback"
        := todo,

    ProblemFillinsol = "fillinsol"
        := todo,

    ProblemFillinsolWidth = "fillinsol-width"
        := todo,

    ProblemFillinsolCase = "fillin-case"
        := todo,

    ProblemFillinsolCaseValue = "fillin-case-value"
        := todo,

    ProblemFillinsolCaseVerdict = "fillin-case-verdict"
        := todo,

    ProblemSolution = "solution"
        := todo,

    ProblemHint = "problemhint"
        := todo,

    ProblemNote = "problemnote"
        := todo,

    ProblemGradingNote = "problemgnote"
        := todo,

    ProblemPoints = "problempoints"
        {-(Problem, SubProblem) }
        := todo,

    Autogradable = "autogradable"
        {-(Problem, SubProblem) }
        := todo,

        // ------------------------------------------------------------------------------------

    /// Denotes the title of the current [`Section`] or [`LogicalParagraph`]
    Title = "title"
        { <=(Section,Definition,Assertion,Example,Paragraph) }
        := (ext,attrs,keys,node) => {
            del!(keys - Invisible);
            attrs.remove(FtmlKey::Invisible);
            let mut iter = ext.iterate_narrative();
            while let Some(e) = iter.next() {
                match e {
                    OpenNarrativeElement::Section { .. } => {
                        drop(iter);
                        return ret!(ext,node <- SectionTitle + SectionTitle);
                    }
                    OpenNarrativeElement::Paragraph { .. } => {
                        drop(iter);
                        return ret!(ext,node <- ParagraphTitle + ParagraphTitle);
                    }
                    OpenNarrativeElement::SkipSection { .. }
                    | OpenNarrativeElement::Notation { .. }
                    | OpenNarrativeElement::NotationComp { .. }
                    | OpenNarrativeElement::ArgSep { .. }
                    | OpenNarrativeElement::VariableDeclaration { .. }
                    | OpenNarrativeElement::Definiendum(_)
                    | OpenNarrativeElement::NotationArg(_) => {
                        break;
                    }
                    OpenNarrativeElement::Module { .. }
                    | OpenNarrativeElement::MathStructure { .. }
                    | OpenNarrativeElement::Invisible => (),
                }
            }
            Err(FtmlExtractionError::NotIn(
                FtmlKey::Title,
                "a section or paragraph",
            ))
        },

    // --------------------------------------------------------------------------------


    /// Denotes a [`Slide`], implying that the [`Document`] is (or contains in some sense)
    /// a presentation.
    Slide = "slide"
        {+(Id) -!"in [LogicalParagraph]s, [Problem]s or [Slide]s" }
        := todo,

    /// A (possibly empty) node that, when being rendered, should be replaced by the
    /// current slide number.
    SlideNumber = "slide-number"
        { !"in [Slide](DocumentElement::Slide)s" }
        := todo,

    // ------------------------------------------------------------------------------------

    /// Denotes a new [`Module`] (or [`NestedModule`], iff already in a module) with the given
    /// [`UriName`] in the current [`Namespace`](PathURI). The full [`ModuleUri`] is computed
    /// from the given name and the current [`DocumentUri`].
    Module = "module"
        {="[UriName]" +(Metatheory, Signature) }
        := (ext,attrs,keys,node) => {
            let uri = attrs.take_new_module_uri(FtmlKey::Module, FtmlKey::Module, ext)?;
            let _ = attrs.take_language(FtmlKey::Language);
            let meta = opt!(attrs.take_module_uri(FtmlKey::Metatheory));
            let signature = opt!(attrs.take_language(FtmlKey::Signature));
            del!(keys - Language, Metatheory, Signature);
            ret!(ext,node <- Module{
                uri,
                meta,
                signature,
            } + Module)
        } => Module{uri:ModuleUri,meta:Option<ModuleUri>,signature:Option<Language>},

    /// <div class="advanced">
    ///
    /// The metatheory of a module, that provides the formal "language" the module
    /// is in.
    ///
    /// </div>
    Metatheory = "metatheory"
        { ="[ModuleUri]" -(Module)}
        := noop,

    /// If this is a translation of an existing module, the language the original
    /// module (containing the actual declarations) is in.
    Signature = "signature"
        { ="[Language]" -(Module)}
        := noop,

    /// Denotes a new [`MathStructure`] or [`Extension`] with the given [`UriName`].
    /// A conservative [`Extension`]'s [`UriName`] is expected to start with `EXTSTRUCT`
    /// and have an [`ImportModule`](StructureDeclaration::Import) as its first child.
    MathStructure = "feature-structure"
        { ="[UriName]" +(Macroname) !"in [Module]s" }
        := (ext,attrs,keys,node) => {
            let uri = attrs.take_new_symbol_uri(FtmlKey::MathStructure, FtmlKey::MathStructure, ext)?;
            let macroname = attrs
                .get(FtmlKey::Macroname)
                .map(|s| s.as_ref().parse())
                .transpose()
                .map_err(|_| (FtmlKey::Macroname, ()))?;
            del!(keys - Macroname);
            ret!(ext,node <- MathStructure{uri,macroname} + MathStructure)
        } => MathStructure{uri:SymbolUri,macroname:Option<Id>},

    /// Denotes that the current [`Module`] a) uses symbols imported from the referenced [`Module`]
    /// and b) reexports them to downstream [`Module`]s that import the current one.
    ///
    /// For [`MathStructure`]s, denotes that an instance of this structure inherits fields
    /// from the referenced one.
    ImportModule = "import"
        { = "[ModuleUri]" !"in [`Module`]s or [`MathStructure`]s" }
        := (ext,attrs,_keys,node) =>{
            let uri = attrs.take_symbol_or_module_uri(FtmlKey::ImportModule)?;
            ret!(ext,node <- ImportModule(uri))
        } => ImportModule(uri:ModuleUri),

    /// <div class="advanced">
    ///
    /// <div class="ftml-wip">TODO</div>
    ///
    /// </div>
    Morphism = "feature-morphism"
        := todo,

    /// Declares a new [`Symbol`] with the given name. Its [`SymbolUri`] is made up of the given
    /// name and the [`ModuleUri`] of the containing [`Module`] or [`MathStructure`].
    Symdecl = "symdecl"
        { ="[UriName]"
            +(Role,AssocType,Args,ArgumentReordering,Macroname)
            &(Type,Definiens,ReturnType,ArgTypes)
            !"in [`Module`]s or [`MathStructure`]s"
        }
        := (ext,attrs,keys,node) => {
            let uri = attrs.get_new_symbol_uri(FtmlKey::Symdecl, FtmlKey::Symdecl, ext)?;
            let role = opt!(attrs.get_typed(FtmlKey::Role, |s| {
                Ok::<_, SegmentParseError>(
                    s.split(',')
                        .map(|s| s.trim().parse::<Id>())
                        .collect::<std::result::Result<Vec<_>, SegmentParseError>>()?
                        .into_boxed_slice(),
                )
            }))
            .unwrap_or_default();
            let assoctype = opt!(attrs.get_typed(FtmlKey::AssocType, |s| {
                AssocType::from_str(s).map_err(|_| ())
            }));
            let arity = opt!(attrs.get_typed(FtmlKey::Args, |s| {
                ArgumentSpec::from_str(s).map_err(|_| ())
            }))
            .unwrap_or_default();
            let reordering = attrs
                .get(FtmlKey::ArgumentReordering)
                .map(|s| s.as_ref().parse())
                .transpose()
                .map_err(|_| (FtmlKey::ArgumentReordering, ()))?;
            let macroname = attrs
                .get(FtmlKey::Macroname)
                .map(|s| s.as_ref().parse())
                .transpose()
                .map_err(|_| (FtmlKey::Macroname, ()))?;
            del!(keys - Role, AssocType, Args, ArgumentReordering, Macroname);
            ret!(ext,node <- SymbolDeclaration {
                uri,
                data: Box::new(SymbolData {
                    arity,
                    macroname,
                    role,
                    assoctype,
                    reordering,
                    return_type:None,
                    argument_types:Box::new([]),
                    tp: None,
                    df: None,
                }),
            } + SymbolDeclaration)
        } => SymbolDeclaration{uri:SymbolUri,data:Box<SymbolData>},

    /// Declares a new [`VariableDeclaration`] with the given name.
    /// Its [`DocumentElementUri`] is made up of the given
    /// name and the current [`NarrativeUri`].
    Vardef = "vardef"
        { ="[UriName]"
            +(Role,AssocType,Args,ArgumentReordering,Macroname,Bind)
            &(Type,Definiens,ReturnType,ArgTypes)
        }
        := (ext,attrs,keys,node) => {do_vardef(ext, attrs, keys, node, FtmlKey::Vardef, false)}
        => VariableDeclaration{uri:DocumentElementUri,data:Box<VariableData>},

    /// Declares a new [`VariableDeclaration`] representing a *sequence* of arbitrary length
    /// with the given name.
    /// Its [`DocumentElementUri`] is made up of the given
    /// name and the current [`NarrativeUri`].
    Varseq = "varseq"
        { ="[UriName]"
            +(Role,AssocType,Args,ArgumentReordering,Macroname,Bind)
            &(Type,Definiens,ReturnType,ArgTypes)
        }
        := (ext,attrs,keys,node) => {do_vardef(ext, attrs, keys, node, FtmlKey::Varseq, true)},

    /// The (optional) macro name of a [`Symbol`] (e.g. in $s\TeX$).
    Macroname = "macroname"
        {="[Id]" -(Symdecl,MathStructure,Vardef,Varseq)}
        := noop,

    /// <div class="ftml-wip">TODO</div>
    AssocType = "assoctype"
        {="[AssocType]" -(Symdecl,Vardef,Varseq)}
        := noop,

    /// <div class="ftml-wip">TODO</div>
    Role = "role"
        {="[Id]*" -(Symdecl,Vardef,Varseq)}
        := noop,

    /// The modes of arguments a [`Symbol`] takes; either an integer (if all arguments
    /// are simple) or a sequence of [`ArgumentMode`] characters (`i`,`a`,`b`,`B`).
    Args = "args"
        {="[ArgumentSpec]" -(Symdecl,Vardef,Varseq)}
        := noop,

    /// <div class="advanced">
    ///
    /// <div class="ftml-wip">TODO</div>
    ///
    /// </div>
    ArgumentReordering = "reorderargs"
        {="([u8],[u8])*" -(Symdecl,Vardef,Varseq)}
        := noop,

    /// <div class="advanced">
    ///
    /// <div class="ftml-wip">TODO</div>
    ///
    /// </div>
    Bind = "bind"
        {="[bool]" -(Vardef,Varseq)}
        := noop,

    // -------------------------------------------------------------------------------

    /// Denotes the *type* of the current [`Symbol`] or [`Variable`]. This node (or its only child)
    /// is interpreted to be a [`Term`].
    Type = "type"
        {<=(Symdecl, Vardef, Varseq) }
        := (ext,_attrs,_keys,node) => {
            if ext.in_term() {
                return Err(FtmlExtractionError::InvalidIn(FtmlKey::Type, "terms"));
            }
            ret!(ext,node <- Type + Type)
        } => Type,

    /// Denotes the *return type* of the current [`Symbol`] or [`Variable`], in absence of a
    /// [`Type`](FtmlKey::Type). In conjunction with [`ArgTypes`](FtmlKey::ArgTypes),
    /// the full type is assembled by binding the argument types.
    ReturnType = "returntype"
        {<=(Symdecl, Vardef, Varseq) }
        := (ext,_attrs,_keys,node) => {
            if ext.in_term() {
                return Err(FtmlExtractionError::InvalidIn(FtmlKey::ReturnType, "terms"));
            }
            ret!(ext,node <- ReturnType + ReturnType)
        } => ReturnType,

    /// Denotes the *types* of the arguments for the current [`Symbol`] or [`Variable`], in absence of a
    /// [`Type`](FtmlKey::Type). In conjunction with [`ArgTypes`](FtmlKey::ArgTypes),
    /// the full type is assembled by binding the argument types.
    ArgTypes = "argtypes"
        {<=(Symdecl, Vardef, Varseq) }
        := todo,

    /// In a [`Symdecl`], [`Vardef`] or [`Varseq`], denotes the *definiens* of the current
    /// [`Symbol`] or [`Variable`]. In a [`Definition`], a definition-like [`Paragraph`]
    /// or an [`Assertion`], denotes the definiens of the *referenced* [`Symbol`] or the
    /// *first* [`Symbol`] in the paragraph's [`Fors`](FtmlKey::Fors)-list.
    ///
    /// This node (or its only child)
    /// is interpreted to be a [`Term`].
    Definiens = "definiens"
        {="[Option]<[SymbolUri]>" <=(Definition, Paragraph, Assertion, Symdecl, Vardef, Varseq) }
        := (ext,attrs,_keys,node) => {
            if ext.in_term() {
                return Err(FtmlExtractionError::InvalidIn(FtmlKey::Definiens, "terms"));
            }
            let uri = opt!(attrs.get_symbol_uri(FtmlKey::Definiens));
            ret!(ext,node <- Definiens(uri) + Definiens)
            } => Definiens(def:Option<SymbolUri>),

    /// <div class="ftml-wip">TODO</div>
    Conclusion = "conclusion"
        := todo,

    // ---------------------------------------------------------------------------------

    /// A [Term] of the given kind with the given head, being presented using the given
    /// [NotationId](FtmlKey::NotationId):
    ///
    /// - `OMID`: a symbol reference; [`Head`](FtmlKey::Head) should be a [`SymbolUri`]
    /// - `OMV`: a variable reference; [`Head`](FtmlKey::Head) should be the [`DocumentElementUri`]
    ///     of a [`VariableDeclaration`] or a [`UriName`] of an (undeclared) variable.
    /// - `OMA`: an application. [`Head`](FtmlKey::Head) can be any of the above, the children
    ///     are expected to contain [`Argument`](FtmlKey::Arg)s.
    /// - `OMBIND`: a binding application. [`Head`](FtmlKey::Head) can be any of the above, the children
    ///     are expected to contain [`Argument`](FtmlKey::Arg)s.
    /// - `OML`: a non-alpha-renamable variable-like identifier; primarily used for the names
    ///     of fields in a record field projection.
    /// - `complex`: An arbitrary term that is being presented using the [`Head`](FtmlKey::Head);
    ///     e.g. a record field projection where the actual [`Head`](FtmlKey::Head)
    ///     is just the [`SymbolUri`] of the record type's field itself.
    Term = "term"
        { ="OMID|OMV|OMA|OMBIND|OML|complex"
            +(Head,NotationId)
            &(HeadTerm,Arg,Comp,VarComp,MainComp,DefComp)
            -!"in [`Comp`], [`VarComp`], [`MainComp`], or [`DefComp`]"
        }
        := (ext,attrs,keys,node) => {
            #[derive(Debug)]
            #[allow(clippy::upper_case_acronyms)]
            enum OpenTermKind {
                OMS,
                //OMMOD,
                OMV,
                OMA,
                OMBIND,
                OML,
                Complex,
            }
            impl std::str::FromStr for OpenTermKind {
                type Err = ();
                fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                    Ok(match s {
                        "OMID" | "OMMOD" => Self::OMS,
                        "OMV" => Self::OMV,
                        "OMA" => Self::OMA,
                        "OMBIND" => Self::OMBIND,
                        "OML" => Self::OML,
                        "complex" => Self::Complex,
                        _ => return Err(()),
                    })
                }
            }

            del!(keys - NotationId, Head);
            if ext.in_notation() {
                attrs.remove(FtmlKey::NotationId);
                attrs.remove(FtmlKey::Head);
                attrs.remove(FtmlKey::Term);
                return ret!(ext, node);
            }

            let head = attrs.get_symbol_or_var(FtmlKey::Head, ext)?;

            let kind: OpenTermKind = attrs.get_typed(FtmlKey::Term, str::parse)?;
            let notation = opt!(attrs.get_typed(FtmlKey::NotationId, str::parse));

            let in_term = |ext: &mut E| {
                Ok(!ext.in_notation()
                    && match ext.iterate_domain().next() {
                        None
                        | Some(
                            OpenDomainElement::Module { .. }
                            | OpenDomainElement::MathStructure { .. }
                            | OpenDomainElement::SymbolDeclaration { .. }
                            | OpenDomainElement::SymbolReference { .. }
                            | OpenDomainElement::VariableReference { .. }
                            | OpenDomainElement::OMA { .. }
                            | OpenDomainElement::OMBIND { .. }
                            | OpenDomainElement::OML { .. }
                            | OpenDomainElement::ComplexTerm { .. }
                            | OpenDomainElement::Type { .. }
                            | OpenDomainElement::ReturnType { .. }
                            | OpenDomainElement::Definiens { .. },
                        ) => false,
                        Some(OpenDomainElement::Argument { .. } | OpenDomainElement::HeadTerm { .. }) => {
                            true
                        }
                        Some(OpenDomainElement::Comp | OpenDomainElement::DefComp) => {
                            return Err(FtmlExtractionError::InvalidIn(
                                FtmlKey::Term,
                                "notation components",
                            ));
                        }
                    })
            };

            if let VarOrSym::Var(Variable::Ref { declaration, .. }) = &head {
                attrs.set(FtmlKey::Head.attr_name(), declaration);
            }

            match (kind, head) {
                (OpenTermKind::OMS | OpenTermKind::OMV, VarOrSym::Sym(uri)) => {
                    ret!(ext,node <- SymbolReference{uri,notation} + SymbolReference)
                }
                (OpenTermKind::OMS | OpenTermKind::OMV, VarOrSym::Var(var)) => {
                    ret!(ext,node <- VariableReference{var,notation} + VariableReference)
                }
                (OpenTermKind::OMA, head) => {
                    let uri = if in_term(ext)? {
                        None
                    } else {
                        Some(attrs.get_elem_uri_from_id(ext, Cow::Borrowed("term"))?)
                    };
                    ret!(ext,node <- OMA{head,notation,uri} + OMA)
                }
                (OpenTermKind::OMBIND, head) => {
                    let uri = if in_term(ext)? {
                        None
                    } else {
                        Some(attrs.get_elem_uri_from_id(ext, Cow::Borrowed("term"))?)
                    };
                    ret!(ext,node <- OMBIND{head,notation,uri} + OMBIND)
                }
                (OpenTermKind::Complex, head) => {
                    let uri = if in_term(ext)? {
                        None
                    } else {
                        Some(attrs.get_elem_uri_from_id(ext, Cow::Borrowed("term"))?)
                    };
                    ret!(ext,node <- ComplexTerm{head,notation,uri} + ComplexTerm)
                }
                (OpenTermKind::OML, VarOrSym::Var(Variable::Name { name, .. })) => {
                    // SAFETY: names are valid UriNames
                    let name = unsafe { name.as_ref().parse().unwrap_unchecked() };
                    ret!(ext,node <- OML{name} + OML)
                }
                (OpenTermKind::OML, VarOrSym::Var(Variable::Ref { declaration, .. })) => {
                    ret!(ext,node <- OML{name:declaration.name} + OML)
                }
                (k, _) => crate::TODO!("{k:?}"),
            }
        },

    /// The [`Id`] of the notation used to present this [`Term`].
    NotationId = "notationid"
        { ="[Id]" -(Term)}
        := noop,

    /// The head symbol of the current [`Term`].
    Head = "head"
        {="[SymbolUri]|[DocumentElementUri]|[UriName]" -(Term)}
        := noop,

    /// An argument for an application or binding application [`Term`]. This node (or its only child)
    /// is interpreted to be a [`Term`].
    Arg = "arg"
        { ="[ArgumentPosition]" +(ArgMode) !"in [`Term`]s of kind `OMA` or `OMBIND`"}
        := (ext,attrs,keys,node) => {
            let Some(index) = attrs.value(FtmlKey::Arg.attr_name()) else {
                return Err(FtmlExtractionError::MissingKey(FtmlKey::Arg));
            };
            let mode: Option<ArgumentMode> = opt!(attrs.get_typed(FtmlKey::ArgMode, |s| {
                s.parse()
                    .map_err(|_| FtmlExtractionError::InvalidValue(FtmlKey::ArgMode))
            }));
            let Some(argument) = ArgumentPosition::from_strs(index.as_ref(), mode) else {
                return Err(FtmlExtractionError::InvalidValue(FtmlKey::Arg));
            };
            del!(keys - Arg, ArgMode);
            if ext.in_term() {
                ret!(ext,node <- Argument(argument) + Argument)
            } else if ext.in_notation() {
                ret!(ext,node <- NotationArg(argument) + NotationArg)
            } else {
                Err(FtmlExtractionError::NotIn(FtmlKey::Arg, "open term"))
            }
        } => Argument(argument:ArgumentPosition),

    /// The [`ArgumentMode`] of this particular argument; one of the characters `i`,`a`,`b`,`B`.
    ArgMode = "argmode"
        {= "[ArgumentMode]" <=(Arg)}
        := noop,

    /// The head of a complex [`Term`], or one where the head used for presentation purposes
    /// differs from the actual head, e.g. a a record field projection, where the applicant
    /// for presentation purposes is the [`SymbolUri`] of the record type's field itself.
    ///
    /// This node (or its only child) is interpreted to be a [`Term`].
    HeadTerm = "headterm"
    { !"in [`Term`]s of kind `OMA`, `OMBIND` or `complex`"}
        := (ext,_attrs,_keys,node) => {
            ret!(ext,node <- HeadTerm + HeadTerm)
        } => HeadTerm,

    // --------------------------------------------------------------------------

    /// Declares a new [`Notation`] for the given symbol or variable with the (optional)
    /// given name, operator precedence, and argument precedences.
    Notation = "notation"
        { = "[SymbolUri]|[DocumentElementUri]|[UriName]"
            +(NotationFragment,Precedence,Argprecs)
            &(NotationComp,NotationOpComp)
        }
        := (ext,attrs,keys,node) => {
            let head = attrs.get_symbol_or_var(FtmlKey::Notation, ext)?;

            let mut fragment = attrs
                .get(FtmlKey::NotationFragment)
                .map(Into::<String>::into);
            if fragment.as_ref().is_some_and(String::is_empty) {
                fragment = None;
            }
            let id = if let Some(id) = fragment {
                Some(
                    id.parse()
                        .map_err(|_| FtmlExtractionError::InvalidValue(FtmlKey::NotationFragment))?,
                )
            } else {
                None
            };
            let uri = if let Some(id) = &id {
                ext.get_narrative_uri() & id
            } else {
                let name = ext.new_id(FtmlKey::NotationFragment, Cow::Borrowed("notation"))?;
                ext.get_narrative_uri() & &name
            };

            let prec = if let Some(v) = attrs.get(FtmlKey::Precedence) {
                if let Ok(v) = i64::from_str(v.as_ref()) {
                    v
                } else {
                    return Err(FtmlExtractionError::InvalidValue(FtmlKey::Precedence));
                }
            } else {
                0
            };

            let mut argprecs = Vec::new();
            if let Some(s) = attrs.get(FtmlKey::Argprecs) {
                for s in s.as_ref().split(',') {
                    if s.is_empty() {
                        continue;
                    }
                    if let Ok(v) = i64::from_str(s.trim()) {
                        argprecs.push(v);
                    } else {
                        return Err(FtmlExtractionError::InvalidValue(FtmlKey::Argprecs));
                    }
                }
            }

            del!(keys - NotationFragment, Precedence, Argprecs);
            ret!(ext,node <- Notation{id,uri,head,prec,argprecs} + Notation)
        } => Notation{
            id:Option<Id>,
            uri:DocumentElementUri,
            head:VarOrSym,
            prec:i64,
            argprecs:Vec<i64>
        },

    /// The actual notation.
    /// This node is interpreted to be a notation.
    NotationComp = "notationcomp"
        { <=(Notation) }
        := (ext,attrs,keys,node) => {
            if !ext.in_notation() {
                return Err(FtmlExtractionError::InvalidIn(
                    FtmlKey::NotationComp,
                    "ouside of a notation",
                ));
            }
            del!(keys - NotationComp, Term, Head, NotationId, Invisible);
            attrs.remove(FtmlKey::NotationComp);
            attrs.remove(FtmlKey::Term);
            attrs.remove(FtmlKey::Head);
            attrs.remove(FtmlKey::NotationId);
            attrs.remove(FtmlKey::Invisible);
            ret!(ext,node <- NotationComp + NotationComp)
        } => NotationComp,

    /// This node is interpreted to be the operator notation.
    NotationOpComp = "notationopcomp"
        { <=(Notation) }
        := (ext,attrs,keys,node) => {
            del!(keys - NotationOpComp, Term, Head, NotationId, Invisible);
            attrs.remove(FtmlKey::NotationOpComp);
            attrs.remove(FtmlKey::Term);
            attrs.remove(FtmlKey::Head);
            attrs.remove(FtmlKey::NotationId);
            attrs.remove(FtmlKey::Invisible);
            ret!(ext,node <- None + NotationOpComp)
        },

    /// This node serves as a separator between the individual components of an
    /// *argument sequence* (`ArgumentMode::Sequence`
    /// or `ArgumentMode::BoundSequence`)
    ArgSep = "argsep"
        { <=(NotationComp) }
        := (ext,attrs,keys,node) => {
            del!(keys - ArgSep, Term, Head, NotationId, Invisible);
            attrs.remove(FtmlKey::Term);
            attrs.remove(FtmlKey::ArgSep);
            attrs.remove(FtmlKey::Head);
            attrs.remove(FtmlKey::NotationId);
            attrs.remove(FtmlKey::Invisible);
            ret!(ext,node <- ArgSep + ArgSep)
        } => ArgSep,

    /// Argument marker in a notation
    ArgNum = "argnum"
        {=""  <=(NotationComp) }
        := (ext,attrs,_keys,node) => {
            let index = attrs.get_typed(FtmlKey::ArgNum, |s| {
                let u = u8::from_str(s).map_err(|_| FtmlExtractionError::InvalidValue(FtmlKey::ArgNum))?;
                NonZeroU8::new(u).ok_or(FtmlExtractionError::InvalidValue(FtmlKey::ArgNum))
            })?;
            let argument = ArgumentPosition::Simple(index, ArgumentMode::Simple);
            let fits = if let Some(OpenNarrativeElement::NotationArg(pos)) = ext.iterate_narrative().next()
                && pos.index() == argument.index()
            {
                true
            } else {
                false
            };
            if fits {
                ret!(ext, node)
            } else if ext.in_notation() {
                ret!(ext,node <- NotationArg(argument) + NotationArg)
            } else {
                Err(FtmlExtractionError::NotIn(FtmlKey::ArgNum, "notations"))
            }
        } => NotationArg(arg:ArgumentPosition),

    ArgMap = "argmap"
        { <=(NotationComp) }
        := todo,

    ArgMapSep = "argmap-sep"
        { <=(ArgComp) }
        := todo,

    /// The [`Id`] of the notation.
    NotationFragment = "notationfragment"
        {="[Id]" -(Notation)}
        := noop,

    /// The operator (upwards) precedence of a notation.
    Precedence = "precedence"
        {="[i64]" -(Notation)}
        := noop,

    /// The (downwards) precedences of the individual arguments of a notation
    Argprecs = "argprecs"
        {="[i64]*" -(Notation)}
        := noop,

    // --------------------------------------------------------------------------

    /// A notation component in a [`Term`] (to be e.g. highlighted)
    Comp = "comp"
        { <=(Term,NotationComp,NotationOpComp)}
        := (ext,attrs,keys,node) => {
            if ext.in_notation() {
                del!(keys - Comp, VarComp, Term, Head, NotationId, Invisible);
                attrs.remove(FtmlKey::Comp);
                attrs.remove(FtmlKey::Term);
                attrs.remove(FtmlKey::Head);
                attrs.remove(FtmlKey::NotationId);
                attrs.remove(FtmlKey::Invisible);
                return ret!(ext,node <- None + CompInNotation);
            }
            do_comp(ext, node)
        } => Comp,

    /// A notation component in a [`Term`] whose head is a variable (to be e.g. highlighted)
    VarComp = "varcomp"
        { <=(Term,NotationComp,OpNotationComp)}
        := (ext,attrs,keys,node) => {
            if ext.in_notation() {
                del!(keys - Comp, VarComp, Term, Head, NotationId, Invisible);
                attrs.remove(FtmlKey::Comp);
                attrs.remove(FtmlKey::Term);
                attrs.remove(FtmlKey::Head);
                attrs.remove(FtmlKey::NotationId);
                attrs.remove(FtmlKey::Invisible);
                return ret!(ext,node <- None + CompInNotation);
            }
            do_comp(ext, node)
        },

    /// A primary notation component in a [`Term`] (to be e.g. highlighted); also serves
    /// e.g. as the notation of the operant/binder itself.
    MainComp = "maincomp"
        { <=(Term,NotationComp,OpNotationComp)}
        := (ext,attrs,keys,node) => {
            if ext.in_notation() {
                del!(keys - MainComp, Term, Head, NotationId, Invisible);
                attrs.remove(FtmlKey::MainComp);
                attrs.remove(FtmlKey::Term);
                attrs.remove(FtmlKey::Head);
                attrs.remove(FtmlKey::NotationId);
                attrs.remove(FtmlKey::Invisible);
                return ret!(ext,node <- None + MainCompInNotation);
            }
            do_comp(ext, node)
        },

    /// A notation component in a [`Term`] whose head is being
    /// *defined* here (to be e.g. highlighted *as a definiendum*)
    DefComp = "defcomp"
        { <=(Term,NotationComp,OpNotationComp) !"in definition-like [`LogicalParagraph`]s"}
        := (ext,_attrs,_keys,node) => {
            match ext.iterate_domain().next() {
                Some(
                    OpenDomainElement::SymbolReference { .. }
                    | OpenDomainElement::OMA { .. }
                    | OpenDomainElement::OMBIND { .. }
                    | OpenDomainElement::OML { .. }
                    | OpenDomainElement::ComplexTerm { .. }
                    | OpenDomainElement::VariableReference { .. },
                ) => (),
                None
                | Some(
                    OpenDomainElement::Module { .. }
                    | OpenDomainElement::MathStructure { .. }
                    | OpenDomainElement::SymbolDeclaration { .. }
                    | OpenDomainElement::Argument { .. }
                    | OpenDomainElement::HeadTerm { .. }
                    | OpenDomainElement::Type { .. }
                    | OpenDomainElement::ReturnType { .. }
                    | OpenDomainElement::Definiens { .. }
                    | OpenDomainElement::Comp
                    | OpenDomainElement::DefComp,
                ) => {
                    return Err(FtmlExtractionError::NotIn(FtmlKey::DefComp, "a term"));
                }
            }
            ret!(ext,node <- DefComp + DefComp)
        } => DefComp,

    /// The *definiendum* in a Definition, i.e. a [`Symbol`] being defined here.
    Definiendum = "definiendum"
        {="[SymbolUri]" <=(Definition, Paragraph, Assertion) !"in definition-like [`LogicalParagraph`]s"}
        := (ext,attrs,_keys,node) => {
            let s = attrs.get_symbol_uri(FtmlKey::Definiendum)?;
            ret!(ext,node <- Definiendum(s) + Definiendum)
        } => Definiendum(s:SymbolUri),

    // --------------------------------------------------------------------------

    /// <div class="ftml-wip">TODO</div>
    Rule = "rule"
        := todo,

    SRef = "sref"
        := todo,
    SRefIn = "srefin"
        := todo,
    Slideshow = "slideshow"
        := todo,
    SlideshowSlide = "slideshow-slide"
        := todo,

    Assign = "assign"
        := todo,
    Rename = "rename"
        := todo,
    RenameTo = "to"
        := todo,
    AssignMorphismFrom = "assignmorphismfrom"
        := todo,
    AssignMorphismTo = "assignmorphismto"
        := todo,

    MorphismDomain = "domain"
        := todo,
    MorphismTotal = "total"
        := todo,

    Language = "language"
        := noop,

    /// An optional [`Id`] used for generating new [`Uri`]s.
    Id = "id"
        {="[Id]" -(Section,Definition, Paragraph, Assertion, Example, Proof, SubProof, Problem, SubProblem, Slide,InputRef)}
        := noop,

    /// This node is only used for providing declarations and does not produce any
    /// "output". Can and will be stripped from the HTML after processing.
    Invisible = "invisible"
        := (ext,attrs,_keys,node) => {
            if attrs.take_bool(FtmlKey::Invisible) {
                ret!(ext,node <- Invisible + Invisible)
            } else {
                ret!(ext, node)
            }
        } => Invisible
}

fn do_vardef<E: crate::extraction::FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
    key: FtmlKey,
    is_sequence: bool,
) -> Result<
    (E::Return, Option<crate::extraction::CloseFtmlElement>),
    crate::extraction::FtmlExtractionError,
> {
    let name: Id = attrs.get_typed(key, |v| v.parse().map_err(|_| ()))?;
    let uri = ext.get_narrative_uri() & &name;

    let role = opt!(attrs.get_typed(FtmlKey::Role, |s| {
        Ok::<_, SegmentParseError>(
            s.split(',')
                .map(|s| s.trim().parse::<Id>())
                .collect::<std::result::Result<Vec<_>, SegmentParseError>>()?
                .into_boxed_slice(),
        )
    }))
    .unwrap_or_default();
    let assoctype = opt!(attrs.get_typed(FtmlKey::AssocType, |s| {
        AssocType::from_str(s).map_err(|_| ())
    }));
    let arity = opt!(attrs.get_typed(FtmlKey::Args, |s| {
        ArgumentSpec::from_str(s).map_err(|_| ())
    }))
    .unwrap_or_default();
    let reordering = attrs
        .get(FtmlKey::ArgumentReordering)
        .map(|s| s.as_ref().parse())
        .transpose()
        .map_err(|_| (FtmlKey::ArgumentReordering, ()))?;
    let macroname = attrs
        .get(FtmlKey::Macroname)
        .map(|s| s.as_ref().parse())
        .transpose()
        .map_err(|_| (FtmlKey::Macroname, ()))?;
    let bind = attrs.get_bool(FtmlKey::Bind);

    del!(
        keys - Role,
        AssocType,
        Args,
        ArgumentReordering,
        Macroname,
        Bind
    );
    ret!(ext,node <- VariableDeclaration {
        uri,
        data: Box::new(VariableData {
            arity,
            macroname,
            role,
            assoctype,
            reordering,
            bind,
            is_seq:is_sequence,
            tp: None,
            df: None,
        }),
    } + VariableDeclaration)
}

fn do_comp<E: FtmlExtractor>(
    ext: &mut E,
    node: &E::Node,
) -> Result<
    (E::Return, Option<crate::extraction::CloseFtmlElement>),
    crate::extraction::FtmlExtractionError,
> {
    match ext.iterate_domain().next() {
        Some(
            OpenDomainElement::SymbolReference { .. }
            | OpenDomainElement::OMA { .. }
            | OpenDomainElement::OMBIND { .. }
            | OpenDomainElement::ComplexTerm { .. }
            | OpenDomainElement::OML { .. }
            | OpenDomainElement::VariableReference { .. },
        ) => (),
        None
        | Some(
            OpenDomainElement::Module { .. }
            | OpenDomainElement::MathStructure { .. }
            | OpenDomainElement::SymbolDeclaration { .. }
            | OpenDomainElement::Argument { .. }
            | OpenDomainElement::HeadTerm { .. }
            | OpenDomainElement::Type { .. }
            | OpenDomainElement::ReturnType { .. }
            | OpenDomainElement::Definiens { .. }
            | OpenDomainElement::Comp
            | OpenDomainElement::DefComp,
        ) => {
            return Err(FtmlExtractionError::NotIn(FtmlKey::Comp, "a term"));
        }
    }
    ret!(ext,node <- Comp + Comp)
}

fn do_paragraph<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
    kind: ParagraphKind,
) -> Result<
    (E::Return, Option<crate::extraction::CloseFtmlElement>),
    crate::extraction::FtmlExtractionError,
> {
    let uri = attrs.get_elem_uri_from_id(ext, Cow::Borrowed(kind.as_str()))?;
    let inline = attrs.get_bool(FtmlKey::Inline);
    let mut fors: Vec<(SymbolUri, Option<Term>)> = Vec::new();
    if let Some(f) = attrs.get(FtmlKey::Fors) {
        for f in f.as_ref().split(',') {
            let uri = f
                .trim()
                .parse()
                .map_err(|_| FtmlExtractionError::InvalidValue(FtmlKey::Fors))?;
            if !fors.iter().any(|(u, _)| *u == uri) {
                fors.push((uri, None));
            }
        }
    }
    let styles = opt!(
        attrs.get_typed_vec::<FtmlExtractionError, _>(FtmlKey::Styles, |s| {
            s.trim()
                .parse()
                .map_err(|_| FtmlExtractionError::InvalidValue(FtmlKey::Fors))
        })
    )
    .unwrap_or_default();

    let formatting = if inline {
        ParagraphFormatting::Inline
    } else if matches!(kind, ParagraphKind::Proof | ParagraphKind::SubProof) {
        let hide = attrs.get_bool(FtmlKey::ProofHide);
        if hide {
            ParagraphFormatting::Collapsed
        } else {
            ParagraphFormatting::Block
        }
    } else {
        ParagraphFormatting::Block
    };

    del!(keys - Id, Inline, Fors, Styles, ProofHide);
    ret!(ext,node <- Paragraph{
        kind,
        formatting,
        styles:styles.into_boxed_slice(),
        uri,
        fors
    } + Paragraph)
}

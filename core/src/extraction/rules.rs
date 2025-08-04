#![allow(clippy::needless_pass_by_ref_mut)]

use crate::{
    FtmlKey,
    extraction::{
        ArgumentPosition, CloseFtmlElement, FtmlExtractionError, FtmlExtractor, KeyList,
        OpenDomainElement, OpenFtmlElement, OpenNarrativeElement, VarOrSym, attributes::Attributes,
    },
};
use ftml_ontology::{
    domain::declarations::symbols::{ArgumentSpec, AssocType, SymbolData},
    narrative::{
        documents::{DocumentCounter, DocumentStyle},
        elements::{
            paragraphs::{ParagraphFormatting, ParagraphKind},
            sections::SectionLevel,
            variables::VariableData,
        },
    },
    terms::{ArgumentMode, Term, Variable},
};
use ftml_uris::{Id, IsNarrativeUri, SymbolUri, errors::SegmentParseError};
use std::{borrow::Cow, num::NonZeroU8, str::FromStr};

type Result<E> = super::Result<(<E as FtmlExtractor>::Return, Option<CloseFtmlElement>)>;

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
        Ok(($ext.add_element(OpenFtmlElement::$id( $($a),* ),$node)?,$r))
    };
    (@I $ext:ident,$node:ident <- $id:ident + $r:expr) => {
        Ok(($ext.add_element(OpenFtmlElement::$id,$node)?,$r))
    };
    ($ext:ident,$node:ident <- $id:ident{$($b:tt)*} + $r:ident) => {
        ret!(@I $ext,$node <- $id{$($b)*} + Some(CloseFtmlElement::$r))
    };
    ($ext:ident,$node:ident <- $id:ident( $($a:expr),* ) + $r:ident) => {
        ret!(@I $ext,$node <- $id( $($a),*) + Some(CloseFtmlElement::$r))
    };
    ($ext:ident,$node:ident <- $id:ident + $r:ident) => {
        ret!(@I $ext,$node <- $id + Some(CloseFtmlElement::$r))
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

#[allow(clippy::unnecessary_wraps)]
pub fn todo<E: FtmlExtractor>(
    key: FtmlKey,
    ext: &mut E,
    _: &mut E::Attributes<'_>,
    _: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    tracing::warn!("Not yet implemented: {key}");
    ret!(ext, node)
}

#[allow(clippy::unnecessary_wraps)]
pub fn no_op<E: FtmlExtractor>(
    ext: &mut E,
    _: &mut E::Attributes<'_>,
    _: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    ret!(ext, node)
}

pub fn invisible<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    _keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    if attrs.take_bool(FtmlKey::Invisible) {
        ret!(ext,node <- Invisible + Invisible)
    } else {
        ret!(ext, node)
    }
}

pub fn doctitle<E: FtmlExtractor>(
    ext: &mut E,
    _attrs: &mut E::Attributes<'_>,
    _keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    ret!(ext, node <- None + DocTitle)
}

pub fn setsectionlevel<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    _keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    let lvl = attrs.get_typed(FtmlKey::SetSectionLevel, |s| {
        u8::from_str(s).map_err(|_| ())
    })?;
    let lvl: SectionLevel = lvl
        .try_into()
        .map_err(|_| FtmlExtractionError::InvalidValue(FtmlKey::SetSectionLevel))?;
    ret!(ext,node <- SetSectionLevel(lvl))
}

pub fn section<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    let uri = attrs.get_elem_uri_from_id(ext, "section")?;
    del!(keys - Id);
    ret!(ext,node <- Section(uri) + Section)
}

pub fn currentsectionlevel<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    let cap = attrs.get_bool(FtmlKey::Capitalize);
    del!(keys - Capitalize);
    ret!(ext,node <- CurrentSectionLevel(cap))
}

pub fn skipsection<E: FtmlExtractor>(
    ext: &mut E,
    _attrs: &mut E::Attributes<'_>,
    _keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    ret!(ext,node <- SkipSection + SkipSection)
}

pub fn title<E: FtmlExtractor>(
    ext: &mut E,
    _attrs: &mut E::Attributes<'_>,
    _keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
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
            | OpenNarrativeElement::NotationArg(_) => {
                break;
            }
            OpenNarrativeElement::Module { .. } | OpenNarrativeElement::Invisible => (),
        }
    }
    Err(FtmlExtractionError::NotIn(
        FtmlKey::Title,
        "a section or paragraph",
    ))
}

pub fn inputref<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    _keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    let target = attrs.get_document_uri(FtmlKey::InputRef)?;
    let uri = attrs.get_elem_uri_from_id(ext, Cow::Owned(target.document_name().to_string()))?;
    ret!(ext,node <- InputRef{uri,target})
}

pub fn ifinputref<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    crate::TODO!()
}

pub fn symdecl<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
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
        .map_err(|_| (FtmlKey::ArgumentReordering, ()))?;
    del!(keys - Role, AssocType, Args, ArgumentReordering, Macroname);
    ret!(ext,node <- SymbolDeclaration {
        uri,
        data: Box::new(SymbolData {
            arity,
            macroname,
            role,
            assoctype,
            reordering,
            tp: None,
            df: None,
        }),
    } + SymbolDeclaration)
}

pub fn vardef<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    do_vardef(ext, attrs, keys, node, FtmlKey::Vardef, false)
}

pub fn varseq<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    do_vardef(ext, attrs, keys, node, FtmlKey::Varseq, true)
}

fn do_vardef<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
    key: FtmlKey,
    is_sequence: bool,
) -> Result<E> {
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
        .map_err(|_| (FtmlKey::ArgumentReordering, ()))?;
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

pub fn type_component<E: FtmlExtractor>(
    ext: &mut E,
    _attrs: &mut E::Attributes<'_>,
    _keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    if ext.in_term() {
        return Err(FtmlExtractionError::InvalidIn(FtmlKey::Type, "terms"));
    }
    ret!(ext,node <- Type + Type)
}

pub fn definiens<E: FtmlExtractor>(
    ext: &mut E,
    _attrs: &mut E::Attributes<'_>,
    _keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    if ext.in_term() {
        return Err(FtmlExtractionError::InvalidIn(FtmlKey::Definiens, "terms"));
    }
    ret!(ext,node <- Definiens + Definiens)
}

pub fn style<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    let mut style = attrs.get_typed(FtmlKey::Style, |s| {
        DocumentStyle::from_str(s).map_err(|_| ())
    })?;
    if let Some(count) = opt!(attrs.get_typed(FtmlKey::Counter, Id::from_str)) {
        style.counter = Some(count);
    }
    del!(keys - Counter);
    ret!(ext,node <- Style(style))
}

pub fn counter_parent<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
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
    del!(keys - Counter);
    ret!(ext,node <- Counter(DocumentCounter { name, parent }))
}

pub fn module<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
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
}

pub fn usemodule<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    _keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    let uri = attrs.take_module_uri(FtmlKey::UseModule)?;
    ret!(ext,node <- UseModule(uri))
}

pub fn importmodule<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    _keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    let uri = attrs.take_module_uri(FtmlKey::UseModule)?;
    ret!(ext,node <- ImportModule(uri))
}

pub fn term<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
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
                    | OpenDomainElement::SymbolDeclaration { .. }
                    | OpenDomainElement::SymbolReference { .. }
                    | OpenDomainElement::VariableReference { .. }
                    | OpenDomainElement::OMA { .. }
                    | OpenDomainElement::OMBIND { .. }
                    | OpenDomainElement::Type { .. }
                    | OpenDomainElement::Definiens { .. },
                ) => false,
                Some(OpenDomainElement::Argument { .. }) => true,
                Some(OpenDomainElement::Comp) => {
                    return Err(FtmlExtractionError::InvalidIn(
                        FtmlKey::Term,
                        "notation components",
                    ));
                }
            })
    };

    match (kind, head) {
        (OpenTermKind::OMS | OpenTermKind::OMV, VarOrSym::S(uri)) => {
            ret!(ext,node <- SymbolReference{uri,notation} + SymbolReference)
        }
        (OpenTermKind::OMS | OpenTermKind::OMV, VarOrSym::V(var)) => {
            if let Variable::Ref { declaration, .. } = &var {
                attrs.set(FtmlKey::Head.attr_name(), declaration);
            }
            ret!(ext,node <- VariableReference{var,notation} + VariableReference)
        }
        (OpenTermKind::OMA, head) => {
            let uri = if in_term(ext)? {
                None
            } else {
                Some(attrs.get_elem_uri_from_id(ext, Cow::Borrowed("term"))?)
            };
            if let VarOrSym::V(Variable::Ref { declaration, .. }) = &head {
                attrs.set(FtmlKey::Head.attr_name(), declaration);
            }
            ret!(ext,node <- OMA{head,notation,uri} + OMA)
        }
        (OpenTermKind::OMBIND, head) => {
            let uri = if in_term(ext)? {
                None
            } else {
                Some(attrs.get_elem_uri_from_id(ext, Cow::Borrowed("term"))?)
            };
            if let VarOrSym::V(Variable::Ref { declaration, .. }) = &head {
                attrs.set(FtmlKey::Head.attr_name(), declaration);
            }
            ret!(ext,node <- OMBIND{head,notation,uri} + OMBIND)
        }
        (k, _) => crate::TODO!("{k:?}"),
    }

    /*
    let term = match (kind, head) {
        (OpenTermKind::OML, VarOrSym::V(PreVar::Unresolved(name))) => {
            extractor.open_decl();
            OpenTerm::OML { name } //, tp: None, df: None }
        }
        (OpenTermKind::Complex, head) => {
            extractor.open_complex_term();
            OpenTerm::Complex(head)
        }
        (k, head) => {
            extractor.add_error(FTMLError::InvalidHeadForTermKind(k, head.clone()));
            extractor.open_args();
            OpenTerm::OMA { head, notation } //, args: SmallVec::new() }
        }
    };
    let is_top = if extractor.in_term() {
        false
    } else {
        extractor.set_in_term(true);
        true
    };
    Some(OpenFTMLElement::OpenTerm { term, is_top })
    */
}

pub fn arg<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
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
}

pub fn headterm<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

fn do_comp<E: FtmlExtractor>(ext: &mut E, node: &E::Node) -> Result<E> {
    match ext.iterate_domain().next() {
        Some(
            OpenDomainElement::SymbolReference { .. }
            | OpenDomainElement::OMA { .. }
            | OpenDomainElement::OMBIND { .. }
            | OpenDomainElement::VariableReference { .. },
        ) => (),
        None
        | Some(
            OpenDomainElement::Module { .. }
            | OpenDomainElement::SymbolDeclaration { .. }
            | OpenDomainElement::Argument { .. }
            | OpenDomainElement::Type { .. }
            | OpenDomainElement::Definiens { .. }
            | OpenDomainElement::Comp,
        ) => {
            return Err(FtmlExtractionError::NotIn(FtmlKey::Comp, "a term"));
        }
    }
    ret!(ext,node <- Comp + Comp)
}

pub fn comp<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    if ext.in_notation() {
        del!(keys - Comp, Term, Head, NotationId, Invisible);
        attrs.remove(FtmlKey::Comp);
        attrs.remove(FtmlKey::Term);
        attrs.remove(FtmlKey::Head);
        attrs.remove(FtmlKey::NotationId);
        attrs.remove(FtmlKey::Invisible);
        return ret!(ext,node <- None + CompInNotation);
    }
    do_comp(ext, node)
}

pub fn maincomp<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
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
}

pub fn notation<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
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
        if let Ok(v) = isize::from_str(v.as_ref()) {
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
            if let Ok(v) = isize::from_str(s.trim()) {
                argprecs.push(v);
            } else {
                return Err(FtmlExtractionError::InvalidValue(FtmlKey::Argprecs));
            }
        }
    }

    del!(keys - NotationFragment, Precedence, Argprecs);
    ret!(ext,node <- Notation{id,uri,head,prec,argprecs} + Notation)

    /*
    extractor.open_notation();
    Some(OpenFTMLElement::Notation {
        id,
        symbol,
        precedence: prec,
        argprecs,
    })
     */
}

pub fn notation_comp<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
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
}

pub fn notation_op_comp<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    del!(keys - NotationOpComp, Term, Head, NotationId, Invisible);
    attrs.remove(FtmlKey::NotationOpComp);
    attrs.remove(FtmlKey::Term);
    attrs.remove(FtmlKey::Head);
    attrs.remove(FtmlKey::NotationId);
    attrs.remove(FtmlKey::Invisible);
    ret!(ext,node <- None + NotationOpComp)
}

pub fn argsep<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    del!(keys - ArgSep, Term, Head, NotationId, Invisible);
    attrs.remove(FtmlKey::Term);
    attrs.remove(FtmlKey::ArgSep);
    attrs.remove(FtmlKey::Head);
    attrs.remove(FtmlKey::NotationId);
    attrs.remove(FtmlKey::Invisible);
    ret!(ext,node <- ArgSep + ArgSep)
}

pub fn argnum<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    _keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
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
}

pub fn argmap<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    crate::TODO!()
}

pub fn argmapsep<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    crate::TODO!()
}

fn do_paragraph<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
    kind: ParagraphKind,
) -> Result<E> {
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

pub fn definition<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    do_paragraph(ext, attrs, keys, node, ParagraphKind::Definition)
}

pub fn paragraph<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    do_paragraph(ext, attrs, keys, node, ParagraphKind::Paragraph)
}

pub fn assertion<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    do_paragraph(ext, attrs, keys, node, ParagraphKind::Assertion)
}

pub fn example<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    do_paragraph(ext, attrs, keys, node, ParagraphKind::Example)
}

pub fn proof<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    do_paragraph(ext, attrs, keys, node, ParagraphKind::Proof)
}
pub fn subproof<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    do_paragraph(ext, attrs, keys, node, ParagraphKind::SubProof)
}

/*

pub fn defcomp<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    crate::TODO!()
}

pub fn mathstructure<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    crate::TODO!()
}

pub fn morphism<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    crate::TODO!()
}

pub fn assign<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    crate::TODO!()
}

pub fn slide<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    crate::TODO!()
}

pub fn slide_number<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    crate::TODO!()
}


pub fn proofbody<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn problem<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn subproblem<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn problem_hint<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn solution<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn gnote<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn answer_class<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn answer_class_feedback<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn multiple_choice_block<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn single_choice_block<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn problem_choice<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn problem_choice_verdict<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn problem_choice_feedback<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn fillinsol<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn fillinsol_case<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn precondition<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn objective<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn prooftitle<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn subprooftitle<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn definiendum<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn conclusion<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

 */

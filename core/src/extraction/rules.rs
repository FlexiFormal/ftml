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
        elements::sections::SectionLevel,
    },
    terms::{ArgumentMode, Variable},
};
use ftml_uris::{Id, IsNarrativeUri, Uri, errors::SegmentParseError};
use std::{borrow::Cow, str::FromStr};

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
    _keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    let uri = attrs.get_elem_uri_from_id(ext, "section")?;
    ret!(ext,node <- Section(uri) + Section)
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
            OpenNarrativeElement::SkipSection { .. } => break,
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
                .map_err(|_| FtmlExtractionError::InvalidValue(FtmlKey::SetSectionLevel))
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

    if ext.in_notation() {
        return ret!(ext, node);
    }

    let Some(headv) = attrs.get(FtmlKey::Head) else {
        return Err(FtmlExtractionError::MissingKey(FtmlKey::Head));
    };
    let head = headv.as_ref().trim();
    let head = if head.contains('?') {
        let uri = head.parse::<Uri>().map_err(|e| (FtmlKey::Term, e))?;
        match uri {
            Uri::Symbol(s) => VarOrSym::S(s),
            Uri::Module(m) => {
                let Some(s) = m.into_symbol() else {
                    return Err(FtmlExtractionError::InvalidValue(FtmlKey::Head));
                };
                VarOrSym::S(s)
            }
            //Uri::Module(_) => VarOrSym::S(m.into()) ???
            Uri::DocumentElement(e) => VarOrSym::V(Variable::Ref {
                declaration: e,
                is_sequence: None,
            }),
            _ => return Err(FtmlExtractionError::InvalidValue(FtmlKey::Head)),
        }
    } else {
        VarOrSym::V(ext.resolve_variable_name(head.parse().map_err(|e| (FtmlKey::Term, e))?))
    };
    drop(headv);

    let kind: OpenTermKind = attrs.get_typed(FtmlKey::Term, str::parse)?;
    let notation = opt!(attrs.get_typed(FtmlKey::NotationId, str::parse));
    del!(keys - NotationId, Head, Term);

    match (kind, head) {
        (OpenTermKind::OMS | OpenTermKind::OMV, VarOrSym::S(uri)) => {
            ret!(ext,node <- SymbolReference{uri,notation} + SymbolReference)
        }
        (OpenTermKind::OMS | OpenTermKind::OMV, VarOrSym::V(var)) => {
            ret!(ext,node <- VariableReference{var,notation} + VariableReference)
        }
        (OpenTermKind::OMA, head) => {
            let uri = if ext.in_term() {
                None
            } else {
                Some(attrs.get_elem_uri_from_id(ext, Cow::Borrowed("term"))?)
            };
            ret!(ext,node <- OMA{head,notation,uri} + OMA)
        }
        (OpenTermKind::OMBIND, head) => {
            let uri = if ext.in_term() {
                None
            } else {
                Some(attrs.get_elem_uri_from_id(ext, Cow::Borrowed("term"))?)
            };
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
    } else {
        /* tracing::debug!("Error: Argument: {argument:?}: {:?}", {
            ext.iterate_domain().collect::<Vec<_>>()
        });*/

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

pub fn comp<E: FtmlExtractor>(
    ext: &mut E,
    _attrs: &mut E::Attributes<'_>,
    _keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
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
            | OpenDomainElement::Definiens { .. },
        ) => {
            return Err(FtmlExtractionError::NotIn(FtmlKey::Comp, "a term"));
        }
    }
    ret!(ext,node <- Comp)
}

pub fn maincomp<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    crate::TODO!()
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

pub fn importmodule<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
    node: &E::Node,
) -> Result<E> {
    crate::TODO!()
}

pub fn usemodule<E: FtmlExtractor>(
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

pub fn definition<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn paragraph<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn assertion<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn example<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn proof<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn subproof<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
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

pub fn doctitle<E: FtmlExtractor>(
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

pub fn vardecl<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn varseq<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn notation<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn notationcomp<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn notationopcomp<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn argsep<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn argmap<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    crate::TODO!()
}

pub fn argmapsep<E: FtmlExtractor>(
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

#![allow(clippy::needless_pass_by_ref_mut)]

use crate::{
    FtmlKey,
    extraction::{
        CloseFtmlElement, FtmlExtractionError, FtmlExtractor, KeyList, OpenFtmlElement,
        OpenNarrativeElement, attributes::Attributes,
    },
};
use ftml_ontology::{
    domain::declarations::symbols::{ArgumentSpec, AssocType, SymbolData},
    narrative::{
        documents::{DocumentCounter, DocumentStyle},
        elements::sections::SectionLevel,
    },
};
use ftml_uris::{Id, errors::SegmentParseError};
use std::str::FromStr;

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
    ($ext:ident) => {Ok(($ext.add_element(OpenFtmlElement::None)?,None))};
    (@I $ext:ident <- $id:ident{$($b:tt)*} = $r:expr) => {
        Ok(($ext.add_element(OpenFtmlElement::$id{$($b)*})?,$r))
    };
    (@I $ext:ident <- $id:ident($($a:expr),*) = $r:expr) => {
        Ok(($ext.add_element(OpenFtmlElement::$id( $($a),* ))?,$r))
    };
    (@I $ext:ident <- $id:ident = $r:expr) => {
        Ok(($ext.add_element(OpenFtmlElement::$id)?,$r))
    };
    ($ext:ident <- $id:ident{$($b:tt)*} = $r:ident) => {
        ret!(@I $ext <- $id{$($b)*} = Some(CloseFtmlElement::$r))
    };
    ($ext:ident <- $id:ident( $($a:expr),* ) = $r:ident) => {
        ret!(@I $ext <- $id( $($a),*) = Some(CloseFtmlElement::$r))
    };
    ($ext:ident <- $id:ident = $r:ident) => {
        ret!(@I $ext <- $id = Some(CloseFtmlElement::$r))
    };
    ($ext:ident <- $id:ident{$($b:tt)*}) => {
        ret!(@I $ext <- $id{$($b)*} = None)
    };
    ($ext:ident <- $id:ident( $($a:expr),* )) => {
        ret!(@I $ext <- $id( $($a),*) = None)
    };
    ($ext:ident <- $id:ident) => {
        ret!(@I $ext <- $id = None)
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
) -> Result<E> {
    tracing::warn!("Not yet implemented: {key}");
    ret!(ext)
}

#[allow(clippy::unnecessary_wraps)]
pub fn no_op<E: FtmlExtractor>(
    ext: &mut E,
    _: &mut E::Attributes<'_>,
    _: &mut KeyList,
) -> Result<E> {
    ret!(ext)
}

pub fn invisible<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    _keys: &mut KeyList,
) -> Result<E> {
    if attrs.take_bool(FtmlKey::Invisible) {
        ret!(ext <- Invisible = Invisible)
    } else {
        ret!(ext)
    }
}

pub fn setsectionlevel<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    _keys: &mut KeyList,
) -> Result<E> {
    let lvl = attrs.get_typed(FtmlKey::SetSectionLevel, |s| {
        u8::from_str(s).map_err(|_| FtmlExtractionError::InvalidValue(FtmlKey::SetSectionLevel))
    })?;
    let lvl: SectionLevel = lvl
        .try_into()
        .map_err(|_| FtmlExtractionError::InvalidValue(FtmlKey::SetSectionLevel))?;
    ret!(ext <- SetSectionLevel(lvl))
}

pub fn symbol<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    let uri = attrs.get_new_symbol_uri(FtmlKey::Symdecl, FtmlKey::Symdecl, ext)?;
    let role = opt!(attrs.get_typed(FtmlKey::Role, |s| {
        Ok::<_, FtmlExtractionError>(
            s.split(',')
                .map(|s| s.trim().parse::<Id>())
                .collect::<std::result::Result<Vec<_>, SegmentParseError>>()?
                .into_boxed_slice(),
        )
    }))
    .unwrap_or_default();
    let assoctype = opt!(attrs.get_typed(FtmlKey::AssocType, |s| {
        AssocType::from_str(s).map_err(|_| FtmlExtractionError::InvalidValue(FtmlKey::AssocType))
    }));
    let arity = opt!(attrs.get_typed(FtmlKey::Args, |s| {
        ArgumentSpec::from_str(s).map_err(|_| FtmlExtractionError::InvalidValue(FtmlKey::Args))
    }))
    .unwrap_or_default();
    let reordering = attrs
        .get(FtmlKey::ArgumentReordering)
        .map(|s| s.as_ref().parse())
        .transpose()?;
    let macroname = attrs
        .get(FtmlKey::Macroname)
        .map(|s| s.as_ref().parse())
        .transpose()?;
    del!(keys - Role, AssocType, Args, ArgumentReordering, Macroname);
    ret!(ext <- Symbol {
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
    } = Symbol)
}

pub fn style<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    let mut style = attrs.get_typed(FtmlKey::Style, |s| {
        DocumentStyle::from_str(s).map_err(|_| FtmlExtractionError::InvalidValue(FtmlKey::Style))
    })?;
    if let Some(count) = opt!(attrs.get_typed(FtmlKey::Counter, Id::from_str)) {
        style.counter = Some(count);
    }
    del!(keys - Counter);
    ret!(ext <- Style(style))
}

pub fn counter_parent<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    let name = attrs.get_typed(FtmlKey::Counter, Id::from_str)?;
    let parent: Option<SectionLevel> = {
        let lvl = opt!(attrs.get_typed(FtmlKey::CounterParent, |s| {
            u8::from_str(s).map_err(|_| FtmlExtractionError::InvalidValue(FtmlKey::SetSectionLevel))
        }));
        lvl.map(|lvl| {
            lvl.try_into()
                .map_err(|_| FtmlExtractionError::InvalidValue(FtmlKey::SetSectionLevel))
        })
        .transpose()?
    };
    del!(keys - Counter);
    ret!(ext <- Counter(DocumentCounter { name, parent }))
}

pub fn importmodule<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    todo!()
}

pub fn usemodule<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    todo!()
}

pub fn module<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    let uri = attrs.take_new_module_uri(FtmlKey::Module, FtmlKey::Module, ext)?;
    let _ = attrs.take_language(FtmlKey::Language);
    let meta = opt!(attrs.take_module_uri(FtmlKey::Metatheory));
    let signature = opt!(attrs.take_language(FtmlKey::Signature));
    del!(keys - Language, Metatheory, Signature);
    ret!(ext <- Module{
        uri,
        meta,
        signature,
    } = Module)
}

pub fn mathstructure<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    todo!()
}

pub fn morphism<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    todo!()
}

pub fn assign<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    todo!()
}

pub fn section<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    _keys: &mut KeyList,
) -> Result<E> {
    let uri = attrs.get_elem_uri_from_id(ext, "section")?;
    ret!(ext <- Section(uri) = Section)
}

pub fn skipsection<E: FtmlExtractor>(
    ext: &mut E,
    _attrs: &mut E::Attributes<'_>,
    _keys: &mut KeyList,
) -> Result<E> {
    ret!(ext <- SkipSection = SkipSection)
}

pub fn title<E: FtmlExtractor>(
    ext: &mut E,
    _attrs: &mut E::Attributes<'_>,
    _keys: &mut KeyList,
) -> Result<E> {
    let mut iter = ext.iterate_narrative();
    while let Some(e) = iter.next() {
        match e {
            OpenNarrativeElement::Section { .. } => {
                drop(iter);
                return ret!(ext <- SectionTitle = SectionTitle);
            }
            OpenNarrativeElement::SkipSection { .. } => break,
            OpenNarrativeElement::Module { .. } => (),
        }
    }
    Err(FtmlExtractionError::NotIn(
        FtmlKey::Title,
        "a section or paragraph",
    ))
}

pub fn slide<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    todo!()
}

pub fn slide_number<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    todo!()
}

pub fn definition<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    todo!()
}

pub fn paragraph<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    todo!()
}

pub fn assertion<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    todo!()
}

pub fn example<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    todo!()
}

pub fn proof<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    todo!()
}

pub fn subproof<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    todo!()
}

pub fn proofbody<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    todo!()
}

pub fn problem<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    todo!()
}

pub fn subproblem<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    todo!()
}

pub fn problem_hint<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    todo!()
}

pub fn solution<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    todo!()
}

pub fn gnote<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    todo!()
}

pub fn answer_class<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    todo!()
}

pub fn answer_class_feedback<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    todo!()
}

pub fn multiple_choice_block<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    todo!()
}

pub fn single_choice_block<E: FtmlExtractor>(
    ext: &mut E,
    attrs: &mut E::Attributes<'_>,
    keys: &mut KeyList,
) -> Result<E> {
    todo!()
}

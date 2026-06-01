use crate::terms::arguments::MaybeSequence;
use crate::terms::helpers::IntoTerm;
use crate::terms::{ApplicationTerm, VarOrSym};

use super::Variable;
use super::{Argument, BoundArgument, Term};
use ftml_uris::{
    DocumentElementUri, Id, Language, ModuleUri, PathUri, SimpleUriName, SymbolUri, UriName,
};
use openmath::ser::{AsOMS, OMAttr, Omv};
use openmath::{OM, OMMaybeForeign, OMSerializable};
use std::hint::unreachable_unchecked;
use std::str::FromStr;

macro_rules! uri {
    ($(  $name:ident  $(  : $t:ty := $l:literal)?   $( = $lb:literal )?  ),* $(,)?) => {
        $(
            uri!{@go
                $name $( : $t := $l )? $( = $lb )?
            }
        )*
    };
    (@go $name:ident = $l:literal) => {
            pub static $name: std::sync::LazyLock<SymbolUri> = std::sync::LazyLock::new(||
                MOD.clone() | $l.parse::<ftml_uris::UriName>().expect("Is a valid URI")
            );
    };
    (@go $name:ident : $t:ty := $l:literal) => {
            pub static $name: std::sync::LazyLock<$t> = std::sync::LazyLock::new(||
                $l.parse().expect("Is a valid URI")
            );
    }
}

uri! {
    MOD:ModuleUri := "http://mathhub.info?a=FTML/meta&m=OpenMath",
    RESOLVED_VARIABLE_URI = "variable uri",
    SEQUENCE_ARGUMENT = "sequence argument",
    SEQUENCE_ARGUMENT_ONE = "sequence argument one",
    NOTATED = "variable notation",
    PRESENTATION = "presentation",
    IS_BOUND_APP = "binding application"
}

impl openmath::ser::OMSerializable for Variable {
    fn as_openmath<'s, S: openmath::ser::OMSerializer<'s>>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Err> {
        match self {
            Self::Name {
                name,
                notated: Some(notated),
            } => serializer.omattr(std::iter::once((&*NOTATED, &Omv(notated))), &Omv(name)),
            Self::Name { name, .. } => serializer.omv(name),
            Self::Ref {
                declaration,
                is_sequence: Some(true),
            } => serializer.omattr(
                [
                    (&*RESOLVED_VARIABLE_URI, &either::Left(declaration.as_oms())),
                    (
                        &*ftml_uris::metatheory::SEQUENCE_TYPE,
                        &either::Right(ftml_uris::metatheory::SEQUENCE_TYPE.as_oms()),
                    ),
                ]
                .into_iter(),
                &Omv(declaration.name()),
            ),
            Self::Ref { declaration, .. } => serializer.omattr(
                std::iter::once((&*RESOLVED_VARIABLE_URI, &declaration.as_oms())),
                &Omv(declaration.name()),
            ),
        }
    }
}

impl openmath::ser::OMSerializable for Term {
    fn as_openmath<'s, S: openmath::ser::OMSerializer<'s>>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Err> {
        match self {
            Self::Symbol { presentation, .. } | Self::Var { presentation, .. } => {
                do_presentation(serializer, presentation.as_ref(), self)
            }
            Self::Application(a) => do_presentation(serializer, a.presentation.as_ref(), self),
            Self::Bound(a) => do_presentation(serializer, a.presentation.as_ref(), self),
            _ => todo!(),
        }
    }
}

fn do_presentation<'s, S: openmath::ser::OMSerializer<'s>>(
    serializer: S,
    presentation: Option<&VarOrSym>,
    then: &Term,
) -> Result<S::Ok, S::Err> {
    if let Some(VarOrSym::Sym(uri)) = presentation {
        serializer.omattr(
            std::iter::once((&*PRESENTATION, &uri.as_oms())),
            NoPres(then),
        )
    } else if let Some(VarOrSym::Var(v)) = presentation {
        serializer.omattr(std::iter::once((&*PRESENTATION, v)), NoPres(then))
    } else {
        then.as_openmath(serializer)
    }
}

struct NoPres<'t>(&'t Term);
impl openmath::ser::OMSerializable for NoPres<'_> {
    // TODO: is_sequence
    fn as_openmath<'s, S: openmath::ser::OMSerializer<'s>>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Err> {
        match self.0 {
            Term::Symbol { uri, .. } => uri.as_oms().as_openmath(serializer),
            Term::Var { variable, .. } => variable.as_openmath(serializer),
            Term::Application(app) => serializer.oma(
                &app.head,
                app.arguments.iter().map(|a| match a {
                    Argument::Simple(a) => either_of::EitherOf3::A(a),
                    Argument::Sequence(MaybeSequence::One(e)) => {
                        either_of::EitherOf3::B(OneAsSequence(e))
                    }
                    Argument::Sequence(MaybeSequence::Seq(e)) => {
                        either_of::EitherOf3::C(SequenceArgument(e))
                    }
                }),
            ),
            Term::Bound(b) => BoundArgs {
                head: &b.head,
                args: &b.arguments,
                //bd: &b.body,
            }
            .as_openmath(serializer),
            _ => todo!(),
        }
    }
}

struct OneAsSequence<'t>(&'t Term);
impl openmath::ser::OMSerializable for OneAsSequence<'_> {
    fn as_openmath<'s, S: openmath::ser::OMSerializer<'s>>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Err> {
        serializer.omattr(
            std::iter::once((&*SEQUENCE_ARGUMENT_ONE, &SEQUENCE_ARGUMENT_ONE.as_oms())),
            self.0,
        )
    }
}

struct SequenceArgument<'t>(&'t [Term]);
impl openmath::ser::OMSerializable for SequenceArgument<'_> {
    fn as_openmath<'s, S: openmath::ser::OMSerializer<'s>>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Err> {
        struct SeqExpr<'t>(&'t [Term]);
        impl openmath::ser::OMSerializable for SeqExpr<'_> {
            fn as_openmath<'s, S: openmath::ser::OMSerializer<'s>>(
                &self,
                serializer: S,
            ) -> Result<S::Ok, S::Err> {
                serializer.oma(
                    ftml_uris::metatheory::SEQUENCE_EXPRESSION.as_oms(),
                    self.0.iter(),
                )
            }
        }
        serializer.omattr(
            std::iter::once((&*SEQUENCE_ARGUMENT, &SEQUENCE_ARGUMENT.as_oms())),
            SeqExpr(self.0),
        )
    }
}

// -----------------------------------------------------------------------------------------

pub enum IntermediateTerm {
    Term(Term),
    Element(DocumentElementUri),
    ArgumentSequenceOne(Term),
    ArgumentSequence(Box<[Term]>),
}
impl TryInto<Term> for IntermediateTerm {
    type Error = Error;
    fn try_into(self) -> Result<Term, Self::Error> {
        match self {
            Self::Term(t) => Ok(t),
            Self::Element(_) => Err(Error::UriParser(
                ftml_uris::errors::UriParseError::TooManyPartsFor {
                    uri_kind: ftml_uris::UriKind::Symbol,
                },
            )),
            Self::ArgumentSequenceOne(_) | Self::ArgumentSequence(_) => Err(Error::Misplaced),
        }
    }
}

// this should be moved to the openmath crate
const fn attrs<'s, 'o, I>(
    om: &'s mut openmath::OM<'o, I>,
) -> &'s mut Vec<openmath::Attr<'o, OMMaybeForeign<'o, I>>> {
    match om {
        OM::OMA { attrs, .. }
        | OM::OMB { attrs, .. }
        | OM::OMBIND { attrs, .. }
        | OM::OME { attrs, .. }
        | OM::OMF { attrs, .. }
        | OM::OMI { attrs, .. }
        | OM::OMS { attrs, .. }
        | OM::OMSTR { attrs, .. }
        | OM::OMV { attrs, .. } => attrs,
    }
}

#[derive(Default)]
struct Attrs {
    seq_arg: bool,
    single_seq: bool,
    seq_var: bool,
    presentation: Option<VarOrSym>,
    resolved_variable_uri: Option<DocumentElementUri>,
    notated: Option<Id>,
}
impl Attrs {
    fn parse(om: &mut openmath::OM<'_, IntermediateTerm>, cd_base: &str) -> Self {
        let mut ret = Self::default();

        for a in std::mem::take(attrs(om)) {
            if PRESENTATION.is_om_attr(&a, cd_base).is_some() {
                match a.value {
                    OMMaybeForeign::OM(IntermediateTerm::Term(Term::Var { variable, .. })) => {
                        ret.presentation = Some(VarOrSym::Var(variable));
                    }
                    OMMaybeForeign::OM(IntermediateTerm::Term(Term::Symbol { uri, .. })) => {
                        ret.presentation = Some(VarOrSym::Sym(uri));
                    }
                    _ => (),
                }
                continue;
            }
            if ftml_uris::metatheory::SEQUENCE_TYPE
                .is_om_attr(&a, cd_base)
                .is_some()
            {
                if let OMMaybeForeign::OM(IntermediateTerm::Term(Term::Symbol { uri, .. })) =
                    a.value
                    && uri == *ftml_uris::metatheory::SEQUENCE_TYPE
                {
                    ret.seq_var = true;
                }
                continue;
            }
            if RESOLVED_VARIABLE_URI.is_om_attr(&a, cd_base).is_some() {
                if let OMMaybeForeign::OM(IntermediateTerm::Element(uri)) = a.value {
                    ret.resolved_variable_uri = Some(uri);
                }
                continue;
            }
            if NOTATED.is_om_attr(&a, cd_base).is_some() {
                if let OMMaybeForeign::OM(IntermediateTerm::Term(Term::Var {
                    variable: Variable::Name { name, .. },
                    ..
                })) = a.value
                {
                    ret.notated = Some(name);
                }
                continue;
            }
            if SEQUENCE_ARGUMENT_ONE.is_om_attr(&a, cd_base).is_some() {
                ret.single_seq = true;
                continue;
            }
            if SEQUENCE_ARGUMENT.is_om_attr(&a, cd_base).is_some() {
                ret.seq_arg = true;
            }
        }
        ret
    }
}

impl openmath::de::OMDeserializable<'_> for Term {
    type Ret = IntermediateTerm;
    type Err = Error;
    #[allow(clippy::too_many_lines)]
    fn from_openmath(
        mut om: openmath::OM<'_, IntermediateTerm>,
        cd_base: &str,
    ) -> Result<IntermediateTerm, Self::Err>
    where
        Self: Sized,
    {
        fn inner(
            mut om: openmath::OM<'_, IntermediateTerm>,
            cd_base: &str,
            attrs: Attrs,
        ) -> Result<IntermediateTerm, Error> {
            match om {
                OM::OMS { cd, name, .. } => {
                    let path: PathUri = cd_base.parse()?;
                    if let Some((d, l)) = cd.split_once("&l=") {
                        let elem =
                            path & (
                                SimpleUriName::from_str(d)?,
                                Language::from_str(l).map_err(|_| {
                                    ftml_uris::errors::UriParseError::InvalidLanguage
                                })?,
                            ) & UriName::from_str(&name)?;
                        Ok(IntermediateTerm::Element(elem))
                    } else {
                        let sym = path | UriName::from_str(&cd)? | UriName::from_str(&name)?;
                        Ok(IntermediateTerm::Term(Term::Symbol {
                            uri: sym,
                            presentation: attrs.presentation,
                        }))
                    }
                }
                OM::OMV { .. } if let Some(uri) = attrs.resolved_variable_uri => {
                    Ok(IntermediateTerm::Term(Term::Var {
                        variable: Variable::Ref {
                            declaration: uri,
                            is_sequence: Some(attrs.seq_var),
                        },
                        presentation: attrs.presentation,
                    }))
                }
                OM::OMV { name, .. } => Ok(IntermediateTerm::Term(Term::Var {
                    variable: Variable::Name {
                        name: Id::from_str(&name)?,
                        notated: attrs.notated,
                    },
                    presentation: attrs.presentation,
                })),
                OM::OMA {
                    applicant: IntermediateTerm::Term(head),
                    arguments,
                    ..
                } => Ok(IntermediateTerm::Term(Term::Application(
                    ApplicationTerm::new(
                        head,
                        arguments
                            .into_iter()
                            .map(|t| match t {
                                IntermediateTerm::Element(_) => Err(Error::UriParser(
                                    ftml_uris::errors::UriParseError::TooManyPartsFor {
                                        uri_kind: ftml_uris::UriKind::Symbol,
                                    },
                                )),
                                IntermediateTerm::Term(t) => Ok(Argument::Simple(t)),
                                IntermediateTerm::ArgumentSequence(s) => {
                                    Ok(Argument::Sequence(MaybeSequence::Seq(s)))
                                }
                                IntermediateTerm::ArgumentSequenceOne(t) => {
                                    Ok(Argument::Sequence(MaybeSequence::One(t)))
                                }
                            })
                            .collect::<Result<Box<[Argument]>, _>>()?,
                        attrs.presentation,
                    ),
                ))),

                o => Err(Error::Unsupported(o.kind())),
            }
        }
        let attrs = Attrs::parse(&mut om, cd_base);
        let is_arg_seq = attrs.seq_arg;
        let is_arg_seq_one = attrs.single_seq;
        // TODO: is_sequence
        let tm = inner(om, cd_base, attrs)?;
        if is_arg_seq {
            let IntermediateTerm::Term(Self::Application(app)) = tm else {
                return Ok(tm);
            };
            if !ftml_uris::metatheory::SEQUENCE_EXPRESSION.term_is(&app.head)
                || app
                    .arguments
                    .iter()
                    .any(|a| !matches!(a, Argument::Simple(_)))
            {
                return Ok(IntermediateTerm::Term(Self::Application(app)));
            }
            Ok(IntermediateTerm::ArgumentSequence(
                app.arguments
                    .iter()
                    .map(|a| {
                        let Argument::Simple(t) = a else {
                            // SAFETY: app.arguments.iter().any() check above
                            unsafe { unreachable_unchecked() }
                        };
                        t.clone()
                    })
                    .collect(),
            ))
        } else if is_arg_seq_one {
            let IntermediateTerm::Term(tm) = tm else {
                return Ok(tm);
            };
            Ok(IntermediateTerm::ArgumentSequenceOne(tm))
        } else {
            Ok(tm)
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("uri parse error: {0}")]
    UriParser(#[from] ftml_uris::errors::UriParseError),
    #[error("uri segment parse error: {0}")]
    SegmentParser(#[from] ftml_uris::errors::SegmentParseError),
    #[error("unsupported OpenMath kind: {0}")]
    Unsupported(openmath::OMKind),
    #[error("misplaced application argument")]
    Misplaced,
}

struct Var<'a>(&'a Variable);
impl openmath::ser::BindVar for Var<'_> {
    fn name(&self) -> impl std::fmt::Display {
        match self.0 {
            Variable::Name { name, .. } => either::Left(name),
            Variable::Ref { declaration, .. } => either::Right(declaration.name()),
        }
    }
    fn attrs(&self) -> impl ExactSizeIterator<Item: openmath::ser::OMAttr> {
        match self.0 {
            Variable::Name { .. } => either::Left(std::iter::empty()),
            Variable::Ref { declaration, .. } => {
                either::Right(std::iter::once((&*RESOLVED_VARIABLE_URI, declaration)))
            }
        }
    }
}

struct Oma<'a, H: OMSerializable, A: OMSerializable> {
    head: H,
    args: &'a [A],
}
impl<H: OMSerializable, A: OMSerializable> openmath::ser::OMSerializable for Oma<'_, H, A> {
    fn as_openmath<'s, S: openmath::ser::OMSerializer<'s>>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Err> {
        serializer.oma(&self.head, self.args.iter())
    }
}

#[allow(clippy::struct_field_names)]
struct BoundArgs<'a, H: OMSerializable = &'a Term> {
    head: &'a H,
    args: &'a [BoundArgument],
    //bd: &'a Term,
}
impl<H: OMSerializable> openmath::ser::OMSerializable for BoundArgs<'_, H> {
    fn as_openmath<'s, S: openmath::ser::OMSerializer<'s>>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Err> {
        match self.args.first() {
            None => self.head.as_openmath(serializer), //self.bd.as_openmath(serializer),
            Some(BoundArgument::Bound(a)) => serializer.ombind(
                &self.head,
                std::iter::once(&a.var),
                &Self {
                    head: self.head,
                    args: &self.args[1..],
                    //bd: self.bd,
                },
            ),
            Some(BoundArgument::BoundSeq(MaybeSequence::One(v))) => serializer.ombind(
                &self.head,
                std::iter::once(Var(&v.var)),
                &Self {
                    head: self.head,
                    args: &self.args[1..],
                    //bd: self.bd,
                },
            ),
            Some(BoundArgument::BoundSeq(MaybeSequence::Seq(s))) => serializer.ombind(
                &self.head,
                s.iter().map(|v| &v.var),
                &Self {
                    head: self.head,
                    args: &self.args[1..],
                    //bd: self.bd,
                },
            ),
            Some(BoundArgument::Simple(t)) => BoundArgs {
                head: &Oma {
                    head: &self.head,
                    args: &[t],
                },
                args: &self.args[1..],
                //bd: self.bd,
            }
            .as_openmath(serializer),
            Some(BoundArgument::Sequence(MaybeSequence::One(t))) => BoundArgs {
                head: &Oma {
                    head: &self.head,
                    args: &[OneAsSequence(t)],
                },
                args: &self.args[1..],
                //bd: self.bd,
            }
            .as_openmath(serializer),
            Some(BoundArgument::Sequence(MaybeSequence::Seq(s))) => BoundArgs {
                head: &Oma {
                    head: &self.head,
                    args: s,
                },
                args: &self.args[1..],
                //bd: self.bd,
            }
            .as_openmath(serializer),
        }
    }
}

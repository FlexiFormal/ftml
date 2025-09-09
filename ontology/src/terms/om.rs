use crate::terms::arguments::MaybeSequence;

use super::Variable;
use super::{Argument, BoundArgument, Term};
use ftml_uris::{Id, PathUri, UriName};
use openmath::ser::{AsOMS, Omv};
use openmath::{OM, OMSerializable};
use std::str::FromStr;

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("uri parse error: {0}")]
    UriParser(#[from] ftml_uris::errors::UriParseError),
    #[error("uri segment parse error: {0}")]
    SegmentParser(#[from] ftml_uris::errors::SegmentParseError),
    #[error("unsupported OpenMath kind: {0}")]
    Unsupported(openmath::OMKind),
}

struct Seq<'t>(&'t [Term]);
impl openmath::ser::OMSerializable for Seq<'_> {
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
struct AsSeq<'t>(&'t Term);
impl openmath::ser::OMSerializable for AsSeq<'_> {
    fn as_openmath<'s, S: openmath::ser::OMSerializer<'s>>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Err> {
        serializer.omattr(
            std::iter::once((
                &*ftml_uris::metatheory::SEQUENCE_EXPRESSION,
                &ftml_uris::metatheory::SEQUENCE_EXPRESSION.as_oms(),
            )),
            self.0,
        )
    }
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
            Variable::Ref { declaration, .. } => either::Right(std::iter::once((
                &*ftml_uris::metatheory::RESOLVED_VARIABLE_URI,
                declaration,
            ))),
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
struct Args<'a, H: OMSerializable = &'a Term> {
    head: &'a H,
    args: &'a [BoundArgument],
    bd: &'a Term,
}
impl<H: OMSerializable> openmath::ser::OMSerializable for Args<'_, H> {
    fn as_openmath<'s, S: openmath::ser::OMSerializer<'s>>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Err> {
        match self.args.first() {
            None => self.bd.as_openmath(serializer),
            Some(BoundArgument::Bound(a)) => serializer.ombind(
                &self.head,
                std::iter::once(a),
                &Self {
                    head: self.head,
                    args: &self.args[1..],
                    bd: self.bd,
                },
            ),
            Some(BoundArgument::BoundSeq(MaybeSequence::One(v))) => serializer.ombind(
                &self.head,
                std::iter::once(Var(v)),
                &Self {
                    head: self.head,
                    args: &self.args[1..],
                    bd: self.bd,
                },
            ),
            Some(BoundArgument::BoundSeq(MaybeSequence::Seq(s))) => serializer.ombind(
                &self.head,
                s.iter(),
                &Self {
                    head: self.head,
                    args: &self.args[1..],
                    bd: self.bd,
                },
            ),
            Some(BoundArgument::Simple(t)) => Args {
                head: &Oma {
                    head: &self.head,
                    args: &[t],
                },
                args: &self.args[1..],
                bd: self.bd,
            }
            .as_openmath(serializer),
            Some(BoundArgument::Sequence(MaybeSequence::One(t))) => Args {
                head: &Oma {
                    head: &self.head,
                    args: &[AsSeq(t)],
                },
                args: &self.args[1..],
                bd: self.bd,
            }
            .as_openmath(serializer),
            Some(BoundArgument::Sequence(MaybeSequence::Seq(s))) => Args {
                head: &Oma {
                    head: &self.head,
                    args: s,
                },
                args: &self.args[1..],
                bd: self.bd,
            }
            .as_openmath(serializer),
        }
    }
}

impl openmath::ser::OMSerializable for Term {
    fn as_openmath<'s, S: openmath::ser::OMSerializer<'s>>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Err> {
        match self {
            Self::Symbol { uri, .. } => uri.as_oms().as_openmath(serializer),
            Self::Var {
                variable: Variable::Name { name, .. },
                ..
            } => serializer.omv(name),
            Self::Var {
                variable: Variable::Ref { declaration, .. },
                ..
            } => serializer.omattr(
                std::iter::once((
                    &*ftml_uris::metatheory::RESOLVED_VARIABLE_URI,
                    &declaration.as_oms(),
                )),
                &Omv(declaration.name()),
            ),
            Self::Application(app) => serializer.oma(
                &app.head,
                app.arguments.iter().map(|a| match a {
                    Argument::Simple(a) => either_of::EitherOf3::A(a),
                    Argument::Sequence(MaybeSequence::One(e)) => either_of::EitherOf3::B(AsSeq(e)),
                    Argument::Sequence(MaybeSequence::Seq(e)) => either_of::EitherOf3::C(Seq(e)),
                }),
            ),
            Self::Bound(b) => Args {
                head: &b.head,
                args: &b.arguments,
                bd: &b.body,
            }
            .as_openmath(serializer),
            _ => todo!(),
        }
    }
}

impl openmath::de::OMDeserializable<'_> for Term {
    type Ret = Self;
    type Err = Error;
    fn from_openmath(om: openmath::OM<'_, Self>, cd_base: &str) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        match om {
            OM::OMS { cd, name, attrs: _ } => {
                let path: PathUri = cd_base.parse()?;
                let sym = path | UriName::from_str(&cd)? | UriName::from_str(&name)?;
                Ok(Self::Symbol {
                    uri: sym,
                    presentation: None,
                })
            }
            OM::OMV { name, .. } => Ok(Self::Var {
                variable: Variable::Name {
                    name: Id::from_str(&name)?,
                    notated: None,
                },
                presentation: None,
            }),

            o => Err(Error::Unsupported(o.kind())),
        }
    }
}

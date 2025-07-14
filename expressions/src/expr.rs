use ftml_uris::SymbolUri;

use crate::{Argument, variables::Variable};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum Expr {
    Symbol(SymbolUri),
    Var(Variable),
    Application {
        head: Box<Self>,
        arguments: Vec<Argument>,
    },
}
impl Expr {
    /*#[must_use]
    #[inline]
    pub const fn normalize(self) -> Self {
        self
    }*/
}

#[cfg(feature = "openmath")]
pub mod om {
    use super::Variable;
    use crate::{Argument, Expr};
    use ftml_uris::{PathUri, UriName};
    use openmath::OM;
    use openmath::ser::AsOMS;
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

    impl openmath::ser::OMSerializable for Expr {
        fn as_openmath<'s, S: openmath::ser::OMSerializer<'s>>(
            &self,
            serializer: S,
        ) -> Result<S::Ok, S::Err> {
            match self {
                Self::Symbol(s) => s.as_oms().as_openmath(serializer),
                Self::Var(Variable::Name(n)) => serializer.omv(n),
                Self::Var(Variable::Ref { declaration, .. }) => serializer.omv(declaration.name()),
                _ => todo!(),
                /*Self::Application { head, arguments }
                    if arguments
                        .iter
                        .any(|a| matches!(a, Argument::Bound(_) | Argument::BoundSeq(_))) =>
                {
                    serializer.ombind(head, vars, body)
                }
                */
                /*
                  Self::Var(Variable::Ref {
                      declaration,
                      is_sequence: None | Some(false),
                  }) => serializer.omattr(
                      std::iter::once((
                          &*ftml_uris::metatheory::RESOLVED_VARIABLE_URI,
                          &declaration.as_oms(),
                      )),
                      openmath::ser::Omv(declaration.name()),
                  ),
                  Self::Var(Variable::Ref { declaration, .. }) => serializer.omattr(
                      [
                          (
                              &*ftml_uris::metatheory::RESOLVED_VARIABLE_URI,
                              &Either::Left(declaration.as_oms()),
                          ),
                          (&*ftml_uris::metatheory::SEQUENCE_TYPE, &Either::Right(1u64)),
                      ]
                      .into_iter(),
                      openmath::ser::Omv(declaration.name()),
                  ), */
            }
        }
    }

    #[cfg(feature = "openmath")]
    impl openmath::de::OMDeserializable<'_> for Expr {
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
                    Ok(Self::Symbol(sym))
                }
                OM::OMV { name, .. } => Ok(Self::Var(Variable::Name(UriName::from_str(&name)?))),

                o => Err(Error::Unsupported(o.kind())),
            }
        }
    }
}

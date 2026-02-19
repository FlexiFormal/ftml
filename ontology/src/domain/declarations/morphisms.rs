use ftml_uris::{DomainUriRef, Id, ModuleUri, SimpleUriName, SymbolUri};

use crate::{
    domain::{
        HasDeclarations,
        declarations::{AnyDeclarationRef, Declaration, IsDeclaration, symbols::Symbol},
        modules::ModuleLike,
    },
    terms::Term,
    utils::SourceRange,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Morphism {
    pub uri: SymbolUri,
    pub domain: ModuleUri,
    pub total: bool,
    pub elements: Box<[Assignment]>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(skip))]
    pub elaboration: Elaboration,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub source: SourceRange,
}

impl crate::__private::Sealed for Morphism {}
impl crate::Ftml for Morphism {
    #[cfg(feature = "rdf")]
    fn triples(&self) -> impl IntoIterator<Item = ulo::rdf_types::Triple> {
        use ftml_uris::FtmlUri;
        use ulo::triple;

        let iri = self.uri.to_iri();
        [
            triple!(<(iri.clone())> : ulo:morphism),
            triple!(<(iri.clone())> rdfs:DOMAIN <(self.domain.to_iri())>),
        ]
        .into_iter()
        .chain(self.declarations().filter_map(move |e| match e {
            AnyDeclarationRef::Import { uri, .. } => {
                Some(triple!(<(iri.clone())> ulo:imports <(uri.to_iri())>))
            }
            AnyDeclarationRef::Extension(e) => {
                Some(triple!(<(iri.clone())> ulo:declares <(e.uri.to_iri())>))
            }
            AnyDeclarationRef::MathStructure(e) => {
                Some(triple!(<(iri.clone())> ulo:declares <(e.uri.to_iri())>))
            }
            AnyDeclarationRef::Morphism(e) => {
                Some(triple!(<(iri.clone())> ulo:declares <(e.uri.to_iri())>))
            }
            AnyDeclarationRef::NestedModule(e) => {
                Some(triple!(<(iri.clone())> ulo:declares <(e.uri.to_iri())>))
            }
            AnyDeclarationRef::Symbol(e) => {
                Some(triple!(<(iri.clone())> ulo:declares <(e.uri.to_iri())>))
            }
            AnyDeclarationRef::Rule { .. } => None,
        }))
    }
    #[inline]
    fn source_range(&self) -> SourceRange {
        self.source
    }
}
impl IsDeclaration for Morphism {
    #[inline]
    fn uri(&self) -> Option<&SymbolUri> {
        Some(&self.uri)
    }
    #[inline]
    fn from_declaration(decl: AnyDeclarationRef<'_>) -> Option<&Self> {
        match decl {
            AnyDeclarationRef::Morphism(m) => Some(m),
            _ => None,
        }
    }
    #[inline]
    fn as_ref(&self) -> AnyDeclarationRef<'_> {
        AnyDeclarationRef::Morphism(self)
    }
}
impl HasDeclarations for Morphism {
    #[inline]
    fn declarations(
        &self,
    ) -> impl ExactSizeIterator<Item = AnyDeclarationRef<'_>> + DoubleEndedIterator {
        self.elaboration.get().iter().map(|d| d.as_ref()) //std::iter::empty() //self.elements.iter().map(Declaration::as_ref)
    }
    #[inline]
    fn domain_uri(&self) -> DomainUriRef<'_> {
        DomainUriRef::Symbol(&self.uri)
    }

    #[inline]
    fn initialize<E: std::fmt::Display>(
        &self,
        get: &mut dyn FnMut(&ModuleUri) -> Result<ModuleLike, E>,
    ) {
        if let Err(e) = Elaboration::initialize(self, get) {
            tracing::error!("Error elaborating: {e}");
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[allow(clippy::unsafe_derive_deserialize)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Assignment {
    pub original: SymbolUri,
    pub morphism: SymbolUri,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub definiens: Option<Term>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub refined_type: Option<Term>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub new_name: Option<SimpleUriName>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub macroname: Option<Id>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub source: SourceRange,
}
impl Assignment {
    #[must_use]
    pub fn elaborated_uri(&self) -> SymbolUri {
        self.new_name.as_ref().map_or_else(
            || {
                // SAFETY: segment already validated
                unsafe {
                    self.morphism.clone() / &self.original.name.last().parse().unwrap_unchecked()
                }
            },
            |name| self.morphism.module.clone() | name.clone(),
        )
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for Assignment {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        self.definiens
            .as_ref()
            .map(|t| t.deep_size_of_children(context))
            .unwrap_or_default()
            + self
                .refined_type
                .as_ref()
                .map(|t| t.deep_size_of_children(context))
                .unwrap_or_default()
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for Morphism {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        self.elements
            .iter()
            .map(|v| std::mem::size_of_val(v) + v.deep_size_of_children(context))
            .sum::<usize>()
    }
}

// -------------------------------------------------------------------------

#[derive(Default, Debug, Clone)]
pub struct Elaboration {
    contents: std::sync::OnceLock<Vec<Declaration>>,
}

impl Elaboration {
    pub fn get(&self) -> &[Declaration] {
        self.contents.get().map_or(&[], Vec::as_slice)
    }

    fn initialize<E: std::fmt::Display>(
        m: &Morphism,
        get: impl FnMut(&ModuleUri) -> Result<ModuleLike, E>,
    ) -> Result<(), E> {
        let mut err = None;
        let errp = &mut err;
        m.elaboration
            .contents
            .get_or_init(move || match Self::initialize_i(m, get) {
                Ok(v) => v,
                Err(e) => {
                    *errp = Some(e);
                    Vec::new()
                }
            });
        err.map_or(Ok(()), Err)
    }

    fn initialize_i<E: std::fmt::Display>(
        m: &Morphism,
        mut get: impl FnMut(&ModuleUri) -> Result<ModuleLike, E>,
    ) -> Result<Vec<Declaration>, E> {
        let full_domain = Self::collect_deps(m.domain.clone(), &mut get)?;
        let mut ret = Vec::new();
        let mut assigns = m.elements.iter().collect::<Vec<_>>();

        for d in full_domain.iter().flat_map(ModuleLike::declarations) {
            match d {
                AnyDeclarationRef::Import(_) => (),
                AnyDeclarationRef::Morphism(_) => todo!("???"),
                AnyDeclarationRef::MathStructure(_) => todo!("???"),
                AnyDeclarationRef::Extension(_) => todo!("???"),
                AnyDeclarationRef::NestedModule(_) => todo!("???"),
                AnyDeclarationRef::Symbol(s) => {
                    // Do something
                }
            }
        }
        Ok(ret)
    }

    fn collect_deps<E: std::fmt::Display>(
        init: ModuleUri,
        mut get: impl FnMut(&ModuleUri) -> Result<ModuleLike, E>,
    ) -> Result<Vec<ModuleLike>, E> {
        let mut dones = rustc_hash::FxHashSet::<ModuleUri>::default();
        let mut todos = vec![init];
        let mut ret = Vec::new();
        while let Some(todo) = todos.pop() {
            if dones.contains(&todo) {
                continue;
            }
            let module = get(&todo)?;
            for d in module.declarations() {
                if let AnyDeclarationRef::Import(uri) = d {
                    todos.push(uri.clone());
                }
            }
            dones.insert(todo);
            ret.push(module);
        }
        Ok(ret)
    }
}

// --------------------------------------------------------------------------

impl std::hash::Hash for Elaboration {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {}
}
impl PartialEq for Elaboration {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}
impl Eq for Elaboration {}
#[cfg(feature = "serde")]
impl bincode::Encode for Elaboration {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        Ok(())
    }
}
#[cfg(feature = "serde")]
impl<'de, C> bincode::BorrowDecode<'de, C> for Elaboration {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = C>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Ok(Self::default())
    }
}
#[cfg(feature = "serde")]
impl<C> bincode::Decode<C> for Elaboration {
    fn decode<D: bincode::de::Decoder<Context = C>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Ok(Self::default())
    }
}

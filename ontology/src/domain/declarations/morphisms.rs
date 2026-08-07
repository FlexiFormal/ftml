use ftml_uris::{DomainUriRef, FtmlUri, Id, ModuleUri, SimpleUriName, SymbolUri, Uri};

use crate::{
    domain::{
        HasDeclarations, SharedDeclaration,
        declarations::{
            AnyDeclarationRef, Declaration, IsSymbol, SharedSymbolLike,
            symbols::{Symbol, SymbolData},
        },
        modules::{Module, ModuleLike},
    },
    terms::{ApplicationTerm, Argument, IsTerm, MaybeSequence, Term, TermContainer, VarOrSym},
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
    //#[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    //pub elaborated_from: Option<SymbolUri>,
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
impl IsSymbol for Morphism {
    #[inline]
    fn symbol_uri(&self) -> &SymbolUri {
        &self.uri
    }
    #[inline]
    fn from_declaration(decl: AnyDeclarationRef<'_>) -> Option<&Self> {
        match decl {
            AnyDeclarationRef::Morphism(m) => Some(m),
            _ => None,
        }
    }
    #[inline]
    fn as_decl(&self) -> AnyDeclarationRef<'_> {
        AnyDeclarationRef::Morphism(self)
    }
    /*
    #[inline]
    fn elaborated_from(&self) -> Option<&SymbolUri> {
        self.elaborated_from.as_ref()
    }
    */
}
impl HasDeclarations for Morphism {
    type DeclIter<'a>
        = std::iter::Map<
        std::slice::Iter<'a, Declaration>,
        fn(&'a Declaration) -> AnyDeclarationRef<'a>,
    >
    where
        Self: 'a;
    #[inline]
    fn declarations(&self) -> Self::DeclIter<'_> {
        self.elaboration.get().iter().map(|d| d.as_ref()) //std::iter::empty() //self.elements.iter().map(Declaration::as_ref)
    }
    #[inline]
    fn domain_uri(&self) -> DomainUriRef<'_> {
        DomainUriRef::Symbol(&self.uri)
    }

    #[inline]
    fn initialize(
        &self,
        get: &mut dyn FnMut(&ModuleUri) -> Option<ModuleLike>,
    ) -> Result<(), ModuleUri> {
        Elaboration::initialize(self, get)
            .inspect_err(|e| tracing::error!("Error elaborating; module missing: {e}"))
    }
    /*
    async fn initialize_async<E: std::fmt::Display, F>(&self, get: impl FnMut(&ModuleUri) -> F)
    where
        F: Future<Output = Result<ModuleLike, E>> + Send,
    {
        if let Err(e) = Elaboration::initialize_async(self, get).await {
            tracing::error!("Error elaborating: {e}");
        }
    }
    */
}
impl Morphism {
    pub fn unapply<'s>(
        t: &'s Term,
        get: &mut impl FnMut(&SymbolUri) -> Option<SharedSymbolLike>,
    ) -> Option<(SharedDeclaration<Self>, &'s Term)> {
        let Term::Application(app) = t else {
            return None;
        };
        if app.arguments.len() != 1 {
            return None;
        }
        let Some(Argument::Simple(arg)) = app.arguments.first() else {
            return None;
        };
        let Term::Symbol { uri: head, .. } = &app.head else {
            return None;
        };
        if let Some(SharedSymbolLike::Morphism(m)) = get(head) {
            Some((m, arg))
        } else {
            None
        }
    }
    ///#### Errors
    /// if not elaborated yet
    pub fn apply<'t>(
        &self,
        t: &'t Term,
        get: &mut impl FnMut(&SymbolUri) -> Option<SharedSymbolLike>,
    ) -> Result<std::borrow::Cow<'t, Term>, Option<SymbolUri>> {
        let _ = &self.elaboration.contents.get().ok_or(None)?;

        //let mut err = None;
        let r = t.modify(|t| {
            // Do not recurse into morphisms:
            if let Some(either::Left(s)) = t.head()
                && let Some(SharedSymbolLike::Morphism(_)) = get(s)
            {
                if let Some((m, t)) = Self::unapply(t, get)
                    && let Ok(r) = m.apply(t, get)
                    && *r != *t
                {
                    return Some(std::ops::ControlFlow::Continue(r.into_owned()));
                }
                return Some(std::ops::ControlFlow::Break(t.clone()));
            }
            match t {
                // emergency break:
                Term::Application(app) if app.head.is(&*ftml_uris::metatheory::APPLY_IMPLICIT) => {
                    if let [
                        Argument::Simple(Term::Symbol { uri, presentation }),
                        Argument::Sequence(MaybeSequence::Seq(_)),
                    ] = &*app.arguments
                        && matches!(
                            self.apply_symbol(uri, presentation),
                            None | Some(std::ops::ControlFlow::Break(Term::Symbol { .. }))
                        )
                    {
                        None
                    } else {
                        Some(std::ops::ControlFlow::Break(Term::Application(
                            ApplicationTerm::new(
                                self.uri.clone().into(),
                                Box::new([Argument::Simple(t.clone())]),
                                None,
                            ),
                        )))
                    }
                }
                Term::Symbol { uri, presentation } => self.apply_symbol(uri, presentation),
                _ => None,
            }
        });
        //if let Some(err) = err {
        //    Err(Some(err))
        //} else {
        /*tracing::warn!(
            "{} applied to {:?}:\n{:?}",
            self.uri,
            t.debug_short(),
            r.debug_short()
        );*/
        Ok(r)
        //}
    }
    #[allow(clippy::ref_option)]
    fn apply_symbol(
        &self,
        uri: &SymbolUri,
        presentation: &Option<VarOrSym>,
    ) -> Option<std::ops::ControlFlow<Term, Term>> {
        let elab = &self.elaboration.contents.get()?;
        if let Some((target, _)) = elab.domain.get(uri) {
            Some(std::ops::ControlFlow::Break(Term::Symbol {
                uri: target.clone(),
                presentation: presentation.clone(),
            }))
        } else if elab.domain.values().any(|(u, _)| u == uri) || elab.identities.contains(uri) {
            None
        }
        /*else if err.is_none() {
            if let Some(df) = exp(uri) {
                //tracing::warn!("Expanding {uri}:\n   {:?}", df.debug_short());
                Some(std::ops::ControlFlow::Continue(df))
            } else {
                err = Some(uri.clone());
                None
            }
        }*/
        else {
            Some(std::ops::ControlFlow::Break(Term::Application(
                ApplicationTerm::new(
                    self.uri.clone().into(),
                    Box::new([Argument::Simple(uri.clone().into())]),
                    None,
                ),
            )))
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
    pub fn default_uri(morphism: &SymbolUri, original: &SymbolUri) -> SymbolUri {
        // SAFETY: segment already validated
        unsafe { morphism.clone() / &original.name.last().parse().unwrap_unchecked() }
    }
    #[must_use]
    pub fn elaborated_uri(&self) -> SymbolUri {
        self.new_name.as_ref().map_or_else(
            || Self::default_uri(&self.morphism, &self.original),
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
    contents: std::sync::OnceLock<InnerElaboration>,
    elaborating: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

#[derive(Debug, Clone)]
struct InnerElaboration {
    contents: Vec<Declaration>,
    identities: Identities,
    domain: rustc_hash::FxHashMap<SymbolUri, (SymbolUri, bool)>,
}

#[derive(Debug, Clone, Default)]
struct Identities {
    symbols: rustc_hash::FxHashSet<SymbolUri>,
    modules: rustc_hash::FxHashSet<ModuleUri>,
}
impl Identities {
    fn contains(&self, uri: &SymbolUri) -> bool {
        self.symbols.iter().any(|s| s.contains(uri))
            || self
                .modules
                .iter()
                .any(|m| m.contains(DomainUriRef::Symbol(uri)))
    }
}

impl Elaboration {
    pub fn get(&self) -> &[Declaration] {
        self.contents.get().map_or(&[], |d| &d.contents)
    }
    // .1: whether had an assignment
    pub fn get_map(&self) -> Option<&rustc_hash::FxHashMap<SymbolUri, (SymbolUri, bool)>> {
        self.contents.get().map(|e| &e.domain)
    }

    fn initialize(
        m: &Morphism,
        get: impl FnMut(&ModuleUri) -> Option<ModuleLike>,
    ) -> Result<(), ModuleUri> {
        if m.elaboration
            .elaborating
            .swap(true, std::sync::atomic::Ordering::Relaxed)
        {
            return Ok(());
        }
        if m.elaboration.contents.get().is_some() {
            m.elaboration
                .elaborating
                .store(false, std::sync::atomic::Ordering::Relaxed);
            return Ok(());
        }
        let r = Self::initialize_i(m, get).inspect_err(|_| {
            m.elaboration
                .elaborating
                .store(false, std::sync::atomic::Ordering::Relaxed);
        })?;

        /*
        println!(
            "Elaborated {}:\nDomain: {:#?}\nIdentities: {:#?}",
            m.uri, r.domain, r.identities
        );
         */

        let _ = m.elaboration.contents.set(r);
        m.elaboration
            .elaborating
            .store(false, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    /*
    async fn initialize_async<F>(
        m: &Morphism,
        get: impl FnMut(&ModuleUri) -> F,
    ) -> Result<(), ModuleUri>
    where
        F: Future<Output = Option<ModuleLike>>,
    {
        if m.elaboration.contents.get().is_some() {
            return Ok(());
        }
        let r = Self::initialize_async_i(m, get).await;
        match r {
            Ok(v) => {
                m.elaboration.contents.get_or_init(move || v);
                Ok(())
            }
            Err(e) => {
                m.elaboration.contents.get_or_init(Vec::new);
                Err(e)
            }
        }
    }
     */

    #[allow(clippy::too_many_lines, clippy::option_if_let_else)]
    fn initialize_i(
        morphism: &Morphism,
        mut get_module: impl FnMut(&ModuleUri) -> Option<ModuleLike>,
    ) -> Result<InnerElaboration, ModuleUri> {
        let mut identities = Identities::default();

        let parent_uri = morphism.uri.clone().parent();
        let Some(parent) = get_module(&parent_uri) else {
            return Err(parent_uri);
        };
        for d in parent.declarations() {
            if let Some(u) = d.uri() {
                if *u == morphism.uri {
                    break;
                }
                identities.symbols.insert(u.clone());
            } else if let AnyDeclarationRef::Import { uri, .. } = d {
                identities.modules.insert(uri.clone());
            }
        }

        let mut parents = parent_uri.ancestors().filter_map(|m| {
            if let Uri::Module(m) = m {
                Some(m)
            } else {
                None
            }
        });
        let _ = parents.next();
        let mut dones = rustc_hash::FxHashSet::default();
        let mut todos = Vec::new();
        if let ModuleLike::Module(m) = parent
            && let Some(meta) = &m.meta_module
        {
            todos.push(get_module(meta).ok_or_else(|| meta.clone())?);
        }
        for m in parents {
            todos.extend(
                Self::collect_deps(m.clone(), &mut get_module, &mut dones)?
                    .into_iter()
                    .rev(),
            );
            if let ModuleLike::Module(m) = get_module(&m).ok_or_else(|| m.clone())?
                && let Some(m) = m.meta_module.as_ref()
            {
                todos.extend(
                    Self::collect_deps(m.clone(), &mut get_module, &mut dones)?
                        .into_iter()
                        .rev(),
                );
            }
        }
        for t in todos {
            identities.modules.insert(match t.domain_uri() {
                DomainUriRef::Module(m) => m.clone(),
                DomainUriRef::Symbol(s) => s.clone().into_module(),
            });
        }

        let full_domain = Self::collect_deps(
            morphism.domain.clone(),
            get_module,
            &mut rustc_hash::FxHashSet::default(),
        )?;
        Ok(Self::initialize_ii(
            morphism,
            identities,
            full_domain.iter(),
        ))
    }
    /*
    async fn initialize_async_i<F>(
        morphism: &Morphism,
        get_module: impl FnMut(&ModuleUri) -> F,
    ) -> Result<Vec<Declaration>, ModuleUri>
    where
        F: Future<Output = Option<ModuleLike>>,
    {
        let full_domain = Self::collect_deps_async(morphism.domain.clone(), get_module).await?;
        Ok(Self::initialize_ii(morphism, full_domain.iter()))
    }
     */

    #[allow(clippy::too_many_lines)]
    fn initialize_ii<'l>(
        morphism: &Morphism,
        identities: Identities,
        full_domain: impl Iterator<Item = &'l ModuleLike>,
    ) -> InnerElaboration {
        fn do_decl(
            d: AnyDeclarationRef,
            ret: &mut Vec<Declaration>,
            map: &mut rustc_hash::FxHashMap<SymbolUri, (SymbolUri, bool)>,
            morphism: &Morphism,
        ) {
            match d {
                AnyDeclarationRef::Import { .. } | AnyDeclarationRef::Rule { .. } => (),
                AnyDeclarationRef::Symbol(original_symbol) => {
                    let mut changed = false;
                    let assignment = morphism
                        .elements
                        .iter()
                        .find(|ass| ass.original == original_symbol.uri);
                    let new_uri = assignment.map_or_else(
                        || Assignment::default_uri(&morphism.uri, &original_symbol.uri),
                        Assignment::elaborated_uri,
                    );
                    let tp = {
                        if let Some(ass) = assignment
                            && let Some(typ) = &ass.refined_type
                        {
                            changed = true;
                            TermContainer::new(typ.clone(), Some(ass.source))
                        } else {
                            original_symbol
                                .data
                                .tp
                                .checked_or_parsed()
                                .map(|(t, _)| {
                                    TermContainer::new(
                                        Term::Application(ApplicationTerm::new(
                                            Term::Symbol {
                                                uri: morphism.uri.clone(),
                                                presentation: None,
                                            },
                                            Box::new([Argument::Simple(t)]),
                                            None,
                                        )),
                                        assignment.map(|ass| ass.source),
                                    )
                                })
                                .unwrap_or_default()
                        }
                    };
                    let df = {
                        if let Some(ass) = assignment
                            && let Some(def) = &ass.definiens
                        {
                            changed = true;
                            TermContainer::new(def.clone(), Some(ass.source))
                        } else {
                            original_symbol
                                .data
                                .df
                                .checked_or_parsed()
                                .map(|(t, _)| {
                                    TermContainer::new(
                                        Term::Application(ApplicationTerm::new(
                                            Term::Symbol {
                                                uri: morphism.uri.clone(),
                                                presentation: None,
                                            },
                                            Box::new([Argument::Simple(t)]),
                                            None,
                                        )),
                                        assignment.map(|ass| ass.source),
                                    )
                                })
                                .unwrap_or_default()
                        }
                    };
                    let new_data = SymbolData {
                        arity: original_symbol.data.arity.clone(),
                        macroname: assignment.and_then(|ass| ass.macroname.clone()),
                        role: original_symbol.data.role.clone(),
                        tp,
                        df,
                        reordering: original_symbol.data.reordering.clone(),
                        assoctype: original_symbol.data.assoctype,
                        source: assignment.map(|ass| ass.source).unwrap_or_default(),
                        ..SymbolData::default()
                    };
                    map.insert(original_symbol.uri.clone(), (new_uri.clone(), changed));
                    ret.push(Declaration::Symbol(Symbol {
                        uri: new_uri,
                        data: Box::new(new_data),
                    }));
                }
                AnyDeclarationRef::Morphism(m) => {
                    // TODO this should probably preserve the morphism itself too
                    // somehow, for name resolution reasons
                    for d in m.elaboration.get() {
                        do_decl(d.as_ref(), ret, map, morphism);
                    }
                }
                AnyDeclarationRef::MathStructure(_) => {
                    tracing::error!("TODO: Structure in morphism {}", morphism.uri);
                }
                AnyDeclarationRef::Extension(_) => {
                    tracing::error!("TODO: Structure extension in morphism {}", morphism.uri);
                }
                AnyDeclarationRef::NestedModule(_) => {
                    tracing::error!("TODO: Nested module in morphism {}", morphism.uri);
                }
            }
        }
        let mut ret = Vec::new();
        let mut map = rustc_hash::FxHashMap::default();

        for d in full_domain.flat_map(ModuleLike::declarations) {
            do_decl(d, &mut ret, &mut map, morphism);
        }
        InnerElaboration {
            contents: ret,
            identities,
            domain: map,
        }
    }

    fn collect_deps(
        init: ModuleUri,
        mut get: impl FnMut(&ModuleUri) -> Option<ModuleLike>,
        dones: &mut rustc_hash::FxHashSet<ModuleUri>,
    ) -> Result<Vec<ModuleLike>, ModuleUri> {
        let mut todos = vec![init];
        let mut ret = Vec::new();
        while let Some(todo) = todos.pop() {
            if dones.contains(&todo) {
                continue;
            }
            let module = get(&todo).ok_or_else(|| todo.clone())?;
            module.initialize(&mut get)?;
            for d in module.declarations() {
                if let AnyDeclarationRef::Import { uri, .. } = d {
                    todos.push(uri.clone());
                }
            }
            dones.insert(todo);
            ret.push(module);
        }
        ret.reverse();
        Ok(ret)
    }
    /*
    async fn collect_deps_async<F>(
        init: ModuleUri,
        mut get_module: impl FnMut(&ModuleUri) -> F,
    ) -> Result<Vec<ModuleLike>, ModuleUri>
    where
        F: Future<Output = Option<ModuleLike>>,
    {
        let mut dones = rustc_hash::FxHashSet::<ModuleUri>::default();
        let mut todos = vec![init];
        let mut ret = Vec::new();
        while let Some(todo) = todos.pop() {
            if dones.contains(&todo) {
                continue;
            }
            let module = get_module(&todo).await.ok_or_else(|| todo.clone())?;
            for d in module.declarations() {
                if let AnyDeclarationRef::Import { uri, .. } = d {
                    todos.push(uri.clone());
                }
            }
            dones.insert(todo);
            ret.push(module);
        }
        Ok(ret)
    }
     */
}

// --------------------------------------------------------------------------

impl std::hash::Hash for Elaboration {
    fn hash<H: std::hash::Hasher>(&self, _: &mut H) {}
}
impl PartialEq for Elaboration {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl Eq for Elaboration {}
#[cfg(feature = "serde")]
impl bincode::Encode for Elaboration {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        _: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        Ok(())
    }
}
#[cfg(feature = "serde")]
impl<'de, C> bincode::BorrowDecode<'de, C> for Elaboration {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = C>>(
        _: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Ok(Self::default())
    }
}
#[cfg(feature = "serde")]
impl<C> bincode::Decode<C> for Elaboration {
    fn decode<D: bincode::de::Decoder<Context = C>>(
        _: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Ok(Self::default())
    }
}

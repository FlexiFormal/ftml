use crate::{
    domain::{
        SharedDeclaration,
        declarations::{
            structures::{MathStructure, StructureDeclaration, StructureExtension},
            symbols::Symbol,
        },
    },
    terms::{Argument, Term, arguments::MaybeSequence},
};
use either::Either;
use ftml_uris::{SymbolUri, UriName, metatheory};
use std::{hint::unreachable_unchecked, pin::Pin};

impl Term {
    pub fn get_in_record_type(
        &self,
        name: &UriName,
        get_struct: impl Fn(
            &SymbolUri,
        ) -> Option<
            Either<SharedDeclaration<MathStructure>, SharedDeclaration<StructureExtension>>,
        >,
    ) -> Option<SharedDeclaration<Symbol>> {
        let mut dones = Vec::new();
        get(self, &mut dones, name, get_struct)
    }

    #[allow(clippy::future_not_send)]
    pub fn get_in_record_type_async<
        Error: Send + 'static,
        Fut: Future<
            Output = Result<
                Either<SharedDeclaration<MathStructure>, SharedDeclaration<StructureExtension>>,
                Error,
            >,
        >,
    >(
        self,
        name: UriName,
        get_struct: impl Fn(SymbolUri) -> Fut + Send + Clone + 'static,
    ) -> impl Future<Output = Result<Option<SharedDeclaration<Symbol>>, Error>> {
        let dones = std::sync::Arc::new(parking_lot::Mutex::new(Vec::new()));
        get_async(self, dones, name, get_struct)
    }
}

fn get(
    term: &Term,
    dones: &mut Vec<SymbolUri>,
    name: &UriName,
    get_struct: impl Fn(
        &SymbolUri,
    ) -> Option<
        Either<SharedDeclaration<MathStructure>, SharedDeclaration<StructureExtension>>,
    >,
) -> Option<SharedDeclaration<Symbol>> {
    match term {
        Term::Symbol { uri, .. } => from_structure(uri, dones, name, get_struct),
        Term::Application(app)
            if matches!(&app.head,Term::Symbol{uri,..} if *uri == *metatheory::RECORD_TYPE_MERGE)
                && matches!(&*app.arguments, [Argument::Sequence(_)]) =>
        {
            let Term::Application(app) = term else {
                // SAFETY: pattern match above
                unsafe { unreachable_unchecked() }
            };
            let Some(Argument::Sequence(s)) = app.arguments.first() else {
                // SAFETY: pattern match above
                unsafe { unreachable_unchecked() }
            };
            match s {
                MaybeSequence::One(t) => get(t, dones, name, get_struct),
                MaybeSequence::Seq(s) => s
                    .iter()
                    .rev()
                    .find_map(|s| get(s, dones, name, &get_struct)),
            }
        }
        _ => None,
    }
}

#[allow(clippy::type_complexity)]
fn get_async<
    Error: Send + 'static,
    Fut: Future<
        Output = Result<
            Either<SharedDeclaration<MathStructure>, SharedDeclaration<StructureExtension>>,
            Error,
        >,
    >,
>(
    term: Term,
    dones: std::sync::Arc<parking_lot::Mutex<Vec<SymbolUri>>>,
    name: UriName,
    get_struct: impl Fn(SymbolUri) -> Fut + Send + Clone + 'static,
) -> Pin<Box<dyn Future<Output = Result<Option<SharedDeclaration<Symbol>>, Error>>>> {
    match term {
        Term::Symbol { uri, .. } => from_structure_async(uri, dones, name, get_struct),
        Term::Application(app)
            if matches!(&app.head,Term::Symbol{uri,..} if *uri == *metatheory::RECORD_TYPE_MERGE)
                && matches!(&*app.arguments, [Argument::Sequence(_)]) =>
        {
            let mut arguments = app.arguments.iter();
            let Some(Argument::Sequence(s)) = arguments.next() else {
                // SAFETY: pattern match above
                unsafe { unreachable_unchecked() }
            };
            match s {
                MaybeSequence::One(t) => get_async(t.clone(), dones, name, get_struct),
                MaybeSequence::Seq(s) => from_terms_async(s.clone(), dones, name, get_struct),
            }
        }
        _ => Box::pin(std::future::ready(Ok(None))),
    }
}

#[allow(clippy::type_complexity)]
fn from_terms_async<
    Error: Send + 'static,
    Fut: Future<
        Output = Result<
            Either<SharedDeclaration<MathStructure>, SharedDeclaration<StructureExtension>>,
            Error,
        >,
    >,
>(
    terms: Box<[Term]>,
    dones: std::sync::Arc<parking_lot::Mutex<Vec<SymbolUri>>>,
    name: UriName,
    get_struct: impl Fn(SymbolUri) -> Fut + Send + Clone + 'static,
) -> Pin<Box<dyn Future<Output = Result<Option<SharedDeclaration<Symbol>>, Error>>>> {
    Box::pin(async move {
        for term in terms.into_iter().rev() {
            if let Some(r) =
                get_async(term, dones.clone(), name.clone(), get_struct.clone()).await?
            {
                return Ok(Some(r));
            }
        }
        Ok(None)
    })
}

fn from_structure(
    s: &SymbolUri,
    dones: &mut Vec<SymbolUri>,
    name: &UriName,
    get_struct: impl Fn(
        &SymbolUri,
    ) -> Option<
        Either<SharedDeclaration<MathStructure>, SharedDeclaration<StructureExtension>>,
    >,
) -> Option<SharedDeclaration<Symbol>> {
    if dones.contains(s) {
        return None;
    }
    dones.push(s.clone());
    let elems = match get_struct(s)? {
        Either::Left(s) => s.0.inherit_infallibly(|s| &s.elements),
        Either::Right(s) => s.0.inherit_infallibly(|s| &s.elements),
    };

    for e in elems.iter().rev() {
        match e {
            StructureDeclaration::Symbol(s) if s.uri.name == *name => {
                return Some(SharedDeclaration(elems.inherit_infallibly(|elems| {
                    // SAFETY: we already found it above
                    unsafe {
                        elems
                            .iter()
                            .rev()
                            .find_map(|s| {
                                if let StructureDeclaration::Symbol(s) = s
                                    && s.uri.name == *name
                                {
                                    Some(s)
                                } else {
                                    None
                                }
                            })
                            .unwrap_unchecked()
                    }
                })));
            }
            StructureDeclaration::Import { uri, .. } => {
                if let Some(s) = uri.clone().into_symbol()
                    && let Some(r) = from_structure(&s, dones, name, &get_struct)
                {
                    return Some(r);
                }
            }
            StructureDeclaration::Symbol(_)
            | StructureDeclaration::Morphism(_)
            | StructureDeclaration::Rule { .. } => (), // TODO
        }
    }
    None
}

#[allow(clippy::type_complexity)]
fn from_structure_async<
    Error: Send,
    Fut: Future<
        Output = Result<
            Either<SharedDeclaration<MathStructure>, SharedDeclaration<StructureExtension>>,
            Error,
        >,
    >,
>(
    s: SymbolUri,
    dones: std::sync::Arc<parking_lot::Mutex<Vec<SymbolUri>>>,
    name: UriName,
    get_struct: impl Fn(SymbolUri) -> Fut + Send + Clone + 'static,
) -> Pin<Box<dyn Future<Output = Result<Option<SharedDeclaration<Symbol>>, Error>>>> {
    Box::pin(async move {
        {
            let mut d = dones.lock();
            if d.contains(&s) {
                return Ok(None);
            }
            d.push(s.clone());
        }
        let elems = match get_struct(s).await? {
            Either::Left(s) => s.0.inherit_infallibly(|s| &s.elements),
            Either::Right(s) => s.0.inherit_infallibly(|s| &s.elements),
        };

        for e in elems.iter().rev() {
            match e {
                StructureDeclaration::Symbol(s) if s.uri.name == name => {
                    return Ok(Some(SharedDeclaration(elems.inherit_infallibly(|elems| {
                        // SAFETY: we already found it above
                        unsafe {
                            elems
                                .iter()
                                .rev()
                                .find_map(|s| {
                                    if let StructureDeclaration::Symbol(s) = s
                                        && s.uri.name == name
                                    {
                                        Some(s)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_unchecked()
                        }
                    }))));
                }
                StructureDeclaration::Import { uri, .. } => {
                    if let Some(s) = uri.clone().into_symbol()
                        && let Some(r) =
                            from_structure_async(s, dones.clone(), name.clone(), get_struct.clone())
                                .await?
                    {
                        return Ok(Some(r));
                    }
                }
                StructureDeclaration::Symbol(_)
                | StructureDeclaration::Morphism(_)
                | StructureDeclaration::Rule { .. } => (), // TODO
            }
        }
        Ok(None)
    })
}

use std::hint::unreachable_unchecked;

use crate::{
    ClonableView, DocumentState, FtmlViews,
    document::CurrentUri,
    notations::NotationExt,
    terms::{ReactiveTerm, TopTerm},
    utils::{
        FutureExt,
        local_cache::{LocalCache, SendBackend, WithLocalCache},
        owned,
    },
};
use ftml_core::extraction::VarOrSym;
use ftml_ontology::{
    narrative::elements::Notation,
    terms::{Argument, BoundArgument, Term, Variable, opaque::Opaque},
};
use ftml_uris::{DocumentElementUri, Id, LeafUri, NamedUri, SymbolUri};
use leptos::{
    either::Either,
    math::{mi, mo},
};
use leptos::{math::mtext, prelude::*};

pub trait TermExt {
    fn into_view<Views: FtmlViews, Be: SendBackend>(self, in_term: bool) -> AnyView;
    fn into_view_safe<Views: FtmlViews, Be: SendBackend>(self) -> impl IntoView
    where
        Self: Sized,
    {
        owned(move || {
            provide_context(None::<TopTerm>);
            provide_context(None::<ReactiveTerm>);
            self.into_view::<Views, Be>(false)
        })
    }
}

macro_rules! maybe_comp {
    ($e:expr) => {
        if with_context::<CurrentUri, _>(|_| ()).is_some() {
            leptos::either::Either::Left(Views::comp(false, ClonableView::new(true, move || $e)))
        } else {
            leptos::either::Either::Right($e)
        }
    };
}

fn no_notation<Views: FtmlViews>(
    name: &str,
    arguments: Vec<Either<ClonableView, Vec<ClonableView>>>,
) -> impl IntoView + use<Views> {
    fn do_view<Views: FtmlViews>(v: Either<ClonableView, Vec<ClonableView>>) -> impl IntoView {
        v.map(ClonableView::into_view::<Views>, |v| {
            let mut args = v.into_iter();
            view! {
                {maybe_comp!(mo().child('('))}
                {args.next().map(ClonableView::into_view::<Views>)}
                {args.map(|v| view!{
                    {maybe_comp!(mo().child(','))}
                    {v.into_view::<Views>()}
                }).collect_view()}
                {maybe_comp!(mo().child(')'))}
            }
        })
    }
    if arguments.is_empty() {
        return Either::Left(mtext().style("color:red").child(name.to_string()));
    }
    let mut args = arguments.into_iter();
    Either::Right(view! {<mrow>
        {mtext().style("color:red").child(name.to_string())}
        {maybe_comp!(mo().child('('))}
        {args.next().map(do_view::<Views>)}
        {args.map(|v| view!{
            {maybe_comp!(mo().child(','))}
            {do_view::<Views>(v)}
        }).collect_view()}
        {maybe_comp!(mo().child(')'))}
    </mrow>})
}

impl TermExt for Term {
    #[allow(clippy::too_many_lines)]
    fn into_view<Views: FtmlViews, Be: SendBackend>(self, in_term: bool) -> AnyView {
        use leptos::either::Either::{Left, Right};
        tracing::trace!("Presenting {self:?}");
        //owned(move || {
        match self {
            Self::Symbol(s) => sym::<Views, Be>(s, in_term).into_any(),
            Self::Var(Variable::Ref {
                declaration,
                is_sequence,
            }) => var_ref::<Views, Be>(declaration, is_sequence, in_term).into_any(),
            Self::Var(Variable::Name { name, notated }) => {
                var_name::<Views>(name, notated, in_term).into_any()
            }
            Self::Application { head, arguments }
                if matches!(*head, Self::Symbol(_) | Self::Var(Variable::Ref { .. })) =>
            {
                let arguments = do_args::<Views, Be>(arguments);
                // SAFETY: pattern match above
                let (leaf, vos) = unsafe { do_head(*head) };
                DocumentState::with_head(vos.clone(), move || {
                    if with_context::<CurrentUri, _>(|_| ()).is_some() {
                        Left(Views::application(
                            vos.clone(),
                            None,
                            None,
                            do_application_inner::<Views, Be>(leaf, vos, arguments),
                        ))
                    } else {
                        Right(
                            do_application_inner::<Views, Be>(leaf, vos, arguments)
                                .into_view::<Views>(),
                        )
                    }
                })
                .into_any()
            }
            Self::Bound {
                head,
                arguments,
                body,
            } if matches!(*head, Self::Symbol(_) | Self::Var(Variable::Ref { .. })) => {
                // SAFETY: pattern match above
                let (leaf, vos) = unsafe { do_head(*head) };
                let mut arguments = do_bound_args::<Views, Be>(arguments);
                let body = *body;
                arguments.push(Either::Left(ClonableView::new(true, move || {
                    body.clone().into_view::<Views, Be>(true)
                })));
                DocumentState::with_head(vos.clone(), move || {
                    if with_context::<CurrentUri, _>(|_| ()).is_some() {
                        Left(Views::binder_application(
                            vos.clone(),
                            None,
                            None,
                            do_application_inner::<Views, Be>(leaf, vos, arguments),
                        ))
                    } else {
                        Right(
                            do_application_inner::<Views, Be>(leaf, vos, arguments)
                                .into_view::<Views>(),
                        )
                    }
                })
                .into_any()
            }
            Self::Opaque {
                tag,
                attributes,
                children,
                terms,
            } => {
                let mut terms = terms
                    .into_iter()
                    .map(|t| Some(move || t.into_view::<Views, Be>(true)))
                    .collect::<Vec<_>>();
                do_opaque(&tag, attributes, children, &mut terms)
            }

            Self::Application { head, arguments }
                if matches!(
                    &*head,
                    Self::Field {
                        record,
                        key,
                        record_type: Some(_)
                    }
                ) =>
            {
                // let arguments = do_args::<Views, Be>(arguments);
                let Self::Field {
                    record,
                    key,
                    record_type: Some(tp),
                } = *head
                else {
                    // SAFETY: pattern match above
                    unsafe { unreachable_unchecked() }
                };
                let tp = *tp;
                FutureExt::into_view(
                    move || {
                        tp.clone().get_in_record_type_async(key.clone(), |uri| {
                            WithLocalCache::<Be>::default().get_structure(uri)
                        })
                    },
                    |r| match r {
                        Err(e) => Right(e.to_string()),
                        Ok(None) => Right("(Structure not found)".to_string()),
                        Ok(Some(r)) => Left(
                            Self::Application {
                                head: Box::new(Self::Symbol(r.uri.clone())),
                                arguments,
                            }
                            .into_view::<Views, Be>(true),
                        ),
                    },
                )
                .into_any()
            }
            t => mtext().child(format!("{t:?}")).into_any(),
        }
        //})

        //
    }
}

fn sym<Views: FtmlViews, Be: SendBackend>(uri: SymbolUri, in_term: bool) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    DocumentState::with_head(VarOrSym::S(uri.clone()), move || {
        if with_context::<CurrentUri, _>(|_| ()).is_some() {
            Left(Views::symbol_reference(
                uri.clone(),
                None,
                in_term,
                ClonableView::new(true, move || {
                    let uri = uri.clone();
                    with_notations::<Be, _, _>(uri.clone().into(), move |t| {
                        if let Some(n) = t {
                            if let Some(n) = n.op {
                                Left(Left(super::view_node(&n)))
                            } else {
                                Left(Right(n.as_view::<Views>(&VarOrSym::S(uri))))
                            }
                        } else {
                            Right(
                                mtext()
                                    .style("color:red")
                                    .child(uri.name().last().to_string()),
                            )
                        }
                    })
                }),
            ))
        } else {
            Right(with_notations::<Be, _, _>(uri.clone().into(), move |t| {
                if let Some(n) = t {
                    if let Some(n) = n.op {
                        Left(Left(Views::comp(
                            false,
                            ClonableView::new(true, move || super::view_node(&n)),
                        )))
                    } else {
                        Left(Right(n.as_view::<Views>(&VarOrSym::S(uri))))
                    }
                } else {
                    Right(
                        mtext()
                            .style("color:red")
                            .child(uri.name().last().to_string()),
                    )
                }
            }))
        }
    })
}

fn var_ref<Views: FtmlViews, Be: SendBackend>(
    uri: DocumentElementUri,
    is_sequence: Option<bool>,
    in_term: bool,
) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    DocumentState::with_head(
        VarOrSym::V(Variable::Ref {
            declaration: uri.clone(),
            is_sequence,
        }),
        move || {
            if with_context::<CurrentUri, _>(|_| ()).is_some() {
                Left(Views::variable_reference(
                    Variable::Ref {
                        declaration: uri.clone(),
                        is_sequence,
                    },
                    None,
                    in_term,
                    ClonableView::new(true, move || {
                        let uri = uri.clone();
                        with_notations::<Be, _, _>(uri.clone().into(), move |t| {
                            if let Some(n) = t {
                                if let Some(n) = n.op {
                                    Left(Left(Views::comp(
                                        false,
                                        ClonableView::new(true, move || super::view_node(&n)),
                                    )))
                                } else {
                                    Left(Right(n.as_view::<Views>(&VarOrSym::V(Variable::Ref {
                                        declaration: uri,
                                        is_sequence,
                                    }))))
                                }
                            } else {
                                Right(
                                    mtext()
                                        .style("color:red")
                                        .child(uri.name().last().to_string()),
                                )
                            }
                        })
                    }),
                ))
            } else {
                Right(with_notations::<Be, _, _>(uri.clone().into(), move |t| {
                    if let Some(n) = t {
                        if let Some(n) = n.op {
                            Left(Left(super::view_node(&n)))
                        } else {
                            Left(Right(n.as_view::<Views>(&VarOrSym::V(Variable::Ref {
                                declaration: uri,
                                is_sequence,
                            }))))
                        }
                    } else {
                        Right(
                            mtext()
                                .style("color:red")
                                .child(uri.name().last().to_string()),
                        )
                    }
                }))
            }
        },
    )
}

fn var_name<Views: FtmlViews>(name: Id, notated: Option<Id>, in_term: bool) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    let not = notated
        .as_ref()
        .map_or_else(|| name.to_string(), Id::to_string);
    DocumentState::with_head(
        VarOrSym::V(Variable::Name {
            name: name.clone(),
            notated: notated.clone(),
        }),
        move || {
            if with_context::<CurrentUri, _>(|_| ()).is_some() {
                Left(Views::variable_reference(
                    Variable::Name { name, notated },
                    None,
                    in_term,
                    ClonableView::new(true, move || mi().child(not.clone())),
                ))
            } else {
                Right(mi().child(not))
            }
        },
    )
}

fn do_application_inner<Views: FtmlViews, Be: SendBackend>(
    leaf: LeafUri,
    vos: VarOrSym,
    arguments: Vec<Either<ClonableView, Vec<ClonableView>>>,
) -> ClonableView {
    use leptos::either::Either::{Left, Right};
    ClonableView::new(true, move || {
        let leaf = leaf.clone();
        let vos = vos.clone();
        let arguments = arguments.clone();
        with_notations::<Be, _, _>(leaf.clone(), move |t| {
            if let Some(n) = t {
                Left(n.with_arguments::<Views, _>(&vos, &arguments))
            } else {
                Right(no_notation::<Views>(leaf.name().last(), arguments))
            }
        })
    })
}

fn do_args<Views: FtmlViews, Be: SendBackend>(
    arguments: Box<[Argument]>,
) -> Vec<Either<ClonableView, Vec<ClonableView>>> {
    arguments
        .into_iter()
        .map(|a| match a {
            Argument::Simple(t) | Argument::Sequence(either::Left(t)) => {
                Either::Left(ClonableView::new(true, move || {
                    t.clone().into_view::<Views, Be>(true)
                }))
            }
            Argument::Sequence(either::Right(s)) => Either::Right(
                s.into_iter()
                    .map(|t| {
                        ClonableView::new(true, move || t.clone().into_view::<Views, Be>(true))
                    })
                    .collect::<Vec<_>>(),
            ),
        })
        .collect::<Vec<_>>()
}

fn do_opaque(
    tag: &Id,
    attributes: Box<[(Id, Box<str>)]>,
    children: Box<[Opaque]>,
    terms: &mut Vec<Option<impl FnOnce() -> AnyView>>,
) -> AnyView {
    use leptos::either::EitherOf4::{A, B, C, D};
    let i = super::tachys_from_tag(
        tag.as_ref(),
        children
            .into_iter()
            .map(|e| match e {
                Opaque::Node {
                    tag,
                    attributes,
                    children,
                } => A(do_opaque(&tag, attributes, children, terms)),
                Opaque::Text(t) => B(t.into_string()),
                Opaque::Term(i) => {
                    let f = terms.get_mut(i as usize).and_then(Option::take);
                    f.map_or_else(|| C(mtext().child("ERROR")), |f| D(f()))
                }
            })
            .collect_view(),
    );
    attributes.into_iter().fold(i, |i, (k, v)| {
        i.attr(k.as_ref().to_string(), v.into_string()).into_any()
    })
}

fn do_bound_args<Views: FtmlViews, Be: SendBackend>(
    arguments: Box<[BoundArgument]>,
) -> Vec<Either<ClonableView, Vec<ClonableView>>> {
    use leptos::either::Either::{Left, Right};
    arguments
        .into_iter()
        .map(|a| match a {
            BoundArgument::Simple(t) | BoundArgument::Sequence(either::Left(t)) => {
                Either::Left(ClonableView::new(true, move || {
                    t.clone().into_view::<Views, Be>(true)
                }))
            }
            BoundArgument::Sequence(either::Right(s)) => Either::Right(
                s.into_iter()
                    .map(|t| {
                        ClonableView::new(true, move || t.clone().into_view::<Views, Be>(true))
                    })
                    .collect::<Vec<_>>(),
            ),
            BoundArgument::Bound(Variable::Ref {
                declaration,
                is_sequence,
            })
            | BoundArgument::BoundSeq(either::Left(Variable::Ref {
                declaration,
                is_sequence,
            })) => Either::Left(ClonableView::new(true, move || {
                let declaration = declaration.clone();
                with_notations::<Be, _, _>(declaration.clone().into(), move |t| {
                    if let Some(n) = t {
                        Left(n.as_view::<Views>(&VarOrSym::V(Variable::Ref {
                            declaration,
                            is_sequence,
                        })))
                    } else {
                        Right(mtext().child(format!("TODO: No notation for {declaration}")))
                    }
                })
            })),
            BoundArgument::BoundSeq(either::Right(v)) => Either::Right(
                v.into_iter()
                    .map(|v| {
                        ClonableView::new(true, move || {
                            if let Variable::Ref {
                                declaration,
                                is_sequence,
                            } = &v
                            {
                                let declaration = declaration.clone();
                                let is_sequence = *is_sequence;
                                Left(with_notations::<Be, _, _>(
                                    declaration.clone().into(),
                                    move |t| {
                                        if let Some(n) = t {
                                            Left(n.as_view::<Views>(&VarOrSym::V(Variable::Ref {
                                                declaration,
                                                is_sequence,
                                            })))
                                        } else {
                                            Right(mtext().child(format!(
                                                "TODO: No notation for {declaration}"
                                            )))
                                        }
                                    },
                                ))
                            } else {
                                Right(mtext().child("TODO: unresolved variable"))
                            }
                        })
                    })
                    .collect(),
            ),
            t => Either::Left(ClonableView::new(true, move || {
                mtext().child(format!("{t:?}")).into_any()
            })),
        })
        .collect::<Vec<_>>()
}

fn with_notations<
    Be: SendBackend,
    V: IntoView + 'static,
    F: FnOnce(Option<Notation>) -> V + Send + Clone + 'static,
>(
    uri: LeafUri,
    then: F,
) -> impl IntoView + use<Be, V, F> {
    use crate::utils::FutureExt;
    FutureExt::into_view(
        move || WithLocalCache::<Be>::default().get_notations(uri.clone()),
        move |gl| {
            let not = gl
                .local
                .and_then(|v| v.first().cloned().map(|p| p.1))
                .or_else(|| {
                    gl.global
                        .and_then(|r| r.ok().and_then(|v| v.first().cloned().map(|p| p.1)))
                });
            then(not)
        },
    )
    /*
    view! {
        <Suspense fallback = || "…">{move || {
            let uri = uri.clone();
            let then = then.clone();
            Suspend::new(async move {
                let mut gl = WithLocalCache::<Be>::default().get_notations(uri).await;
                let not = gl.local.and_then(|v| v.first().cloned().map(|p| p.1))
                    .or_else(|| gl.global.and_then(|r| r.ok().and_then(|v| v.first().cloned().map(|p| p.1))));
                then(not)
            })
        }}
        </Suspense>
    }
    */
}

// SAFETY: requires head be Sym or Var::Ref
unsafe fn do_head(head: Term) -> (LeafUri, VarOrSym) {
    match head {
        Term::Symbol(s) => (s.clone().into(), VarOrSym::S(s)),
        Term::Var(Variable::Ref {
            declaration,
            is_sequence,
        }) => (
            declaration.clone().into(),
            VarOrSym::V(Variable::Ref {
                declaration,
                is_sequence,
            }),
        ),
        _ => unsafe { unreachable_unchecked() },
    }
}

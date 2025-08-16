use std::hint::unreachable_unchecked;

use crate::{
    ClonableView, DocumentState, FtmlViews,
    document::CurrentUri,
    notations::NotationExt,
    terms::{ReactiveTerm, TopTerm},
    utils::{
        FutureExt,
        local_cache::{SendBackend, WithLocalCache},
        owned,
    },
};
use ftml_ontology::{
    narrative::elements::Notation,
    terms::{Argument, BoundArgument, Term, VarOrSym, Variable, opaque::Opaque},
};
use ftml_uris::{DocumentElementUri, Id, LeafUri, NamedUri, SymbolUri};
use leptos::{
    either::Either,
    math::{mi, mo},
    tachys::view::any_view::AnyViewWithAttrs,
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
            Self::Symbol {
                uri,
                presentation: None,
            } => sym::<Views, Be>(uri, None, in_term).into_any(),
            Self::Var {
                variable:
                    Variable::Ref {
                        declaration,
                        is_sequence,
                    },
                presentation: None,
            } => var_ref::<Views, Be>(declaration, is_sequence, None, in_term).into_any(),
            Self::Var {
                variable: Variable::Name { name, notated },
                ..
            } => var_name::<Views>(name, notated, None, in_term).into_any(),
            Self::Application {
                head,
                arguments,
                presentation: None,
            } if matches!(
                *head,
                Self::Symbol { .. }
                    | Self::Var {
                        variable: Variable::Ref { .. },
                        ..
                    }
            ) =>
            {
                // SAFETY: pattern match above
                let (leaf, vos) = unsafe { do_head(*head) };
                application::<Views, Be>(vos, leaf, None, arguments).into_any()
            }
            Self::Application {
                head,
                arguments,
                presentation: Some(pres),
            } => {
                let head = match *head {
                    Self::Field { record, .. } => *record,
                    t => t,
                };
                let head =
                    ClonableView::new(true, move || head.clone().into_view::<Views, Be>(true));
                let uri = match &pres {
                    VarOrSym::Sym(s) => s.clone().into(),
                    VarOrSym::Var(Variable::Ref { declaration, .. }) => declaration.clone().into(),
                    VarOrSym::Var(Variable::Name { .. }) => {
                        return "TODO: unresolved variable".into_any();
                    }
                };
                application::<Views, Be>(pres, uri, Some(head), arguments).into_any()
            }
            Self::Bound {
                head,
                arguments,
                body,
                presentation: None,
            } if matches!(
                *head,
                Self::Symbol { .. }
                    | Self::Var {
                        variable: Variable::Ref { .. },
                        ..
                    }
            ) =>
            {
                // SAFETY: pattern match above
                let (leaf, vos) = unsafe { do_head(*head) };
                bound::<Views, Be>(vos, leaf, *body, None, arguments).into_any()
            }
            Self::Bound {
                head,
                arguments,
                body,
                presentation: Some(pres),
            } => {
                let head = match *head {
                    Self::Field { record, .. } => *record,
                    t => t,
                };
                let head =
                    ClonableView::new(true, move || head.clone().into_view::<Views, Be>(true));
                let uri = match &pres {
                    VarOrSym::Sym(s) => s.clone().into(),
                    VarOrSym::Var(Variable::Ref { declaration, .. }) => declaration.clone().into(),
                    VarOrSym::Var(Variable::Name { .. }) => {
                        return "TODO: unresolved variable".into_any();
                    }
                };
                bound::<Views, Be>(pres, uri, *body, Some(head), arguments).into_any()
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
                do_opaque(&tag, attributes, children, &mut terms).into_any()
            }
            Self::Field {
                record,
                presentation: Some(pres),
                ..
            } => {
                let record =
                    ClonableView::new(true, move || record.clone().into_view::<Views, Be>(true));
                match pres {
                    VarOrSym::Sym(uri) => sym::<Views, Be>(uri, Some(record), in_term).into_any(),
                    VarOrSym::Var(Variable::Ref {
                        declaration,
                        is_sequence,
                    }) => var_ref::<Views, Be>(declaration, is_sequence, Some(record), in_term)
                        .into_any(),
                    VarOrSym::Var(Variable::Name { name, notated }) => {
                        var_name::<Views>(name, notated, Some(record), in_term).into_any()
                    }
                }
            }

            Self::Application {
                head,
                arguments,
                presentation: None,
            } if matches!(
                &*head,
                Self::Field {
                    record_type: Some(_),
                    presentation: None,
                    ..
                }
            ) =>
            {
                // let arguments = do_args::<Views, Be>(arguments);
                let Self::Field {
                    record,
                    key,
                    record_type: Some(tp),
                    ..
                } = *head
                else {
                    // SAFETY: pattern match above
                    unsafe { unreachable_unchecked() }
                };
                let tp = *tp;
                let record =
                    ClonableView::new(true, move || record.clone().into_view::<Views, Be>(true));
                FutureExt::into_view(
                    move || {
                        tp.clone().get_in_record_type_async(key.clone(), |uri| {
                            WithLocalCache::<Be>::default().get_structure(uri)
                        })
                    },
                    |r| match r {
                        Err(e) => Right(e.to_string()),
                        Ok(None) => Right("(Structure not found)".to_string()),
                        Ok(Some(r)) => Left(application::<Views, Be>(
                            VarOrSym::Sym(r.uri.clone()),
                            r.uri.clone().into(),
                            Some(record),
                            arguments,
                        )),
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

fn application<Views: FtmlViews, Be: SendBackend>(
    head: VarOrSym,
    uri: LeafUri,
    real_term: Option<ClonableView>,
    arguments: Box<[Argument]>,
) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    let arguments = do_args::<Views, Be>(arguments);
    DocumentState::with_head(head.clone(), move || {
        if with_context::<CurrentUri, _>(|_| ()).is_some() {
            Left(Views::application(
                head.clone(),
                None,
                None,
                do_application_inner::<Views, Be>(uri, head, real_term, arguments),
            ))
        } else {
            Right(
                do_application_inner::<Views, Be>(uri, head, real_term, arguments)
                    .into_view::<Views>(),
            )
        }
    })
}

fn bound<Views: FtmlViews, Be: SendBackend>(
    head: VarOrSym,
    uri: LeafUri,
    body: Term,
    real_term: Option<ClonableView>,
    arguments: Box<[BoundArgument]>,
) -> impl IntoView {
    use leptos::either::Either::{Left, Right};

    let mut arguments = do_bound_args::<Views, Be>(arguments);
    arguments.push(Either::Left(ClonableView::new(true, move || {
        body.clone().into_view::<Views, Be>(true)
    })));
    DocumentState::with_head(head.clone(), move || {
        if with_context::<CurrentUri, _>(|_| ()).is_some() {
            Left(Views::binder_application(
                head.clone(),
                None,
                None,
                do_application_inner::<Views, Be>(uri, head, real_term, arguments),
            ))
        } else {
            Right(
                do_application_inner::<Views, Be>(uri, head, real_term, arguments)
                    .into_view::<Views>(),
            )
        }
    })
}

fn sym<Views: FtmlViews, Be: SendBackend>(
    uri: SymbolUri,
    this: Option<ClonableView>,
    in_term: bool,
) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    DocumentState::with_head(VarOrSym::Sym(uri.clone()), move || {
        if with_context::<CurrentUri, _>(|_| ()).is_some() {
            Left(Views::symbol_reference(
                uri.clone(),
                None,
                in_term,
                ClonableView::new(true, move || {
                    let uri = uri.clone();
                    let this = this.clone();
                    with_notations::<Be, _, _>(uri.clone().into(), move |t| {
                        if let Some(n) = t {
                            if let Some(n) = n.op {
                                Left(Left(super::view_node(&n)))
                            } else {
                                Left(Right(
                                    n.as_view::<Views>(&VarOrSym::Sym(uri), this.as_ref()),
                                ))
                            }
                        } else {
                            let name = uri.name;
                            Right(Views::comp(
                                false,
                                ClonableView::new(true, move || {
                                    mtext().style("color:red").child(name.last().to_string())
                                }),
                            ))
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
                        Left(Right(
                            n.as_view::<Views>(&VarOrSym::Sym(uri), this.as_ref()),
                        ))
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
    this: Option<ClonableView>,
    in_term: bool,
) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    DocumentState::with_head(
        VarOrSym::Var(Variable::Ref {
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
                        let this = this.clone();
                        with_notations::<Be, _, _>(uri.clone().into(), move |t| {
                            if let Some(n) = t {
                                if let Some(n) = n.op {
                                    Left(Left(Views::comp(
                                        false,
                                        ClonableView::new(true, move || super::view_node(&n)),
                                    )))
                                } else {
                                    Left(Right(n.as_view::<Views>(
                                        &VarOrSym::Var(Variable::Ref {
                                            declaration: uri,
                                            is_sequence,
                                        }),
                                        this.as_ref(),
                                    )))
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
                            Left(Right(n.as_view::<Views>(
                                &VarOrSym::Var(Variable::Ref {
                                    declaration: uri,
                                    is_sequence,
                                }),
                                this.as_ref(),
                            )))
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

fn var_name<Views: FtmlViews>(
    name: Id,
    notated: Option<Id>,
    this: Option<ClonableView>,
    in_term: bool,
) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    let not = notated
        .as_ref()
        .map_or_else(|| name.to_string(), Id::to_string);
    DocumentState::with_head(
        VarOrSym::Var(Variable::Name {
            name: name.clone(),
            notated: notated.clone(),
        }),
        move || {
            let outer = if with_context::<CurrentUri, _>(|_| ()).is_some() {
                let inner = ClonableView::new(true, move || mi().child(not.clone()));
                Left(Views::variable_reference(
                    Variable::Name { name, notated },
                    None,
                    in_term,
                    ClonableView::new(true, move || Views::comp(false, inner.clone())),
                ))
            } else {
                Right(mi().child(not))
            };
            if let Some(this) = this {
                Left(
                    leptos::math::msub()
                        .child(outer)
                        .child(this.into_view::<Views>()),
                )
            } else {
                Right(outer)
            }
        },
    )
}

fn do_application_inner<Views: FtmlViews, Be: SendBackend>(
    leaf: LeafUri,
    vos: VarOrSym,
    this: Option<ClonableView>,
    arguments: Vec<Either<ClonableView, Vec<ClonableView>>>,
) -> ClonableView {
    use leptos::either::Either::{Left, Right};
    ClonableView::new(true, move || {
        let leaf = leaf.clone();
        let vos = vos.clone();
        let arguments = arguments.clone();
        let this = this.clone();
        with_notations::<Be, _, _>(leaf.clone(), move |t| {
            if let Some(n) = t {
                Left(n.with_arguments::<Views, _>(&vos, this.as_ref(), &arguments))
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
) -> leptos::either::Either<AnyView, AnyViewWithAttrs> {
    use leptos::either::{
        Either::Left,
        EitherOf4::{A, B, C, D},
    };
    let make_red = !children.iter().any(|e| matches!(e, Opaque::Term(_)));
    let i = super::html_from_tag(
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
    let r = attributes.into_iter().fold(Left(i), |i, (k, v)| {
        super::attr(i, k.as_ref().to_string(), v.into_string())
    });
    if make_red {
        super::attr(r, "style", "color:red")
    } else {
        r
    }
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
                        Left(n.as_view::<Views>(
                            &VarOrSym::Var(Variable::Ref {
                                declaration,
                                is_sequence,
                            }),
                            None,
                        ))
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
                                            Left(n.as_view::<Views>(
                                                &VarOrSym::Var(Variable::Ref {
                                                    declaration,
                                                    is_sequence,
                                                }),
                                                None,
                                            ))
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
}

// SAFETY: requires head be Sym or Var::Ref
unsafe fn do_head(head: Term) -> (LeafUri, VarOrSym) {
    match head {
        Term::Symbol { uri, .. } => (uri.clone().into(), VarOrSym::Sym(uri)),
        Term::Var {
            variable:
                Variable::Ref {
                    declaration,
                    is_sequence,
                },
            ..
        } => (
            declaration.clone().into(),
            VarOrSym::Var(Variable::Ref {
                declaration,
                is_sequence,
            }),
        ),
        _ => unsafe { unreachable_unchecked() },
    }
}

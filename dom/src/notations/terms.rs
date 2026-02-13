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
    terms::{
        ApplicationTerm, Argument, BindingTerm, BoundArgument, ComponentVar, MaybeSequence,
        Numeric, Term, VarOrSym, Variable,
        opaque::{AnyOpaque, OpaqueNode},
    },
};
use ftml_parser::FtmlKey;
use ftml_uris::{
    DocumentElementUri, FtmlUri, Id, IsDomainUri, IsNarrativeUri, LeafUri, NamedUri, SymbolUri,
    UriRef, UriWithArchive, UriWithPath,
};
use leptos::{
    either::Either,
    math::{mi, mn, mo},
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
            leptos::either::Either::Left(Views::comp(ClonableView::new(true, move || $e)))
        } else {
            leptos::either::Either::Right($e)
        }
    };
}

fn no_notation<Views: FtmlViews>(
    name: &str,
    uri: &LeafUri,
    arguments: Vec<Either<ClonableView, Vec<ClonableView>>>,
) -> AnyView {
    fn do_view<Views: FtmlViews>(v: Either<ClonableView, Vec<ClonableView>>) -> AnyView {
        match v {
            Either::Left(v) => v.into_view::<Views>(),
            Either::Right(v) => {
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
                .into_any()
            }
        }
    }
    let kind = match uri {
        LeafUri::Element(_) => "OMV",
        LeafUri::Symbol(_) => "OMID",
    };
    if arguments.is_empty() {
        return mtext()
            .style("color:red")
            .child(name.to_string())
            .attr(FtmlKey::Head.attr_name(), uri.to_string())
            .attr(FtmlKey::Term.attr_name(), kind)
            .attr(FtmlKey::Comp.attr_name(), "")
            .into_any();
    }
    let mut args = arguments.into_iter();
    view! {<mrow>
        {mtext().style("color:red").child(name.to_string())
            .attr(FtmlKey::Head.attr_name(), uri.to_string())
            .attr(FtmlKey::Term.attr_name(), kind)
            .attr(FtmlKey::Comp.attr_name(), "")
        }
        {maybe_comp!(mo().child('('))}
        {args.next().map(do_view::<Views>)}
        {args.map(|v| view!{
            {maybe_comp!(mo().child(','))}
            {do_view::<Views>(v)}
        }).collect_view()}
        {maybe_comp!(mo().child(')'))}
    </mrow>}
    .into_any()
}

impl TermExt for Term {
    #[allow(clippy::too_many_lines)]
    fn into_view<Views: FtmlViews, Be: SendBackend>(self, in_term: bool) -> AnyView {
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
            Self::Application(app)
                if app.presentation.is_none()
                    && matches!(
                        app.head,
                        Self::Symbol { .. }
                            | Self::Var {
                                variable: Variable::Ref { .. },
                                ..
                            }
                    ) =>
            {
                // SAFETY: pattern match above
                let (leaf, vos) = unsafe { do_head(app.head.clone()) };
                application::<Views, Be>(vos, leaf, None, app).into_any()
            }
            Self::Application(app) if app.presentation.is_some() => {
                let head = match &app.head {
                    Self::Field(f) => f.record.clone(),
                    t => t.clone(),
                };
                let head =
                    ClonableView::new(true, move || head.clone().into_view::<Views, Be>(true));
                // SAFETY: app.presentation.is_some()
                let pres = unsafe { app.presentation.as_ref().unwrap_unchecked() };
                let uri = match pres {
                    VarOrSym::Sym(s) => s.clone().into(),
                    VarOrSym::Var(Variable::Ref { declaration, .. }) => declaration.clone().into(),
                    VarOrSym::Var(Variable::Name { .. }) => {
                        return "TODO: unresolved variable".into_any();
                    }
                };
                application::<Views, Be>(pres.clone(), uri, Some(head), app).into_any()
            }
            Self::Bound(b)
                if b.presentation.is_none()
                    && matches!(
                        b.head,
                        Self::Symbol { .. }
                            | Self::Var {
                                variable: Variable::Ref { .. },
                                ..
                            }
                    ) =>
            {
                // SAFETY: pattern match above
                let (leaf, vos) = unsafe { do_head(b.head.clone()) };
                bound::<Views, Be>(vos, leaf, /*b.body.clone(),*/ None, b).into_any()
            }
            Self::Bound(b) if b.presentation.is_some() => {
                // SAFETY: presentation.is_some();
                let pres = unsafe { b.presentation.clone().unwrap_unchecked() };
                let head = match &b.head {
                    Self::Field(f) => f.record.clone(),
                    t => t.clone(),
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
                bound::<Views, Be>(pres, uri, /*b.body.clone(),*/ Some(head), b).into_any()
            }
            Self::Opaque(o) => {
                let mut terms = o
                    .terms
                    .iter()
                    .map(|t| {
                        let t = t.clone();
                        Some(move || t.into_view::<Views, Be>(true))
                    })
                    .collect::<Vec<_>>();
                do_opaque(&o.node, &mut terms).into_any()
            }
            Self::Field(f) if f.presentation.is_some() => {
                // SAFETY: presentation.is_some();
                let pres = unsafe { f.presentation.clone().unwrap_unchecked() };
                let record = f.record.clone();
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

            Self::Application(app)
                if app.presentation.is_none()
                    && matches!(
                        &app.head,
                        Self::Field(f)
                        if f.presentation.is_none() && f.record_type.is_some()
                    ) =>
            {
                // let arguments = do_args::<Views, Be>(arguments);
                let Self::Field(f) = &app.head else {
                    // SAFETY: pattern match above
                    unsafe { unreachable_unchecked() }
                };
                // SAFETY: pattern match above
                let tp = unsafe { f.record_type.clone().unwrap_unchecked() };
                let record = f.record.clone();
                let key = f.key.clone();
                let record =
                    ClonableView::new(true, move || record.clone().into_view::<Views, Be>(true));
                // TODO I think this clone can be avoided
                //let arguments = app.arguments.clone();
                FutureExt::into_view(
                    move || {
                        tp.clone().get_in_record_type_async(key.clone(), |uri| {
                            WithLocalCache::<Be>::default().get_structure(uri)
                        })
                    },
                    move |r| match r {
                        Err(e) => e.to_string().into_any(),
                        Ok(None) => "(Structure not found)".into_any(),
                        Ok(Some(r)) => application::<Views, Be>(
                            VarOrSym::Sym(r.uri.clone()),
                            r.uri.clone().into(),
                            Some(record),
                            app,
                        ),
                    },
                )
                .into_any()
            }

            Self::Number(n) => match n {
                Numeric::Int(i) => mn().child(i.to_string()).into_any(),
                Numeric::Float(f) => mn().child(f.to_string()).into_any(),
            },
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
    app: ApplicationTerm,
) -> AnyView {
    let arguments = do_args::<Views, Be>(&app.arguments);
    DocumentState::with_head(head.clone(), move || {
        if with_context::<CurrentUri, _>(|_| ()).is_some() {
            Views::application(
                head.clone(),
                None,
                None,
                do_application_inner::<Views, Be>(
                    Some(Term::Application(app)),
                    uri,
                    head,
                    real_term,
                    arguments,
                ),
            )
        } else {
            do_application_inner::<Views, Be>(
                Some(Term::Application(app)),
                uri,
                head,
                real_term,
                arguments,
            )
            .into_view::<Views>()
        }
    })
}

fn bound<Views: FtmlViews, Be: SendBackend>(
    head: VarOrSym,
    uri: LeafUri,
    //body: Term,
    real_term: Option<ClonableView>,
    app: BindingTerm,
) -> AnyView {
    let arguments = do_bound_args::<Views, Be>(&app.arguments);
    /*arguments.push(Either::Left(ClonableView::new(true, move || {
        body.clone().into_view::<Views, Be>(true)
    })));*/
    DocumentState::with_head(head.clone(), move || {
        if with_context::<CurrentUri, _>(|_| ()).is_some() {
            Views::binder_application(
                head.clone(),
                None,
                None,
                do_application_inner::<Views, Be>(
                    Some(Term::Bound(app)),
                    uri,
                    head,
                    real_term,
                    arguments,
                ),
            )
        } else {
            do_application_inner::<Views, Be>(
                Some(Term::Bound(app)),
                uri,
                head,
                real_term,
                arguments,
            )
            .into_view::<Views>()
        }
    })
}

fn sym<Views: FtmlViews, Be: SendBackend>(
    uri: SymbolUri,
    this: Option<ClonableView>,
    in_term: bool,
) -> AnyView {
    DocumentState::with_head(VarOrSym::Sym(uri.clone()), move || {
        if with_context::<CurrentUri, _>(|_| ()).is_some() {
            Views::symbol_reference(
                uri.clone(),
                None,
                in_term,
                ClonableView::new(true, move || {
                    let uri = uri.clone();
                    let this = this.clone();
                    with_notations::<Be, _>(uri.clone().into(), move |t| {
                        if let Some(n) = t {
                            if let Some(n) = n.op {
                                Views::comp(ClonableView::new(true, move || super::view_node(&n)))
                            } else {
                                n.as_view::<Views>(&VarOrSym::Sym(uri), this.as_ref())
                            }
                        } else {
                            let name = uri.name;
                            Views::comp(ClonableView::new(true, move || {
                                mtext().style("color:red").child(name.last().to_string())
                            }))
                        }
                    })
                }),
            )
        } else {
            with_notations::<Be, _>(uri.clone().into(), move |t| {
                if let Some(n) = t {
                    if let Some(n) = n.op {
                        Views::comp(ClonableView::new(true, move || super::view_node(&n)))
                    } else {
                        n.as_view::<Views>(&VarOrSym::Sym(uri), this.as_ref())
                    }
                } else {
                    mtext()
                        .style("color:red")
                        .child(uri.name().last().to_string())
                        .into_any()
                }
            })
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
                Views::variable_reference(
                    Variable::Ref {
                        declaration: uri.clone(),
                        is_sequence,
                    },
                    None,
                    in_term,
                    ClonableView::new(true, move || {
                        let uri = uri.clone();
                        let this = this.clone();
                        with_notations::<Be, _>(uri.clone().into(), move |t| {
                            if let Some(n) = t {
                                if let Some(n) = n.op {
                                    Views::comp(ClonableView::new(true, move || {
                                        super::view_node(&n)
                                    }))
                                } else {
                                    n.as_view::<Views>(
                                        &VarOrSym::Var(Variable::Ref {
                                            declaration: uri,
                                            is_sequence,
                                        }),
                                        this.as_ref(),
                                    )
                                }
                            } else {
                                mtext()
                                    .style("color:red")
                                    .child(uri.name().last().to_string())
                                    .into_any()
                            }
                        })
                    }),
                )
            } else {
                with_notations::<Be, _>(uri.clone().into(), move |t| {
                    if let Some(n) = t {
                        if let Some(n) = n.op {
                            super::view_node(&n).into_any()
                        } else {
                            n.as_view::<Views>(
                                &VarOrSym::Var(Variable::Ref {
                                    declaration: uri,
                                    is_sequence,
                                }),
                                this.as_ref(),
                            )
                        }
                    } else {
                        mtext()
                            .style("color:red")
                            .child(uri.name().last().to_string())
                            .into_any()
                    }
                })
            }
        },
    )
}

fn var_name<Views: FtmlViews>(
    name: Id,
    notated: Option<Id>,
    this: Option<ClonableView>,
    in_term: bool,
) -> AnyView {
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
                    ClonableView::new(true, move || Views::comp(inner.clone())),
                ))
            } else {
                Right(mi().child(not))
            };
            if let Some(this) = this {
                leptos::math::msub()
                    .child(outer)
                    .child(this.into_view::<Views>())
                    .into_any()
            } else {
                outer.into_any()
            }
        },
    )
}

fn do_application_inner<Views: FtmlViews, Be: SendBackend>(
    term: Option<Term>,
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
        let term = term.clone();
        with_notations::<Be, _>(leaf.clone(), move |t| {
            if let Some(n) = t {
                n.with_arguments::<Views, _>(term, &vos, this.as_ref(), &arguments)
            } else {
                no_notation::<Views>(leaf.name().last(), &leaf, arguments)
            }
        })
    })
}

fn do_args<Views: FtmlViews, Be: SendBackend>(
    arguments: &[Argument],
) -> Vec<Either<ClonableView, Vec<ClonableView>>> {
    arguments
        .iter()
        .map(|a| match a {
            Argument::Simple(t) | Argument::Sequence(MaybeSequence::One(t)) => {
                let t = t.clone();
                Either::Left(ClonableView::new(true, move || {
                    t.clone().into_view::<Views, Be>(true)
                }))
            }
            Argument::Sequence(MaybeSequence::Seq(s)) => Either::Right(
                s.iter()
                    .map(|t| {
                        let t = t.clone();
                        ClonableView::new(true, move || t.clone().into_view::<Views, Be>(true))
                    })
                    .collect::<Vec<_>>(),
            ),
        })
        .collect::<Vec<_>>()
}

fn do_opaque(
    node: &OpaqueNode,
    terms: &mut Vec<Option<impl FnOnce() -> AnyView>>,
) -> leptos::either::Either<AnyView, AnyViewWithAttrs> {
    use leptos::either::{
        Either::Left,
        EitherOf4::{A, B, C, D},
    };
    let make_red = !node
        .children
        .iter()
        .any(|e| matches!(e, AnyOpaque::Term(_)));
    let i = super::html_from_tag(
        node.tag.as_ref(),
        node.children
            .iter()
            .map(|e| match e {
                AnyOpaque::Node(node) => A(do_opaque(node, terms)),
                AnyOpaque::Text(t) => B(t.to_string()),
                AnyOpaque::Term(i) => {
                    let f = terms.get_mut(*i as usize).and_then(Option::take);
                    f.map_or_else(|| C(mtext().child("ERROR")), |f| D(f()))
                }
            })
            .collect_view(),
    );
    let r = node.attributes.iter().fold(Left(i), |i, (k, v)| {
        super::attr(i, k.as_ref().to_string(), v.to_string())
    });
    if make_red {
        super::attr(r, "style", "color:red")
    } else {
        r
    }
}

#[allow(clippy::too_many_lines)]
fn do_bound_args<Views: FtmlViews, Be: SendBackend>(
    arguments: &[BoundArgument],
) -> Vec<Either<ClonableView, Vec<ClonableView>>> {
    arguments
        .iter()
        .map(|a| match a {
            BoundArgument::Simple(t) | BoundArgument::Sequence(MaybeSequence::One(t)) => {
                let t = t.clone();
                Either::Left(ClonableView::new(true, move || {
                    t.clone().into_view::<Views, Be>(true)
                }))
            }
            BoundArgument::Sequence(MaybeSequence::Seq(s)) => Either::Right(
                s.iter()
                    .map(|t| {
                        let t = t.clone();
                        ClonableView::new(true, move || t.clone().into_view::<Views, Be>(true))
                    })
                    .collect::<Vec<_>>(),
            ),
            BoundArgument::Bound(ComponentVar {
                var:
                    Variable::Ref {
                        declaration,
                        is_sequence,
                    },
                tp,
                df,
            })
            | BoundArgument::BoundSeq(MaybeSequence::One(ComponentVar {
                var:
                    Variable::Ref {
                        declaration,
                        is_sequence,
                    },
                tp,
                df,
            })) => {
                let declaration = declaration.clone();
                let tp = tp.clone();
                let df = df.clone();
                let is_sequence = *is_sequence;
                Either::Left(ClonableView::new(true, move || {
                    let declaration = declaration.clone();
                    let tp = tp.clone();
                    let df = df.clone();
                    with_notations::<Be, _>(declaration.clone().into(), move |t| {
                        let r = if let Some(n) = t {
                            n.as_view::<Views>(
                                &VarOrSym::Var(Variable::Ref {
                                    declaration,
                                    is_sequence,
                                }),
                                None,
                            )
                        } else {
                            mi().child(declaration.name().last().to_string())
                                .attr(FtmlKey::Head.attr_name(), declaration.to_string())
                                .attr(FtmlKey::Term.attr_name(), "OMV")
                                .attr(FtmlKey::Comp.attr_name(), "")
                                .into_any()
                        };
                        if tp.is_none() && df.is_none() {
                            return r;
                        }
                        let tp = tp.map(|t| {
                            view! {
                                <mo>":"</mo>{t.into_view::<Views, Be>(true)}
                            }
                        });
                        let df = df.map(|t| {
                            view! {
                                <mo>":="</mo>{t.into_view::<Views, Be>(true)}
                            }
                        });
                        view! {<mrow>{r}{tp}{df}</mrow>}.into_any()
                    })
                }))
            }
            BoundArgument::BoundSeq(MaybeSequence::Seq(v)) => Either::Right(
                v.iter()
                    .map(|v| {
                        let v = v.clone();
                        ClonableView::new(true, move || {
                            let tp = v.tp.clone();
                            let df = v.df.clone();
                            if let Variable::Ref {
                                declaration,
                                is_sequence,
                            } = &v.var
                            {
                                let declaration = declaration.clone();
                                let is_sequence = *is_sequence;
                                with_notations::<Be, _>(declaration.clone().into(), move |t| {
                                    let r = if let Some(n) = t {
                                        n.as_view::<Views>(
                                            &VarOrSym::Var(Variable::Ref {
                                                declaration,
                                                is_sequence,
                                            }),
                                            None,
                                        )
                                    } else {
                                        mi().child(declaration.name().last().to_string())
                                            .attr(
                                                FtmlKey::Head.attr_name(),
                                                declaration.to_string(),
                                            )
                                            .attr(FtmlKey::Term.attr_name(), "OMV")
                                            .attr(FtmlKey::Comp.attr_name(), "")
                                            .into_any()
                                    };
                                    if tp.is_none() && df.is_none() {
                                        return r;
                                    }
                                    let tp = tp.map(|t| {
                                        view! {
                                            <mo>":"</mo>{t.into_view::<Views, Be>(true)}
                                        }
                                    });
                                    let df = df.map(|t| {
                                        view! {
                                            <mo>":="</mo>{t.into_view::<Views, Be>(true)}
                                        }
                                    });
                                    view! {<mrow>{r}{tp}{df}</mrow>}.into_any()
                                })
                            } else {
                                let r = mtext().child("TODO: unresolved variable"); //.into_any();
                                if tp.is_none() && df.is_none() {
                                    return r.into_any();
                                }
                                let tp = tp.map(|t| {
                                    view! {
                                        <mo>":"</mo>{t.into_view::<Views, Be>(true)}
                                    }
                                });
                                let df = df.map(|t| {
                                    view! {
                                        <mo>":="</mo>{t.into_view::<Views, Be>(true)}
                                    }
                                });
                                view! {<mrow>{r}{tp}{df}</mrow>}.into_any()
                            }
                        })
                    })
                    .collect(),
            ),
            t => {
                let t = t.clone();
                Either::Left(ClonableView::new(true, move || {
                    mtext().child(format!("{t:?}")).into_any()
                }))
            }
        })
        .collect::<Vec<_>>()
}

fn with_notations<
    Be: SendBackend,
    F: FnOnce(Option<Notation>) -> AnyView + Send + Clone + 'static,
>(
    uri: LeafUri,
    then: F,
) -> AnyView {
    use crate::utils::FutureExt;
    let uricl = uri.clone();
    FutureExt::into_view(
        move || WithLocalCache::<Be>::default().get_notations(uricl.clone()),
        move |gl| {
            let not = gl.local.and_then(|v| select_notation(v, &uri)).or_else(|| {
                gl.global
                    .and_then(|r| r.ok().and_then(|v| select_notation(v, &uri)))
            });
            then(not)
        },
    )
}

#[allow(clippy::cast_possible_truncation)]
fn select_notation(
    notations: Vec<(DocumentElementUri, Notation)>,
    uri: &LeafUri,
) -> Option<Notation> {
    fn score(not: &DocumentElementUri, sym: &LeafUri) -> u8 {
        let mut ret = 0;
        if not.name.as_ref().starts_with("notation") {
            ret += 1;
        }
        if not.archive_uri() == sym.archive_uri() {
            ret += 1;
        } else {
            return ret;
        }
        if not.path().is_none() && sym.path().is_none() {
            ret += 1;
        } else if let Some(np) = not.path()
            && let Some(up) = sym.path()
        {
            if np == up {
                ret += np.steps().count() as u8;
            } else {
                let mut i = np.steps().zip(up.steps());
                while let Some((a, b)) = i.next()
                    && a == b
                {
                    ret += 1;
                }
                return ret;
            }
        } else {
            return ret;
        }
        match sym {
            LeafUri::Element(e) if not.document_name() == e.document_name() => ret += 1,
            LeafUri::Symbol(s) if not.document_name().as_ref() == s.module_name().first() => {
                ret += 1;
            }
            _ => (),
        }
        ret
    }
    notations
        .into_iter()
        .max_by_key(|(u, _)| score(u, uri))
        .map(|(_, n)| n)
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

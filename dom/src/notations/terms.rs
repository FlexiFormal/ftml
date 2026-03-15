use std::{hint::unreachable_unchecked, isize};

use crate::{
    ClonableView, DocumentState, FtmlViews,
    document::CurrentUri,
    notations::{AnyMaybeAttr, ArgumentRender, NotationExt},
    terms::{ReactiveTerm, TopTerm},
    utils::{
        FutureExt,
        local_cache::{LocalCache, SendBackend},
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
    DocumentElementUri, Id, IsDomainUri, IsNarrativeUri, LeafUri, NamedUri, SymbolUri,
    UriWithArchive, UriWithPath,
};
use leptos::{
    either::Either,
    math::{mi, mn, mo},
};
use leptos::{math::mtext, prelude::*};

macro_rules! commata {
    ($args:expr) => {{
        $args.next().map(|first| {
            view! {
                {first}
                {$args.map(|a| view!{{mo().child(',')}{a}}).collect_view()}
            }
        })
    }};
}

pub trait TermExt: Sized {
    fn into_view<Views: FtmlViews, Be: SendBackend>(self, in_term: bool) -> AnyView {
        self.into_view_with_precedence::<Views, Be>(in_term, i64::MAX)
    }

    fn into_view_with_precedence<Views: FtmlViews, Be: SendBackend>(
        self,
        in_term: bool,
        precedence: i64,
    ) -> AnyView;

    fn into_view_safe<Views: FtmlViews, Be: SendBackend>(self) -> impl IntoView {
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

 #[allow(clippy::cast_possible_truncation)]
fn no_notation<Views: FtmlViews,Be:SendBackend,A:ArgumentRender>(
    name: &str,
    uri: &LeafUri,
    arguments: &A,
) -> AnyView {
    fn do_view<Views: FtmlViews,Be:SendBackend,A:ArgumentRender>(arguments:&A,index:u8) -> AnyView {
        if arguments.is_sequence(index) {
            view! {
                {maybe_comp!(mo().child('('))}
                {arguments.render_arg::<Views,Be>(index,ftml_ontology::terms::ArgumentMode::Sequence,i64::MAX)}
                {maybe_comp!(mo().child(')'))}
            }.into_any()
        } else {
            arguments.render_arg::<Views,Be>(index, ftml_ontology::terms::ArgumentMode::Simple,i64::MAX)
        }
    }
    let kind = match uri {
        LeafUri::Element(_) => "OMV",
        LeafUri::Symbol(_) => "OMID",
    };
    if arguments.is_empty() {
        return
            mtext()
                .style("color:red")
                .child(name.to_string())
                .attr(FtmlKey::Head.attr_name(), uri.to_string())
                .attr(FtmlKey::Term.attr_name(), kind)
                .attr(FtmlKey::Comp.attr_name(), "").into_any()
        ;
    }
    //let mut args = arguments.into_iter();
    view! {<mrow>
        {mtext().style("color:red").child(name.to_string())
            .attr(FtmlKey::Head.attr_name(), uri.to_string())
            .attr(FtmlKey::Term.attr_name(), kind)
            .attr(FtmlKey::Comp.attr_name(), "")
        }
        {maybe_comp!(mo().child('('))}
        {do_view::<Views,Be,_>(arguments,0)}
        //{args.next().map(do_view::<Views>)}
        {(1..arguments.num_args()).map(|i| view!{
            {maybe_comp!(mo().child(','))}
            {do_view::<Views,Be,_>(arguments,i as u8)}
        }).collect_view()}
        /*{args.map(|v| view!{
            {maybe_comp!(mo().child(','))}
            {do_view::<Views>(v)}
        }).collect_view()}
        */
        {maybe_comp!(mo().child(')'))}
    </mrow>}.into_any()
}

impl TermExt for Term {
    fn into_view_with_precedence<Views: FtmlViews, Be: SendBackend>(
        self,
        in_term: bool,
        precedence: i64,
    ) -> AnyView {
        tracing::trace!("Presenting {self:?}");
        //owned(move || {
        match self {
            Self::Symbol {
                uri,
                presentation: None,
            } => sym::<Views, Be>(uri, None, in_term, precedence),
            Self::Var {
                variable:
                    Variable::Ref {
                        declaration,
                        is_sequence,
                    },
                presentation: None,
            } => var_ref::<Views, Be>(
                declaration,
                is_sequence,
                None,
                in_term,
                precedence,
            ),
            Self::Var {
                variable: Variable::Name { name, notated },
                ..
            } => var_name::<Views>(name, notated, None, in_term),
            Self::Application(app) => {
                app.into_view_with_precedence::<Views, Be>(in_term, precedence)
            }
            Self::Bound(b) => b.into_view_with_precedence::<Views, Be>(in_term, precedence),
            Self::Opaque(o) => {
                let mut terms = o
                    .terms
                    .iter()
                    .map(|t| {
                        let t = t.clone();
                        Some(move || t.into_view::<Views, Be>(true))
                    })
                    .collect::<Vec<_>>();
                do_opaque(&o.node, &mut terms)
            }
            Self::Field(f) if f.presentation.is_some() => {
                // SAFETY: presentation.is_some();
                let pres = unsafe { f.presentation.clone().unwrap_unchecked() };
                let record = f.record.clone();
                let record =
                    ClonableView::new(true, move || record.clone().into_view::<Views, Be>(true));
                match pres {
                    VarOrSym::Sym(uri) => {
                        sym::<Views, Be>(uri, Some(record), in_term, precedence)
                    }
                    VarOrSym::Var(Variable::Ref {
                        declaration,
                        is_sequence,
                    }) => var_ref::<Views, Be>(
                        declaration,
                        is_sequence,
                        Some(record),
                        in_term,
                        precedence,
                    ),
                    VarOrSym::Var(Variable::Name { name, notated }) => {
                        var_name::<Views>(name, notated, Some(record), in_term)
                    }
                }
            }
            Self::Number(n) => mn().child(match n {
                Numeric::Int(i) => i.to_string(),
                Numeric::Float(f) => f.to_string(),
            }).into_any(),
            t => mtext().child(format!("{t:?}")).into_any(),
        }
        //})

        //
    }
}

impl TermExt for ApplicationTerm {
    fn into_view_with_precedence<Views: FtmlViews, Be: SendBackend>(
        self,
        in_term: bool,
        precedence: i64,
    ) -> AnyView {
        use leptos::either::EitherOf3::{A as A3, B as B3, C as C3};
        match &self.head {
            Term::Symbol { .. }
            | Term::Var {
                variable: Variable::Ref { .. },
                ..
            } if self.presentation.is_none() => {
                // SAFETY: pattern match above
                let (leaf, vos) = unsafe { do_head(self.head.clone()) };
                application::<Views, Be>(vos, leaf, None, self, precedence)
            }
            _ if self.presentation.is_some() => {
                let head = match &self.head {
                    Term::Field(f) => f.record.clone(),
                    t => t.clone(),
                };
                let head =
                    ClonableView::new(true, move || head.clone().into_view::<Views, Be>(true));
                // SAFETY: app.presentation.is_some()
                let pres = unsafe { self.presentation.as_ref().unwrap_unchecked() };
                let uri = match pres {
                    VarOrSym::Sym(s) => s.clone().into(),
                    VarOrSym::Var(Variable::Ref { declaration, .. }) => declaration.clone().into(),
                    VarOrSym::Var(Variable::Name { .. }) => {
                        return "TODO: unresolved variable".into_any();
                    }
                };
                application::<Views, Be>(
                    pres.clone(),
                    uri,
                    Some(head),
                    self,
                    precedence,
                )
            }
            Term::Field(f) if f.presentation.is_none() && f.record_type.is_some() => {
                // let arguments = do_args::<Views, Be>(arguments);
                let Term::Field(f) = &self.head else {
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
                            LocalCache::get().get_structure(Be::get(), uri)
                        })
                    },
                    move |r| match r {
                        Err(e) => A3(e.to_string()),
                        Ok(None) => B3("(Structure not found)"),
                        Ok(Some(r)) => C3(application::<Views, Be>(
                            VarOrSym::Sym(r.uri.clone()),
                            r.uri.clone().into(),
                            Some(record),
                            self,
                            precedence,
                        )),
                    },
                ).into_any()
            }
            _ => {
                use leptos::either::Either::{Left, Right};
                let head = self.head.clone().into_view::<Views, Be>(true);
                let args = commata!(self.arguments.iter().map(|a| match a {
                    Argument::Simple(t) | Argument::Sequence(MaybeSequence::One(t)) => {
                        Left(t.clone().into_view::<Views, Be>(true))
                    }
                    Argument::Sequence(MaybeSequence::Seq(ts)) => Right({
                        let args =
                            commata!(ts.iter().map(|t| t.clone().into_view::<Views, Be>(true)));
                        view! {
                            {mo().child('[')}
                            {args}
                            {mo().child(']')}
                        }
                    }),
                })); // avoid recursive types
                leptos::tachys::mathml::mrow()
                    .child(head)
                    .child(mo().child('('))
                    .child(args)
                    .child(mo().child(')')).into_any()
            }
        }
    }
}

impl TermExt for BindingTerm {
    fn into_view_with_precedence<Views: FtmlViews, Be: SendBackend>(
        self,
        in_term: bool,
        precedence: i64,
    ) -> AnyView {
        use leptos::either::EitherOf3::{A as A3, B as B3, C as C3};
        use leptos::either::EitherOf4::{A, B, C, D};
        match &self.head {
            Term::Symbol { .. }
            | Term::Var {
                variable: Variable::Ref { .. },
                ..
            } if self.presentation.is_none() => {
                // SAFETY: pattern match above
                let (leaf, vos) = unsafe { do_head(self.head.clone()) };
                bound::<Views, Be>(
                    vos, leaf, /*b.body.clone(),*/ None, self, precedence,
                )
            }
            _ if self.presentation.is_some() => {
                // SAFETY: presentation.is_some();
                let pres = unsafe { self.presentation.clone().unwrap_unchecked() };
                let head = match &self.head {
                    Term::Field(f) => f.record.clone(),
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
                bound::<Views, Be>(
                    pres,
                    uri,
                    /*b.body.clone(),*/ Some(head),
                    self,
                    precedence,
                )
            }
            Term::Field(f) if f.presentation.is_none() && f.record_type.is_some() => {
                // let arguments = do_args::<Views, Be>(arguments);
                let Term::Field(f) = &self.head else {
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
                            LocalCache::get().get_structure(Be::get(), uri)
                        })
                    },
                    move |r| match r {
                        Err(e) => A3(e.to_string()),
                        Ok(None) => B3("(Structure not found)"),
                        Ok(Some(r)) => C3(bound::<Views, Be>(
                            VarOrSym::Sym(r.uri.clone()),
                            r.uri.clone().into(),
                            Some(record),
                            self,
                            precedence,
                        )),
                    },
                ).into_any()
            }
            _ => {
                //use leptos::either::EitherOf4::{A, B, C, D};
                let head = self.head.clone().into_view::<Views, Be>(true);
                let args = commata!(self.arguments.iter().map(|a| match a {
                    BoundArgument::Simple(t) | BoundArgument::Sequence(MaybeSequence::One(t)) => {
                        A(t.clone().into_view::<Views, Be>(true))
                    }
                    BoundArgument::Sequence(MaybeSequence::Seq(ts)) => B({
                        let args =
                            commata!(ts.iter().map(|t| t.clone().into_view::<Views, Be>(true)));
                        view! {
                            {mo().child('[')}
                            {args}
                            {mo().child(']')}
                        }
                    }),
                    BoundArgument::Bound(cv) | BoundArgument::BoundSeq(MaybeSequence::One(cv)) => {
                        C(do_cv::<Views, Be>(cv.clone(),i64::MAX))
                    }
                    BoundArgument::BoundSeq(MaybeSequence::Seq(ts)) => D({
                        let args = commata!(ts.iter().map(|t| do_cv::<Views, Be>(t.clone(),i64::MAX)));
                        view! {
                            {mo().child('[')}
                            {args}
                            {mo().child(']')}
                        }
                    }),
                }));
                leptos::tachys::mathml::mrow()
                    .child(head)
                    .child(mo().child('('))
                    .child(args)
                    .child(mo().child(')')).into_any()
            }
        }
    }
}

fn application<Views: FtmlViews, Be: SendBackend>(
    head: VarOrSym,
    uri: LeafUri,
    real_term: Option<ClonableView>,
    app: ApplicationTerm,
    precedence: i64,
) -> AnyView {
    let arguments = app.arguments.clone(); //do_args::<Views, Be>(&app.arguments);
    DocumentState::with_head(head.clone(), move || {
        if with_context::<CurrentUri, _>(|_| ()).is_some() {
            Views::application(
                head.clone(),
                None,
                None,
                do_application_inner::<Views, Be,_>(
                    Some(Term::Application(app)),
                    uri,
                    head,
                    real_term,
                    arguments,
                    precedence,
                ),
            )
        } else {
            do_application_inner::<Views, Be,_>(
                Some(Term::Application(app)),
                uri,
                head,
                real_term,
                arguments,
                precedence,
            )
            .into_view::<Views>()
        }
    }).into_any()
}

fn bound<Views: FtmlViews, Be: SendBackend>(
    head: VarOrSym,
    uri: LeafUri,
    //body: Term,
    real_term: Option<ClonableView>,
    app: BindingTerm,
    precedence: i64,
) -> AnyView {
    let arguments = app.arguments.clone();//do_bound_args::<Views, Be>(&app.arguments);
    /*arguments.push(Either::Left(ClonableView::new(true, move || {
        body.clone().into_view::<Views, Be>(true)
    })));*/
    DocumentState::with_head(head.clone(), move || {
        if with_context::<CurrentUri, _>(|_| ()).is_some() {
            Views::binder_application(
                head.clone(),
                None,
                None,
                do_application_inner::<Views, Be,_>(
                    Some(Term::Bound(app)),
                    uri,
                    head,
                    real_term,
                    arguments,
                    precedence,
                ),
            )
        } else {
            do_application_inner::<Views, Be,_>(
                Some(Term::Bound(app)),
                uri,
                head,
                real_term,
                arguments,
                precedence,
            )
            .into_view::<Views>()
        }
    }).into_any()
}

fn sym<Views: FtmlViews, Be: SendBackend>(
    uri: SymbolUri,
    this: Option<ClonableView>,
    in_term: bool,
    precedence: i64,
) -> AnyView {
    use leptos::either::EitherOf3::{A, B, C};
    DocumentState::with_head(VarOrSym::Sym(uri.clone()), move || {
        if with_context::<CurrentUri, _>(|_| ()).is_some() {
            Views::symbol_reference(
                uri.clone(),
                None,
                in_term,
                ClonableView::new(true, move || {
                    let uri = uri.clone();
                    let this = this.clone();
                    with_notations::<Be, _, _>(uri.clone().into(), move |t| {
                        if let Some(n) = t {
                            let prec = n.precedence;
                            if let Some(n) = n.op {
                                A(super::with_precedences(
                                    precedence,
                                    prec,
                                    Views::comp(ClonableView::new(true, move || {
                                        super::view_node(&n,false)
                                    })),
                                ))
                            } else {
                                B(n.as_view::<Views,Be>(&VarOrSym::Sym(uri), this.as_ref(),precedence),
                                )
                            }
                        } else {
                            let name = uri.name;
                            C(Views::comp(ClonableView::new(true, move || {
                                mtext().style("color:red").child(name.last().to_string())
                            })))
                        }
                    })
                }),
            )
        } else {
            with_notations::<Be, _, _>(uri.clone().into(), move |t| {
                if let Some(n) = t {
                    if let Some(n) = n.op {
                        A(Views::comp(ClonableView::new(true, move || {
                            super::view_node(&n,false)
                        })))
                    } else {
                        B(n.as_view::<Views,Be>(&VarOrSym::Sym(uri), this.as_ref(),precedence))
                    }
                } else {
                    C(mtext()
                        .style("color:red")
                        .child(uri.name().last().to_string()))
                }
            })
            .into_any()
        }
    }).into_any()
}

fn var_ref<Views: FtmlViews, Be: SendBackend>(
    uri: DocumentElementUri,
    is_sequence: Option<bool>,
    this: Option<ClonableView>,
    in_term: bool,
    precedence: i64,
) -> AnyView {
    use leptos::either::EitherOf3::{A, B, C};
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
                        with_notations::<Be, _, _>(uri.clone().into(), move |t| {
                            if let Some(n) = t {
                                let prec = n.precedence;
                                if let Some(n) = n.op {
                                    A(super::with_precedences(
                                        precedence,
                                        prec,
                                        Views::comp(ClonableView::new(true, move || {
                                            super::view_node(&n,false)
                                        })),
                                    ))
                                } else {
                                    B(n.as_view::<Views,Be>(
                                            &VarOrSym::Var(Variable::Ref {
                                                declaration: uri,
                                                is_sequence,
                                            }),
                                            this.as_ref(),
                                            precedence
                                        ),
                                    )
                                }
                            } else {
                                C(mtext()
                                    .style("color:red")
                                    .child(uri.name().last().to_string()))
                            }
                        })
                    }),
                )
            } else {
                with_notations::<Be, _, _>(uri.clone().into(), move |t| {
                    if let Some(n) = t {
                        if let Some(n) = n.op {
                            A(super::view_node(&n,false))
                        } else {
                            B(n.as_view::<Views,Be>(
                                &VarOrSym::Var(Variable::Ref {
                                    declaration: uri,
                                    is_sequence,
                                }),
                                this.as_ref(),
                                precedence
                            ))
                        }
                    } else {
                        C(mtext()
                            .style("color:red")
                            .child(uri.name().last().to_string()))
                    }
                })
                .into_any()
            }
        },
    ).into_any()
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
                Left(
                    leptos::math::msub()
                        .child(outer)
                        .child(this.into_view::<Views>()),
                )
            } else {
                Right(outer)
            }
        },
    ).into_any()
}

fn do_application_inner<Views: FtmlViews, Be: SendBackend,A:ArgumentRender>(
    term: Option<Term>,
    leaf: LeafUri,
    vos: VarOrSym,
    this: Option<ClonableView>,
    arguments: A,//Vec<Either<ClonableView, Vec<ClonableView>>>,
    precedence: i64,
) -> ClonableView {
    use leptos::either::Either::{Left, Right};
    ClonableView::new(true, move || {
        let leaf = leaf.clone();
        let vos = vos.clone();
        let arguments = arguments.clone();
        let this = this.clone();
        let term = term.clone();
        with_notations::<Be, _, _>(leaf.clone(), move |t| t.map_or_else(
            || Right(no_notation::<Views,Be,_>(leaf.name().last(), &leaf, &arguments)),
            |n| Left(n.with_arguments::<Views, Be,_>(term, &vos, this.as_ref(), &arguments,precedence)))
        )
    })
}

/*
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
 */

fn do_opaque<F: FnOnce() -> AnyView>(
    node: &OpaqueNode,
    terms: &mut Vec<Option<F>>,
) -> AnyView {
    use leptos::either::EitherOf4::{A, B, C, D};
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
    let r = node
        .attributes
        .iter()
        .fold(AnyMaybeAttr::Any(i), |i, (k, v)| {
            i.attr(k.as_ref().to_string(), v.to_string())
        });
    if make_red {
        r.attr("style", "color:red").into_any()
    } else {
        r.into_any()
    }
}

/*
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
            BoundArgument::Bound(cv) | BoundArgument::BoundSeq(MaybeSequence::One(cv)) => {
                let cv = cv.clone();
                Either::Left(ClonableView::new(true, move || {
                    do_cv::<Views, Be>(cv.clone())
                }))
            }
            BoundArgument::BoundSeq(MaybeSequence::Seq(v)) => Either::Right(
                v.iter()
                    .map(|cv| {
                        let cv = cv.clone();
                        ClonableView::new(true, move || do_cv::<Views, Be>(cv.clone()))
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
 */

pub fn do_cv<Views: FtmlViews, Be: SendBackend>(cv: ComponentVar,precedence:i64) -> AnyView {
    use leptos::either::Either::{Left, Right};
    match cv.var {
        Variable::Ref {
            declaration,
            is_sequence,
        } => with_notations::<Be, _, _>(
            declaration.clone().into(),
            move |t| {
                let r = if let Some(n) = t {
                    Left(n.as_view::<Views, Be>(
                        &VarOrSym::Var(Variable::Ref {
                            declaration,
                            is_sequence,
                        }),
                        None,
                        precedence
                    ))
                } else {
                    Right(
                        mi().child(declaration.name().last().to_string())
                            .attr(FtmlKey::Head.attr_name(), declaration.to_string())
                            .attr(FtmlKey::Term.attr_name(), "OMV")
                            .attr(FtmlKey::Comp.attr_name(), ""),
                    )
                };
                if cv.tp.is_none() && cv.df.is_none() {
                    return Left(r);
                }
                let tp = cv.tp.map(|t| {
                    view! {
                        <mo>":"</mo>{t.into_view::<Views, Be>(true)}
                    }
                });
                let df = cv.df.map(|t| {
                    view! {
                        <mo>":="</mo>{t.into_view::<Views, Be>(true)}
                    }
                });
                Right(view! {<mrow>{r}{tp}{df}</mrow>})
            },
        ),
        Variable::Name { name, notated } => {
            let r = var_name::<Views>(name, notated, None, true); //mtext().child("TODO: unresolved variable"); //.into_any();
            if cv.tp.is_none() && cv.df.is_none() {
                return r;
            }
            let tp = cv.tp.map(|t| {
                view! {
                    <mo>":"</mo>{t.into_view::<Views, Be>(true)}
                }
            });
            let df = cv.df.map(|t| {
                view! {
                    <mo>":="</mo>{t.into_view::<Views, Be>(true)}
                }
            });
            view! {<mrow>{r}{tp}{df}</mrow>}.into_any()
        }
    }
}

fn with_notations<
    Be: SendBackend,
    V: IntoView + Send + 'static,
    F: FnOnce(Option<Notation>) -> V + Send + Clone + 'static,
>(
    uri: LeafUri,
    then: F,
) -> AnyView {
    use crate::utils::FutureExt;
    let uricl = uri.clone();
    FutureExt::into_view(
        move || LocalCache::get().get_notations(Be::get(), uricl.clone()),
        move |gl| {
            let not = gl.local.and_then(|v| select_notation(v, &uri)).or_else(|| {
                gl.global
                    .and_then(|r| r.ok().and_then(|v| select_notation(v, &uri)))
            });
            then(not)
        },
    ).into_any()
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

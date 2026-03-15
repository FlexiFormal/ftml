mod terms;

use crate::ClonableView;
use crate::FtmlViews;
use crate::document::{CurrentUri, WithHead};
use crate::terms::{ReactiveApplication, ReactiveTerm, TopTerm};
use crate::utils::local_cache::SendBackend;
use crate::utils::owned;
use ftml_ontology::terms::Argument;
use ftml_ontology::terms::BoundArgument;
use ftml_ontology::terms::ComponentVar;
use ftml_ontology::terms::MaybeSequence;
use ftml_ontology::terms::{Term, VarOrSym};
use ftml_ontology::{
    narrative::elements::{
        Notation,
        notations::{NodeOrText, NotationComponent, NotationNode},
    },
    terms::ArgumentMode,
};
use ftml_parser::FtmlKey;
use ftml_parser::extraction::ArgumentPosition;
use leptos::attr::AttributeValue;
use leptos::attr::custom::CustomAttributeKey;
use leptos::math::mtext;
use leptos::tachys::view::any_view::AnyViewWithAttrs;
use leptos::{either::Either, prelude::*};
use std::num::NonZeroU8;
pub use terms::*;

pub fn with_precedences(down: i64, up: i64, view: impl IntoView) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    if up > down {
        Left(view! {
            <mo lspace="0" rspace="0" stretchy="true" attr:data-ftml-comp="">"("</mo>
            {view}
            <mo lspace="0" rspace="0" stretchy="true" attr:data-ftml-comp="">")"</mo>
        })
    } else {
        Right(view)
    }
}

pub trait NotationExt {
    fn with_arguments<Views: FtmlViews, Be: SendBackend, R: ArgumentRender>(
        &self,
        term: Option<Term>,
        head: &VarOrSym,
        this: Option<&ClonableView>,
        args: &R,
        precedence: i64,
    ) -> impl IntoView + use<Self, Views, Be, R> + 'static;

    fn with_arguments_safe<Views: FtmlViews, Be: SendBackend, R: ArgumentRender>(
        &self,
        term: Option<Term>,
        head: &VarOrSym,
        this: Option<&ClonableView>,
        args: &R,
    ) -> impl IntoView + use<Self, Views, Be, R> {
        owned(move || {
            provide_context(WithHead(Some(head.clone())));
            provide_context(None::<TopTerm>);
            provide_context(None::<ReactiveTerm>);
            self.with_arguments::<Views, Be, R>(term, head, this, args, i64::MAX)
        })
    }

    fn as_view<Views: FtmlViews, Be: SendBackend>(
        &self,
        head: &VarOrSym,
        this: Option<&ClonableView>,
        precedence: i64,
    ) -> impl IntoView + use<Self, Views, Be>;

    fn as_view_safe<Views: FtmlViews, Be: SendBackend>(
        &self,
        head: &VarOrSym,
        this: Option<&ClonableView>,
    ) -> impl IntoView + use<Self, Views, Be> {
        owned(move || {
            provide_context(WithHead(Some(head.clone())));
            self.as_view::<Views, Be>(head, this, i64::MAX)
        })
        //DocumentState::with_head(head.clone(), move || self.as_view::<Views>(head, this))
    }
    fn as_op<Views: FtmlViews, Be: SendBackend>(
        &self,
        head: &VarOrSym,
        this: Option<&ClonableView>,
        precedence: i64,
    ) -> impl IntoView + use<Self, Views, Be>;

    fn as_op_safe<Views: FtmlViews, Be: SendBackend>(
        &self,
        head: &VarOrSym,
        this: Option<&ClonableView>,
    ) -> impl IntoView + use<Self, Views, Be> {
        owned(move || {
            provide_context(WithHead(Some(head.clone())));
            self.as_op::<Views, Be>(head, this, i64::MAX)
        })
        //DocumentState::with_head(head.clone(), move || self.as_op::<Views>(head, this))
    }
}

pub trait ArgumentRender: Clone + Send + Sync + 'static {
    #[inline]
    fn is_empty(&self) -> bool {
        self.num_args() > 0
    }
    fn num_args(&self) -> usize;
    fn is_sequence(&self, index: u8) -> bool;
    fn render_arg<Views: FtmlViews, Be: SendBackend>(
        &self,
        index: u8,
        mode: ArgumentMode,
        argument_prec: i64,
    ) -> impl IntoView + use<Self, Views, Be>;
    fn render_arg_at<Views: FtmlViews, Be: SendBackend>(
        &self,
        index: u8,
        seq_index: usize,
        mode: ArgumentMode,
    ) -> impl IntoView + use<Self, Views, Be>;
    fn length_at(&self, index: u8) -> usize;
}

fn error() -> impl IntoView {
    mtext().style("color:red").child("ERROR")
}

fn render_arg<Views: FtmlViews, Be: SendBackend>(
    term: &Term,
    index: u8,
    seq_index: Option<usize>,
    mode: ArgumentMode,
    argument_prec: i64,
) -> impl IntoView + use<Views, Be> {
    use_context::<Option<ReactiveTerm>>().flatten().map_or_else(
        || {
            leptos::either::Either::Right(
                term.clone()
                    .into_view_with_precedence::<Views, Be>(true, argument_prec),
            )
        },
        |r| {
            let position = seq_index.map_or_else(
                || {
                    // SAFETY: +1 > 0
                    unsafe { ArgumentPosition::Simple(NonZeroU8::new_unchecked(index + 1), mode) }
                },
                |i| unsafe {
                    ArgumentPosition::Sequence {
                        argument_number: NonZeroU8::new_unchecked(index + 1),
                        #[allow(clippy::cast_possible_truncation)]
                        sequence_index: NonZeroU8::new_unchecked((i + 1) as u8),
                        mode,
                    }
                },
            );
            let t = term.clone();
            leptos::either::Either::Left(r.add_argument::<Views>(
                position,
                ClonableView::new(true, move || {
                    t.clone()
                        .into_view_with_precedence::<Views, Be>(true, argument_prec)
                }),
            ))
        },
    )
}

fn render_cv<Views: FtmlViews, Be: SendBackend>(
    cv: &ComponentVar,
    index: u8,
    seq_index: Option<usize>,
    mode: ArgumentMode,
    argument_prec: i64,
) -> impl IntoView + use<Views, Be> {
    use_context::<Option<ReactiveTerm>>().flatten().map_or_else(
        || leptos::either::Either::Right(terms::do_cv::<Views, Be>(cv.clone(), argument_prec)),
        |r| {
            let position = seq_index.map_or_else(
                || {
                    // SAFETY: +1 > 0
                    unsafe { ArgumentPosition::Simple(NonZeroU8::new_unchecked(index + 1), mode) }
                },
                |i| unsafe {
                    ArgumentPosition::Sequence {
                        argument_number: NonZeroU8::new_unchecked(index + 1),
                        #[allow(clippy::cast_possible_truncation)]
                        sequence_index: NonZeroU8::new_unchecked((i + 1) as u8),
                        mode,
                    }
                },
            );
            let t = cv.clone();
            leptos::either::Either::Left(r.add_argument::<Views>(
                position,
                ClonableView::new(true, move || {
                    terms::do_cv::<Views, Be>(t.clone(), argument_prec)
                }),
            ))
        },
    )
}

impl ArgumentRender for Box<[Argument]> {
    #[inline]
    fn is_empty(&self) -> bool {
        (**self).is_empty()
    }
    #[inline]
    fn num_args(&self) -> usize {
        self.len()
    }
    fn is_sequence(&self, index: u8) -> bool {
        matches!(self.get(index as usize), Some(Argument::Sequence(_)))
    }
    fn render_arg<Views: FtmlViews, Be: SendBackend>(
        &self,
        index: u8,
        mode: ArgumentMode,
        argument_prec: i64,
    ) -> impl IntoView + use<Views, Be> {
        use leptos::either::Either::{Left, Right};
        use leptos::either::EitherOf5::{A, B, C, D, E};
        tracing::trace!("rendering arg {index}@{mode:?} of {}", self.len());
        match self.get(index as usize) {
            Some(Argument::Simple(v) | Argument::Sequence(MaybeSequence::One(v))) => {
                A(render_arg::<Views, Be>(v, index, None, mode, argument_prec))
            }
            Some(Argument::Sequence(MaybeSequence::Seq(v))) => {
                let len = v.len();
                if len == 0 {
                    return E(());
                }
                // SAFETY: len > 0
                let first = || {
                    render_arg::<Views, Be>(
                        unsafe { v.first().unwrap_unchecked() },
                        index,
                        Some(0),
                        mode,
                        argument_prec,
                    )
                };
                if len == 1 {
                    return B(first());
                }
                C(view! {
                    <mrow>
                        {first()}
                    {
                        v.iter().enumerate().skip(1).map(|(i,v)| view!{
                            {

                                if with_context::<CurrentUri, _>(|_| ()).is_some() {
                                    Left(Views::comp(ClonableView::new(true,|| leptos::math::mo().child(","))))
                                } else {
                                    Right(leptos::math::mo().child(","))
                                }
                            }
                            {render_arg::<Views,Be>(v,index, Some(i), mode,argument_prec)}
                        }).collect_view()
                    }
                    </mrow>
                })
            }
            _ => D(error()),
        }
    }
    fn render_arg_at<Views: FtmlViews, Be: SendBackend>(
        &self,
        index: u8,
        seq_index: usize,
        mode: ArgumentMode,
    ) -> impl IntoView + use<Views, Be> {
        use leptos::either::Either::{Left, Right};
        tracing::trace!(
            "rendering arg sequence@{mode:?} {index}/{seq_index} of {}",
            self.len()
        );
        match self.get(index as usize) {
            Some(Argument::Sequence(MaybeSequence::Seq(v))) => v.get(seq_index).map_or_else(
                || Right(error()),
                |v| {
                    Left(render_arg::<Views, Be>(
                        v,
                        index,
                        Some(seq_index),
                        mode,
                        i64::MAX,
                    ))
                },
            ),
            Some(Argument::Simple(t) | Argument::Sequence(MaybeSequence::One(t)))
                if seq_index == 0 =>
            {
                Left(render_arg::<Views, Be>(
                    t,
                    index,
                    Some(seq_index),
                    mode,
                    i64::MAX,
                ))
            }
            _ => Right(error()),
        }
    }
    fn length_at(&self, index: u8) -> usize {
        self.get(index as usize).map_or(0, |l| {
            if let Argument::Sequence(MaybeSequence::Seq(v)) = l {
                v.len()
            } else {
                1
            }
        })
    }
}

impl ArgumentRender for Box<[BoundArgument]> {
    #[inline]
    fn is_empty(&self) -> bool {
        (&**self).is_empty()
    }
    #[inline]
    fn num_args(&self) -> usize {
        self.len()
    }
    fn is_sequence(&self, index: u8) -> bool {
        matches!(
            self.get(index as usize),
            Some(BoundArgument::Sequence(_) | BoundArgument::BoundSeq(_))
        )
    }
    fn render_arg<Views: FtmlViews, Be: SendBackend>(
        &self,
        index: u8,
        mode: ArgumentMode,
        argument_precedence: i64,
    ) -> impl IntoView + use<Views, Be> {
        use leptos::either::Either::{Left, Right};
        use leptos::either::EitherOf6::{A, B, C, D, E, F};
        tracing::trace!("rendering arg {index}@{mode:?} of {}", self.len());
        match self.get(index as usize) {
            Some(BoundArgument::Simple(v) | BoundArgument::Sequence(MaybeSequence::One(v))) => A(
                render_arg::<Views, Be>(v, index, None, mode, argument_precedence),
            ),
            Some(BoundArgument::Sequence(MaybeSequence::Seq(v))) => {
                let len = v.len();
                if len == 0 {
                    return E(());
                }
                // SAFETY: len > 0
                let first = || {
                    render_arg::<Views, Be>(
                        unsafe { v.first().unwrap_unchecked() },
                        index,
                        Some(0),
                        mode,
                        argument_precedence,
                    )
                };
                if len == 1 {
                    return A(first());
                }
                C(view! {
                    <mrow>
                        {first()}
                    {
                        v.iter().enumerate().skip(1).map(|(i,v)| view!{
                            {

                                if with_context::<CurrentUri, _>(|_| ()).is_some() {
                                    Left(Views::comp(ClonableView::new(true,|| leptos::math::mo().child(","))))
                                } else {
                                    Right(leptos::math::mo().child(","))
                                }
                            }
                            {render_arg::<Views,Be>(v,index, Some(i), mode,argument_precedence)}
                        }).collect_view()
                    }
                    </mrow>
                })
            }
            Some(BoundArgument::Bound(cv) | BoundArgument::BoundSeq(MaybeSequence::One(cv))) => B(
                render_cv::<Views, Be>(cv, index, None, mode, argument_precedence),
            ),
            Some(BoundArgument::BoundSeq(MaybeSequence::Seq(v))) => {
                let len = v.len();
                if len == 0 {
                    return E(());
                }
                // SAFETY: len > 0
                let first = || {
                    render_cv::<Views, Be>(
                        unsafe { v.first().unwrap_unchecked() },
                        index,
                        Some(0),
                        mode,
                        i64::MAX,
                    )
                };
                if len == 1 {
                    return B(first());
                }
                F(view! {
                    <mrow>
                        {first()}
                    {
                        v.iter().enumerate().skip(1).map(|(i,v)| view!{
                            {

                                if with_context::<CurrentUri, _>(|_| ()).is_some() {
                                    Left(Views::comp(ClonableView::new(true,|| leptos::math::mo().child(","))))
                                } else {
                                    Right(leptos::math::mo().child(","))
                                }
                            }
                            {render_cv::<Views,Be>(v,index, Some(i), mode,i64::MAX)}
                        }).collect_view()
                    }
                    </mrow>
                })
            }
            _ => D(error()),
        }
    }
    fn render_arg_at<Views: FtmlViews, Be: SendBackend>(
        &self,
        index: u8,
        seq_index: usize,
        mode: ArgumentMode,
    ) -> impl IntoView + use<Views, Be> {
        use leptos::either::EitherOf3::{A, B, C};
        tracing::trace!(
            "rendering arg sequence@{mode:?} {index}/{seq_index} of {}",
            self.len()
        );
        match self.get(index as usize) {
            Some(BoundArgument::Sequence(MaybeSequence::Seq(v))) => v.get(seq_index).map_or_else(
                || C(error()),
                |v| {
                    A(render_arg::<Views, Be>(
                        v,
                        index,
                        Some(seq_index),
                        mode,
                        i64::MAX,
                    ))
                },
            ),
            Some(BoundArgument::Simple(t) | BoundArgument::Sequence(MaybeSequence::One(t)))
                if seq_index == 0 =>
            {
                A(render_arg::<Views, Be>(
                    t,
                    index,
                    Some(seq_index),
                    mode,
                    i64::MAX,
                ))
            }
            Some(BoundArgument::BoundSeq(MaybeSequence::Seq(v))) => v.get(seq_index).map_or_else(
                || C(error()),
                |cv| {
                    B(render_cv::<Views, Be>(
                        cv,
                        index,
                        Some(seq_index),
                        mode,
                        i64::MAX,
                    ))
                },
            ),
            Some(BoundArgument::Bound(cv) | BoundArgument::BoundSeq(MaybeSequence::One(cv)))
                if seq_index == 0 =>
            {
                B(render_cv::<Views, Be>(
                    cv,
                    index,
                    Some(seq_index),
                    mode,
                    i64::MAX,
                ))
            }

            _ => C(error()),
        }
    }
    fn length_at(&self, index: u8) -> usize {
        self.get(index as usize).map_or(0, |l| {
            if let BoundArgument::Sequence(MaybeSequence::Seq(v)) = l {
                v.len()
            } else {
                1
            }
        })
    }
}

impl ArgumentRender for Vec<Either<ClonableView, Vec<ClonableView>>> {
    #[inline]
    fn is_empty(&self) -> bool {
        (*self).is_empty()
    }
    #[inline]
    fn num_args(&self) -> usize {
        self.len()
    }
    fn is_sequence(&self, index: u8) -> bool {
        matches!(self.get(index as usize), Some(Either::Right(_)))
    }
    fn render_arg<Views: FtmlViews, Be: SendBackend>(
        &self,
        index: u8,
        mode: ArgumentMode,
        _: i64,
    ) -> impl IntoView + use<Views, Be> {
        use leptos::either::Either::{Left, Right};
        use leptos::either::EitherOf5::{A, B, C, D, E};
        tracing::trace!("rendering arg {index}@{mode:?} of {}", self.len());
        match self.get(index as usize) {
            Some(Either::Left(v)) => A(do_arg::<Views>(v.clone(), index, None, mode)),
            Some(Either::Right(v)) => {
                let len = v.len();
                if len == 0 {
                    return E(());
                }
                // SAFETY: len > 0
                let first = || {
                    do_arg::<Views>(
                        unsafe { v.first().unwrap_unchecked() }.clone(),
                        index,
                        Some(0),
                        mode,
                    )
                };
                if len == 1 {
                    return B(first());
                }
                C(view! {
                    <mrow>
                        {first()}
                    {
                        v.iter().enumerate().skip(1).map(|(i,v)| view!{
                            {

                                if with_context::<CurrentUri, _>(|_| ()).is_some() {
                                    Left(Views::comp(ClonableView::new(true,|| leptos::math::mo().child(","))))
                                } else {
                                    Right(leptos::math::mo().child(","))
                                }
                            }
                            {do_arg::<Views>(v.clone(), index, Some(i), mode)}
                        }).collect_view()
                    }
                    </mrow>
                })
            }
            _ => D(error()),
        }
    }
    fn render_arg_at<Views: FtmlViews, Be: SendBackend>(
        &self,
        index: u8,
        seq_index: usize,
        mode: ArgumentMode,
    ) -> impl IntoView + use<Views, Be> {
        use leptos::either::Either::{Left, Right};
        tracing::trace!(
            "rendering arg sequence@{mode:?} {index}/{seq_index} of {}",
            self.len()
        );
        match self.get(index as usize) {
            Some(Either::Right(v)) => v.get(seq_index).map_or_else(
                || Right(error()),
                |v| Left(do_arg::<Views>(v.clone(), index, Some(seq_index), mode)),
            ),
            Some(Either::Left(t)) if seq_index == 0 => {
                Left(do_arg::<Views>(t.clone(), index, Some(seq_index), mode))
            }
            _ => Right(error()),
        }
    }
    fn length_at(&self, index: u8) -> usize {
        self.get(index as usize)
            .map_or(0, |l| if let Either::Right(v) = l { v.len() } else { 1 })
    }
}

fn do_arg<Views: FtmlViews>(
    v: ClonableView,
    index: u8,
    seq_index: Option<usize>,
    mode: ArgumentMode,
) -> impl IntoView {
    if let Some(r) = use_context::<Option<ReactiveTerm>>().flatten() {
        let position = seq_index.map_or_else(
            || {
                // SAFETY: +1 > 0
                unsafe { ArgumentPosition::Simple(NonZeroU8::new_unchecked(index + 1), mode) }
            },
            |i| unsafe {
                ArgumentPosition::Sequence {
                    argument_number: NonZeroU8::new_unchecked(index + 1),
                    #[allow(clippy::cast_possible_truncation)]
                    sequence_index: NonZeroU8::new_unchecked((i + 1) as u8),
                    mode,
                }
            },
        );
        leptos::either::Either::Left(r.add_argument::<Views>(position, v))
    } else {
        leptos::either::Either::Right(v.into_view::<Views>())
    }
}

impl NotationExt for Notation {
    fn with_arguments<Views: FtmlViews, Be: SendBackend, R: ArgumentRender>(
        &self,
        term: Option<Term>,
        head: &VarOrSym,
        this: Option<&ClonableView>,
        args: &R,
        precedence: i64,
    ) -> impl IntoView + use<Views, Be, R> {
        if args.is_empty() {
            return leptos::either::Either::Left(self.as_op::<Views, Be>(head, this, precedence));
        }
        /*owned(move ||*/
        {
            let h = head.to_string();
            //provide_context(WithHead(Some(head.clone())));
            let r = with_precedences(
                precedence,
                self.precedence,
                view_component_with_args::<Views, Be, _>(
                    &self.component,
                    args,
                    this,
                    self.precedence,
                    &self.argprecs,
                )
                .attr(FtmlKey::Term.attr_name(), "OMBIND")
                .attr(FtmlKey::Head.attr_name(), h),
            );
            ReactiveApplication::close(term);
            leptos::either::Either::Right(r)
        } //)
    }

    fn as_op<Views: FtmlViews, Be: SendBackend>(
        &self,
        head: &VarOrSym,
        this: Option<&ClonableView>,
        precedence: i64,
    ) -> impl IntoView + use<Views, Be> {
        use leptos::either::EitherOf3::{A, B, C};
        self.op
            .as_ref()
            .map_or_else(
                || {
                    A(with_precedences(
                        precedence,
                        self.precedence,
                        view_component_with_args::<Views, Be, _>(
                            &self.component,
                            &DummyRender,
                            this,
                            self.precedence,
                            &self.argprecs,
                        ),
                    ))
                },
                |op| {
                    let op = op.clone();
                    let prec = self.precedence;
                    if with_context::<CurrentUri, _>(|_| ()).is_some() {
                        B(Views::comp(ClonableView::new(true, move || {
                            with_precedences(
                                precedence,
                                prec,
                                view_node(&op).attr("data-ftml-comp", ""),
                            )
                        })))
                    } else {
                        C(with_precedences(
                            precedence,
                            self.precedence,
                            view_node(&op).attr("data-ftml-comp", ""),
                        ))
                    }
                },
            )
            .attr(FtmlKey::Term.attr_name(), "OMID")
            .attr(FtmlKey::Head.attr_name(), head.to_string())
    }

    fn as_view<Views: FtmlViews, Be: SendBackend>(
        &self,
        head: &VarOrSym,
        this: Option<&ClonableView>,
        precedence: i64,
    ) -> impl IntoView + use<Views, Be> {
        let h = head.to_string();
        with_precedences(
            precedence,
            self.precedence,
            view_component_with_args::<Views, Be, _>(
                &self.component,
                &DummyRender,
                this,
                self.precedence,
                &self.argprecs,
            )
            .attr(FtmlKey::Term.attr_name(), "OMID")
            .attr(FtmlKey::Head.attr_name(), h),
        )
    }
}

pub enum AnyMaybeAttr {
    Any(AnyView),
    Attr(AnyViewWithAttrs),
}
impl AnyMaybeAttr {
    fn attr<K: CustomAttributeKey, V: AttributeValue>(self, k: K, v: V) -> Self {
        Self::Attr(match self {
            Self::Any(a) => a.attr(k, v),
            Self::Attr(a) => a.attr(k, v),
        })
    }
    fn into_view(self) -> impl IntoView {
        match self {
            Self::Any(a) => leptos::either::Either::Left(a),
            Self::Attr(a) => leptos::either::Either::Right(a),
        }
    }
}

pub(crate) fn view_component_with_args<Views: FtmlViews, Be: SendBackend, A: ArgumentRender>(
    comp: &NotationComponent,
    args: &A,
    this: Option<&ClonableView>,
    prec: i64,
    argument_precs: &[i64],
) -> impl IntoView + use<Views, A, Be> {
    use leptos::either::EitherOf8::{A, B, C, D, E, F, G, H};
    match comp {
        NotationComponent::Text(s) => B(s.to_string()),
        NotationComponent::Node {
            tag,
            attributes,
            children,
        } => A(attributes
            .iter()
            .fold(
                AnyMaybeAttr::Any(html_from_tag(
                    tag.as_ref(),
                    children
                        .iter()
                        .map(|c| {
                            view_component_with_args::<Views, Be, _>(
                                c,
                                args,
                                this,
                                prec,
                                argument_precs,
                            )
                        })
                        .collect_view(),
                )),
                |n, (k, v)| n.attr(k.to_string(), v.to_string()),
            )
            .into_view()),
        NotationComponent::MainComp(n) if this.is_some() => {
            // SAFETY: defined
            let this = unsafe { this.unwrap_unchecked().clone() };
            let n = n.clone();
            let inner = if with_context::<CurrentUri, _>(|_| ()).is_some() {
                leptos::either::Either::Left(Views::comp(ClonableView::new(true, move || {
                    view_node(&n).attr("data-ftml-comp", "")
                })))
            } else {
                leptos::either::Either::Right(view_node(&n).attr("data-ftml-comp", ""))
            };
            C(view!(<msub>{inner}{this.into_view::<Views>()}</msub>))
        }
        NotationComponent::Comp(n) | NotationComponent::MainComp(n) => {
            let n = n.clone();
            if with_context::<CurrentUri, _>(|_| ()).is_some() {
                H(Views::comp(ClonableView::new(true, move || {
                    view_node(&n).attr("data-ftml-comp", "")
                })))
            } else {
                D(view_node(&n).attr("data-ftml-comp", ""))
            }
        }
        NotationComponent::Argument { index, mode } => {
            let prec = argument_precs.get(*index as usize).copied().unwrap_or(prec);
            E(args.render_arg::<Views, Be>(*index, *mode, prec))
        }
        NotationComponent::ArgSep { index, mode, sep } => {
            let len = args.length_at(*index);
            if len == 0 {
                return F(());
            }
            if len == 1 {
                let prec = argument_precs.get(*index as usize).copied().unwrap_or(prec);
                return E(args.render_arg::<Views, Be>(*index, *mode, prec));
            }

            G(view! {
                <mrow>
                    {args.render_arg_at::<Views,Be>(*index, 0, *mode)}
                {
                    (1..len).map(|i| view!{
                        {sep.iter().map(|s| view_component_with_args::<Views,Be,_>(s,args,this,prec,argument_precs)).collect_view().into_any()}
                        {args.render_arg_at::<Views,Be>(*index, i, *mode)}
                    }).collect_view()
                }
                </mrow>
            })
        }
        NotationComponent::ArgMap { .. } => todo!(),
    }
}

#[derive(Copy, Clone)]
struct DummyRender;
impl ArgumentRender for DummyRender {
    #[inline]
    fn is_empty(&self) -> bool {
        true
    }
    #[inline]
    fn num_args(&self) -> usize {
        0
    }
    fn is_sequence(&self, _: u8) -> bool {
        true
    }
    fn render_arg<Views: FtmlViews, Be: SendBackend>(
        &self,
        index: u8,
        mode: ArgumentMode,
        _: i64,
    ) -> impl IntoView + use<Views, Be> {
        view!(<msub><mi>{mode.as_char()}</mi><mn>{index + 1}</mn></msub>)
    }
    #[inline]
    fn length_at(&self, _index: u8) -> usize {
        3
    }
    fn render_arg_at<Views: FtmlViews, Be: SendBackend>(
        &self,
        index: u8,
        seq_index: usize,
        mode: ArgumentMode,
    ) -> impl IntoView + use<Views, Be> {
        use leptos::either::EitherOf3::{A, B, C};
        match seq_index {
            0 => A(
                view!(<msubsup><mi>{mode.as_char()}</mi><mn>{index + 1}</mn><mn>{1}</mn></msubsup>),
            ),
            1 => B(view!(<mo>"…"</mo>)),
            _ => C(view!(<msubsup>
                 <mi>{mode.as_char()}</mi>
                 <mn>{index + 1}</mn>
                 <msub>
                     <mn>"ℓ"</mn>
                     <mn>{index + 1}</mn>
                 </msub>
             </msubsup>)),
        }
    }
}

pub(crate) fn view_node(n: &NotationNode) -> impl IntoView + use<> {
    let NotationNode {
        tag,
        attributes,
        children,
    } = n;
    attributes
        .iter()
        .fold(
            AnyMaybeAttr::Any(html_from_tag(
                tag.as_ref(),
                children.iter().map(node_or_text).collect_view(),
            )),
            |n, (k, v)| n.attr(k.to_string(), v.to_string()),
        )
        .into_view()
}

fn node_or_text(n: &NodeOrText) -> impl IntoView {
    match n {
        NodeOrText::Node(n) => leptos::either::Either::Left(view_node(n)),
        NodeOrText::Text(t) => leptos::either::Either::Right(t.to_string()),
    }
}

pub(crate) fn html_from_tag(id: &str, children: impl IntoView) -> AnyView {
    macro_rules! tags {
        ( $(  {$($name:ident $($actual:ident)? ),* $(,)? } )*) => {
            match id {
                $( $(
                    stringify!($name) => view!(<$name>{children}</$name>)/* leptos::tachys::html::element::$name()
                        .child(children)*/.into_any(),//tags!(@NAME $name $($actual)?)::TAG,
                )*  )*
                _ => mtext().child("ERROR").into_any()
            }
        };
    }

    tags! {
        {
            //area,base,br,col,embed,hr,img,input,link,meta,source,track,wbr
        }
        {
            a,abbr,address,article,aside,audio,b,bdi,bdo,blockquote,body,
            button,canvas,caption,cite,code,colgroup,data,datalist,dd,
            del,details,dfn,dialog,div,dl,dt,em,fieldset,figcaption,figure,
            footer,form,h1,h2,h3,h4,h5,h6,head,header,hgroup,html,i,iframe,ins,
            kbd,label,legend,li,main,map,mark,menu,meter,nav,noscript,object,
            ol,optgroup,output,p,picture,portal,pre,progress,q,rp,rt,ruby,s,samp,
            script,search,section,select,slot,small,span,strong,style,sub,summary,
            sup,table,tbody,td,template,textarea,tfoot,th,thead,time,title,tr,u,
            ul,var,video
        }
        {option}
        {
            math,mi,mn,mo,ms,mspace,mtext,menclose,merror,mfenced,mfrac,mpadded,
            mphantom,mroot,mrow,msqrt,mstyle,mmultiscripts,mover,mprescripts,
            msub,msubsup,msup,munder,munderover,mtable,mtd,mtr,maction,annotation,
            semantics
        }
        /*{
            a,animate,animateMotion,animateTransform,circle,clipPath,defs,desc,
            discard,ellipse,feBlend,feColorMatrix,feComponentTransfer,
            feComposite,feConvolveMatrix,feDiffuseLighting,feDisplacementMap,
            feDistantLight,feDropShadow,feFlood,feFuncA,feFuncB,feFuncG,feFuncR,
            feGaussianBlur,feImage,feMerge,feMergeNode,feMorphology,feOffset,
            fePointLight,feSpecularLighting,feSpotLight,feTile,feTurbulence,
            filter,foreignObject,g,hatch,hatchpath,image,line,linearGradient,
            marker,mask,metadata,mpath,path,pattern,polygon,polyline,radialGradient,
            rect,script,set,stop,style,svg,switch,symbol,text,textPath,title,
            tspan,view
        }*/
    }
}

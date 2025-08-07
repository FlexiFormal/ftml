mod terms;

use crate::ClonableView;
use crate::document::CurrentUri;
use crate::terms::{ReactiveApplication, ReactiveTerm, TopTerm};
use crate::{DocumentState, FtmlViews};
use ftml_core::extraction::ArgumentPosition;
use ftml_core::{FtmlKey, extraction::VarOrSym};
use ftml_ontology::{
    narrative::elements::{
        Notation,
        notations::{NodeOrText, NotationComponent, NotationNode},
    },
    terms::ArgumentMode,
};
use leptos::math::mtext;
use leptos::{either::Either, prelude::*};
use std::num::NonZeroU8;
pub use terms::*;

pub trait NotationExt {
    fn with_arguments<Views: FtmlViews, R: ArgumentRender + ?Sized>(
        &self,
        head: &VarOrSym,
        args: &R,
    ) -> impl IntoView + use<Self, Views, R>;
    fn with_arguments_safe<Views: FtmlViews, R: ArgumentRender + ?Sized>(
        &self,
        head: &VarOrSym,
        args: &R,
    ) -> impl IntoView + use<Self, Views, R> {
        DocumentState::with_head(head.clone(), move || {
            provide_context(None::<TopTerm>);
            provide_context(None::<ReactiveTerm>);
            self.with_arguments::<Views, R>(head, args)
        })
    }
    fn as_view<Views: FtmlViews>(&self, head: &VarOrSym) -> impl IntoView + use<Self, Views>;
    fn as_view_safe<Views: FtmlViews>(&self, head: &VarOrSym) -> impl IntoView + use<Self, Views> {
        DocumentState::with_head(head.clone(), move || self.as_view::<Views>(head))
    }
    fn as_op<Views: FtmlViews>(&self, head: &VarOrSym) -> impl IntoView + use<Self, Views>;
    fn as_op_safe<Views: FtmlViews>(&self, head: &VarOrSym) -> impl IntoView + use<Self, Views> {
        DocumentState::with_head(head.clone(), move || self.as_op::<Views>(head))
    }
}

pub trait ArgumentRender {
    #[inline]
    fn is_empty(&self) -> bool {
        self.num_args() > 0
    }
    fn num_args(&self) -> usize;
    fn render_arg<Views: FtmlViews>(&self, index: u8, mode: ArgumentMode) -> AnyView;
    fn render_arg_at<Views: FtmlViews>(
        &self,
        index: u8,
        seq_index: usize,
        mode: ArgumentMode,
    ) -> AnyView;
    fn length_at(&self, index: u8) -> usize;
}

fn error() -> AnyView {
    mtext().style("color:red").child("ERROR").into_any()
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
    fn render_arg<Views: FtmlViews>(&self, index: u8, mode: ArgumentMode) -> AnyView {
        use leptos::either::Either::{Left, Right};
        tracing::trace!("rendering arg {index}@{mode:?} of {}", self.len());
        match self.get(index as usize) {
            Some(Either::Left(v)) => do_arg::<Views>(v.clone(), index, None, mode),
            Some(Either::Right(v)) => {
                let len = v.len();
                if len == 0 {
                    return ().into_any();
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
                    return first();
                }
                view! {
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
                }
                .into_any()
            }
            _ => error(),
        }
    }
    fn render_arg_at<Views: FtmlViews>(
        &self,
        index: u8,
        seq_index: usize,
        mode: ArgumentMode,
    ) -> AnyView {
        tracing::trace!(
            "rendering arg sequence@{mode:?} {index}/{seq_index} of {}",
            self.len()
        );
        match self.get(index as usize) {
            Some(Either::Right(v)) => v.get(seq_index).map_or_else(error, |v| {
                do_arg::<Views>(v.clone(), index, Some(seq_index), mode)
            }),
            Some(Either::Left(t)) if seq_index == 0 => {
                do_arg::<Views>(t.clone(), index, Some(seq_index), mode)
            }
            _ => error(),
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
) -> AnyView {
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
        r.add_argument::<Views>(position, v).into_any()
    } else {
        v.into_view::<Views>()
    }
}

impl NotationExt for Notation {
    fn with_arguments<Views: FtmlViews, R: ArgumentRender + ?Sized>(
        &self,
        head: &VarOrSym,
        args: &R,
    ) -> impl IntoView + use<Views, R> {
        use leptos::either::Either::{Left, Right};
        if args.is_empty() {
            return Left(self.as_op::<Views>(head));
        }
        Right(/*owned(move ||*/ {
            let h = head.to_string();
            //provide_context(WithHead(Some(head.clone())));
            let r = view_component_with_args::<Views>(&self.component, args)
                .attr(FtmlKey::Term.attr_name(), "OMID")
                .attr(FtmlKey::Head.attr_name(), h);
            ReactiveApplication::close();
            r
        }) //)
    }

    fn as_op<Views: FtmlViews>(&self, head: &VarOrSym) -> impl IntoView + use<Views> {
        self.op
            .as_ref()
            .map_or_else(
                || view_component_with_args::<Views>(&self.component, &DummyRender),
                |op| {
                    let op = op.clone();
                    if with_context::<CurrentUri, _>(|_| ()).is_some() {
                        Views::comp(ClonableView::new(true, move || {
                            view_node(&op).attr("data-ftml-comp", "")
                        }))
                        .into_any()
                    } else {
                        view_node(&op).attr("data-ftml-comp", "").into_any()
                    }
                },
            )
            .attr(FtmlKey::Term.attr_name(), "OMID")
            .attr(FtmlKey::Head.attr_name(), head.to_string())
    }

    fn as_view<Views: FtmlViews>(&self, head: &VarOrSym) -> impl IntoView + use<Views> {
        //owned(move || {
        let h = head.to_string();
        //provide_context(WithHead(Some(head.clone())));
        view_component_with_args::<Views>(&self.component, &DummyRender)
            .attr(FtmlKey::Term.attr_name(), "OMID")
            .attr(FtmlKey::Head.attr_name(), h)
        //})
    }
}

pub(crate) fn view_component_with_args<Views: FtmlViews>(
    comp: &NotationComponent,
    args: &(impl ArgumentRender + ?Sized),
) -> AnyView {
    match comp {
        NotationComponent::Text(s) => s.to_string().into_any(),
        NotationComponent::Node {
            tag,
            attributes,
            children,
        } => attributes.iter().fold(
            tachys_from_tag(
                tag.as_ref(),
                children
                    .iter()
                    .map(|c| view_component_with_args::<Views>(c, args))
                    .collect_view(),
            ),
            |n, (k, v)| n.attr(k.to_string(), v.to_string()).into_any(),
        ),
        NotationComponent::Comp(n) | NotationComponent::MainComp(n) => {
            let n = n.clone();
            if with_context::<CurrentUri, _>(|_| ()).is_some() {
                Views::comp(ClonableView::new(true, move || {
                    view_node(&n).attr("data-ftml-comp", "")
                }))
                .into_any()
            } else {
                view_node(&n).attr("data-ftml-comp", "").into_any()
            }
        }
        NotationComponent::Argument { index, mode } => args.render_arg::<Views>(*index, *mode),
        NotationComponent::ArgSep { index, mode, sep } => {
            let len = args.length_at(*index);
            if len == 0 {
                return ().into_any();
            }
            if len == 1 {
                return args.render_arg::<Views>(*index, *mode);
            }

            view! {
                <mrow>
                    {args.render_arg_at::<Views>(*index, 0, *mode)}
                {
                    (1..len).map(|i| view!{
                        {sep.iter().map(|s| view_component_with_args::<Views>(s,args)).collect_view()}
                        {args.render_arg_at::<Views>(*index, i, *mode)}
                    }).collect_view()
                }
                </mrow>
            }
            .into_any()
        }
        NotationComponent::ArgMap { .. } => todo!(),
    }
}

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
    fn render_arg<Views: FtmlViews>(&self, index: u8, mode: ArgumentMode) -> AnyView {
        view!(<msub><mi>{mode.as_char()}</mi><mn>{index + 1}</mn></msub>).into_any()
    }
    #[inline]
    fn length_at(&self, _index: u8) -> usize {
        3
    }
    fn render_arg_at<Views: FtmlViews>(
        &self,
        index: u8,
        seq_index: usize,
        mode: ArgumentMode,
    ) -> AnyView {
        match seq_index {
            0 => {
                view!(<msubsup><mi>{mode.as_char()}</mi><mn>{index + 1}</mn><mn>{1}</mn></msubsup>)
                    .into_any()
            }
            1 => view!(<mo>"…"</mo>).into_any(),
            _ => view!(<msubsup>
                 <mi>{mode.as_char()}</mi>
                 <mn>{index + 1}</mn>
                 <msub>
                     <mn>"ℓ"</mn>
                     <mn>{index + 1}</mn>
                 </msub>
             </msubsup>)
            .into_any(),
        }
    }
}

pub(crate) fn view_node(n: &NotationNode) -> AnyView {
    let NotationNode {
        tag,
        attributes,
        children,
    } = n;
    attributes.iter().fold(
        tachys_from_tag(
            tag.as_ref(),
            children.iter().map(node_or_text).collect_view(),
        ),
        |n, (k, v)| n.attr(k.to_string(), v.to_string()).into_any(),
    )
}

fn node_or_text(n: &NodeOrText) -> AnyView {
    match n {
        NodeOrText::Node(n) => view_node(n),
        NodeOrText::Text(t) => t.to_string().into_any(),
    }
}

pub(crate) fn tachys_from_tag(id: &str, children: impl IntoView) -> AnyView {
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

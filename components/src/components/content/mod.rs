pub mod documents;
pub mod domain;
pub mod paragraphs;
pub mod symbols;

use crate::{config::FtmlConfig, utils::LocalCacheExt};
use ftml_backend::{FtmlBackend, GlobalBackend};
use ftml_dom::{
    FtmlViews,
    utils::{
        css::{CssExt, inject_css},
        local_cache::{LocalCache, SendBackend},
    },
};
use ftml_ontology::terms::VarOrSym;
use ftml_uris::{DocumentElementUri, DocumentUri, IsDomainUri, ModuleUri, SymbolUri};
use leptos::prelude::*;

pub trait FtmlViewable {
    fn as_view<Be: SendBackend>(&self) -> impl IntoView + use<Self, Be> + 'static;
}
impl<F: FtmlViewable> FtmlViewable for &F {
    #[allow(refining_impl_trait_reachable)]
    fn as_view<Be: SendBackend>(&self) -> impl IntoView + use<F, Be> + 'static {
        F::as_view::<Be>(self)
    }
}

pub struct CommaSep<V: IntoView + 'static, I: IntoIterator<Item = V>>(pub &'static str, pub I);

impl<V: IntoView + 'static, I: IntoIterator<Item = V>> CommaSep<V, I> {
    pub fn into_view(self) -> impl IntoView + use<V, I> + 'static {
        use thaw::Text;
        let mut elems = self.1.into_iter();
        let first = elems.next()?;
        let v = elems.map(|e| view!(", "{e.into_view()})).collect_view();
        Some(view! {
          <div style="display:inline-block;width:max-content;">
            <Text>{self.0}": "</Text>
            {first.into_view()}
            {v}
          </div>
        })
    }
}

#[inline]
#[must_use]
pub fn uses<'a, Be: SendBackend, I: IntoIterator<Item = &'a ModuleUri>>(
    header: &'static str,
    uses: I,
) -> impl IntoView + 'static {
    CommaSep(header, uses.into_iter().map(FtmlViewable::as_view::<Be>)).into_view()
}

impl FtmlViewable for DocumentUri {
    fn as_view<Be: SendBackend>(&self) -> impl IntoView + use<Be> {
        use thaw::Text;
        let uristring = self.to_string();
        let name = self.name.to_string();
        view! {
          <div style="display:inline-block;">
            <Text class="ftml-comp" attr:title=uristring>{name}</Text>
            <a
                style="display:inline-block;"
                target="_blank"
                href={<Be as GlobalBackend>::get().document_link_url(self)}
            >
                <thaw::Icon icon=icondata_bi::BiLinkRegular />
            </a>
          </div>
        }
    }
}

impl FtmlViewable for DocumentElementUri {
    fn as_view<Be: SendBackend>(&self) -> impl IntoView + use<Be> {
        use thaw::Text;
        let name = self.name.last().to_string();
        let title = view!(<Text class="ftml-comp">{name}</Text>);
        hover_paragraph::<Be>(self.clone(), title)
    }
}

pub fn hover_paragraph<Be: SendBackend>(
    uri: DocumentElementUri,
    title: impl IntoView + 'static,
) -> impl IntoView {
    use thaw::{Popover, PopoverTrigger};
    let uristring = uri.to_string();
    inject_css("ftml-symbol-popup", include_str!("../popup.css"));

    view! {
        <Popover>
          <PopoverTrigger slot>{title}</PopoverTrigger>
          <div style="font-size:small;">{uristring}</div>
          <div style="margin-bottom:5px;"><thaw::Divider/></div>
          <div class="ftml-symbol-popup">
          {
              LocalCache::with_or_err::<Be,_,_,_,_>(
                  |b| b.get_fragment(uri.into(), None),
                  |(html,css,_)| {
                      for c in css {
                          c.inject();
                      }
                      crate::Views::<Be>::render_ftml(html.into_string(),None)
                  },
                  |e| view!(<code>{e.to_string()}</code>)
              )
          }
          </div>
        </Popover>
    }
}

impl FtmlViewable for ModuleUri {
    fn as_view<Be: SendBackend>(&self) -> impl IntoView + use<Be> {
        use thaw::{Popover, PopoverTrigger, Text};
        let name = self.module_name().to_string();
        let uri = self.to_string();
        view! {<Popover>
            <PopoverTrigger slot>
                <Text class="ftml-comp">{name}</Text>
            </PopoverTrigger>
            <Text>{uri}</Text>
        </Popover>}
        /*
        use flams_web_utils::components::{OnClickModal, Popover, PopoverTrigger};
        use thaw::Scrollbar;
        let name = uri.module_name().last().to_string();
        let uristring = uri.to_string();
        let uriclone = uri.clone();
        let uri = uri.clone();
        view! {
          <div style="display:inline-block;"><Popover>
            <PopoverTrigger slot><b class="ftml-comp">{name}</b></PopoverTrigger>
            <OnClickModal slot><Scrollbar style="max-height:80vh">{
              crate::remote::get!(omdoc(uriclone.clone().into()) = (css,s) => {
                for c in css { do_css(c); }
                s.into_view()
              })
            }</Scrollbar></OnClickModal>
            <div style="font-size:small;">{uristring}</div>
            <div style="margin-bottom:5px;"><thaw::Divider/></div>
            <Scrollbar style="max-height:300px">
            {
              crate::remote::get!(omdoc(uri.clone().into()) = (css,s) => {
                for c in css { do_css(c); }
                s.into_view()
              })
            }
            </Scrollbar>
          </Popover></div>
        } */
    }
}

impl FtmlViewable for SymbolUri {
    fn as_view<Be: SendBackend>(&self) -> impl IntoView + use<Be> + 'static {
        symbol_uri::<Be>(self.name().last().to_string(), self)
    }
}

pub fn symbol_uri<Be: SendBackend>(
    name: String,
    uri: &SymbolUri,
) -> impl IntoView + use<Be> + 'static {
    use leptos::either::Either::{Left, Right};
    use thaw::Text;
    inject_css("ftml-comp", include_str!("../comp.css"));
    if !FtmlConfig::allow_hovers() {
        tracing::trace!("hovers disabled");
        return Left(view!(<Text class="ftml-comp">{name}</Text>));
    }
    let vos = VarOrSym::Sym(uri.clone());
    Right(super::terms::comp_like::<Be, _>(
        vos,
        None,
        false,
        move || view!(<Text>{name}</Text>),
    ))
}

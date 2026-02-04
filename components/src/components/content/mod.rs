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
use ftml_ontology::terms::{VarOrSym, Variable};
use ftml_uris::{DocumentElementUri, DocumentUri, IsDomainUri, LeafUri, ModuleUri, SymbolUri};
use leptos::prelude::*;

pub trait FtmlViewable {
    fn as_view<Be: SendBackend>(&self) -> AnyView;
}
impl<F: FtmlViewable> FtmlViewable for &F {
    #[allow(refining_impl_trait_reachable)]
    fn as_view<Be: SendBackend>(&self) -> AnyView {
        F::as_view::<Be>(self)
    }
}
impl FtmlViewable for LeafUri {
    fn as_view<Be: SendBackend>(&self) -> AnyView {
        match self {
            Self::Symbol(s) => s.as_view::<Be>(),
            Self::Element(v) => variable_uri::<Be>(v.name().last().to_string(), v),
        }
    }
}

pub struct CommaSep<V: IntoView + 'static, I: IntoIterator<Item = V>>(pub &'static str, pub I);

impl<V: IntoView + 'static, I: IntoIterator<Item = V>> CommaSep<V, I> {
    pub fn into_view(self) -> AnyView {
        use thaw::Text;
        let mut elems = self.1.into_iter();
        let Some(first) = elems.next() else {
            return ().into_any();
        };
        let v = elems.map(|e| view!(", "{e.into_view()})).collect_view();
        view! {
          <div style="display:inline-block;width:max-content;">
            <Text>{self.0}": "</Text>
            {first.into_view()}
            {v}
          </div>
        }
        .into_any()
    }
}

#[inline]
#[must_use]
pub fn uses<'a, Be: SendBackend, I: IntoIterator<Item = &'a ModuleUri>>(
    header: &'static str,
    uses: I,
) -> AnyView {
    CommaSep(header, uses.into_iter().map(FtmlViewable::as_view::<Be>)).into_view()
}

impl FtmlViewable for DocumentUri {
    fn as_view<Be: SendBackend>(&self) -> AnyView {
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
        .into_any()
    }
}

impl FtmlViewable for DocumentElementUri {
    fn as_view<Be: SendBackend>(&self) -> AnyView {
        use thaw::Text;
        let name = self.name.last().to_string();
        let title = view!(<Text class="ftml-comp">{name}</Text>).into_any();
        hover_paragraph::<Be>(self.clone(), title)
    }
}

#[must_use]
pub fn hover_paragraph<Be: SendBackend>(uri: DocumentElementUri, title: AnyView) -> AnyView {
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
              LocalCache::with_or_err::<Be,_,_>(
                  |b| b.get_fragment(uri.into(), None),
                  |(html,css,_)| {
                      for c in css {
                          c.inject();
                      }
                      crate::Views::<Be>::render_ftml(html.into_string(),None).into_any()
                  },
                  |e| view!(<code>{e.to_string()}</code>).into_any()
              )
          }
          </div>
        </Popover>
    }
    .into_any()
}

#[must_use]
pub fn module_with_hover(uri: &ModuleUri) -> AnyView {
    use thaw::{Popover, PopoverTrigger, Text};
    let name = uri.module_name().to_string();
    let uri = uri.to_string();
    view! {
        <Popover>
            <PopoverTrigger slot>
                <Text class="ftml-comp">{name}</Text>
            </PopoverTrigger>
            <Text>{uri}</Text>
        </Popover>
    }
    .into_any()
}

impl FtmlViewable for ModuleUri {
    fn as_view<Be: SendBackend>(&self) -> AnyView {
        use thaw::{Dialog, DialogSurface, Popover, PopoverTrigger, Scrollbar, Text};
        let name = self.module_name().to_string();
        let uri = self.to_string();
        let on_click = RwSignal::new(false);
        let origuri = self.clone();

        view! {
        <Dialog open = on_click>
            <DialogSurface>{
                LocalCache::with_or_toast::<Be,_,_>(move |c| c.get_module(origuri),
                |m| view!{
                    <Scrollbar style="max-height:75vh;">{m.as_view::<Be>()}</Scrollbar>
                }.into_any(),
                || view!(<Text style="color:red">"Error"</Text>).into_any()
                )
            }</DialogSurface>
        </Dialog>
        <Popover>
            <PopoverTrigger slot>
                <Text class="ftml-comp" on:click=move|_| on_click.set(true)>{name}</Text>
            </PopoverTrigger>
            <Text>{uri}</Text>
        </Popover>
        }
        .into_any()
    }
}

impl FtmlViewable for SymbolUri {
    fn as_view<Be: SendBackend>(&self) -> AnyView {
        symbol_uri::<Be>(self.name().last().to_string(), self)
    }
}

pub fn symbol_uri<Be: SendBackend>(name: String, uri: &SymbolUri) -> AnyView {
    use thaw::Text;
    inject_css("ftml-comp", include_str!("../comp.css"));
    if !FtmlConfig::allow_hovers() {
        tracing::trace!("hovers disabled");
        return view!(<Text class="ftml-comp">{name}</Text>).into_any();
    }
    let vos = VarOrSym::Sym(uri.clone());
    super::terms::comp_like::<Be, _>(vos, None, false, move || view!(<Text>{name}</Text>))
        .into_any()
}

pub fn variable_uri<Be: SendBackend>(name: String, uri: &DocumentElementUri) -> AnyView {
    use thaw::Text;
    inject_css("ftml-comp", include_str!("../comp.css"));
    if !FtmlConfig::allow_hovers() {
        tracing::trace!("hovers disabled");
        return view!(<Text class="ftml-comp">{name}</Text>).into_any();
    }
    let vos = VarOrSym::Var(Variable::Ref {
        declaration: uri.clone(),
        is_sequence: None,
    });
    super::terms::comp_like::<Be, _>(vos, None, false, move || view!(<Text>{name}</Text>))
        .into_any()
}

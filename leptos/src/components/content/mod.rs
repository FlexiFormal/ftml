pub mod symbols;

use ftml_backend::{FtmlBackend, GlobalBackend};
use ftml_dom::{
    FtmlViews,
    utils::{
        css::CssExt,
        local_cache::{LocalCache, SendBackend},
    },
};
use ftml_uris::{DocumentElementUri, DocumentUri, Uri};
use leptos::prelude::*;

use crate::utils::LocalCacheExt;

pub trait FtmlViewable {
    fn as_view<Be: SendBackend>(&self) -> impl IntoView + use<Self, Be>;
}

pub struct CommaSep<V: FtmlViewable, I: IntoIterator<Item = V>>(pub &'static str, pub I);

impl<V: FtmlViewable, I: IntoIterator<Item = V>> CommaSep<V, I> {
    pub fn into_view<Be: SendBackend>(self) -> impl IntoView + use<Be, V, I> {
        let mut elems = self.1.into_iter();
        let first = elems.next()?;
        Some(view! {
          <div style="display:inline-block;width:max-content;">
            {self.0}
            ": "
            {first.as_view::<Be>()}
            {elems.map(|e| view!(", "{e.as_view::<Be>()})).collect_view()}
          </div>
        })
    }
}

impl FtmlViewable for DocumentUri {
    fn as_view<Be: SendBackend>(&self) -> impl IntoView + use<Be> {
        let uristring = self.to_string();
        let name = self.name.to_string();
        view! {
          <div style="display:inline-block;">
            <span class="ftml-comp" title=uristring>{name}</span>
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
        use thaw::{Popover, PopoverTrigger};
        let uristring = self.to_string();
        let name = self.name.last().to_string();
        let uri = Uri::DocumentElement(self.clone());

        view! {
          //<div style="display:inline-block;">
            <div style="display:inline-block;"><Popover>
              <PopoverTrigger slot><span class="ftml-comp">{name}</span></PopoverTrigger>
              <div style="font-size:small;">{uristring}</div>
              <div style="margin-bottom:5px;"><thaw::Divider/></div>
              <div style="background-color:white;color:black;">
              {
                  LocalCache::with_or_err::<Be,_,_,_,_>(
                      |b| b.get_fragment(uri, None),
                      |(html,css)| {
                          for c in css {
                              c.inject();
                          }
                          crate::Views::<Be>::render_ftml(html)
                      },
                      |e| view!(<code>{e.to_string()}</code>)
                  )
              }
              </div>
            </Popover></div>
        }
    }
}

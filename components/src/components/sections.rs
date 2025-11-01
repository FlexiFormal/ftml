use crate::{
    config::FtmlConfig,
    utils::collapsible::{collapse_marker, fancy_collapsible},
};
use ftml_dom::{
    DocumentState,
    counters::LogicalLevel,
    structure::SectionInfo,
    utils::{css::inject_css, get_true_rect},
};
use ftml_ontology::narrative::elements::SectionLevel;
use leptos::prelude::*;
use leptos_posthoc::OriginalNode;
use send_wrapper::SendWrapper;
/*
pub fn section<V: IntoView>(info: SectionInfo, children: impl FnOnce() -> V) -> impl IntoView {
    inject_css("ftml-sections", include_str!("sections.css"));
    let SectionInfo {
        id,
        style,
        class,
        lvl,
        uri,
    } = info;
    tracing::debug!("section {id} at level {lvl:?}");

    view! {
        <div id=id style=style class=class>
          {
            if let LogicalLevel::Section(lvl) = lvl {
                tracing::trace!("Section at level {lvl}");
            }
            FtmlConfig::wrap_section(&uri,children)
          }
        </div>
    }
}

pub fn section_title(
    lvl: SectionLevel,
    class: &'static str,
    children: OriginalNode,
) -> impl IntoView {
    tracing::debug!("section title at level {lvl:?}");
    view! {
      {children.attr("class",class)}
      {
          FtmlConfig::insert_section_title(lvl)
      }
    }
}
 */

#[derive(Default, Clone)]
struct SectionTitle(Option<(&'static str, SendWrapper<OriginalNode>)>);

#[allow(clippy::needless_pass_by_value)]
pub fn section<V: IntoView>(info: SectionInfo, children: impl FnOnce() -> V) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    inject_css("ftml-sections", include_str!("sections.css"));
    let lvl = info.level();
    tracing::debug!("section {} at level {lvl:?}", info.id);
    let title = RwSignal::new(SectionTitle::default());
    provide_context(title);
    if let LogicalLevel::Section(lvl) = lvl {
        tracing::trace!("Section at level {lvl}");
        if lvl <= SectionLevel::Paragraph {
            return Left(view! {
                <div id=info.id.to_string() style=info.style() class=info.class()>
                  {
                    FtmlConfig::wrap_section(&info.uri,Some(lvl),children)
                  }
                </div>
            });
        }
    }
    let visible = RwSignal::new(true);
    let uri = info.uri.clone();
    let inner = fancy_collapsible(
        move || {
            FtmlConfig::wrap_section(
                &uri,
                if let LogicalLevel::Section(lvl) = lvl {
                    Some(lvl)
                } else {
                    None
                },
                children,
            )
        },
        visible,
        "",
        "",
    );
    let title = move || {
        title.get().0.map(|(class, title)| {
            /*
            let pos_ref = NodeRef::new();
            let marker_ref = NodeRef::new();
            let _ = Effect::new(move || {
                if let Some(pos) = pos_ref.get()
                    && let Some(marker) = marker_ref.get()
                {
                    position_marker(&pos, &marker);
                }
            });
            */
            view! {
                <div /*node_ref=pos_ref*/ style="display:contents;">
                    {title.take().attr("class",class)}
                </div>
                <div /*node_ref=marker_ref*/ on:click=move |_| visible.set(!visible.get_untracked())
                    style="width:0;height:0;left:-15px;top:-15px;position:relative;"
                >
                    {collapse_marker(visible,false)}
                </div>
            }
        })
    };

    Right(view! {
        <div id=info.id.to_string() style=info.style() class=info.class()>
            {title}
            {inner}
        </div>
    })
}

pub fn section_title(class: &'static str, children: OriginalNode) -> impl IntoView {
    use leptos::either::Either::{Left, Right};
    let lvl = DocumentState::current_section_level();
    let lvl = if let LogicalLevel::Section(lvl) = lvl {
        lvl
    } else {
        SectionLevel::Subparagraph
    };
    tracing::debug!("section title at level {lvl:?}");
    if lvl <= SectionLevel::Paragraph {
        return Left(view! {
          {children.attr("class",class)}
          {
              FtmlConfig::insert_section_title(lvl)
          }
        });
    }
    let ttl = expect_context::<RwSignal<SectionTitle>>();
    ttl.set(SectionTitle(Some((class, SendWrapper::new(children)))));
    Right(FtmlConfig::insert_section_title(lvl))
}

use leptos::web_sys::HtmlDivElement;
pub fn position_marker(pos: &HtmlDivElement, marker: &HtmlDivElement) {
    let mut elem: leptos::web_sys::Element = (***pos).clone();
    let ht = get_true_rect(marker).height();
    let mut rect = get_true_rect(pos);
    loop {
        if rect.height() == 0.0 {
            if let Some(fc) = elem.first_element_child() {
                elem = fc;
                rect = get_true_rect(&elem);
                continue;
            }
            if let Some(fc) = elem.next_element_sibling() {
                elem = fc;
                rect = get_true_rect(&elem);
                continue;
            }
        }
        break;
    }
    let target_height = ((rect.height() - ht) / 2.0).max(0.0); //+ (rect.height() / 2.0);
    let ht = target_height; //ht.max(target_height);
    let font_size = rect.height().max(15.0);
    let _ = marker.set_attribute(
        "style",
        &format!(
            "display:block;\
            position:absolute;\
            width:fit-content;\
            margin-top:{ht}px;\
            margin-left:-1em;\
            font-size:{font_size}px"
        ),
    );
}

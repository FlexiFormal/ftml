use crate::config::FtmlConfig;
use ftml_dom::{counters::LogicalLevel, markers::SectionInfo, utils::css::inject_css};
use ftml_ontology::narrative::elements::SectionLevel;
use leptos::prelude::*;
use leptos_posthoc::OriginalNode;

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

/* Collapsible; needs debugging:

#[derive(Default, Clone)]
struct SectionTitle(Option<(&'static str, SendWrapper<OriginalNode>)>);

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
    let title = RwSignal::new(SectionTitle::default());
    provide_context(title);
    if let LogicalLevel::Section(lvl) = lvl {
        tracing::trace!("Section at level {lvl}");
    }
    let visible = RwSignal::new(true);
    let inner = fancy_collapsible(
        move || FtmlConfig::wrap_section(&uri, children),
        visible,
        "",
        "",
    );
    let title = move || {
        title.get().0.map(|(class, title)| {
            use thaw::Flex;
            view! {
                <Flex>
                    <a on:click=move |_| visible.set(!visible.get_untracked())>
                        {collapse_marker(visible)}
                    </a>
                    {title.take().attr("class",class)}
                </Flex>
            }
        })
    };

    view! {
        <div id=id style=style class=class>
            {title}
            {inner}
        </div>
    }
}

pub fn section_title(
    lvl: SectionLevel,
    class: &'static str,
    children: OriginalNode,
) -> impl IntoView {
    tracing::debug!("section title at level {lvl:?}");
    let ttl = expect_context::<RwSignal<SectionTitle>>();
    ttl.set(SectionTitle(Some((class, SendWrapper::new(children)))));
    FtmlConfig::insert_section_title(lvl)
}

 */

use crate::config::FtmlConfig;
use ftml_dom::{counters::LogicalLevel, markers::SectionInfo, utils::css::inject_css};
use ftml_ontology::narrative::elements::SectionLevel;
use leptos::prelude::*;

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

pub fn section_title<V: IntoView>(
    lvl: SectionLevel,
    class: &'static str,
    children: impl FnOnce() -> V,
) -> impl IntoView {
    tracing::debug!("section title at level {lvl:?}");
    view! {
      <div class=class>{children()}</div>
      {
          FtmlConfig::insert_section_title(lvl)
      }
    }
}

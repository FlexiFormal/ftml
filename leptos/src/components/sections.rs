use ftml_dom::{counters::LogicalLevel, markers::SectionInfo, utils::css::inject_css};
use leptos::prelude::*;

pub fn section<V: IntoView>(info: SectionInfo, children: impl FnOnce() -> V) -> impl IntoView {
    inject_css("ftml-sections", include_str!("sections.css"));
    let SectionInfo {
        id,
        style,
        class,
        lvl,
        ..
    } = info;
    view! {
        <div id=id style=style class=class>
          {
            if let LogicalLevel::Section(lvl) = lvl {
                tracing::info!("Section at level {lvl}");
            }
            children()
          }
          //{end}
        </div>
    }
}

use ftml_dom::{counters::LogicalLevel, markers::SectionInfo, utils::css::inject_css};
use ftml_ontology::narrative::elements::SectionLevel;
use leptos::prelude::*;

use crate::config::FtmlConfigState;

pub fn section<V: IntoView>(info: SectionInfo, children: impl FnOnce() -> V) -> impl IntoView {
    inject_css("ftml-sections", include_str!("sections.css"));
    let SectionInfo {
        id,
        style,
        class,
        lvl,
        uri,
    } = info;
    view! {
        <div id=id style=style class=class>
          {
            if let LogicalLevel::Section(lvl) = lvl {
                tracing::trace!("Section at level {lvl}");
            }
            FtmlConfigState::wrap_section(&uri,children)
          }
        </div>
    }
}

pub fn section_title<V: IntoView>(
    lvl: SectionLevel,
    class: &'static str,
    children: impl FnOnce() -> V,
) -> impl IntoView {
    view! {
      <div class=class>{children()}</div>
      {
          FtmlConfigState::insert_section_title(lvl)
      }
    }
}

/*
pub fn section_title<V: IntoView + 'static>(
    children: impl FnOnce() -> V + Send + 'static,
) -> impl IntoView {
    let counters: SectionCounters = expect_context();
    let (begin, cls) = match counters.current_level() {
        LogicalLevel::Section(l) => (
            if let Some(NarrativeUri::Element(uri)) = use_context() {
                expect_context::<Option<OnSectionTitle>>()
                    .map(|s| TsCont::res_into_view(s.0.apply(&(uri, l))))
            } else {
                tracing::error!("Sectioning error");
                None
            },
            match l {
                SectionLevel::Part => "ftml-title-part",
                SectionLevel::Chapter => "ftml-title-chapter",
                SectionLevel::Section => "ftml-title-section",
                SectionLevel::Subsection => "ftml-title-subsection",
                SectionLevel::Subsubsection => "ftml-title-subsubsection",
                SectionLevel::Paragraph => "ftml-title-paragraph",
                SectionLevel::Subparagraph => "ftml-title-subparagraph",
            },
        ),
        LogicalLevel::BeamerSlide => (None, "ftml-title-slide"),
        LogicalLevel::Paragraph => (None, "ftml-title-paragraph"),
        _ => (None, "ftml-title"),
    };
    view! {
      <div class=cls>{children()}</div>
      {begin}
    }
}
 */

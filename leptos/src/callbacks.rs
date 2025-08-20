use ftml_ontology::narrative::elements::{SectionLevel, paragraphs::ParagraphKind};
use ftml_uris::DocumentElementUri;
use leptos::prelude::*;

leptos_react::wrapper!(SectionWrap(u:DocumentElementUri));
leptos_react::wrapper!(ParagraphWrap(u:DocumentElementUri,kind:ParagraphKind));
leptos_react::wrapper!(SlideWrap(u:DocumentElementUri));
leptos_react::insertion!(OnSectionTitle(u:DocumentElementUri,lvl: SectionLevel));

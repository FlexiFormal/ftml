use std::hint::unreachable_unchecked;

#[allow(clippy::wildcard_imports)]
use super::ever::*;
use crate::HtmlExtractor;
use ftml_core::{
    FtmlKey,
    extraction::{
        CloseFtmlElement, FtmlExtractionError, FtmlExtractor, KeyList, attributes::Attributes,
        nodes::FtmlNode,
    },
};
use ftml_ontology::{narrative::DocumentRange, utils::Css};
use html5ever::{
    QualName,
    interface::{NodeOrText, TreeSink},
    tendril::StrTendril,
};
use smallvec::SmallVec;

pub struct HtmlParser<Img: Fn(&str) -> Option<String>, CS: Fn(&str) -> Option<Box<str>>> {
    document_node: NodeRef,
    //rel_path: &'p str,
    extractor: std::cell::RefCell<HtmlExtractor>,
    body: std::cell::Cell<(DocumentRange, usize)>,
    errors: std::cell::RefCell<Vec<FtmlExtractionError>>,
    img: Img,
    /*
    let path = std::path::Path::new(src);
    if let Some(newsrc) =
        self.extractor.borrow().backend.archive_of(path, |a, rp| {
            format!("srv:/img?a={}&rp={}", a.id(), &rp[1..])
        })
    {
        attributes.set("src", "");
        attributes.new_attr("data-flams-src", newsrc);
    } else {
        let kpsewhich = &*tex_engine::engine::filesystem::kpathsea::KPATHSEA;
        let last = src.rsplit_once('/').map_or(src, |(_, p)| p);
        if let Some(file) = kpsewhich.which(last) {
            if file == path {
                let file = format!("srv:/img?kpse={last}");
                attributes.set("src", "");
                attributes.new_attr("data-flams-src", file);
            }
        } else {
            let file = format!("srv:/img?file={src}");
            attributes.set("src", "");
            attributes.new_attr("data-flams-src", file);
        }
    };
    */
    css: CS,
    /*
    static CSS_SUBSTS: [(&str, &str); 1] = [(
        "https://raw.githack.com/Jazzpirate/RusTeX/main/rustex/src/resources/rustex.css",
        "srv:/rustex.css",
    )];
     */
}

impl<Img: Fn(&str) -> Option<String>, CS: Fn(&str) -> Option<Box<str>>> TreeSink
    for HtmlParser<Img, CS>
{
    type Handle = NodeRef;
    type Output = (); //Result<(OMDocResult, String), String>;
    type ElemName<'a>
        = &'a QualName
    where
        Self: 'a;

    fn finish(self) -> Self::Output {
        /*
        for c in self.document_node.children() {
            self.pop(&c);
        }
        let mut html = Vec::new();
        let HtmlExtractor {
            errors,
            mut css,
            refs,
            title,
            //triples,
            state,
            //backend,
            ..
        } = self.extractor.into_inner();
        if !errors.is_empty() {
            return (); //Err(errors);
        }
        css = Css::merge(std::mem::take(&mut css));
        let Ok((uri, elems, modules, styles)) = state.take() else {
            return Err("Unbalanced FTML document".to_string());
        };

        let _ = html5ever::serialize(&mut html, &self.document_node, SerializeOpts::default());
        let html = String::from_utf8_lossy(&html).into();
        backend.submit_triples(&uri, self.rel_path, triples.into_iter());
        let (body, inner_offset) = self.body.get();
        Ok((
            OMDocResult {
                document: UncheckedDocument {
                    uri,
                    title,
                    styles,
                    elements: elems,
                },
                html: HTMLData {
                    html,
                    css,
                    refs,
                    body,
                    inner_offset,
                },
                modules,
            },
            errors,
        ))
         */
    }

    #[inline]
    fn parse_error(&self, msg: std::borrow::Cow<'static, str>) {
        self.extractor.borrow_mut().errors.push_str(&msg);
    }
    #[inline]
    fn get_document(&self) -> Self::Handle {
        self.document_node.clone()
    }
    #[inline]
    fn set_quirks_mode(&self, mode: html5ever::interface::QuirksMode) {
        let NodeData::Document(r) = self.document_node.data() else {
            unreachable!()
        };
        r.set(mode);
    }

    #[inline]
    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x == y
    }

    #[inline]
    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> Self::ElemName<'a> {
        &target.as_element().unwrap_or_else(|| unreachable!()).name
    }

    #[inline]
    fn create_element(
        &self,
        name: QualName,
        attrs: Vec<html5ever::Attribute>,
        _: html5ever::interface::ElementFlags,
    ) -> Self::Handle {
        NodeRef::new_element(name, attrs.into())
    }
    #[inline]
    fn create_comment(&self, text: StrTendril) -> NodeRef {
        NodeRef::new_comment(text)
    }
    #[inline]
    fn create_pi(&self, target: StrTendril, data: StrTendril) -> Self::Handle {
        NodeRef::new_processing_instruction(target, data)
    }

    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::too_many_lines)]
    fn append(&self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        if let Some(e) = parent.last_child() {
            self.pop(&e);
        }
        match child {
            NodeOrText::AppendNode(child) => {
                if child
                    .as_element()
                    .is_some_and(|n| n.name.local.as_ref().eq_ignore_ascii_case("img"))
                {
                    let Some(child_elem) = child.as_element() else {
                        // SAFETY: we literally just checked that
                        unsafe { unreachable_unchecked() }
                    };
                    let mut attributes = child_elem.attributes.borrow_mut();
                    if let Some(src) = attributes.value("src") {
                        if let Some(newsrc) = (self.img)(src) {
                            attributes.set("src", "");
                            attributes.new_attr("data-ftml-src", newsrc);
                            drop(attributes);
                            NodeRef::update_len(child_elem);
                        }
                    }
                }
                if parent.as_document().is_some() {
                    if let Some(child_elem) = child.as_element() {
                        let new_start = parent.len();
                        let len = child.len();
                        child_elem.start_offset.set(new_start);
                        child_elem.end_offset.set(new_start + len);
                    }
                } else if let Some(parent_elem) = parent.as_element() {
                    let new_start = parent_elem.end_offset.get() - tag_len(&parent_elem.name) - 1;
                    if let Some(child_elem) = child.as_element() {
                        {
                            let mut attributes = child_elem.attributes.borrow_mut();
                            let mut extractor = self.extractor.borrow_mut();
                            let rules: KeyList = attributes
                                .0
                                .iter()
                                .filter_map(|(k, _)| {
                                    if k.local.starts_with(ftml_core::PREFIX) {
                                        FtmlKey::from_attr(&k.local)
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            if !rules.is_empty() {
                                let mut closes = SmallVec::<_, 2>::new();
                                for r in rules.apply(&mut *extractor, &mut *attributes, &child) {
                                    match r {
                                        Ok(((), c)) => {
                                            if let Some(c) = c {
                                                closes.push(c);
                                            }
                                        }
                                        Err(e) => self.errors.borrow_mut().push(e),
                                    }
                                }
                                drop(attributes);
                                if !closes.is_empty() {
                                    closes.reverse();
                                    update_attributes(&closes, child_elem);
                                    child_elem.ftml.set(closes);
                                }
                                NodeRef::update_len(child_elem);
                            }
                        }
                        let len = child.len();
                        child_elem.start_offset.set(new_start);
                        child_elem.end_offset.set(new_start + len);
                    }
                    prolong(parent, child.len() as isize);
                }
                parent.append(child);
            }
            NodeOrText::AppendText(text) => {
                if let Some(elem) = parent.as_element() {
                    let len = if matches!(
                        &*elem.name.local,
                        "style"
                            | "script"
                            | "xmp"
                            | "iframe"
                            | "noembed"
                            | "noframes"
                            | "plaintext"
                            | "noscript"
                    ) {
                        text.as_bytes().len()
                    } else {
                        escaped_len(&text, false)
                    };
                    prolong(parent, len as isize);
                }
                if let Some(last_child) = parent.last_child() {
                    if let Some(existing) = last_child.as_text() {
                        existing.borrow_mut().extend(text.chars());
                        return;
                    }
                }
                parent.append(NodeRef::new_text(text));
            }
        }
    }

    #[inline]
    fn append_doctype_to_document(
        &self,
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    ) {
        self.document_node
            .append(NodeRef::new_doctype(name, public_id, system_id));
    }

    #[inline]
    fn append_based_on_parent_node(
        &self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        if element.parent().is_some() {
            self.append_before_sibling(element, child);
        } else {
            self.append(prev_element, child);
        }
    }

    fn pop(&self, node: &Self::Handle) {
        let Some(elem) = node.as_element() else {
            return;
        };
        if elem.closed.get() {
            return;
        }
        elem.closed.set(true);
        for c in node.children() {
            self.pop(&c);
        }
        if &elem.name.local == "body" {
            let range = DocumentRange {
                start: elem.start_offset.get(),
                end: elem.end_offset.get(),
            };
            let off = elem.attributes.borrow().len();
            self.body.set((range, "<body>".len() + off));
        } else if matches!(&*elem.name.local, "link" | "style") {
            if let Some(p) = node.parent() {
                if let Some(pe) = p.as_element() {
                    if &pe.name.local == "head" {
                        match &*elem.name.local {
                            "link" => {
                                let attrs = elem.attributes.borrow();
                                if attrs.value("rel") == Some("stylesheet") {
                                    if let Some(lnk) = attrs.value("href") {
                                        let val = (self.css)(lnk)
                                            .unwrap_or_else(|| lnk.to_string().into_boxed_str());
                                        self.extractor.borrow_mut().css.push(Css::Link(val));
                                        node.delete();
                                        return;
                                    }
                                }
                            }
                            "style" => {
                                let str = node
                                    .children()
                                    .filter_map(|c| c.as_text().map(|s| s.borrow().to_string()))
                                    .collect::<String>();
                                // update: will get sorted / processed in bulk later
                                self.extractor
                                    .borrow_mut()
                                    .css
                                    .push(Css::Inline(str.into())); //.extend(CSS::split(&str));
                                node.delete();
                                return;
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }
        }
        let closes = elem.ftml.take();
        if !closes.is_empty() {
            let mut extractor = self.extractor.borrow_mut();
            for c in closes {
                if let Err(e) = extractor.close(c, node) {
                    self.errors.borrow_mut().push(e);
                }
            }
        }
    }

    #[inline]
    fn append_before_sibling(&self, _sibling: &Self::Handle, _child: NodeOrText<Self::Handle>) {
        unreachable!()
    }

    #[inline]
    fn remove_from_parent(&self, _target: &Self::Handle) {
        unreachable!()
    }
    #[inline]
    fn reparent_children(&self, _node: &Self::Handle, _new_parent: &Self::Handle) {
        unreachable!()
    }
    #[inline]
    fn mark_script_already_started(&self, _node: &Self::Handle) {
        unreachable!()
    }
    fn get_template_contents(&self, _target: &Self::Handle) -> Self::Handle {
        unreachable!()
    }
    #[inline]
    fn add_attrs_if_missing(&self, _target: &Self::Handle, _attrs: Vec<html5ever::Attribute>) {
        unreachable!()
    }
}

const fn update_attributes(_elements: &[CloseFtmlElement], _child: &ElementData) {
    /*
    let mut attrs = child.attributes.borrow_mut();
    for e in &elements.elems {
        match e {
            OpenFTMLElement::ImportModule(uri) => attrs.update(FTMLTag::ImportModule, uri),
            OpenFTMLElement::UseModule(uri) => attrs.update(FTMLTag::UseModule, uri),
            OpenFTMLElement::MathStructure { uri, .. } => {
                attrs.update(FTMLTag::MathStructure, &uri.clone().into_module());
            }
            OpenFTMLElement::Morphism { uri, domain, .. } => {
                attrs.update(FTMLTag::MorphismDomain, domain);
                attrs.update(FTMLTag::Morphism, &uri.clone().into_module());
            }
            OpenFTMLElement::Assign(uri) => {
                attrs.update(FTMLTag::Assign, uri);
            }
            // Paragraphs: fors-list
            OpenFTMLElement::Symdecl { uri, .. } => {
                attrs.update(FTMLTag::Symdecl, uri);
            }
            OpenFTMLElement::Notation {
                symbol: VarOrSym::S(uri),
                ..
            } => {
                attrs.update(FTMLTag::Notation, uri);
            }
            OpenFTMLElement::Definiendum(uri) => {
                attrs.update(FTMLTag::Definiendum, uri);
            }
            OpenFTMLElement::Conclusion { uri, .. } => {
                attrs.update(FTMLTag::Conclusion, uri);
            }
            OpenFTMLElement::Definiens { uri: Some(uri), .. } => {
                attrs.update(FTMLTag::Definiens, uri);
            }
            OpenFTMLElement::Inputref { uri, .. } => {
                attrs.update(FTMLTag::InputRef, uri);
            }
            OpenFTMLElement::OpenTerm {
                term:
                    OpenTerm::Symref { uri, .. }
                    | OpenTerm::OMA {
                        head: VarOrSym::S(uri),
                        ..
                    }
                    | OpenTerm::Complex(VarOrSym::S(uri), ..),
                ..
            } => attrs.update(FTMLTag::Head, uri),
            _ => (),
        }
    }
    drop(attrs);
    NodeRef::update_len(child);
     */
}

#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_wrap)]
fn prolong(parent: &NodeRef, len: isize) {
    if let Some(elem) = parent.as_element() {
        let end = elem.end_offset.get();
        elem.end_offset.set(((end as isize) + len) as usize);
        if let Some(p) = parent.parent() {
            prolong(&p, len);
        }
    }
}

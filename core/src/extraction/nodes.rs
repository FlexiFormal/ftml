use ftml_ontology::narrative::DocumentRange;

pub trait FtmlNode: Clone {
    //type Ancestors<'a>: Iterator<Item = Self> where Self: 'a;
    //fn ancestors(&self) -> Self::Ancestors<'_>;
    //fn with_elements<R>(&mut self, f: impl FnMut(Option<&mut FTMLElements>) -> R) -> R;
    fn delete(&self);
    //fn delete_children(&self);
    fn range(&self) -> DocumentRange;
    fn inner_range(&self) -> DocumentRange;
    //fn string(&self) -> Cow<'_, str>;
    //fn inner_string(&self) -> Cow<'_, str>;
    //fn as_notation(&self) -> Option<NotationSpec>;
    //fn as_op_notation(&self) -> Option<OpNotation>;
    //fn as_term(&self) -> Term;
}

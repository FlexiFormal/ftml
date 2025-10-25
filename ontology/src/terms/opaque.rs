#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub enum AnyOpaque {
    Term(u32),
    Node(OpaqueNode),
    Text(Box<str>),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize, bincode::Decode, bincode::Encode)
)]
#[cfg_attr(
    feature = "serde-lite",
    derive(serde_lite::Serialize, serde_lite::Deserialize)
)]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct OpaqueNode {
    pub tag: ftml_uris::Id,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub attributes: Box<[(ftml_uris::Id, Box<str>)]>,
    #[cfg_attr(any(feature = "serde", feature = "serde-lite"), serde(default))]
    pub children: Box<[AnyOpaque]>,
}

impl AnyOpaque {
    #[must_use]
    pub fn iter_opt(&self) -> Option<impl Iterator<Item = &Self>> {
        match self {
            Self::Term(_) | Self::Text(_) => None,
            Self::Node(node) => Some(OpaqueIter {
                curr: node.children.iter(),
                stack: smallvec::SmallVec::new(),
            }),
        }
    }
}

struct OpaqueIter<'a> {
    curr: std::slice::Iter<'a, AnyOpaque>,
    stack: smallvec::SmallVec<std::slice::Iter<'a, AnyOpaque>, 4>,
}
impl<'a> Iterator for OpaqueIter<'a> {
    type Item = &'a AnyOpaque;
    fn next(&mut self) -> Option<Self::Item> {
        let r = self.curr.next().or_else(|| {
            self.curr = self.stack.pop()?;
            self.curr.next()
        });
        if let Some(AnyOpaque::Node(node)) = r {
            self.stack
                .push(std::mem::replace(&mut self.curr, node.children.iter()));
        }
        r
    }
}

impl std::fmt::Display for AnyOpaque {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(t) => write!(f, "\"{t}\""),
            Self::Term(t) => write!(f, "<term {t}/>"),
            Self::Node(node) => {
                write!(f, "<{}", node.tag)?;
                for (k, v) in &node.attributes {
                    write!(f, " {k}=\"{v}\"")?;
                }
                f.write_str(">\n")?;
                for t in &node.children {
                    writeln!(f, "{t}")?;
                }
                write!(f, "</{}>", node.tag)
            }
        }
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for AnyOpaque {
    fn deep_size_of_children(&self, _: &mut deepsize::Context) -> usize {
        match self {
            Self::Node(node) => {
                node.attributes
                    .iter()
                    .map(|p| std::mem::size_of_val(p) + p.1.len())
                    .sum::<usize>()
                    + node.children.iter().map(Self::deep_size_of).sum::<usize>()
            }
            Self::Text(t) => t.len(),
            Self::Term(_) => 0,
        }
    }
}

#[cfg(feature = "deepsize")]
impl deepsize::DeepSizeOf for OpaqueNode {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        self.attributes
            .iter()
            .map(|p| std::mem::size_of_val(p) + p.1.len())
            .sum::<usize>()
            + self
                .children
                .iter()
                .map(|t| std::mem::size_of_val(t) + t.deep_size_of_children(context))
                .sum::<usize>()
    }
}

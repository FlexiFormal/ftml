const MML_TAGS: [&str; 31] = [
    "math",
    "mi",
    "mn",
    "mo",
    "ms",
    "mspace",
    "mtext",
    "menclose",
    "merror",
    "mfenced",
    "mfrac",
    "mpadded",
    "mphantom",
    "mroot",
    "mrow",
    "msqrt",
    "mstyle",
    "mmultiscripts",
    "mover",
    "mprescripts",
    "msub",
    "msubsup",
    "msup",
    "munder",
    "munderover",
    "mtable",
    "mtd",
    "mtr",
    "maction",
    "annotation",
    "semantics",
];

#[must_use]
pub fn is(tag: &str) -> Option<&'static str> {
    MML_TAGS
        .iter()
        .find(|e| tag.eq_ignore_ascii_case(e))
        .copied()
}

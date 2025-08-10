use std::hint::unreachable_unchecked;

use ftml_uris::{Id, metatheory};

use crate::terms::{Argument, Term, Variable, opaque::Opaque};

macro_rules! destruct {
    ([$($p:pat),*] = $e:expr ) => {
        let mut iter = $e.into_iter();
        $(
        let Some($p) = iter.next() else {
            // SAFETY: pattern match above
            unsafe { unreachable_unchecked() }
        };
        )*
    }
}

impl Term {
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn simplify(self) -> Self {
        // for opaques:
        const IGNORE_ATTRS: [&str; 8] = [
            "data-ftml-arg",
            "data-ftml-argmode",
            "data-ftml-type",
            "data-ftml-definiens",
            "data-ftml-invisible",
            "data-ftml-headterm",
            "style",
            "class",
        ];
        match self {
            // Opaques
            Self::Opaque {
                tag,
                attributes,
                children,
                terms,
            } if (tag.as_ref() == "mrow" || tag.as_ref().eq_ignore_ascii_case("span"))
                && terms.len() == 1
                && *children == [Opaque::Term(0)]
                && attributes
                    .iter()
                    .all(|(k, _)| IGNORE_ATTRS.contains(&k.as_ref())) =>
            {
                destruct!([tm] = terms);
                tm
            }
            Self::Opaque {
                tag,
                children,
                terms,
                attributes,
            } if tag.as_ref() == "mi"
                && terms.is_empty()
                && matches!(*children, [Opaque::Text(_)]) =>
            {
                // SAFETY: we just matched
                let txt = unsafe {
                    if let Some(Opaque::Text(txt)) = children.first() {
                        txt
                    } else {
                        unreachable_unchecked();
                    }
                };
                let txt = txt.trim();
                let mut chars = txt.chars();
                let Some(c) = chars.next() else {
                    return Self::Opaque {
                        tag,
                        children,
                        terms,
                        attributes,
                    };
                };
                if chars.next().is_some() {
                    return Self::Opaque {
                        tag,
                        children,
                        terms,
                        attributes,
                    };
                }
                let Some(name) = VAR_NAMES.get(&c) else {
                    return Self::Opaque {
                        tag,
                        children,
                        terms,
                        attributes,
                    };
                };
                // SAFETY: name is in map
                let name: Id = unsafe { name.parse().unwrap_unchecked() };
                // SAFETY: txt is key in map
                let notated = Some(unsafe { txt.parse().unwrap_unchecked() });
                Self::Var {
                    variable: Variable::Name { name, notated },
                    presentation: None,
                }
            }
            Self::Opaque {
                tag,
                attributes,
                children,
                terms,
            } if (tag.as_ref() == "mrow" || tag.as_ref().eq_ignore_ascii_case("span"))
                && matches!(*children, [Opaque::Node { .. }])
                && attributes
                    .iter()
                    .all(|(k, _)| IGNORE_ATTRS.contains(&k.as_ref())) =>
            {
                destruct!(
                    [Opaque::Node {
                        tag,
                        attributes,
                        children,
                    }] = children
                );
                Self::Opaque {
                    tag,
                    attributes,
                    children,
                    terms,
                }
                .simplify()
            }
            Self::Opaque {
                tag,
                children,
                terms,
                ..
            } if tag.as_ref() == "math" && matches!(*children, [Opaque::Node { .. }]) => {
                destruct!(
                    [Opaque::Node {
                        tag,
                        attributes,
                        children,
                    }] = children
                );
                Self::Opaque {
                    tag,
                    attributes,
                    children,
                    terms,
                }
                .simplify()
            }

            // structure field projections:
            Self::Application {
                head,
                arguments,
                presentation,
            } if matches!(&*head, Self::Symbol{uri,..} if uri == &*metatheory::FIELD_PROJECTION)
                && matches!(
                    &*arguments,
                    [Argument::Simple(_), Argument::Simple(Self::Label { .. })]
                ) =>
            {
                destruct!(
                    [
                        Argument::Simple(record),
                        Argument::Simple(Self::Label {
                            name: key,
                            df: None,
                            tp: None,
                        })
                    ] = arguments
                );

                let (record, record_type) = match record {
                    Self::Application {
                        head, arguments, ..
                    } if matches!(&*head, Self::Symbol{uri,..} if uri == &*metatheory::OF_TYPE)
                        && matches!(&*arguments, [Argument::Simple(_), Argument::Simple(_)]) =>
                    {
                        destruct!(
                            [Argument::Simple(record), Argument::Simple(record_type)] = arguments
                        );
                        (record, Some(Box::new(record_type)))
                    }
                    _ => (record, None),
                };
                Self::Field {
                    record: Box::new(record),
                    key,
                    record_type,
                    presentation,
                }
            }

            // module type (redundant):
            Self::Application {
                head, arguments, ..
            } if matches!(&*head, Self::Symbol{uri,..} if uri == &*metatheory::MODULE_TYPE)
                && matches!(&*arguments, [Argument::Simple(Self::Symbol { .. })]) =>
            {
                destruct!([Argument::Simple(head)] = arguments);
                head
            }

            // default
            _ => self,
        }
    }
}

/*
macro_rules! reverse {
    ($($a:literal => $b:literal),*) => {
        phf::map! {
            $(
                $b => $a
            ),*
        }
    }
}
reverse! {}
 */

// yes, systematically hardcoding this is actually simpler then doing the
// offset calculations, given that there's exceptions/gaps in unicode blocks
// all over the place -.-
// TODO: combinations, greek letters => copy from rustex
static VAR_NAMES: phf::Map<char, &str> = phf::phf_map! {
    // identity
    'a' => "a", 'b' => "b", 'c' => "c", 'd' => "d", 'e' => "e", 'f' => "f", 'g' => "g",
    'h' => "h", 'i' => "i", 'j' => "j", 'k' => "k", 'l' => "l", 'm' => "m", 'n' => "n",
    'o' => "o", 'p' => "p", 'q' => "q", 'r' => "r", 's' => "s", 't' => "t", 'u' => "u",
    'v' => "v", 'w' => "w", 'x' => "x", 'y' => "y", 'z' => "z",
    'A' => "A", 'B' => "B", 'C' => "C", 'D' => "D", 'E' => "E", 'F' => "F", 'G' => "G",
    'H' => "H", 'I' => "I", 'J' => "J", 'K' => "K", 'L' => "L", 'M' => "M", 'N' => "N",
    'O' => "O", 'P' => "P", 'Q' => "Q", 'R' => "R", 'S' => "S", 'T' => "T", 'U' => "U",
    'V' => "V", 'W' => "W", 'X' => "X", 'Y' => "Y", 'Z' => "Z",
    // monospaced
    '𝚊' => "a", '𝚋' => "b", '𝚌' => "c", '𝚍' => "d", '𝚎' => "e", '𝚏' => "f", '𝚐' => "g",
    '𝚑' => "h", '𝚒' => "i", '𝚓' => "j", '𝚔' => "k", '𝚕' => "l", '𝚖' => "m", '𝚗' => "n",
    '𝚘' => "o", '𝚙' => "p", '𝚚' => "q", '𝚛' => "r", '𝚜' => "s", '𝚝' => "t", '𝚞' => "u",
    '𝚟' => "v", '𝚠' => "w", '𝚡' => "x", '𝚢' => "y", '𝚣' => "z",
    '𝙰' => "A", '𝙱' => "B", '𝙲' => "C", '𝙳' => "D", '𝙴' => "E", '𝙵' => "F", '𝙶' => "G",
    '𝙷' => "H", '𝙸' => "I", '𝙹' => "J", '𝙺' => "K", '𝙻' => "L", '𝙼' => "M", '𝙽' => "N",
    '𝙾' => "O", '𝙿' => "P", '𝚀' => "Q", '𝚁' => "R", '𝚂' => "S", '𝚃' => "T", '𝚄' => "U",
    '𝚅' => "V", '𝚆' => "W", '𝚇' => "X", '𝚈' => "Y", '𝚉' => "Z",
    // smallcaps
     'ᴀ' => "a", 'ʙ' => "b", 'ᴄ' => "c", 'ᴅ' => "d", 'ᴇ' => "e", 'ғ' => "f", 'ɢ' => "g",
     'ʜ' => "h", 'ɪ' => "i", 'ᴊ' => "j", 'ᴋ' => "k", 'ʟ' => "l", 'ᴍ' => "m", 'ɴ' => "n",
     'ᴏ' => "o", 'ᴘ' => "p", 'ǫ' => "q", 'ʀ' => "r", /* s    */ 'ᴛ' => "t", 'ᴜ' => "u",
     'ᴠ' => "v", 'ᴡ' => "w", /* x     */ 'ʏ' => "y", 'ᴢ' => "z",
     '𝖠' => "A", '𝖡' => "B", '𝖢' => "C", '𝖣' => "D", '𝖤' => "E", '𝖥' => "F", '𝖦' => "G",
     '𝖧' => "H", '𝖨' => "I", '𝖩' => "J", '𝖪' => "K", '𝖫' => "L", '𝖬' => "M", '𝖭' => "N",
     '𝖮' => "O", '𝖯' => "P", '𝖰' => "Q", '𝖱' => "R", '𝖲' => "S", '𝖳' => "T", '𝖴' => "U",
     '𝖵' => "V", '𝖶' => "W", '𝖷' => "X", '𝖸' => "Y", '𝖹' => "Z",
    // script
    '𝒶' => "a", '𝒷' => "b", '𝒸' => "c", '𝒹' => "d", 'ℯ' => "e", '𝒻' => "f", 'ℊ' => "g",
    '𝒽' => "h", '𝒾' => "i", '𝒿' => "j", '𝓀' => "k", '𝓁' => "l", '𝓂' => "m", '𝓃' => "n",
    'ℴ' => "o", '𝓅' => "p", '𝓆' => "q", '𝓇' => "r", '𝓈' => "s", '𝓉' => "t", '𝓊' => "u",
    '𝓋' => "v", '𝓌' => "w", '𝓍' => "x", '𝓎' => "y", '𝓏' => "z",
    '𝒜' => "A", 'ℬ' => "B", '𝒞' => "C", '𝒟' => "D", 'ℰ' => "E", 'ℱ' => "F", '𝒢' => "G",
    'ℋ' => "H", 'ℐ' => "I", '𝒥' => "J", '𝒦' => "K", 'ℒ' => "L", 'ℳ' => "M", '𝒩' => "N",
    '𝒪' => "O", '𝒫' => "P", '𝒬' => "Q", 'ℛ' => "R", '𝒮' => "S", '𝒯' => "T", '𝒰' => "U",
    '𝒱' => "V", '𝒲' => "W", '𝒳' => "X", '𝒴' => "Y", '𝒵' => "Z",
    // fraktur
    '𝔞' => "a", '𝔟' => "b", '𝔠' => "c", '𝔡' => "d", '𝔢' => "e", '𝔣' => "f", '𝔤' => "g",
    '𝔥' => "h", '𝔦' => "i", '𝔧' => "j", '𝔨' => "k", '𝔩' => "l", '𝔪' => "m", '𝔫' => "n",
    '𝔬' => "o", '𝔭' => "p", '𝔮' => "q", '𝔯' => "r", '𝔰' => "s", '𝔱' => "t", '𝔲' => "u",
    '𝔳' => "v", '𝔴' => "w", '𝔵' => "x", '𝔶' => "y", '𝔷' => "z",
    '𝔄' => "A", '𝔅' => "B", 'ℭ' => "C", '𝔇' => "D", '𝔈' => "E", '𝔉' => "F", '𝔊' => "G",
    'ℌ' => "H", 'ℑ' => "I", '𝔍' => "J", '𝔎' => "K", '𝔏' => "L", '𝔐' => "M", '𝔑' => "N",
    '𝔒' => "O", '𝔓' => "P", '𝔔' => "Q", 'ℜ' => "R", '𝔖' => "S", '𝔗' => "T", '𝔘' => "U",
    '𝔙' => "V", '𝔚' => "W", '𝔛' => "X", '𝔜' => "Y", 'ℨ' => "Z",
    // sans
     '𝖺' => "a", '𝖻' => "b", '𝖼' => "c", '𝖽' => "d", '𝖾' => "e", '𝖿' => "f", '𝗀' => "g",
     '𝗁' => "h", '𝗂' => "i", '𝗃' => "j", '𝗄' => "k", '𝗅' => "l", '𝗆' => "m", '𝗇' => "n",
     '𝗈' => "o", '𝗉' => "p", '𝗊' => "q", '𝗋' => "r", '𝗌' => "s", '𝗍' => "t", '𝗎' => "u",
     '𝗏' => "v", '𝗐' => "w", '𝗑' => "x", '𝗒' => "y", '𝗓' => "z",
     /* capitals are in capitals already */
    // bold
    '𝐚' => "a", '𝐛' => "b", '𝐜' => "c", '𝐝' => "d", '𝐞' => "e", '𝐟' => "f", '𝐠' => "g",
    '𝐡' => "h", '𝐢' => "i", '𝐣' => "j", '𝐤' => "k", '𝐥' => "l", '𝐦' => "m", '𝐧' => "n",
    '𝐨' => "o", '𝐩' => "p", '𝐪' => "q", '𝐫' => "r", '𝐬' => "s", '𝐭' => "t", '𝐮' => "u",
    '𝐯' => "v", '𝐰' => "w", '𝐱' => "x", '𝐲' => "y", '𝐳' => "z",
    '𝐀' => "A", '𝐁' => "B", '𝐂' => "C", '𝐃' => "D", '𝐄' => "E", '𝐅' => "F", '𝐆' => "G",
    '𝐇' => "H", '𝐈' => "I", '𝐉' => "J", '𝐊' => "K", '𝐋' => "L", '𝐌' => "M", '𝐍' => "N",
    '𝐎' => "O", '𝐏' => "P", '𝐐' => "Q", '𝐑' => "R", '𝐒' => "S", '𝐓' => "T", '𝐔' => "U",
    '𝐕' => "V", '𝐖' => "W", '𝐗' => "X", '𝐘' => "Y", '𝐙' => "Z",
    // italic
    '𝑎' => "a", '𝑏' => "b", '𝑐' => "c", '𝑑' => "d", '𝑒' => "e", '𝑓' => "f", '𝑔' => "g",
    'ℎ' => "h", '𝑖' => "i", '𝑗' => "j", '𝑘' => "k", '𝑙' => "l", '𝑚' => "m", '𝑛' => "n",
    '𝑜' => "o", '𝑝' => "p", '𝑞' => "q", '𝑟' => "r", '𝑠' => "s", '𝑡' => "t", '𝑢' => "u",
    '𝑣' => "v", '𝑤' => "w", '𝑥' => "x", '𝑦' => "y", '𝑧' => "z",
    '𝐴' => "A", '𝐵' => "B", '𝐶' => "C", '𝐷' => "D", '𝐸' => "E", '𝐹' => "F", '𝐺' => "G",
    '𝐻' => "H", '𝐼' => "I", '𝐽' => "J", '𝐾' => "K", '𝐿' => "L", '𝑀' => "M", '𝑁' => "N",
    '𝑂' => "O", '𝑃' => "P", '𝑄' => "Q", '𝑅' => "R", '𝑆' => "S", '𝑇' => "T", '𝑈' => "U",
    '𝑉' => "V", '𝑊' => "W", '𝑋' => "X", '𝑌' => "Y", '𝑍' => "Z"
};

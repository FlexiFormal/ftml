use std::sync::LazyLock;

use crate::{DocumentUri, ModuleUri, SymbolUri};

macro_rules! uri {
    ($(  $name:ident  $(  : $t:ty := $l:literal)?   $( = $lb:literal )?  ),* $(,)?) => {
        $(
            uri!{@go
                $name $( : $t := $l )? $( = $lb )?
            }
        )*
    };
    (@go $name:ident = $l:literal) => {
            pub static $name: LazyLock<SymbolUri> = LazyLock::new(||
                URI.clone() | $l.parse::<crate::UriName>().expect("Is a valid URI")
            );
    };
    (@go $name:ident : $t:ty := $l:literal) => {
            pub static $name: LazyLock<$t> = LazyLock::new(||
                $l.parse().expect("Is a valid URI")
            );
    }
}

pub static NAMESPACE: &str = "http://mathhub.info?a=FTML/meta";

uri! {
    DOC_URI:DocumentUri := "http://mathhub.info?a=FTML/meta&d=Metatheory&l=en",
    URI:ModuleUri := "http://mathhub.info?a=FTML/meta&m=Metatheory",

    OBJECT = "object",
    OF_TYPE = "of type",
    APPLY = "apply",
    BIND = "bind",
    IMPLICIT_BIND = "implicit bind",
    PARENTHESES = "internal parentheses",

    PROP = "prop",
    JUDGMENT = "judgment holds",

    INTEGERS = "integer literal",
    ORDINAL = "ordinal",

    ELLIPSES = "ellipses",
    SEQUENCE_EXPRESSION = "sequence expression",
    SEQUENCE_TYPE = "sequence type",
    SEQUENCE_MAP = "sequence map",
    FOLD_RIGHT = "fold right",
    LAST = "last",
    INIT = "init",

    MODULE_TYPE = "module type",
    RECORD_TYPE = "record type",
    RECORD_TYPE_MERGE = "module type merge",
    ANONYMOUS_RECORD = "anonymous record",
    FIELD_PROJECTION = "record field",
    MATH_STRUCTURE = "mathematical structure",

    // -------------------------------------

    RESOLVED_VARIABLE_URI = "variable uri"
}

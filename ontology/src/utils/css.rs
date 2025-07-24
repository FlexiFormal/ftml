#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "typescript", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum Css {
    Link(Box<str>),
    Inline(Box<str>),
    Class { name: Box<str>, css: Box<str> },
}
impl PartialOrd for Css {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Css {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        fn classnum(s: &str) -> u8 {
            match s {
                s if s.starts_with("ftml-subproblem") => 1,
                s if s.starts_with("ftml-problem") => 2,
                s if s.starts_with("ftml-example") => 3,
                s if s.starts_with("ftml-definition") => 4,
                s if s.starts_with("ftml-paragraph") => 5,
                "ftml-subsubsection" => 6,
                "ftml-subsection" => 7,
                "ftml-section" => 8,
                "ftml-chapter" => 9,
                "ftml-part" => 10,
                _ => 0,
            }
        }
        use std::cmp::Ordering;
        match (self, other) {
            (Self::Link(l1), Self::Link(l2)) | (Self::Inline(l1), Self::Inline(l2)) => l1.cmp(l2),
            (Self::Link(_), Self::Inline(_))
            | (Self::Link(_) | Self::Inline(_), Self::Class { .. }) => Ordering::Less,
            (Self::Inline(_), Self::Link(_))
            | (Self::Class { .. }, Self::Inline(_) | Self::Link(_)) => Ordering::Greater,
            (Self::Class { name: n1, css: c1 }, Self::Class { name: n2, css: c2 }) => {
                (classnum(n1), n1, c1).cmp(&(classnum(n2), n2, c2))
            }
        }
    }
}
impl Css {
    #[cfg(feature = "css_normalize")]
    pub fn merge(v: Vec<Self>) -> Vec<Self> {
        use lightningcss::traits::ToCss;
        use lightningcss::{
            printer::PrinterOptions,
            rules::{CssRule, CssRuleList},
            selector::Component,
            stylesheet::{MinifyOptions, ParserOptions, StyleSheet},
        };
        use std::hint::unreachable_unchecked;

        let mut links = Vec::new();
        let mut strings = Vec::new();
        for c in v {
            match c {
                Self::Link(_) => links.push(c),
                Self::Inline(css) | Self::Class { css, .. } => strings.push(css),
            }
        }

        let mut sheet = StyleSheet::new(
            Vec::new(),
            CssRuleList(Vec::new()),
            ParserOptions::default(),
        );
        for s in &strings {
            if let Ok(rs) = StyleSheet::parse(s, ParserOptions::default()) {
                sheet.rules.0.extend(rs.rules.0.into_iter());
            } else {
                tracing::warn!("Not class-able: {s}");
            }
        }
        let _ = sheet.minify(MinifyOptions::default());

        let mut classes = Vec::new();
        for rule in std::mem::take(&mut sheet.rules.0) {
            match rule {
                CssRule::Style(style) => {
                    if style.vendor_prefix.is_empty()
                        && style.selectors.0.len() == 1
                        && style.selectors.0[0].len() == 1
                        && matches!(
                            style.selectors.0[0].iter().next(),
                            Some(Component::Class(_))
                        )
                    {
                        let Some(Component::Class(class_name)) = style.selectors.0[0].iter().next()
                        else {
                            // SAFETY: we just checked that
                            unsafe { unreachable_unchecked() }
                        };
                        if let Ok(s) = style.to_css_string(PrinterOptions::default()) {
                            classes.push(Self::Class {
                                name: class_name.to_string().into(),
                                css: s.into(),
                            });
                        } else {
                            tracing::warn!("Illegal CSS: {style:?}");
                        }
                    } else if let Ok(s) = style.to_css_string(PrinterOptions::default()) {
                        tracing::warn!("Not class-able: {s}");
                        links.push(Self::Inline(s.into()));
                    } else {
                        tracing::warn!("Illegal CSS: {style:?}");
                    }
                }
                rule => {
                    if let Ok(s) = rule.to_css_string(PrinterOptions::default()) {
                        tracing::warn!("Not class-able: {s}");
                        links.push(Self::Inline(s.into()));
                    } else {
                        tracing::warn!("Illegal CSS: {rule:?}");
                    }
                }
            }
        }
        drop(sheet);
        links.extend(classes);
        links
    }

    /*
    #[must_use]
    pub fn split(css: &str) -> Vec<Self> {
        use lightningcss::traits::ToCss;
        use lightningcss::{
            printer::PrinterOptions,
            rules::CssRule,
            selector::Component,
            stylesheet::{ParserOptions, StyleSheet},
        };
        let Ok(ruleset) = StyleSheet::parse(css, ParserOptions::default()) else {
            tracing::warn!("Not class-able: {css}");
            return vec![Self::Inline(css.to_string().into())];
        };
        if ruleset.sources.iter().any(|s| !s.is_empty()) {
            tracing::warn!("Not class-able: {css}");
            return vec![Self::Inline(css.to_string().into())];
        }
        ruleset
            .rules
            .0
            .into_iter()
            .filter_map(|rule| match rule {
                CssRule::Style(style) => {
                    if style.vendor_prefix.is_empty()
                        && style.selectors.0.len() == 1
                        && style.selectors.0[0].len() == 1
                        && matches!(
                            style.selectors.0[0].iter().next(),
                            Some(Component::Class(_))
                        )
                    {
                        let Some(Component::Class(class_name)) = style.selectors.0[0].iter().next()
                        else {
                            // SAFETY: we just checked that
                            unsafe { unreachable_unchecked() }
                        };
                        style
                            .to_css_string(PrinterOptions::default())
                            .ok()
                            .map(|s| Self::Class {
                                name: class_name.to_string().into(),
                                css: s.into(),
                            })
                    } else {
                        style
                            .to_css_string(PrinterOptions::default())
                            .ok()
                            .map(|s| {
                                tracing::warn!("Not class-able: {s}");
                                Self::Inline(s.into())
                            })
                    }
                }
                o => o.to_css_string(PrinterOptions::default()).ok().map(|s| {
                    tracing::warn!("Not class-able: {s}");
                    Self::Inline(s.into())
                }),
            })
            .collect()
    }
     */
}

//! What the playground needs to know about a component library, and nothing about Spectrum: the options of each
//! component (their values and defaults), the rules that exclude combinations, and the adapters that build the
//! previewed ones. A library implements [`Library`]; `spectrum.rs` is the Spectrum one.

use crate::adapters::Adapter;
use crate::oracle::Oracle;

/// What the playground shows a component in: the appearance and whether it is the compact/touch scale.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Env {
    pub dark: bool,
    pub mobile: bool,
}

/// An option value in the catalog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lit {
    Str(&'static str),
    Bool(bool),
    Num(i64),
}

/// A value of an option the playground checks against the rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptionValue<'a> {
    Str(&'a str),
    Bool(bool),
    Num(i64),
    /// A free-form option (a label, an icon) that is set; leave it out when unset.
    Present,
}

impl From<&Lit> for OptionValue<'static> {
    fn from(l: &Lit) -> Self {
        match *l {
            Lit::Str(s) => OptionValue::Str(s),
            Lit::Bool(b) => OptionValue::Bool(b),
            Lit::Num(n) => OptionValue::Num(n),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    /// The combination is invalid or undefined: its values are greyed out.
    Error,
    /// Valid, but it has no distinct design: a note.
    Advisory,
}

/// An option a component has: its values (empty when free-form, like a label) and default (`None`: unset).
#[derive(Clone, Debug)]
pub struct OptionDef {
    pub name: &'static str,
    pub values: Vec<Lit>,
    pub default: Option<Lit>,
}

/// A rule, as the rules panel lists it.
#[derive(Clone, Copy, Debug)]
pub struct Rule {
    pub severity: Severity,
    pub description: Option<&'static str>,
}

/// A rule the options break: its index in [`Library::rules`].
#[derive(Clone, Copy, Debug)]
pub struct Violation {
    pub rule: usize,
    pub severity: Severity,
    pub description: Option<&'static str>,
}

/// A component library: what the playground lists, controls and previews.
pub trait Library: Sync {
    /// Every component's catalog name; the previewed ones (those with an adapter) are listed first.
    fn components(&self) -> Vec<&'static str>;
    /// The options of `comp`, in the order they are shown. Empty for an unknown component.
    fn options(&self, comp: &str) -> Vec<OptionDef>;
    fn rules(&self, comp: &str) -> Vec<Rule>;
    /// The rules the chosen `options` break. An option left out takes its default.
    fn check(&self, comp: &str, options: &[(&str, OptionValue)]) -> Vec<Violation>;
    /// The components that can be previewed.
    fn adapters(&self) -> &'static [&'static dyn Adapter];
    /// What the previews are compared with, if the library has one: the oracle panel shows only then.
    fn oracle(&self) -> Option<&dyn Oracle> {
        None
    }
}

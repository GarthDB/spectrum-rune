//! What a previewed component adds to the catalog and how it is built: a trait the library implements per
//! component. An adapter sees the chosen values as a [`Choice`] and the environment as an [`Env`], never the
//! playground's state.

use crate::model::{Env, OptionValue};
use rune::El;

pub type Opts = Vec<(&'static str, OptionValue<'static>)>;
/// Option name and its presets as (id, shown); the first preset is the default.
pub type Extras = Vec<(&'static str, Vec<(&'static str, &'static str)>)>;

/// The chosen values of one component: its catalog options (defaults filled in) and the preset id chosen for
/// each of its extras.
pub struct Choice {
    pub options: Opts,
    pub extras: Vec<(&'static str, String)>,
}

impl Choice {
    /// A catalog option's value as a string (`""` when unset).
    pub fn v(&self, opt: &str) -> String {
        match self.options.iter().find(|(n, _)| *n == opt) {
            Some((_, OptionValue::Str(x))) => (*x).to_owned(),
            Some((_, OptionValue::Bool(b))) => b.to_string(),
            Some((_, OptionValue::Num(n))) => n.to_string(),
            _ => String::new(),
        }
    }
    pub fn on(&self, opt: &str) -> bool {
        self.v(opt) == "true"
    }
    /// The chosen preset id of an extra (`""` when the component has no such extra).
    pub fn extra(&self, opt: &str) -> &str {
        self.extras
            .iter()
            .find(|(n, _)| *n == opt)
            .map_or("", |(_, v)| v.as_str())
    }
}

/// What an extra is, which decides how the playground treats it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtraKind {
    /// An option with no catalog values (a label, an icon): reported to the rules as set (`OptionValue::Present`)
    /// unless its preset is `none`, so a rule can exclude values by it.
    FreeForm,
    /// A request the builder takes but no rule is about (a width): shown as controls only.
    Request,
    /// A state to render the oracle in (hover, down): its control is listed with the oracle panel.
    Oracle,
}

/// One previewed component.
pub trait Adapter: Sync {
    /// The catalog name, e.g. `"button"`.
    fn id(&self) -> &'static str;
    /// The name shown in the playground.
    fn name(&self) -> &'static str;
    /// Free-form options and the request width: they have no catalog values, so a few presets each.
    fn extras(&self) -> Extras;
    /// What the extra `opt` is. By default a [`ExtraKind::Request`].
    fn extra_kind(&self, _opt: &str) -> ExtraKind {
        ExtraKind::Request
    }
    /// The built component under `key` (its parts are `<key>/container`, ...) and the builder's own `options()`.
    fn build(&self, c: &Choice, env: Env, key: &str) -> (Opts, El);
    /// The built element on the stage: a library may put it on a backdrop (a static-colour button on a contrasting
    /// surface, say). By default as it is.
    fn stage(&self, _c: &Choice, _env: Env, built: El) -> El {
        built
    }
    /// Arguments for the library's oracle beyond the chosen options, opaque to the playground: a label, an icon,
    /// a state.
    fn oracle_args(&self, _c: &Choice) -> Vec<String> {
        Vec::new()
    }
}

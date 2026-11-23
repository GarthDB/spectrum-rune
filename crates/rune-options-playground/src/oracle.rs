//! An oracle: an independent rendering of a component the preview is compared with, such as a design system's
//! reference renderer. A library supplies one through [`crate::model::Library::oracle`]; the playground runs it on
//! demand and shows, per part, whether Rune's preview matches and the tokens the part reads.

use crate::adapters::Choice;
use crate::model::Env;

/// A token a part reads: the property, the token name, and its resolved value.
#[derive(Clone, Debug)]
pub struct Tok {
    pub prop: String,
    pub token: String,
    pub value: String,
}

/// One part of the oracle's scene: its box, corner radius, and the tokens it reads. `fill` and `color` token values
/// are CSS colours (`rgb(59, 99, 251)`).
#[derive(Clone, Debug)]
pub struct Part {
    pub name: String,
    pub rect: [f32; 4],
    pub radius: f32,
    pub tokens: Vec<Tok>,
}

pub trait Oracle: Sync {
    /// The arguments `run` takes for the chosen values (and the component's adapter's own).
    fn args(&self, comp: &str, choice: &Choice, env: Env) -> Vec<String>;
    /// Renders `comp`: the oracle's version and the parts of its scene. May block (it is a process, say).
    fn run(&self, comp: &str, args: &[String]) -> Result<(String, Vec<Part>), String>;
}

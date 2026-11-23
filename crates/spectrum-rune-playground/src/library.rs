//! Spectrum as a [`Library`]: the generated catalog constraints (`spectrum_rune::constraints`) and the adapters
//! for the components `spectrum-rune` builds.

use crate::adapters::ADAPTERS;
use rune_options_playground::adapters::Adapter;
use rune_options_playground::model as m;
use rune_options_playground::oracle::Oracle;
use spectrum_rune::constraints as c;

pub struct Spectrum;

fn lit(l: &c::Lit) -> m::Lit {
    match *l {
        c::Lit::Str(s) => m::Lit::Str(s),
        c::Lit::Bool(b) => m::Lit::Bool(b),
        c::Lit::Num(n) => m::Lit::Num(n),
    }
}

fn severity(s: c::Severity) -> m::Severity {
    match s {
        c::Severity::Error => m::Severity::Error,
        c::Severity::Advisory => m::Severity::Advisory,
    }
}

impl m::Library for Spectrum {
    fn components(&self) -> Vec<&'static str> {
        c::COMPONENTS.iter().map(|k| k.name).collect()
    }
    fn options(&self, comp: &str) -> Vec<m::OptionDef> {
        c::component(comp).map_or_else(Vec::new, |k| {
            k.options
                .iter()
                .map(|d| m::OptionDef {
                    name: d.name,
                    values: d.values.iter().map(lit).collect(),
                    default: d.default.as_ref().map(lit),
                })
                .collect()
        })
    }
    fn rules(&self, comp: &str) -> Vec<m::Rule> {
        c::component(comp).map_or_else(Vec::new, |k| {
            k.rules
                .iter()
                .map(|r| m::Rule {
                    severity: severity(r.severity),
                    description: r.description,
                })
                .collect()
        })
    }
    fn check(&self, comp: &str, options: &[(&str, m::OptionValue)]) -> Vec<m::Violation> {
        let opts: Vec<(&str, c::OptionValue)> = options
            .iter()
            .map(|(n, v)| {
                (
                    *n,
                    match *v {
                        m::OptionValue::Str(s) => c::OptionValue::Str(s),
                        m::OptionValue::Bool(b) => c::OptionValue::Bool(b),
                        m::OptionValue::Num(n) => c::OptionValue::Num(n),
                        m::OptionValue::Present => c::OptionValue::Present,
                    },
                )
            })
            .collect();
        let rules = c::component(comp).map_or(&[][..], |k| k.rules);
        c::check(comp, &opts)
            .into_iter()
            .map(|v| m::Violation {
                rule: rules.iter().position(|r| std::ptr::eq(r, v.rule)).unwrap(),
                severity: severity(v.severity),
                description: v.description,
            })
            .collect()
    }
    fn adapters(&self) -> &'static [&'static dyn Adapter] {
        &ADAPTERS
    }
    fn touch_scale(&self) -> bool {
        true
    }
    fn oracle(&self) -> Option<&dyn Oracle> {
        Some(&crate::oracle::DesignData)
    }
}

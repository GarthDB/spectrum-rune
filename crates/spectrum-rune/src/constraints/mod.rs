//! Option constraints from the Spectrum component catalogs (`components/<c>.json` `constraints[]`,
//! spec: `option-constraints.md`), and a small evaluator for them. The table in [`generated`] covers
//! every catalog, not only the components built here.
//!
//! A rule is "if `when`, then `require`". `error` means the combination is undefined (the oracle
//! renders nothing sensible for it); `advisory` means it is valid but has no distinct design.
//! Builders call [`assert_valid`]: an `error` is a `debug_assert!` failure. In release builds the
//! check is skipped and the component renders what the oracle renders. Apps can call [`check`]
//! themselves to validate options.
//!
//! Options are named as in the catalog (`isLabelHidden`, not `.label_hidden()`). Each component maps
//! its builder state to those names in one place (its `options()` method). An option left out of
//! the list takes its catalog default; a default of `null` means unset, so no `when` matches it and
//! a `require` on it fails.

mod generated;

pub use generated::{component, COMPONENTS};

/// A literal in a catalog rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lit {
    Str(&'static str),
    Bool(bool),
}

/// A value for an option passed to [`check`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptionValue<'a> {
    Str(&'a str),
    Bool(bool),
    /// An option with a free-form value (an icon, say) that is set. Leave it out when unset.
    Present,
}

impl OptionValue<'_> {
    fn is(&self, lit: &Lit) -> bool {
        match (self, lit) {
            (OptionValue::Str(a), Lit::Str(b)) => a == b,
            (OptionValue::Bool(a), Lit::Bool(b)) => a == b,
            _ => false,
        }
    }
}

impl From<&Lit> for OptionValue<'static> {
    fn from(l: &Lit) -> Self {
        match *l {
            Lit::Str(s) => OptionValue::Str(s),
            Lit::Bool(b) => OptionValue::Bool(b),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    /// The combination is invalid or undefined.
    Error,
    /// Valid, but it has no distinct design.
    Advisory,
}

/// What a rule requires of one option.
#[derive(Debug, PartialEq, Eq)]
pub enum Require {
    OneOf(&'static [Lit]),
    Present,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Rule {
    /// Conditions, ANDed. An option matches if it equals any of its literals.
    pub when: &'static [(&'static str, &'static [Lit])],
    /// Requirements, ANDed.
    pub require: &'static [(&'static str, Require)],
    pub severity: Severity,
    pub description: Option<&'static str>,
}

#[derive(Debug)]
pub struct ComponentConstraints {
    pub name: &'static str,
    /// Catalog defaults of the options the rules mention (a `null` default is left out).
    pub defaults: &'static [(&'static str, Lit)],
    pub rules: &'static [Rule],
}

/// A rule the options break.
#[derive(Clone, Copy, Debug)]
pub struct Violation {
    pub rule: &'static Rule,
    pub severity: Severity,
    pub description: Option<&'static str>,
}

/// The rules `options` break for the component `name` (its catalog name, e.g. `"button"`). An unknown
/// component has no rules.
pub fn check(name: &str, options: &[(&str, OptionValue)]) -> Vec<Violation> {
    component(name).map_or_else(Vec::new, |c| evaluate(c, options))
}

fn evaluate(c: &'static ComponentConstraints, options: &[(&str, OptionValue)]) -> Vec<Violation> {
    let value = |name: &str| -> Option<OptionValue<'_>> {
        options
            .iter()
            .rev()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| *v)
            .or_else(|| {
                c.defaults
                    .iter()
                    .find(|(n, _)| *n == name)
                    .map(|(_, l)| l.into())
            })
    };
    c.rules
        .iter()
        .filter(|r| {
            r.when
                .iter()
                .all(|(n, lits)| value(n).is_some_and(|v| lits.iter().any(|l| v.is(l))))
        })
        .filter(|r| {
            r.require.iter().any(|(n, req)| match (req, value(n)) {
                // Present means set: any value but `false` (same as constraints.jq).
                (Require::Present, v) => matches!(v, None | Some(OptionValue::Bool(false))),
                (Require::OneOf(lits), v) => !v.is_some_and(|v| lits.iter().any(|l| v.is(l))),
            })
        })
        .map(|r| Violation {
            rule: r,
            severity: r.severity,
            description: r.description,
        })
        .collect()
}

/// What every builder's `build()` calls: an `error` violation is a `debug_assert!` failure carrying
/// the rule's description; `advisory` violations are allowed. Does nothing in release builds.
pub(crate) fn assert_valid(component: &str, options: &[(&str, OptionValue)]) {
    if cfg!(debug_assertions) {
        let errors: Vec<_> = check(component, options)
            .into_iter()
            .filter(|v| v.severity == Severity::Error)
            .map(|v| v.description.unwrap_or("excluded option combination"))
            .collect();
        debug_assert!(errors.is_empty(), "{component}: {}", errors.join("; "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Lit::{Bool, Str};

    static C: ComponentConstraints = ComponentConstraints {
        name: "t",
        defaults: &[("kind", Str("a")), ("on", Bool(false))],
        rules: &[
            // literal when, array require
            Rule {
                when: &[("on", &[Bool(true)])],
                require: &[("kind", Require::OneOf(&[Str("b"), Str("c")]))],
                severity: Severity::Error,
                description: Some("on needs kind b or c"),
            },
            // any-of when, present require
            Rule {
                when: &[("kind", &[Str("x"), Str("y")])],
                require: &[("icon", Require::Present)],
                severity: Severity::Advisory,
                description: None,
            },
            // ANDed conditions
            Rule {
                when: &[("on", &[Bool(true)]), ("tone", &[Str("hot")])],
                require: &[("kind", Require::OneOf(&[Str("c")]))],
                severity: Severity::Error,
                description: Some("both"),
            },
            // ANDed requirements
            Rule {
                when: &[("mode", &[Str("m")])],
                require: &[
                    ("flag", Require::OneOf(&[Bool(true)])),
                    ("icon", Require::Present),
                ],
                severity: Severity::Error,
                description: Some("multi"),
            },
        ],
    };

    fn sev(options: &[(&str, OptionValue)]) -> Vec<(Severity, Option<&'static str>)> {
        evaluate(&C, options)
            .iter()
            .map(|v| (v.severity, v.description))
            .collect()
    }

    #[test]
    fn literal_when_and_array_require() {
        use OptionValue::{Bool as B, Str as S};
        let e = Severity::Error;
        assert_eq!(sev(&[("on", B(true))]), [(e, Some("on needs kind b or c"))]);
        assert!(sev(&[("on", B(true)), ("kind", S("b"))]).is_empty());
        assert!(sev(&[("on", B(false))]).is_empty());
        // The first rule's value is not the literal `true`.
        assert!(sev(&[("on", S("true"))]).is_empty());
    }

    #[test]
    fn any_of_when_and_present_require() {
        use OptionValue::{Present, Str as S};
        let a = [(Severity::Advisory, None)];
        assert_eq!(sev(&[("kind", S("x"))]), a);
        assert_eq!(sev(&[("kind", S("y"))]), a);
        assert!(sev(&[("kind", S("x")), ("icon", Present)]).is_empty());
        assert!(sev(&[("kind", S("z"))]).is_empty());
    }

    #[test]
    fn unset_option_uses_its_default() {
        use OptionValue::{Bool as B, Str as S};
        // `kind` is unset: its default "a" is not in [b, c].
        assert_eq!(sev(&[("on", B(true))]).len(), 1);
        // `on` is unset: its default false does not match, so nothing applies.
        assert!(sev(&[("kind", S("a"))]).is_empty());
        // An option with a null default (`tone`, `icon`) is unset: no `when` matches, `present` fails.
        assert!(sev(&[("on", B(true)), ("kind", S("b"))]).is_empty());
        assert_eq!(sev(&[("kind", S("x"))]).len(), 1);
    }

    #[test]
    fn conditions_are_anded() {
        use OptionValue::{Bool as B, Str as S};
        let both = |kind| sev(&[("on", B(true)), ("tone", S("hot")), ("kind", S(kind))]);
        assert_eq!(both("b"), [(Severity::Error, Some("both"))]);
        assert!(both("c").is_empty());
        // Only one of the two conditions holds.
        assert_eq!(sev(&[("on", B(true)), ("kind", S("b"))]), []);
    }

    #[test]
    fn every_requirement_must_hold() {
        use OptionValue::{Bool as B, Present, Str as S};
        let m = [(Severity::Error, Some("multi"))];
        assert_eq!(sev(&[("mode", S("m"))]), m);
        assert_eq!(sev(&[("mode", S("m")), ("flag", B(true))]), m);
        assert_eq!(sev(&[("mode", S("m")), ("icon", Present)]), m);
        assert!(sev(&[("mode", S("m")), ("flag", B(true)), ("icon", Present)]).is_empty());
    }

    #[test]
    fn present_means_set_and_not_false() {
        use OptionValue::{Bool as B, Present, Str as S};
        let with = |v| sev(&[("mode", S("m")), ("flag", B(true)), ("icon", v)]);
        assert!(with(Present).is_empty());
        assert!(with(S("x")).is_empty());
        assert!(with(B(true)).is_empty());
        assert_eq!(with(B(false)).len(), 1);
    }

    #[test]
    fn unknown_component_has_no_rules() {
        assert!(check("no-such-component", &[]).is_empty());
    }

    #[test]
    fn table_has_the_catalog_rules() {
        let rules = |n| component(n).unwrap().rules;
        let b = rules("button");
        assert_eq!(b.len(), 3);
        assert_eq!(b[0].when, &[("style", &[Str("outline")][..])]);
        assert_eq!(
            b[0].require,
            &[(
                "variant",
                Require::OneOf(&[Str("primary"), Str("secondary")])
            )]
        );
        assert_eq!(b[0].severity, Severity::Error);
        assert_eq!(
            b[1].when,
            &[("staticColor", &[Str("white"), Str("black")][..])]
        );
        assert_eq!(b[1].severity, Severity::Advisory);
        assert_eq!(b[2].when, &[("isLabelHidden", &[Bool(true)][..])]);
        assert_eq!(b[2].require, &[("icon", Require::Present)]);
        assert_eq!(b[2].severity, Severity::Error);

        let a = rules("action-button");
        assert_eq!(a.len(), 2);
        assert_eq!(a[0].when, &[("hideLabel", &[Bool(true)][..])]);
        assert_eq!(a[0].require, &[("icon", Require::Present)]);
        assert_eq!(a[0].severity, Severity::Error);
        assert_eq!(a[1].when, &[("isEmphasized", &[Bool(true)][..])]);
        assert_eq!(
            a[1].require,
            &[("isSelected", Require::OneOf(&[Bool(true)]))]
        );
        assert_eq!(a[1].severity, Severity::Advisory);

        let c = rules("checkbox");
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].when, &[("isIndeterminate", &[Bool(true)][..])]);
        assert_eq!(
            c[0].require,
            &[("isSelected", Require::OneOf(&[Bool(false)]))]
        );
        assert_eq!(c[0].severity, Severity::Advisory);

        // Present in the table, no rules yet.
        for n in ["switch", "radio-button", "badge", "accordion"] {
            assert!(component(n).unwrap().rules.is_empty(), "{n}");
        }
        assert!(COMPONENTS.len() > 90);
    }
}

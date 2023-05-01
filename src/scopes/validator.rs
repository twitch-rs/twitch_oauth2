#![allow(missing_docs)]
use std::borrow::Cow;

use super::Scope;

pub type Validators = Cow<'static, [Validator]>;

#[derive(Clone)]
pub enum Validator {
    Scope(Scope, bool),
    And(Sized<Validators>, bool),
    Or(Sized<Validators>, bool),
}

impl std::fmt::Debug for Validator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Validator::Scope(scope, _) => scope.fmt(f),
            Validator::And(Sized(and), false) => f.debug_tuple("And").field(and).finish(),
            Validator::And(Sized(and), true) => f.debug_tuple("NotAnd").field(and).finish(),
            Validator::Or(Sized(or), false) => f.debug_tuple("Or").field(or).finish(),
            Validator::Or(Sized(or), true) => f.debug_tuple("NotOr").field(or).finish(),
        }
    }
}

impl Validator {
    #[must_use]
    pub fn matches(&self, scopes: &[Scope]) -> bool {
        match &self {
            Validator::Scope(scope, _) => scopes.contains(scope),
            Validator::And(Sized(validators), neg) => {
                validators.iter().all(|v| v.matches(scopes)) ^ neg
            }
            Validator::Or(Sized(validators), neg) => {
                validators.iter().any(|v| v.matches(scopes)) ^ neg
            }
        }
    }

    pub const fn scope(scope: Scope) -> Self { Validator::Scope(scope, false) }

    pub const fn and_multiple(ands: &'static [Validator]) -> Self {
        Validator::And(Sized(Cow::Borrowed(ands)), false)
    }

    pub const fn or_multiple(ands: &'static [Validator]) -> Self {
        Validator::Or(Sized(Cow::Borrowed(ands)), false)
    }
}


// https://github.com/rust-lang/rust/issues/47032#issuecomment-568784919
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Sized<T>(T);

impl std::ops::Not for Validator {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Validator::Scope(s, b) => Validator::Scope(s, !b),
            Validator::And(s, b) => Validator::And(s, !b),
            Validator::Or(s, b) => Validator::Or(s, !b),
        }
    }
}

impl From<Scope> for Validator {
    fn from(scope: Scope) -> Self { Validator::scope(scope) }
}

/// Syntax should be
/// ```rust
/// use twitch_oauth2::{Scope, validator};
/// let scopes: &[Scope] = &[Scope::ChatEdit, Scope::ChatRead];
/// let validator = validator!(Scope::ChatEdit, Scope::ChatRead);
///    assert!(validator.matches(scopes));
/// ```
///
/// ```rust
/// use twitch_oauth2::{Scope, validator};
/// let scopes: &[Scope] = &[Scope::ChatEdit, Scope::ChatRead];
/// let validator = validator!(and(Scope::ChatEdit, Scope::ChatRead));
///    assert!(validator.matches(scopes));
/// ```
///
/// ```rust
/// use twitch_oauth2::{Scope, validator};
/// let scopes: &[Scope] = &[Scope::ChatEdit, Scope::ChatRead];
/// let validator = validator!(and(Scope::ChatEdit, or(Scope::ChatRead, Scope::ChannelReadSubscriptions)));
///    assert!(validator.matches(scopes));
/// ```
#[macro_export]
macro_rules! validator {
    (@and $($scope:tt),+) => {{
        static MULT: &[$crate::scopes::Validator] = &[$(validator!($scope)),*];
        $crate::scopes::Validator::and_multiple(MULT)
    }};
    (@or $($scope:tt),+) => {{
        static MULT: &[$crate::scopes::Validator] = &[$(validator!($scope)),*];
        $crate::scopes::Validator::or_multiple(MULT)
    }};
    // Replacing these with tt doesn't work
    (and($($and:tt),+)) => {{
        validator!(@and $($and),*)
    }};
    (or($($or:tt),+)) => {{
        validator!(@or $($or)*)
    }};
    ($scope:expr) => {{
        $crate::scopes::Validator::scope($scope)
    }};
    ($scope:expr,) => {{
        $crate::scopes::Validator::scope($scope)
    }};
    ($($scope:tt),+) => {{
        validator!(and($($scope),*))
    }};
}


//#[cfg(test)]
mod tests {
    use crate::Scope;

    use super::*;
    fn valid_basic() {
        let scopes = &[Scope::ChatEdit, Scope::ChatRead];
        let validator = validator!(and(Scope::ChatEdit, Scope::ChatRead));
        dbg!(&validator);
        assert!(validator.matches(scopes));
    }
    #[test]
    fn valid_and() {
       let scopes = &[Scope::ChatEdit, Scope::ChatRead];
       let validator = validator!(and(Scope::ChatEdit, Scope::ChatRead));
       dbg!(&validator);
       assert!(validator.matches(scopes));
    }
}


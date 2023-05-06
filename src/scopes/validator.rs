//! Validator used for checking scopes in a token.
use std::borrow::Cow;

use super::Scope;

/// A collection of validators
pub type Validators = Cow<'static, [Validator]>;

/// A [validator](Validator) is a way to check if an array of scopes matches a predicate.
///
/// Can be constructed easily with the [validator!](crate::validator) macro.
///
/// # Examples
///
/// ```rust, no_run
/// use twitch_oauth2::{validator, AppAccessToken, Scope, TwitchToken as _};
///
/// let token: AppAccessToken = token();
/// let validator = validator!(Scope::ChatEdit, Scope::ChatRead);
/// assert!(validator.matches(token.scopes()));
///
/// # pub fn token() -> AppAccessToken { todo!() }
/// ```
#[derive(Clone)]
#[non_exhaustive]
pub enum Validator {
    /// A scope
    Scope(Scope),
    /// Matches true if all validators passed inside return true
    All(Sized<Validators>),
    /// Matches true if **any** validator passed inside returns true
    Any(Sized<Validators>),
    /// Matches true if all validators passed inside matches false
    Not(Sized<Validators>),
}

impl std::fmt::Debug for Validator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Validator::Scope(scope) => scope.fmt(f),
            Validator::All(Sized(all)) => f.debug_tuple("All").field(all).finish(),
            Validator::Any(Sized(any)) => f.debug_tuple("Any").field(any).finish(),
            Validator::Not(Sized(not)) => f.debug_tuple("Not").field(not).finish(),
        }
    }
}

impl Validator {
    /// Checks if the given scopes match the predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// use twitch_oauth2::{validator, Scope};
    ///
    /// let scopes: &[Scope] = &[Scope::ChatEdit, Scope::ChatRead];
    /// let validator = validator!(Scope::ChatEdit, Scope::ChatRead);
    /// assert!(validator.matches(scopes));
    /// assert!(!validator.matches(&scopes[..1]));
    /// ```
    #[must_use]
    pub fn matches(&self, scopes: &[Scope]) -> bool {
        match &self {
            Validator::Scope(scope) => scopes.contains(scope),
            Validator::All(Sized(validators)) => validators.iter().all(|v| v.matches(scopes)),
            Validator::Any(Sized(validators)) => validators.iter().any(|v| v.matches(scopes)),
            Validator::Not(Sized(validator)) => !validator.iter().any(|v| v.matches(scopes)),
        }
    }

    /// Create a [Validator] which matches if the scope is present.
    pub const fn scope(scope: Scope) -> Self { Validator::Scope(scope) }

    /// Create a [Validator] which matches if all validators passed inside matches true.
    pub const fn all_multiple(ands: &'static [Validator]) -> Self {
        Validator::All(Sized(Cow::Borrowed(ands)))
    }

    /// Create a [Validator] which matches if **any** validator passed inside matches true.
    pub const fn any_multiple(anys: &'static [Validator]) -> Self {
        Validator::Any(Sized(Cow::Borrowed(anys)))
    }

    /// Create a [Validator] which matches if all validators passed inside matches false.
    pub const fn not(not: &'static Validator) -> Self {
        Validator::Not(Sized(Cow::Borrowed(std::slice::from_ref(not))))
    }

    /// Convert [Self] to [Self]
    ///
    /// # Notes
    ///
    /// This function doesn't do anything, but it powers the [validator!] macro
    #[doc(hidden)]
    pub const fn to_validator(self) -> Self { self }
}

// https://github.com/rust-lang/rust/issues/47032#issuecomment-568784919
/// Hack for making `T: Sized`
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Sized<T>(pub T);

impl From<Scope> for Validator {
    fn from(scope: Scope) -> Self { Validator::scope(scope) }
}

/// A [validator](Validator) is a way to check if a slice of scopes matches a predicate.
///
/// Uses a functional style to compose the predicate. Can be used in const context.
///
/// # Supported operators
///
/// * `not(...)`
///   * negates the validator passed inside, can only take one argument
/// * `all(...)`
///   * returns true if all validators passed inside return true
/// * `any(...)`
///   * returns true if **any** validator passed inside returns true
///
/// # Examples
///
/// ```rust, no_run
/// use twitch_oauth2::{validator, AppAccessToken, Scope, TwitchToken as _};
///
/// let token: AppAccessToken = token();
/// let validator = validator!(Scope::ChatEdit, Scope::ChatRead);
/// assert!(validator.matches(token.scopes()));
///
/// # pub fn token() -> AppAccessToken { todo!() }
/// ```
///
/// ## Multiple scopes
///
/// ```rust
/// use twitch_oauth2::{validator, Scope};
///
/// let scopes: &[Scope] = &[Scope::ChatEdit, Scope::ChatRead];
/// let validator = validator!(Scope::ChatEdit, Scope::ChatRead);
/// assert!(validator.matches(scopes));
/// assert!(!validator.matches(&scopes[..1]));
/// ```
///
/// ## Multiple scopes with explicit all(...)
///
/// ```rust
/// use twitch_oauth2::{validator, Scope};
///
/// let scopes: &[Scope] = &[Scope::ChatEdit, Scope::ChatRead];
/// let validator = validator!(all(Scope::ChatEdit, Scope::ChatRead));
/// assert!(validator.matches(scopes));
/// assert!(!validator.matches(&scopes[..1]));
/// ```
///
/// ## Multiple scopes with nested any(...)
///
/// ```rust
/// use twitch_oauth2::{validator, Scope};
///
/// let scopes: &[Scope] = &[Scope::ChatEdit, Scope::ChatRead];
/// let validator = validator!(
///     Scope::ChatEdit,
///     any(Scope::ChatRead, Scope::ChannelReadSubscriptions)
/// );
/// assert!(validator.matches(scopes));
/// assert!(!validator.matches(&scopes[1..]));
/// ```
///
/// ## Not
///
/// ```rust
/// use twitch_oauth2::{validator, Scope};
///
/// let scopes: &[Scope] = &[Scope::ChatRead];
/// let validator = validator!(not(Scope::ChatEdit));
/// assert!(validator.matches(scopes));
/// ```
///
/// ## Combining other validators
///
/// ```
/// use twitch_oauth2::{validator, Scope, Validator};
///
/// let scopes: &[Scope] = &[
///     Scope::ChatEdit,
///     Scope::ChatRead,
///     Scope::ModeratorManageAutoMod,
///     Scope::ModerationRead,
/// ];
/// const CHAT_SCOPES: Validator = validator!(all(Scope::ChatEdit, Scope::ChatRead));
/// const MODERATOR_SCOPES: Validator =
///     validator!(Scope::ModerationRead, Scope::ModeratorManageAutoMod);
/// const COMBINED: Validator = validator!(CHAT_SCOPES, MODERATOR_SCOPES);
/// assert!(COMBINED.matches(scopes));
/// assert!(!COMBINED.matches(&scopes[1..]));
/// ```
///
/// ## Empty
///
/// ```rust
/// use twitch_oauth2::{validator, Scope};
///
/// let scopes: &[Scope] = &[Scope::ChatRead];
/// let validator = validator!();
/// assert!(validator.matches(scopes));
/// ```
///
/// ## Invalid examples
///
/// ### Invalid usage of not(...)
///
/// ```compile_fail
/// use twitch_oauth2::{Scope, validator};
///
/// let scopes: &[Scope] = &[Scope::ChatEdit, Scope::ChatRead];
/// let validator = validator!(not(Scope::ChatEdit, Scope::ChatRead));
/// assert!(validator.matches(scopes));
/// ```
///
/// ### Invalid operator
///
/// ```compile_fail
/// use twitch_oauth2::{Scope, validator};
///
/// let scopes: &[Scope] = &[Scope::ChatEdit, Scope::ChatRead];
/// let validator = validator!(xor(Scope::ChatEdit, Scope::ChatRead));
/// assert!(validator.matches(scopes));
/// ```
#[macro_export]
macro_rules! validator {
    ($operator:ident($($scopes:tt)+)) => {{
        $crate::validator_logic!(@$operator $($scopes)*)
    }};
    ($scope:expr $(,)?) => {{
        $scope.to_validator()
    }};
    ($($all:tt)+) => {{
        $crate::validator_logic!(@all $($all)*)
    }};
    () => {{
        $crate::Validator::all_multiple(&[])
    }};
}

/// Logical operators for the [validator!] macro.
#[doc(hidden)]
#[macro_export]
macro_rules! validator_logic {
    (@all $($scope:tt)+) => {{
        const MULT: &[$crate::Validator] = &$crate::validator_accumulate![@down [] $($scope)*];
        $crate::Validator::all_multiple(MULT)
    }};
    (@any $($scope:tt)+) => {{
        const MULT: &[$crate::Validator] = &$crate::validator_accumulate![@down [] $($scope)*];
        $crate::Validator::any_multiple(MULT)
    }};
    (@not $($scope:tt)+) => {{
        $crate::validator_logic!(@notend $($scope)*);
        const NOT: &[$crate::Validator] = &[$crate::validator!($($scope)*)];
        $crate::Validator::not(&NOT[0])
    }};
    (@notend $e:expr) => {};
    (@notend $e:expr, $($t:tt)*) => {compile_error!("not(...) takes only one argument")};
    (@$operator:ident $($rest:tt)*) => {
        compile_error!(concat!("unknown operator `", stringify!($operator), "`, only `all`, `any` and `not` are supported"))
    }
}

/// Accumulator for the [validator!] macro.
// Thanks to danielhenrymantilla, the macro wizard
#[doc(hidden)]
#[macro_export]
macro_rules! validator_accumulate {
    // inner operator
    (@down
        [$($acc:tt)*]
        $operator:ident($($all:tt)*) $(, $($rest:tt)* )?
    ) => (
        $crate::validator_accumulate![@down
            [$($acc)* $crate::validator!($operator($($all)*)),]
            $($($rest)*)?
        ]
    );
    // inner scope
    (@down
        [$($acc:tt)*]
        $scope:expr $(, $($rest:tt)* )?
    ) => (
        $crate::validator_accumulate![@down
            [$($acc)* $crate::validator!($scope),]
            $($($rest)*)?
        ]
    );
    // nothing left
    (@down
        [$($output:tt)*] $(,)?
    ) => (
        [ $($output)* ]
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Scope;

    #[test]
    fn valid_basic() {
        let scopes = &[Scope::ChatEdit, Scope::ChatRead];
        const VALIDATOR: Validator = validator!(Scope::ChatEdit, Scope::ChatRead);
        dbg!(&VALIDATOR);
        assert!(VALIDATOR.matches(scopes));
    }

    #[test]
    fn valid_all() {
        let scopes = &[Scope::ChatEdit, Scope::ChatRead];
        const VALIDATOR: Validator = validator!(all(Scope::ChatEdit, Scope::ChatRead));
        dbg!(&VALIDATOR);
        assert!(VALIDATOR.matches(scopes));
    }

    #[test]
    fn valid_any() {
        let scopes = &[Scope::ChatEdit, Scope::ModerationRead];
        const VALIDATOR: Validator =
            validator!(Scope::ChatEdit, any(Scope::ChatRead, Scope::ModerationRead));
        dbg!(&VALIDATOR);
        assert!(VALIDATOR.matches(scopes));
    }

    #[test]
    fn valid_not() {
        let scopes = &[Scope::ChannelEditCommercial, Scope::ChatRead];
        const VALIDATOR: Validator = validator!(not(Scope::ChatEdit));
        dbg!(&VALIDATOR);
        assert!(VALIDATOR.matches(scopes));
    }

    #[test]
    fn valid_strange() {
        let scopes = &[Scope::ChatEdit, Scope::ModerationRead, Scope::UserEdit];
        let scopes_1 = &[Scope::ChatEdit, Scope::ChatRead];
        const VALIDATOR: Validator = validator!(
            Scope::ChatEdit,
            any(Scope::ChatRead, all(Scope::ModerationRead, Scope::UserEdit))
        );
        dbg!(&VALIDATOR);
        assert!(VALIDATOR.matches(scopes));
        assert!(VALIDATOR.matches(scopes_1));
    }
    #[test]
    fn valid_strange_not() {
        let scopes = &[Scope::ModerationRead, Scope::UserEdit];
        let scopes_1 = &[Scope::ChatEdit, Scope::ChatRead];
        const VALIDATOR: Validator = validator!(
            not(Scope::ChatEdit),
            any(Scope::ChatRead, all(Scope::ModerationRead, Scope::UserEdit))
        );
        dbg!(&VALIDATOR);
        assert!(VALIDATOR.matches(scopes));
        assert!(!VALIDATOR.matches(scopes_1));
    }
}

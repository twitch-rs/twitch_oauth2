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
#[derive(Clone, PartialEq)]
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

impl std::fmt::Display for Validator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // dont allocate if we can avoid it, instead we map over the validators, and use write!
        match self {
            Validator::Scope(scope) => scope.fmt(f),
            Validator::All(Sized(all)) => {
                write!(f, "(")?;
                for (i, v) in all.iter().enumerate() {
                    if i != 0 {
                        write!(f, " and ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, ")")
            }
            Validator::Any(Sized(any)) => {
                write!(f, "(")?;
                for (i, v) in any.iter().enumerate() {
                    if i != 0 {
                        write!(f, " or ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, ")")
            }
            Validator::Not(Sized(not)) => {
                write!(f, "not(")?;
                for (i, v) in not.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, ")")
            }
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

    /// Returns a validator only containing the unmatched scopes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use twitch_oauth2::{validator, Scope};
    ///
    /// let validator = validator!(Scope::ChatEdit, Scope::ChatRead);
    ///
    /// let scopes = &[Scope::ChatEdit, Scope::ChatRead];
    /// assert_eq!(validator.missing(scopes), None);
    /// ```
    ///
    /// ```rust
    /// use twitch_oauth2::{validator, Scope};
    ///
    /// let validator = validator!(Scope::ChatEdit, Scope::ChatRead);
    ///
    /// let scopes = &[Scope::ChatEdit];
    /// if let Some(v) = validator.missing(scopes) {
    ///     println!("Missing scopes: {}", v);
    /// }
    /// ```
    ///
    /// ```rust
    /// use twitch_oauth2::{validator, Scope};
    ///
    /// let validator = validator!(
    ///     any(
    ///         Scope::ModeratorReadBlockedTerms,
    ///         Scope::ModeratorManageBlockedTerms
    ///     ),
    ///     any(
    ///         Scope::ModeratorReadChatSettings,
    ///         Scope::ModeratorManageChatSettings
    ///     )
    /// );
    ///
    /// let scopes = &[Scope::ModeratorReadBlockedTerms];
    /// let missing = validator.missing(scopes).unwrap();
    /// // We're missing either of the chat settings scopes
    /// assert!(missing.matches(&[Scope::ModeratorReadChatSettings]));
    /// assert!(missing.matches(&[Scope::ModeratorManageChatSettings]));
    /// ```
    pub fn missing(&self, scopes: &[Scope]) -> Option<Validator> {
        if self.matches(scopes) {
            return None;
        }
        // a recursive prune approach, if a validator matches, we prune it.
        // TODO: There's a bit of allocation going on here, maybe we can remove it with some kind of descent
        match &self {
            Validator::Scope(scope) => {
                if scopes.contains(scope) {
                    None
                } else {
                    Some(Validator::Scope(scope.clone()))
                }
            }
            Validator::All(Sized(validators)) => {
                let mut missing = validators
                    .iter()
                    .filter_map(|v| v.missing(scopes))
                    .collect::<Vec<_>>();

                if missing.is_empty() {
                    None
                } else if missing.len() == 1 {
                    Some(missing.remove(0))
                } else {
                    Some(Validator::All(Sized(Cow::Owned(missing))))
                }
            }
            Validator::Any(Sized(validators)) => {
                let mut missing = validators
                    .iter()
                    .filter(|v| !v.matches(scopes))
                    .filter_map(|v| v.missing(scopes))
                    .collect::<Vec<_>>();

                if missing.is_empty() {
                    None
                } else if missing.len() == 1 {
                    Some(missing.remove(0))
                } else {
                    Some(Validator::Any(Sized(Cow::Owned(missing))))
                }
            }
            Validator::Not(Sized(validators)) => {
                // not is special, it's a negation, so a match is a failure.
                // we find out if the validators inside matches (e.g the scopes exists),
                // if they exist they are bad.
                // a validator should preferably not use not, because scopes are additive.

                let matching = validators
                    .iter()
                    .filter(|v| v.matches(scopes))
                    .collect::<Vec<_>>();

                if matching.is_empty() {
                    None
                } else {
                    Some(Validator::Not(Sized(Cow::Owned(
                        matching.into_iter().cloned().collect(),
                    ))))
                }
            }
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
#[derive(Debug, Clone, PartialEq)]
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

    #[test]
    fn missing() {
        let scopes = &[Scope::ChatEdit, Scope::ModerationRead];
        const VALIDATOR: Validator = validator!(
            Scope::ChatEdit,
            any(Scope::ChatRead, all(Scope::ModerationRead, Scope::UserEdit))
        );
        dbg!(&VALIDATOR);
        let missing = VALIDATOR.missing(scopes).unwrap();
        dbg!(&missing);
        assert_eq!(format!("{}", missing), "(chat:read or user:edit)");

        const NOT_VALIDATOR: Validator = validator!(all(
            not(all(Scope::ChatEdit, Scope::ModerationRead)), // we don't want both of these
            Scope::ChatRead,
            Scope::UserEdit,
            any(Scope::ModerationRead, not(Scope::UserEdit)) // we don't want user:edit or we want moderation:read
        ));
        let missing = NOT_VALIDATOR.missing(scopes).unwrap();
        dbg!(&missing);
        assert_eq!(
            format!("{}", missing),
            "(not((chat:edit and moderation:read)) and chat:read and user:edit)"
        );
    }

    #[test]
    fn display() {
        const COMPLEX_VALIDATOR: Validator = validator!(
            Scope::ChatEdit,
            any(Scope::ChatRead, all(Scope::ModerationRead, Scope::UserEdit))
        );
        assert_eq!(
            format!("{}", COMPLEX_VALIDATOR),
            "(chat:edit and (chat:read or (moderation:read and user:edit)))"
        );
    }
}

# Change Log

<!-- next-header -->

## [Unreleased] - ReleaseDate

[Commits](https://github.com/twitch-rs/twitch_oauth2/compare/v0.7.0...Unreleased)

## Changed

* Organization moved to `twitch-rs`

## [v0.7.0] - 2022-05-08

[Commits](https://github.com/twitch-rs/twitch_oauth2/compare/v0.6.1...v0.7.0)

### Breaking changes

* switch to [`twitch_types`](https://crates.io/crates/twitch_types) for `UserId` and `Nickname`/`UserName`
* bump MSRV to 1.60, also changes the feature names for clients to their simpler variant `surf` and `client`

## [v0.6.1] - 2021-11-23

[Commits](https://github.com/twitch-rs/twitch_oauth2/compare/v0.6.0...v0.6.1)

### Added

* Added new scopes `moderator:manage:automod_settings`, `moderator:manage:banned_users`,
  `moderator:manage:blocked_terms`, `moderator:manage:chat_settings`, `moderator:read:automod_settings`,
  `moderator:read:blocked_terms` and `moderator:read:chat_settings`

## [v0.6.0] - 2021-09-27

[Commits](https://github.com/twitch-rs/twitch_oauth2/compare/v0.5.2...v0.6.0)

### Breaking changes

* All types associated with tokens are now defined in this crate. This is a consequence of the `oauth2` dependency being removed from tree.
  Additionally, as another consequence, clients are now able to be specified as a `for<'a> &'a T where T: Client<'a>`, meaning `twitch_api` can use its clients as an interface to token requests,
  and clients can persist instead of being rebuilt every call. Care should be taken when making clients, as SSRF and similar attacks are possible with improper client configurations.

### Added

* Added types/braids `ClientId`, `ClientSecret`, `AccessToken`, `RefreshToken` and `CsrfToken`.
* Added way to interact with the Twitch-CLI [mock API](https://github.com/twitchdev/twitch-cli/blob/main/docs/mock-api.md) using environment variables.
  See static variables `AUTH_URL`, `TOKEN_URL`, `VALIDATE_URL` and `REVOKE_URL` for more information.
* Added `impl Borrow<str> for Scope`, meaning it can be used in places it couldn't be used before. Primarily, it allows the following code to work:
  ```rust
  let scopes = vec![Scope::ChatEdit, Scope::ChatRead];
  let space_separated_scope: String = scopes.as_slice().join(" ");
  ```
* Added scope `channel:read:goals`

### Changed

* Requests to `id.twitch.tv` now follow the documentation, instead of following a subset of the RFC for oauth2.
* URLs are now initialized lazily and specified as `url::Url`s.

### Removed

* Removed `oauth2` dependency.

## [v0.5.2] - 2021-06-18

[Commits](https://github.com/twitch-rs/twitch_oauth2/compare/v0.5.1...v0.5.2)

### Added

* Added new scope `channel:manage:schedule`

## [v0.5.1] - 2021-05-16

[Commits](https://github.com/twitch-rs/twitch_oauth2/compare/v0.5.0...v0.5.1)

### Added

* Added new scopes `channel:manage:polls`, `channel:manage:predictions`, `channel:read:polls`, `channel:read:predictions`, and `moderator:manage:automod`,
* Added function `Scope::description` to get the description of the scope

## [v0.5.0] - 2021-05-08

[Commits](https://github.com/twitch-rs/twitch_oauth2/compare/49a083ceda6768cc52a1f8f1714bb7f942f24c01...v0.5.0)

### Added

* Made crate runtime agnostic with custom clients.
* Updated deps.
* Add an extra (optional) client secret field to `UserToken::from_existing` (thanks [Dinnerbone](https://github.com/Dinnerbone))
* Added `channel:manage:redemptions`, `channel:read:editors`, `channel:manage:videos`, `user:read:blocked_users`,  `user:manage:blocked_users`, `user:read:subscriptions` and `user:read:follows`
* Implemented [OAuth Authorization Code Flow](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#oauth-authorization-code-flow) with `UserTokenBuilder`
* Added a way to suggest or infer that an user token is never expiring, making `is_elapsed` return false and `expires_in` a bogus (max) duration.
### Changed

* MSRV: 1.51
* Made scope take `Cow<&'static str>`
* Made fields `access_token`, `refresh_token`, `user_id` and `login` `pub` on `UserToken` and `AppAccessToken` (where applicable)
* Fixed wrong scope `user:read:stream_key` -> `channel:read:stream_key`
* BREAKING: changed `TwitchToken::expires` -> `TwitchToken::expires_in` to calculate current lifetime of token

## End of Changelog 

Changelog starts on v0.5.0
# Change Log

<!-- next-header -->

## [Unreleased] - ReleaseDate

[Commits](https://github.com/Emilgardis/twitch_oauth2/compare/0.5.1...Unreleased)

## [v0.5.1] - 2021-05-16

[Commits](https://github.com/Emilgardis/twitch_oauth2/compare/0.5.0...v0.5.1)

# Added

* Added new scopes `channel:manage:polls`, `channel:manage:predictions`, `channel:read:polls`, `channel:read:predictions`, and `moderator:manage:automod`,
* Added function `Scope::description` to get the description of the scope

## [v0.5.0] - 2021-05-08

[Commits](https://github.com/Emilgardis/twitch_oauth2/compare/49a083ceda6768cc52a1f8f1714bb7f942f24c01...v0.5.0)

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
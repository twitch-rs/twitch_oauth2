# Change Log

<!-- next-header -->

## [Unreleased] - ReleaseDate

[Commits](https://github.com/Emilgardis/twitch_oauth2/compare/49a083ceda6768cc52a1f8f1714bb7f942f24c01...Unreleased)

### Added

* Made crate runtime agnostic with custom clients.
* Updated deps.
* Added old `channel_subscriptions` scope
* Add an extra (optional) client secret field to `UserToken::from_existing` (thanks [Dinnerbone](https://github.com/Dinnerbone))
* Added `channel:manage:redemptions`, `channel:read:editors`, `channel:manage:videos`, `user:read:blocked_users` and `user:manage:blocked_users`

### Changed

* Made scope take `Cow<&'static str>`
* Made fields `access_token` and `refresh_token` `pub` on `UserToken`
* Fixed wrong scope `user:read:stream_key` -> `channel:read:stream_key`

## End of Changelog 

Changelog starts on Unreleased
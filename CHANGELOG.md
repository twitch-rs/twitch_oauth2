# Change Log

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Added

* Made crate runtime agnostic with custom clients.
* Updated deps
* Added old `channel_subscriptions` scope
* Add an extra (optional) client secret field to `UserToken::from_existing` (thanks [@Dinnerbone](https://github.com/Dinnerbone))
* Added `channel:manage:redemptions` scope

### Changed

* Made scope take `Cow<&'static str>`
* Made fields `access_token` and `refresh_token` `pub` on `UserToken`

## End of Changelog 

Changelog starts on v0.5.0
//! Module for all possible scopes in twitch.

use serde::{Deserialize, Serialize};
use std::borrow::Cow;

macro_rules! scope_impls {
    ($($i:ident,$rename:literal,$doc:literal);* $(;)? ) => {
        #[doc = "Scopes for twitch."]
        #[doc = ""]
        #[doc = "<https://dev.twitch.tv/docs/authentication/#scopes>"]
        #[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
        #[non_exhaustive]
        #[serde(from = "String")]
        #[serde(into = "String")]
        pub enum Scope {
            $(
                #[doc = $doc]
                #[doc = "\n\n"]
                #[doc = "`"]
                #[doc = $rename]
                #[doc = "`"]
                #[serde(rename = $rename)] // Is this even needed?
                $i,
            )*
            #[doc = "Other scope that is not implemented."]
            Other(Cow<'static, str>),
        }

        impl std::fmt::Display for Scope {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(match self {
                    Scope::Other(s) => &s,
                    $(
                        Scope::$i => $rename,
                    )*
                })
            }
        }

        impl Scope {
            #[doc = "Get a vec of all defined twitch [Scopes][Scope]."]
            #[doc = "\n\n"]
            #[doc = "Please note that this may not work for you, as some auth flows and \"apis\" don't accept all scopes"]
            pub fn all() -> Vec<Scope> {
                vec![
                    $(Scope::$i,)*
                ]
            }

            #[doc = "Make a scope from a cow string"]
            pub fn parse<C>(s: C) -> Scope where C: Into<Cow<'static, str>> {
                use std::borrow::Borrow;
                let s = s.into();
                match s.borrow() {
                    $($rename => {Scope::$i})*,
                    _ => Scope::Other(s)
                }
            }
        }

    };
}

scope_impls!(
    AnalyticsReadExtensions, "analytics:read:extensions", "View analytics data for the Twitch Extensions owned by the authenticated account.";
    AnalyticsReadGames, "analytics:read:games", "View analytics data for the games owned by the authenticated account.";
    BitsRead, "bits:read", "View Bits information for a channel.";
    ChannelEditCommercial, "channel:edit:commercial", "Run commercials on a channel.";
    ChannelManageBroadcast, "channel:manage:broadcast", "Manage a channel’s broadcast configuration, including updating channel configuration and managing stream markers and stream tags.";
    ChannelManageExtensions, "channel:manage:extensions", "Manage a channel’s Extension configuration, including activating Extensions.";
    ChannelManageRedemptions, "channel:manage:redemptions", "Manage Channel Points custom rewards and their redemptions on a channel.";
    ChannelManageVideos, "channel:manage:videos", "Manage a channel’s videos, including deleting videos.";
    ChannelModerate, "channel:moderate", "Perform moderation actions in a channel. The user requesting the scope must be a moderator in the channel.";
    ChannelReadEditors, "channel:read:editors", "View a list of users with the editor role for a channel.";
    ChannelReadHypeTrain, "channel:read:hype_train", "View Hype Train information for a channel.";
    ChannelReadRedemptions, "channel:read:redemptions", "View Channel Points custom rewards and their redemptions on a channel.";
    ChannelReadStreamKey, "channel:read:stream_key", "View an authorized user’s stream key.";
    ChannelReadSubscriptions, "channel:read:subscriptions", "View a list of all subscribers to a channel and check if a user is subscribed to a channel.";
    ChannelSubscriptions, "channel_subscriptions", "\\[DEPRECATED\\] Read all subscribers to your channel.";
    ChatEdit, "chat:edit", "Send live stream chat and rooms messages.";
    ChatRead, "chat:read", "View live stream chat and rooms messages.";
    ClipsEdit, "clips:edit", "Manage Clips for a channel.";
    ModerationRead, "moderation:read", "View a channel’s moderation data including Moderators, Bans, Timeouts, and Automod settings.";
    UserEdit, "user:edit", "Manage a user object.";
    UserEditBroadcast, "user:edit:broadcast", "Edit your channel's broadcast configuration, including extension configuration. (This scope implies user:read:broadcast capability.)";
    UserEditFollows, "user:edit:follows", "Edit a user’s follows.";
    UserManageBlockedUsers, "user:manage:blocked_users", "Manage the block list of a user.";
    UserReadBlockedUsers, "user:read:blocked_users", "View the block list of a user.";
    UserReadBroadcast, "user:read:broadcast", "View a user’s broadcasting configuration, including Extension configurations.";
    UserReadFollos, "user:read:follows", "View the list of channels a user follows.";
    UserReadEmail, "user:read:email", "Read an authorized user’s email address.";
    UserReadSubscriptions, "user:read:subscriptions", "View if an authorized user is subscribed to specific channels.";
    WhispersEdit, "whispers:edit", "Send whisper messages.";
    WhispersRead, "whispers:read", "View your whisper messages.";
);

impl Scope {
    /// Get [Scope] as an oauth2 Scope
    pub fn as_oauth_scope(&self) -> oauth2::Scope { oauth2::Scope::new(self.to_string()) }
}

impl From<oauth2::Scope> for Scope {
    fn from(scope: oauth2::Scope) -> Self { Scope::parse(scope.to_string()) }
}

impl From<String> for Scope {
    fn from(s: String) -> Self { Scope::parse(s) }
}

impl From<Scope> for String {
    fn from(s: Scope) -> Self { s.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn custom_scope() {
        assert_eq!(
            Scope::Other(Cow::from("custom_scope")),
            Scope::parse("custom_scope")
        )
    }

    #[test]
    fn roundabout() {
        for scope in Scope::all() {
            assert_eq!(scope, Scope::parse(scope.to_string()))
        }
    }
}

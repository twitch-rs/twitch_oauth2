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
    AnalyticsReadExtensions, "analytics:read:extensions", "View analytics data for your extensions.";
    AnalyticsReadGames, "analytics:read:games", "View analytics data for your games.";
    BitsRead, "bits:read", "View bits information for your channel.";
    ChannelSubscriptions, "channel_subscriptions", "\\[DEPRECATED\\] Read all subscribers to your channel.";
    ChannelEditCommercial, "channel:edit:commercial", "Start a commercial on authorized channels";
    ChannelManageBroadcast, "channel:manage:broadcast", "Manage your channel’s broadcast configuration, including updating channel configuration and managing stream markers and stream tags.";
    ChannelManageExtension, "channel:manage:extension", "Manage your channel’s extension configuration, including activating extensions.";
    ChannelModerate, "channel:moderate", "Perform moderation actions in a channel";
    ChannelReadHypeTrain, "channel:read:hype_train", "Read hype trains";
    ChannelReadRedemptions, "channel:read:redemptions", "View your channel points custom reward redemptions";
    ChannelManageRedemptions, "channel:manage:redemptions", "Manage Channel Points custom rewards and their redemptions on a channel.";
    ChannelReadSubscriptions, "channel:read:subscriptions", "Get a list of all subscribers to your channel and check if a user is subscribed to your channel";
    ChatEdit, "chat:edit", "Send live Stream Chat and Rooms messages";
    ChatRead, "chat:read", "View live Stream Chat and Rooms messages";
    ClipsEdit, "clips:edit", "Create and edit clips as a specific user.";
    ModerationRead, "moderation:read", "View your channel's moderation data including Moderators, Bans, Timeouts and Automod settings";
    UserEdit, "user:edit", "Manage a user object.";
    UserEditBroadcast, "user:edit:broadcast", "Edit your channel's broadcast configuration, including extension configuration. (This scope implies user:read:broadcast capability.)";
    UserEditFollows, "user:edit:follows", "Edit your follows.";
    UserReadBroadcast, "user:read:broadcast", "View your broadcasting configuration, including extension configurations.";
    UserReadEmail, "user:read:email", "Read authorized user's email address.";
    UserReadStreamKey, "user:read:stream_key", "Read authorized user’s stream key.";
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

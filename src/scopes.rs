//! Module for all possible scopes in twitch.

use serde::{Deserialize, Serialize};
use std::borrow::Cow;

macro_rules! scope_impls {
    ($($i:ident,scope: $rename:literal, doc: $doc:literal);* $(;)? ) => {
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

            #[doc = "Get a description for the token"]
            pub fn description(&self) -> &'static str {
                match self {
                    $(Self::$i => $doc,)*
                    _ => "unknown scope"
                }
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

            /// Get the scope as a borrowed string.
            pub fn as_str(&self) -> &str {
                match self {
                    $(Scope::$i => $rename,)*
                    Self::Other(c) =>  c.as_ref()
                }
            }
        }

    };
}

scope_impls!(
    AnalyticsReadExtensions,  scope: "analytics:read:extensions",  doc: "View analytics data for the Twitch Extensions owned by the authenticated account.";
    AnalyticsReadGames,       scope: "analytics:read:games",       doc: "View analytics data for the games owned by the authenticated account.";
    BitsRead,                 scope: "bits:read",                  doc: "View Bits information for a channel.";
    ChannelEditCommercial,    scope: "channel:edit:commercial",    doc: "Run commercials on a channel.";
    ChannelManageBroadcast,   scope: "channel:manage:broadcast",   doc: "Manage a channel’s broadcast configuration, including updating channel configuration and managing stream markers and stream tags.";
    ChannelManageExtensions,  scope: "channel:manage:extensions",  doc: "Manage a channel’s Extension configuration, including activating Extensions.";
    ChannelManagePolls,       scope: "channel:manage:polls",       doc: "Manage a channel’s polls.";
    ChannelManagePredictions, scope: "channel:manage:predictions", doc: "Manage of channel’s Channel Points Predictions";
    ChannelManageRedemptions, scope: "channel:manage:redemptions", doc: "Manage Channel Points custom rewards and their redemptions on a channel.";
    ChannelManageSchedule,    scope: "channel:manage:schedule",    doc: "Manage a channel’s stream schedule.";
    ChannelManageVideos,      scope: "channel:manage:videos",      doc: "Manage a channel’s videos, including deleting videos.";
    ChannelModerate,          scope: "channel:moderate",           doc: "Perform moderation actions in a channel. The user requesting the scope must be a moderator in the channel.";
    ChannelReadEditors,       scope: "channel:read:editors",       doc: "View a list of users with the editor role for a channel.";
    ChannelReadHypeTrain,     scope: "channel:read:hype_train",    doc: "View Hype Train information for a channel.";
    ChannelReadPolls,         scope: "channel:read:polls",         doc: "View a channel’s polls.";
    ChannelReadPredictions,   scope: "channel:read:predictions",   doc: "View a channel’s Channel Points Predictions.";
    ChannelReadRedemptions,   scope: "channel:read:redemptions",   doc: "View Channel Points custom rewards and their redemptions on a channel.";
    ChannelReadStreamKey,     scope: "channel:read:stream_key",    doc: "View an authorized user’s stream key.";
    ChannelReadSubscriptions, scope: "channel:read:subscriptions", doc: "View a list of all subscribers to a channel and check if a user is subscribed to a channel.";
    ChannelSubscriptions,     scope: "channel_subscriptions",      doc: "\\[DEPRECATED\\] Read all subscribers to your channel.";
    ChatEdit,                 scope: "chat:edit",                  doc: "Send live stream chat and rooms messages.";
    ChatRead,                 scope: "chat:read",                  doc: "View live stream chat and rooms messages.";
    ClipsEdit,                scope: "clips:edit",                 doc: "Manage Clips for a channel.";
    ModerationRead,           scope: "moderation:read",            doc: "View a channel’s moderation data including Moderators, Bans, Timeouts, and Automod settings.";
    ModeratorManageAutoMod,   scope: "moderator:manage:automod",   doc: "Manage messages held for review by AutoMod in channels where you are a moderator.";
    UserEdit,                 scope: "user:edit",                  doc: "Manage a user object.";
    UserEditBroadcast,        scope: "user:edit:broadcast",        doc: "Edit your channel's broadcast configuration, including extension configuration. (This scope implies user:read:broadcast capability.)";
    UserEditFollows,          scope: "user:edit:follows",          doc: "Edit a user’s follows.";
    UserManageBlockedUsers,   scope: "user:manage:blocked_users",  doc: "Manage the block list of a user.";
    UserReadBlockedUsers,     scope: "user:read:blocked_users",    doc: "View the block list of a user.";
    UserReadBroadcast,        scope: "user:read:broadcast",        doc: "View a user’s broadcasting configuration, including Extension configurations.";
    UserReadEmail,            scope: "user:read:email",            doc: "Read an authorized user’s email address.";
    UserReadFollows,          scope: "user:read:follows",          doc: "View the list of channels a user follows.";
    UserReadSubscriptions,    scope: "user:read:subscriptions",    doc: "View if an authorized user is subscribed to specific channels.";
    WhispersEdit,             scope: "whispers:edit",              doc: "Send whisper messages.";
    WhispersRead,             scope: "whispers:read",              doc: "View your whisper messages.";
);

impl std::borrow::Borrow<str> for Scope {
    fn borrow(&self) -> &str { self.as_str() }
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

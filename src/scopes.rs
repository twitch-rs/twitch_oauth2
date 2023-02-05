//! Module for all possible scopes in twitch.
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

macro_rules! scope_impls {
    (@omit #[deprecated($depr:tt)] $i:ident) => {
        #[cfg(_internal_never)]
        Self::$i
    };
    (@omit $i:ident) => {
        Self::$i
    };

    ($($(#[cfg(($cfg:meta))])* $(#[deprecated($depr:meta)])? $i:ident,scope: $rename:literal, doc: $doc:literal);* $(;)? ) => {
        #[doc = "Scopes for twitch."]
        #[doc = ""]
        #[doc = "<https://dev.twitch.tv/docs/authentication/#scopes>"]
        #[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
        #[non_exhaustive]
        #[serde(from = "String")]
        #[serde(into = "String")]
        pub enum Scope {
            $(
                $(#[cfg($cfg)])*
                $(#[deprecated($depr)])*
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
                #![allow(deprecated)]

                f.write_str(match self {
                    Scope::Other(s) => &s,
                    $(
                        $(#[cfg($cfg)])*
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
                    $(
                        scope_impls!(@omit $(#[deprecated($depr)])* $i),
                    )*
                ]
            }

            #[doc = "Get a slice of all defined twitch [Scopes][Scope]."]
            #[doc = "\n\n"]
            #[doc = "Please note that this may not work for you, as some auth flows and \"apis\" don't accept all scopes"]
            pub const fn all_slice() -> &'static [Scope] {
                &[
                    $(
                        scope_impls!(@omit $(#[deprecated($depr)])* $i),
                    )*
                ]
            }

            #[doc = "Get a description for the token"]
            pub const fn description(&self) -> &'static str {
                #![allow(deprecated)]

                match self {

                    $(
                        $(#[cfg($cfg)])*
                        Self::$i => $doc,
                    )*
                    _ => "unknown scope"
                }
            }

            #[doc = "Make a scope from a cow string"]
            pub fn parse<C>(s: C) -> Scope where C: Into<Cow<'static, str>> {
                #![allow(deprecated)]
                use std::borrow::Borrow;
                let s = s.into();
                match s.borrow() {

                    $(
                        $(#[cfg($cfg)])*
                        $rename => {Scope::$i}
                    )*,
                    _ => Scope::Other(s)
                }
            }

            /// Get the scope as a borrowed string.
            pub fn as_str(&self) -> &str {
                #![allow(deprecated)]
                match self {
                    $(
                        $(#[cfg($cfg)])*
                        Scope::$i => $rename,
                    )*
                    Self::Other(c) => c.as_ref()
                }
            }

            /// Get the scope as a static string slice.
            ///
            /// ## Panics
            ///
            /// This function panics if the scope can't be represented as a static string slice
            pub const fn as_static_str(&self) -> &'static str {
                #![allow(deprecated)]
                match self {
                    $(
                        $(#[cfg($cfg)])*
                        Scope::$i => $rename,
                    )*
                    Self::Other(Cow::Borrowed(s)) => s,
                    _ => panic!(),
                }
            }
        }
        #[test]
        #[cfg(test)]
        fn sorted() {
            let slice = [$(
                $(#[cfg($cfg)])*
                $rename,
            )*];
            let mut slice_sorted = [$(
                $(#[cfg($cfg)])*
                $rename,
            )*];
            slice_sorted.sort();
            for (scope, sorted) in slice.iter().zip(slice_sorted.iter()) {
                assert_eq!(scope, sorted);
            }
            assert_eq!(slice, slice_sorted);
        }
    };
}

scope_impls!(
    AnalyticsReadExtensions,        scope: "analytics:read:extensions",         doc: "View analytics data for the Twitch Extensions owned by the authenticated account.";
    AnalyticsReadGames,             scope: "analytics:read:games",              doc: "View analytics data for the games owned by the authenticated account.";
    BitsRead,                       scope: "bits:read",                         doc: "View Bits information for a channel.";
    ChannelEditCommercial,          scope: "channel:edit:commercial",           doc: "Run commercials on a channel.";
    ChannelManageBroadcast,         scope: "channel:manage:broadcast",          doc: "Manage a channel’s broadcast configuration, including updating channel configuration and managing stream markers and stream tags.";
    ChannelManageExtensions,        scope: "channel:manage:extensions",         doc: "Manage a channel’s Extension configuration, including activating Extensions.";
    ChannelManageModerators,        scope: "channel:manage:moderators",         doc: "Add or remove the moderator role from users in your channel.";
    ChannelManagePolls,             scope: "channel:manage:polls",              doc: "Manage a channel’s polls.";
    ChannelManagePredictions,       scope: "channel:manage:predictions",        doc: "Manage of channel’s Channel Points Predictions";
    ChannelManageRaids,             scope: "channel:manage:raids",              doc: "Manage a channel raiding another channel.";
    ChannelManageRedemptions,       scope: "channel:manage:redemptions",        doc: "Manage Channel Points custom rewards and their redemptions on a channel.";
    ChannelManageSchedule,          scope: "channel:manage:schedule",           doc: "Manage a channel’s stream schedule.";
    ChannelManageVideos,            scope: "channel:manage:videos",             doc: "Manage a channel’s videos, including deleting videos.";
    ChannelManageVips,              scope: "channel:manage:vips",               doc: "Add or remove the VIP role from users in your channel.";
    ChannelModerate,                scope: "channel:moderate",                  doc: "Perform moderation actions in a channel. The user requesting the scope must be a moderator in the channel.";
    ChannelReadCharity,             scope: "channel:read:charity",              doc: "Read charity campaign details and user donations on your channel.";
    ChannelReadEditors,             scope: "channel:read:editors",              doc: "View a list of users with the editor role for a channel.";
    ChannelReadGoals,               scope: "channel:read:goals",                doc: "View Creator Goals for a channel.";
    ChannelReadHypeTrain,           scope: "channel:read:hype_train",           doc: "View Hype Train information for a channel.";
    ChannelReadPolls,               scope: "channel:read:polls",                doc: "View a channel’s polls.";
    ChannelReadPredictions,         scope: "channel:read:predictions",          doc: "View a channel’s Channel Points Predictions.";
    ChannelReadRedemptions,         scope: "channel:read:redemptions",          doc: "View Channel Points custom rewards and their redemptions on a channel.";
    ChannelReadStreamKey,           scope: "channel:read:stream_key",           doc: "View an authorized user’s stream key.";
    ChannelReadSubscriptions,       scope: "channel:read:subscriptions",        doc: "View a list of all subscribers to a channel and check if a user is subscribed to a channel.";
    ChannelReadVips,                scope: "channel:read:vips",                 doc: "Read the list of VIPs in your channel.";
    #[deprecated(note = "Use `ChannelReadSubscriptions` (`channel:read:subscriptions`) instead")]
    ChannelSubscriptions,           scope: "channel_subscriptions",             doc: "Read all subscribers to your channel.";
    ChatEdit,                       scope: "chat:edit",                         doc: "Send live stream chat and rooms messages.";
    ChatRead,                       scope: "chat:read",                         doc: "View live stream chat and rooms messages.";
    ClipsEdit,                      scope: "clips:edit",                        doc: "Manage Clips for a channel.";
    ModerationRead,                 scope: "moderation:read",                   doc: "View a channel’s moderation data including Moderators, Bans, Timeouts, and Automod settings.";
    ModeratorManageAnnouncements,   scope: "moderator:manage:announcements",    doc: "Send announcements in channels where you have the moderator role.";
    ModeratorManageAutoMod,         scope: "moderator:manage:automod",          doc: "Manage messages held for review by AutoMod in channels where you are a moderator.";
    ModeratorManageAutomodSettings, scope: "moderator:manage:automod_settings", doc: "Manage a broadcaster’s AutoMod settings";
    ModeratorManageBannedUsers,     scope: "moderator:manage:banned_users",     doc: "Ban and unban users.";
    ModeratorManageBlockedTerms,    scope: "moderator:manage:blocked_terms",    doc: "Manage a broadcaster’s list of blocked terms.";
    ModeratorManageChatMessages,    scope: "moderator:manage:chat_messages",    doc: "Delete chat messages in channels where you have the moderator role";
    ModeratorManageChatSettings,    scope: "moderator:manage:chat_settings",    doc: "View a broadcaster’s chat room settings.";
    ModeratorManageShieldMode,      scope: "moderator:manage:shield_mode",      doc: "Manage a broadcaster’s Shield Mode status.";
    ModeratorManageShoutouts,       scope: "moderator:manage:shoutouts",        doc: "Manage a broadcaster’s shoutouts.";
    ModeratorReadAutomodSettings,   scope: "moderator:read:automod_settings",   doc: "View a broadcaster’s AutoMod settings.";
    ModeratorReadBlockedTerms,      scope: "moderator:read:blocked_terms",      doc: "View a broadcaster’s list of blocked terms.";
    ModeratorReadChatSettings,      scope: "moderator:read:chat_settings",      doc: "View a broadcaster’s chat room settings.";
    ModeratorReadChatters,          scope: "moderator:read:chatters",           doc: "View the chatters in a broadcaster’s chat room.";
    ModeratorReadFollowers,         scope: "moderator:read:followers",          doc: "Read the followers of a broadcaster.";
    ModeratorReadShieldMode,        scope: "moderator:read:shield_mode",        doc: "View a broadcaster’s Shield Mode status.";
    ModeratorReadShoutouts,         scope: "moderator:read:shoutouts",          doc: "View a broadcaster’s shoutouts.";
    UserEdit,                       scope: "user:edit",                         doc: "Manage a user object.";
    UserEditBroadcast,              scope: "user:edit:broadcast",               doc: "Edit your channel's broadcast configuration, including extension configuration. (This scope implies user:read:broadcast capability.)";
    #[deprecated(note = "Not used anymore, see https://discuss.dev.twitch.tv/t/deprecation-of-create-and-delete-follows-api-endpoints/32351")]
    UserEditFollows,                scope: "user:edit:follows",                 doc: "\\[DEPRECATED\\] Was previously used for “Create User Follows” and “Delete User Follows.";
    UserManageBlockedUsers,         scope: "user:manage:blocked_users",         doc: "Manage the block list of a user.";
    UserManageChatColor,            scope: "user:manage:chat_color",            doc: "Update the color used for the user’s name in chat.Update User Chat Color";
    UserManageWhispers,             scope: "user:manage:whispers",              doc: "Read whispers that you send and receive, and send whispers on your behalf.";
    UserReadBlockedUsers,           scope: "user:read:blocked_users",           doc: "View the block list of a user.";
    UserReadBroadcast,              scope: "user:read:broadcast",               doc: "View a user’s broadcasting configuration, including Extension configurations.";
    UserReadEmail,                  scope: "user:read:email",                   doc: "View a user’s email address.";
    UserReadFollows,                scope: "user:read:follows",                 doc: "View the list of channels a user follows.";
    UserReadSubscriptions,          scope: "user:read:subscriptions",           doc: "View if an authorized user is subscribed to specific channels.";
    WhispersEdit,                   scope: "whispers:edit",                     doc: "Send whisper messages.";
    WhispersRead,                   scope: "whispers:read",                     doc: "View your whisper messages.";
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

    #[test]
    #[allow(deprecated)]
    fn no_deprecated() {
        for scope in Scope::all() {
            assert!(scope != Scope::ChannelSubscriptions)
        }
    }
}

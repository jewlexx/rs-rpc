use std::default::Default;

use serde::Deserializer;

#[cfg(feature = "activity_type")]
use serde_repr::{Deserialize_repr, Serialize_repr};

use super::events::PartialUser;
use crate::utils;

/// Args to set Discord activity
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct SetActivityArgs {
    pid: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    activity: Option<Activity>,
}

impl SetActivityArgs {
    /// Create a new `SetActivityArgs`
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(Activity) -> Activity,
    {
        Self {
            pid: utils::pid(),
            activity: Some(f(Activity::new())),
        }
    }
}

impl Default for SetActivityArgs {
    fn default() -> Self {
        Self {
            pid: utils::pid(),
            activity: None,
        }
    }
}

/// Args to invite a player to join a game
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct SendActivityJoinInviteArgs {
    /// The user to invite
    pub user_id: String,
}

/// The args to close an activity request
pub type CloseActivityRequestArgs = SendActivityJoinInviteArgs;

impl SendActivityJoinInviteArgs {
    #[must_use]
    /// Create a new `SendActivityJoinInviteArgs`
    pub fn new(user_id: u64) -> Self {
        Self {
            user_id: user_id.to_string(),
        }
    }
}

/// [`ActivityType`] enum
///
/// Lists all activity types currently supported by Discord.
///
/// This may change in future if Discord adds support for more types,
/// or removes support for some.
#[cfg(feature = "activity_type")]
#[cfg_attr(docsrs, doc(cfg(feature = "activity_type")))]
#[repr(u8)]
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize_repr, Serialize_repr, Hash)]
pub enum ActivityType {
    /// Playing a game
    Playing = 0,
    /// Listening to...
    Listening = 2,
    /// Watching...
    Watching = 3,
    /// Competing in...
    Competing = 5,
}

builder! {ActivityJoinEvent
    secret: String,
}

builder! {ActivitySpectateEvent
    secret: String,
}

builder! {ActivityJoinRequestEvent
    user: PartialUser,
}

builder! {Activity
    state: String,
    details: String,
    instance: bool,
    _type: ActivityType alias = "type" => if feature = "activity_type",
    timestamps: ActivityTimestamps func,
    assets: ActivityAssets func,
    party: ActivityParty func,
    secrets: ActivitySecrets func,
    buttons: ActivityButton as array,
}

builder! {ActivityTimestamps
    start: u64,
    end: u64,
}

builder! {ActivityAssets
    large_image: String,
    large_text: String,
    small_image: String,
    small_text: String,
}

builder! {ActivityParty
    id: String,
    size: (u32, u32),
}

builder! {ActivitySecrets
    join: String,
    spectate: String,
    game: String alias = "match",
}

// pub type ActivityButtons = Vec<ActivityButton>;

// A probably overcomplicated way to convert the array of strings returned by Discord, into buttons
fn serialize_activity_button<'de, D>(data: D) -> Result<Vec<ActivityButton>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de;
    use std::fmt;

    struct JsonStringVisitor;

    impl<'de> de::Visitor<'de> for JsonStringVisitor {
        type Value = Vec<ActivityButton>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a string containing the label for the button")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut buttons = vec![];

            while let Ok(Some(label)) = seq.next_element::<String>() {
                let button = ActivityButton {
                    label: Some(label.clone()),
                    url: None,
                };

                buttons.push(button);
            }

            Ok(buttons)
        }
    }

    data.deserialize_any(JsonStringVisitor)
}

builder! {ActivityButton
    // Text shown on the button (1-32 characters)
    label: String,
    // URL opened when clicking the button (1-512 characters)
    url: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn can_serialize_full_activity() {
        let expected = include_str!("../../tests/fixtures/activity_full.json");
        let parsed_expected = serde_json::from_str::<Activity>(expected).unwrap();

        let activity = Activity::new()
            .state("rusting")
            .details("detailed")
            .instance(true)
            .timestamps(|t| t.start(1000).end(2000))
            .assets(|a| {
                a.large_image("ferris")
                    .large_text("Ferris")
                    .small_image("rusting")
                    .small_text("Rusting...")
            })
            .append_buttons(|button| button.label("Click Me!"))
            .party(|p| p.id(String::from("party")).size((3, 6)))
            .secrets(|s| {
                s.join("025ed05c71f639de8bfaa0d679d7c94b2fdce12f")
                    .spectate("e7eb30d2ee025ed05c71ea495f770b76454ee4e0")
                    .game("4b2fdce12f639de8bfa7e3591b71a0d679d7c93f")
            });

        assert_eq!(parsed_expected, activity);
    }

    #[test]
    fn can_serialize_empty_activity() {
        let activity = Activity::new();
        let json = serde_json::to_string(&activity).expect("Failed to serialize into String");
        assert_eq![json, "{}"];
    }
}

#[cfg(test)]
#[cfg(feature = "activity_type")]
mod activity_type_tests {
    use super::*;

    #[test]
    fn can_serialize_activity_type() {
        let activity = Activity::new()._type(ActivityType::Watching);
        let json = serde_json::to_string(&activity).expect("Failed to serialize into String");

        assert_eq![json, r#"{"type":3}"#];
    }
}

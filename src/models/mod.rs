/// The Discord commands module
pub mod commands;
/// The events module
pub mod events;
/// The module to handle messages
pub mod message;
/// The module to handle payloads
pub mod payload;
/// The rich presence module
pub mod rich_presence;

use quork::traits::list::ListVariants;

/// Different Discord commands
#[derive(Debug, PartialEq, Eq, Copy, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Command {
    /// Dispatch something to Discord
    Dispatch,
    /// Authorize connection
    Authorize,
    /// Subscribe to an event
    Subscribe,
    /// Unsubscribe from Discord
    Unsubscribe,
    /// Set the current user's activity
    SetActivity,
    /// Send an invite to join a game
    SendActivityJoinInvite,
    /// Close the invite to join a game
    CloseActivityRequest,
}

// NOTE: ListVariants is required to bevy-discord-rpc
/// Discord events
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Copy, Clone, Hash, ListVariants)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Event {
    /// [`Event::Ready`] event, fired when the client is ready, but not if an error occurs
    Ready,
    /// [`Event::Error`] event, overrides the `Ready` event
    Error,
    /// [`Event::ActivityJoin`] event, fired when the client's game is joined by a player
    ActivityJoin,
    /// [`Event::ActivitySpectate`] event, fired when the client receives a spectate request
    ActivitySpectate,
    /// [`Event::ActivityJoinRequest`] event, fired when the client receives a join request
    ActivityJoinRequest,
}

impl Event {
    #[must_use]
    /// Parse event data from a [`JsonValue`]
    pub fn parse_data(self, data: JsonValue) -> EventData {
        match self {
            Event::Ready => serde_json::from_value(data.clone())
                .map(EventData::Ready)
                .unwrap_or(EventData::Unknown(data)),

            Event::Error => serde_json::from_value(data.clone())
                .map(EventData::Error)
                .unwrap_or(EventData::Unknown(data)),

            Event::ActivityJoin => serde_json::from_value(data.clone())
                .map(EventData::ActivityJoin)
                .unwrap_or(EventData::Unknown(data)),

            Event::ActivitySpectate => serde_json::from_value(data.clone())
                .map(EventData::ActivitySpectate)
                .unwrap_or(EventData::Unknown(data)),

            Event::ActivityJoinRequest => serde_json::from_value(data.clone())
                .map(EventData::ActivityJoinRequest)
                .unwrap_or(EventData::Unknown(data)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
/// Internal data for the [`Event`] enum
pub enum EventData {
    /// [`EventData::Ready`] event data
    Ready(ReadyEvent),
    /// [`EventData::Error`] event data
    Error(ErrorEvent),
    /// [`EventData::ActivityJoin`] event data
    ActivityJoin(ActivityJoinEvent),
    /// [`EventData::ActivitySpectate`] event data
    ActivitySpectate(ActivitySpectateEvent),
    /// [`EventData::ActivityJoinRequest`] event data
    ActivityJoinRequest(ActivityJoinRequestEvent),
    /// [`EventData::Unknown`] event data
    Unknown(JsonValue),
}

pub use commands::*;
pub use events::*;
pub use message::{Message, OpCode};

pub use rich_presence::*;
use serde_json::Value as JsonValue;

/// Prelude for all Discord RPC types
pub mod prelude {
    pub use super::commands::{Subscription, SubscriptionArgs};
    pub use super::events::{ErrorEvent, ReadyEvent};
    pub use super::rich_presence::{
        ActivityJoinEvent, ActivityJoinRequestEvent, ActivitySpectateEvent,
        CloseActivityRequestArgs, SendActivityJoinInviteArgs, SetActivityArgs,
    };
    pub use super::Command;
    pub use super::Event;
}

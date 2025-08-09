use std::{any::Any, collections::HashMap};

use dioxus::prelude::*;
use futures_util::stream::StreamExt;

/// Trait for types that can provide a unique identifier for their event type
///
/// This trait enables types to be used with the event bus system by providing
/// a static method to retrieve their unique event identifier. Types implementing
/// this trait can automatically implement the [`Event`] trait through the blanket
/// implementation.
///
/// # Examples
///
/// ```
/// #[derive(Debug, Clone)]
/// struct MyEvent {
///     data: String,
/// }
///
/// impl EventId for MyEvent {
///     fn event_id() -> &'static str {
///         "my_event"
///     }
/// }
/// ```
pub trait EventId {
    /// Returns a unique identifier for this event type
    ///
    /// This identifier is used by the event bus to route events to the correct
    /// subscribers. It must be unique across all event types in the system.
    fn event_id() -> &'static str;
}

/// Trait for events that can be published through the event bus
///
/// This trait defines the core functionality required for types to participate
/// in the event bus system. Events must be debuggable and cloneable to support
/// multiple subscribers and logging.
///
/// Most types should not implement this trait directly. Instead, implement
/// [`EventId`] and the blanket implementation will automatically provide
/// [`Event`] functionality.
///
/// # Examples
///
/// ```
/// // Don't implement Event directly - implement EventId instead
/// #[derive(Debug, Clone)]
/// struct MyEvent {
///     message: String,
/// }
///
/// impl EventId for MyEvent {
///     fn event_id() -> &'static str {
///         "my_event"
///     }
/// }
/// // MyEvent now automatically implements Event
/// ```
pub trait Event: core::fmt::Debug + CloneEvent {
    /// Returns a unique identifier for this event type
    ///
    /// This instance method returns the same identifier as the associated
    /// [`EventId::event_id`] static method for types that implement both traits.
    fn event_id(&self) -> &'static str;

    /// Converts this boxed event into a boxed `Any` for type downcasting
    ///
    /// This method enables the event bus to downcast generic event trait objects
    /// back to their concrete types when delivering to type-specific subscribers.
    fn into_box_any(self: Box<Self>) -> Box<dyn Any>;
}

/// Blanket implementation of [`Event`] for types implementing [`EventId`]
///
/// This implementation automatically provides [`Event`] functionality for any type
/// that implements [`EventId`], [`Clone`], [`Debug`], and has a `'static` lifetime.
impl<E: EventId + Clone + 'static + core::fmt::Debug> Event for E {
    fn event_id(&self) -> &'static str {
        E::event_id()
    }
    fn into_box_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

/// Trait for cloning event trait objects
///
/// This trait enables cloning of `Box<dyn Event>` by providing a method that
/// returns a new boxed trait object. This is necessary because trait objects
/// cannot implement `Clone` directly due to object safety requirements.
///
/// This trait is automatically implemented for all types that implement [`Event`]
/// and [`Clone`], so it should not be implemented manually.
pub trait CloneEvent {
    /// Creates a cloned boxed event trait object
    ///
    /// Returns a new `Box<dyn Event>` containing a clone of this event.
    /// This enables multiple subscribers to receive copies of the same event.
    fn clone_event(&self) -> Box<dyn Event>;
}

/// Blanket implementation of [`CloneEvent`] for cloneable events
///
/// This implementation automatically provides [`CloneEvent`] functionality for any
/// type that implements [`Event`], [`Clone`], and has a `'static` lifetime.
impl<E: Event + Clone + 'static> CloneEvent for E {
    fn clone_event(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}

/// Implementation of [`Clone`] for boxed event trait objects
///
/// This implementation enables `Box<dyn Event>` to be cloned by delegating
/// to the [`CloneEvent::clone_event`] method. This allows events to be
/// distributed to multiple subscribers in the event bus system.
impl Clone for Box<dyn Event> {
    fn clone(&self) -> Self {
        self.clone_event()
    }
}

/// Commands for the event bus service
enum EventBusCommandInner {
    /// Subscribe to an event
    Subscribe {
        /// The event id
        event_id: &'static str,
        /// Handler function
        handler: Box<dyn Fn(Box<dyn Event>)>,
    },
    /// Publish an event
    Publish {
        /// Event data
        event: Box<dyn Event>,
    },
    // /// Subscribe to database reload events
    // SubscribeDatabaseReload {
    //     /// Handler function
    //     handler: Box<dyn Fn(DatabaseReloadEvent) + Send + Sync>,
    // },
    // /// Subscribe to heritage service config change events
    // SubscribeHeritageServiceConfigChanged {
    //     /// Handler function
    //     handler: Box<dyn Fn(HeritageServiceConfigChangedEvent) + Send + Sync>,
    // },
    // /// Subscribe to blockchain provider config change events
    // SubscribeBlockchainProviderConfigChanged {
    //     /// Handler function
    //     handler: Box<dyn Fn(BlockchainProviderConfigChangedEvent) + Send + Sync>,
    // },
    // /// Publish a database reload event
    // PublishDatabaseReload {
    //     /// Event data
    //     event: DatabaseReloadEvent,
    // },
    // /// Publish a heritage service config change event
    // PublishHeritageServiceConfigChanged {
    //     /// Event data
    //     event: HeritageServiceConfigChangedEvent,
    // },
    // /// Publish a blockchain provider config change event
    // PublishBlockchainProviderConfigChanged {
    //     /// Event data
    //     event: BlockchainProviderConfigChangedEvent,
    // },
}

impl std::fmt::Debug for EventBusCommandInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Subscribe { event_id, .. } => f
                .debug_struct("Subscribe")
                .field("event_id", event_id)
                .finish_non_exhaustive(),
            Self::Publish { event } => f.debug_struct("Publish").field("event", event).finish(),
            // Self::SubscribeDatabaseReload { .. } => f
            //     .debug_struct("SubscribeDatabaseReload")
            //     .finish_non_exhaustive(),
            // Self::SubscribeHeritageServiceConfigChanged { .. } => f
            //     .debug_struct("SubscribeHeritageServiceConfigChanged")
            //     .finish_non_exhaustive(),
            // Self::SubscribeBlockchainProviderConfigChanged { .. } => f
            //     .debug_struct("SubscribeBlockchainProviderConfigChanged")
            //     .finish_non_exhaustive(),
            // Self::PublishDatabaseReload { event } => f
            //     .debug_struct("PublishDatabaseReload")
            //     .field("event", event)
            //     .finish(),
            // Self::PublishHeritageServiceConfigChanged { event } => f
            //     .debug_struct("PublishHeritageServiceConfigChanged")
            //     .field("event", event)
            //     .finish(),
            // Self::PublishBlockchainProviderConfigChanged { event } => f
            //     .debug_struct("PublishBlockchainProviderConfigChanged")
            //     .field("event", event)
            //     .finish(),
        }
    }
}
pub struct EventBusCommand(EventBusCommandInner);
pub type EventBus = Coroutine<EventBusCommand>;
/// Event bus service coroutine
pub(super) fn use_event_bus_service() -> EventBus {
    use_coroutine(
        move |mut rx: UnboundedReceiver<EventBusCommand>| async move {
            log::info!("event_bus_service (coroutine) - start");

            let mut subscribers: HashMap<&str, Vec<Box<dyn Fn(Box<dyn Event>)>>> = HashMap::new();

            // let mut database_reload_subscribers: Vec<
            //     Box<dyn Fn(DatabaseReloadEvent) + Send + Sync>,
            // > = Vec::new();
            // let mut heritage_service_subscribers: Vec<
            //     Box<dyn Fn(HeritageServiceConfigChangedEvent) + Send + Sync>,
            // > = Vec::new();
            // let mut blockchain_provider_subscribers: Vec<
            //     Box<dyn Fn(BlockchainProviderConfigChangedEvent) + Send + Sync>,
            // > = Vec::new();

            while let Some(cmd) = rx.next().await {
                let EventBusCommand(cmd) = cmd;
                log::debug!("event_bus_service (coroutine) - Processing command {cmd:?}...");

                match cmd {
                    EventBusCommandInner::Subscribe { event_id, handler } => {
                        subscribers
                            .entry(event_id)
                            .or_insert(Vec::new())
                            .push(handler);
                        log::debug!("event_bus_service - Subscribed to {event_id} events");
                    }
                    EventBusCommandInner::Publish { event } => {
                        let event_id = event.event_id();
                        let sub_vec = subscribers.get(&event_id);
                        log::debug!(
                            "event_bus_service - Publishing {event_id} event to {} subscribers",
                            sub_vec.map(|v| v.len()).unwrap_or_default()
                        );
                        if let Some(sub_vec) = sub_vec {
                            for handler in sub_vec {
                                handler(event.clone());
                            }
                        }
                    } // EventBusCommand::SubscribeDatabaseReload { handler } => {
                      //     database_reload_subscribers.push(handler);
                      //     log::debug!("event_bus_service - Subscribed to DatabaseReload events");
                      // }
                      // EventBusCommand::SubscribeHeritageServiceConfigChanged { handler } => {
                      //     heritage_service_subscribers.push(handler);
                      //     log::debug!(
                      //         "event_bus_service - Subscribed to HeritageServiceConfigChanged events"
                      //     );
                      // }
                      // EventBusCommand::SubscribeBlockchainProviderConfigChanged { handler } => {
                      //     blockchain_provider_subscribers.push(handler);
                      //     log::debug!("event_bus_service - Subscribed to BlockchainProviderConfigChanged events");
                      // }
                      // EventBusCommand::PublishDatabaseReload { event } => {
                      //     log::debug!(
                      //         "event_bus_service - Publishing DatabaseReload event to {} subscribers",
                      //         database_reload_subscribers.len()
                      //     );
                      //     for handler in &database_reload_subscribers {
                      //         handler(event.clone());
                      //     }
                      // }
                      // EventBusCommand::PublishHeritageServiceConfigChanged { event } => {
                      //     log::debug!(
                      //         "event_bus_service - Publishing HeritageServiceConfigChanged \
                      //             event to {} subscribers",
                      //         heritage_service_subscribers.len()
                      //     );
                      //     for handler in &heritage_service_subscribers {
                      //         handler(event.clone());
                      //     }
                      // }
                      // EventBusCommand::PublishBlockchainProviderConfigChanged { event } => {
                      //     log::debug!(
                      //         "event_bus_service - Publishing BlockchainProviderConfigChanged \
                      //             event to {} subscribers",
                      //         blockchain_provider_subscribers.len()
                      //     );
                      //     for handler in &blockchain_provider_subscribers {
                      //         handler(event.clone());
                      //     }
                      // }
                }
                log::debug!("event_bus_service (coroutine) - Command processed");
            }
        },
    )
}

pub fn publish_event<E: Event + 'static>(event_bus_service: EventBus, event: E) {
    event_bus_service.send(EventBusCommand(EventBusCommandInner::Publish {
        event: Box::new(event),
    }));
}

pub fn subscribe_event<E: Event + EventId + 'static, F: Fn(E) + 'static>(
    event_bus_service: EventBus,
    handler: F,
) {
    let event_id = <E as EventId>::event_id();
    let handler = Box::new(move |boxed: Box<dyn Event>| {
        let event = *boxed.into_box_any().downcast().unwrap();
        handler(event)
    });
    event_bus_service.send(EventBusCommand(EventBusCommandInner::Subscribe {
        event_id,
        handler,
    }));
}

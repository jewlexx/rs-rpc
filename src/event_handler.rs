use std::{collections::HashMap, sync::Arc};
use std::{sync::Weak, thread};

use parking_lot::RwLock;

use crate::models::{Event, EventData};

type Handler = dyn Fn(Context) + 'static + Send + Sync;

type HandlerList = Vec<Arc<Handler>>;

#[derive(Debug, Clone)]
pub struct Context {
    // TODO: implement event data structures
    pub event: EventData,
}

impl Context {
    pub fn new(event: EventData) -> Self {
        Self { event }
    }
}

type Handlers = RwLock<HashMap<Event, HandlerList>>;

#[must_use = "event listeners will be immediately dropped if the handle is not kept. Use `.persist` to stop them from being removed."]
pub struct EventCallbackHandle {
    event: Event,
    registry: Weak<HandlerRegistry>,
    handler: Weak<Handler>,
}

impl EventCallbackHandle {
    /// Immediately drops the event handler, thus removing the handler from the registry.
    pub fn remove(self) {
        drop(self);
    }

    /// "Forgets" event handler, removing the variable, but keeping the handler in the registry until the registry itself is dropped.
    pub fn persist(self) {
        std::mem::forget(self);
    }
}

impl Drop for EventCallbackHandle {
    fn drop(&mut self) {
        // if the registry or this event handler has already been dropped, there's no reason to try and do it again
        if let (Some(registry), Some(handler)) = (self.registry.upgrade(), self.handler.upgrade()) {
            let handler = registry.remove(self.event, &handler);
            if handler.is_err() {
                error!("Failed to remove event handler. This can usually be ignored.");
            }
        }
    }
}

pub struct HandlerRegistry {
    handlers: Handlers,
}

impl HandlerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
        }
    }

    pub fn register<F>(self: &Arc<Self>, event: Event, handler: F) -> EventCallbackHandle
    where
        F: Fn(Context) + Send + Sync + 'static,
    {
        let handler: Arc<Handler> = Arc::new(handler);
        let callback_handle = EventCallbackHandle {
            event,
            registry: Arc::downgrade(self),
            handler: Arc::downgrade(&handler),
        };

        let mut event_handlers = self.handlers.write();
        let event_handler = event_handlers.entry(event).or_default();
        event_handler.push(handler);

        callback_handle
    }

    // TODO: Replace data type with stronger types
    pub fn handle(&self, event: Event, data: EventData) {
        let handlers = self.handlers.read();
        if let Some(handlers) = handlers.get(&event) {
            let context = Context::new(data);

            for handler in handlers {
                let handler = handler.clone();
                let context = context.clone();
                thread::spawn(move || {
                    handler(context);
                });
            }
        }
    }

    /// Removes a handler from the registry, if it exists
    ///
    /// # Errors
    /// - Returns an error if no changes were made to the registry. This generally means that the handler has already been removed, and can thus generally be ignored.
    // TODO: Change return type to Result
    pub fn remove(
        self: &Arc<Self>,
        event: Event,
        target: &Arc<Handler>,
    ) -> crate::Result<Arc<Handler>> {
        let mut handlers = self.handlers.write();
        if let Some(handlers) = handlers.get_mut(&event) {
            if let Some(index) = handlers
                .iter()
                .position(|handler| Arc::ptr_eq(handler, target))
            {
                return Ok(handlers.remove(index));
            }
        }

        Err(crate::DiscordError::NoChangesMade)
    }
}

#[cfg(test)]
mod tests {
    use std::mem::forget;

    use super::*;

    #[test]
    fn can_register_event_handlers() {
        let registry = Arc::new(HandlerRegistry::new());
        let _ready1 = registry.register(Event::Ready, |_| unimplemented!());
        let _ready2 = registry.register(Event::Ready, |_| unimplemented!());
        let _error = registry.register(Event::Error, |_| unimplemented!());

        let handlers = registry.handlers.read();
        assert_eq!(handlers.len(), 2);
        assert_eq!(handlers[&Event::Ready].len(), 2);
        assert_eq!(handlers[&Event::Error].len(), 1);
    }

    /// Removes event handlers once they go out of scope to prevent memory leaks
    #[test]
    fn auto_remove_event_handlers() {
        let registry = Arc::new(HandlerRegistry::new());
        let _ready1 = registry.register(Event::Ready, |_| unimplemented!());
        let _error = registry.register(Event::Error, |_| unimplemented!());

        {
            let _ready2 = registry.register(Event::Ready, |_| unimplemented!());
        }
        // _ready2 is automatically removed

        let handlers = registry.handlers.read();
        assert_eq!(handlers.len(), 2);
        assert_eq!(handlers[&Event::Ready].len(), 1);
        assert_eq!(handlers[&Event::Error].len(), 1);
    }

    /// Enables keeping an event callback for the entire lifetime of the client.
    /// This disables the functionality tested in `auto_remove_event_handlers`.
    #[test]
    fn forget_cb_handles() {
        let registry = Arc::new(HandlerRegistry::new());

        {
            let ready = registry.register(Event::Ready, |_| unimplemented!());
            // skip the Drop impl by running std::mem::forget
            forget(ready);
        }
        // _ready2 is not automatically removed

        let handlers = registry.handlers.read();
        assert_eq!(handlers.len(), 1);
        assert_eq!(handlers[&Event::Ready].len(), 1);
    }
}

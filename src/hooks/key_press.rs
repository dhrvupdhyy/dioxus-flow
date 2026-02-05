//! Key press hook

use dioxus::prelude::*;
use std::collections::HashSet;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

struct WindowListener {
    event_type: String,
    closure: Option<Closure<dyn FnMut(web_sys::KeyboardEvent)>>,
}

impl WindowListener {
    fn new(event_type: &str, handler: impl FnMut(web_sys::KeyboardEvent) + 'static) -> Self {
        let Some(window) = web_sys::window() else {
            return Self {
                event_type: event_type.to_string(),
                closure: None,
            };
        };
        let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(web_sys::KeyboardEvent)>);
        window
            .add_event_listener_with_callback(event_type, closure.as_ref().unchecked_ref())
            .ok();

        Self {
            event_type: event_type.to_string(),
            closure: Some(closure),
        }
    }
}

impl Drop for WindowListener {
    fn drop(&mut self) {
        if let Some(window) = web_sys::window() {
            if let Some(closure) = &self.closure {
                window
                    .remove_event_listener_with_callback(
                        &self.event_type,
                        closure.as_ref().unchecked_ref(),
                    )
                    .ok();
            }
        }
    }
}

pub fn use_key_press(key: impl Into<String>) -> Signal<bool> {
    let key = key.into();
    let pressed = use_signal(|| false);
    let mut tracked_key = use_signal(|| key.clone());

    use_effect(move || {
        if *tracked_key.read() != key {
            tracked_key.set(key.clone());
        }
    });

    use_hook(move || {
        let mut pressed_down = pressed;
        let mut pressed_up = pressed;
        let key_down = tracked_key;
        let key_up = tracked_key;

        let listener_down = WindowListener::new("keydown", move |evt| {
            if evt.key() == *key_down.read() {
                pressed_down.set(true);
            }
        });

        let listener_up = WindowListener::new("keyup", move |evt| {
            if evt.key() == *key_up.read() {
                pressed_up.set(false);
            }
        });

        struct Cleanup {
            _down: WindowListener,
            _up: WindowListener,
        }

        Rc::new(Cleanup {
            _down: listener_down,
            _up: listener_up,
        })
    });

    pressed
}

pub fn use_key_press_multi(keys: Vec<String>) -> Signal<bool> {
    let pressed = use_signal(|| false);
    let pressed_keys = use_signal(|| HashSet::<String>::new());
    let mut tracked_keys = use_signal(|| keys.clone());

    use_effect(move || {
        if *tracked_keys.read() != keys {
            tracked_keys.set(keys.clone());
        }
    });

    use_hook(move || {
        let mut pressed_down = pressed;
        let mut pressed_up = pressed;
        let mut pressed_keys_down = pressed_keys;
        let mut pressed_keys_up = pressed_keys;
        let keys_down = tracked_keys;
        let keys_up = tracked_keys;

        let listener_down = WindowListener::new("keydown", move |evt| {
            if keys_down.read().iter().any(|key| key == &evt.key()) {
                let mut set = pressed_keys_down.write();
                set.insert(evt.key());
                pressed_down.set(!set.is_empty());
            }
        });

        let listener_up = WindowListener::new("keyup", move |evt| {
            if keys_up.read().iter().any(|key| key == &evt.key()) {
                let mut set = pressed_keys_up.write();
                set.remove(&evt.key());
                pressed_up.set(!set.is_empty());
            }
        });

        struct Cleanup {
            _down: WindowListener,
            _up: WindowListener,
        }

        Rc::new(Cleanup {
            _down: listener_down,
            _up: listener_up,
        })
    });

    pressed
}

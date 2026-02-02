//! Key press hook

use dioxus::prelude::*;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

struct WindowListener {
    event_type: String,
    closure: Option<Closure<dyn FnMut(web_sys::KeyboardEvent)>>,
}

impl WindowListener {
    fn new(event_type: &str, handler: impl FnMut(web_sys::KeyboardEvent) + 'static) -> Self {
        let window = web_sys::window().expect("window not available");
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
    let mut pressed = use_signal(|| false);

    use_hook(move || {
        let mut pressed_down = pressed;
        let mut pressed_up = pressed;
        let key_down = key.clone();
        let key_up = key.clone();

        let listener_down = WindowListener::new("keydown", move |evt| {
            if evt.key() == key_down {
                pressed_down.set(true);
            }
        });

        let listener_up = WindowListener::new("keyup", move |evt| {
            if evt.key() == key_up {
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
    let mut pressed = use_signal(|| false);
    let mut pressed_keys = use_signal(|| HashSet::<String>::new());

    use_hook(move || {
        let mut pressed_down = pressed;
        let mut pressed_up = pressed;
        let mut pressed_keys_down = pressed_keys;
        let mut pressed_keys_up = pressed_keys;
        let keys_down = keys.clone();
        let keys_up = keys.clone();

        let listener_down = WindowListener::new("keydown", move |evt| {
            if keys_down.iter().any(|key| key == &evt.key()) {
                let mut set = pressed_keys_down.write();
                set.insert(evt.key());
                pressed_down.set(!set.is_empty());
            }
        });

        let listener_up = WindowListener::new("keyup", move |evt| {
            if keys_up.iter().any(|key| key == &evt.key()) {
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

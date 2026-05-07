use leptos::wasm_bindgen::{JsCast, prelude::Closure};
use leptos::web_sys::EventTarget;
use leptos::{leptos_dom::helpers::TimeoutHandle, prelude::*};
use std::{sync::Arc, time::Duration};

pub struct EventListenerHandle(Box<dyn FnOnce() + Send + Sync>);

impl std::fmt::Debug for EventListenerHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("EventListenerHandle").finish()
    }
}

impl EventListenerHandle {
    pub fn remove(self) {
        (self.0)();
    }
}

pub fn add_event_listener_with_bool<E: leptos::ev::EventDescriptor + 'static>(
    target: impl Into<EventTarget>,
    event: E,
    cb: impl Fn(E::EventType) + 'static,
    use_capture: bool,
) -> EventListenerHandle
where
    E::EventType: JsCast,
{
    add_event_listener_untyped_with_bool(
        target,
        &event.name(),
        move |e| cb(e.unchecked_into::<E::EventType>()),
        use_capture,
    )
}

fn add_event_listener_untyped_with_bool(
    target: impl Into<EventTarget>,
    event_name: &str,
    cb: impl Fn(leptos::web_sys::Event) + 'static,
    use_capture: bool,
) -> EventListenerHandle {
    fn wel(
        target: EventTarget,
        cb: Box<dyn FnMut(leptos::web_sys::Event)>,
        event_name: &str,
        use_capture: bool,
    ) -> EventListenerHandle {
        let cb = Closure::wrap(cb).into_js_value();
        _ = target.add_event_listener_with_callback_and_bool(
            event_name,
            cb.unchecked_ref(),
            use_capture,
        );

        EventListenerHandle({
            let event_name = event_name.to_string();
            let cb = send_wrapper::SendWrapper::new(cb);
            let target = send_wrapper::SendWrapper::new(target);
            Box::new(move || {
                let _ = target.remove_event_listener_with_callback_and_bool(
                    &event_name,
                    cb.unchecked_ref(),
                    use_capture,
                );
            })
        })
    }

    wel(target.into(), Box::new(cb), event_name, use_capture)
}

pub fn throttle(cb: impl Fn() + Send + Sync + 'static, duration: Duration) -> impl Fn() -> () {
    let cb = Arc::new(cb);
    let timeout_handle = StoredValue::new(None::<TimeoutHandle>);
    on_cleanup(move || {
        timeout_handle.update_value(move |handle| {
            if let Some(handle) = handle.take() {
                handle.clear();
            }
        });
    });

    move || {
        if timeout_handle.with_value(|handle| handle.is_some()) {
            return;
        }
        let cb = cb.clone();
        let handle = set_timeout_with_handle(
            move || {
                cb();
                timeout_handle.update_value(move |handle| {
                    *handle = None;
                });
            },
            duration,
        )
        .unwrap();
        timeout_handle.set_value(Some(handle));
    }
}

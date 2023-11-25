#![allow(non_snake_case)]
use dioxus::prelude::*;
use js_sys::Array as JsArray;
use wasm_bindgen::closure::Closure as JsClosure;
use wasm_bindgen::JsCast;
use web_sys::Element as JsElement;
use web_sys::ResizeObserver as JsResizeObserver;
use web_sys::ResizeObserverEntry as JsResizeObserverEntry;

// fn main() {
//   dioxus_web::launch(App);
// }

// fn App(cx: Scope) -> Element {
//   let size = use_ref(cx, ComponentSize::default);

//   let on_resize = use_state(cx, || {
//     to_owned![size];
//     OnResize::new(move |new_size: ComponentSize| {
//       size.set(new_size);
//     })
//   });

//   render! {
//     div {
//       onmounted: |event| {
//         on_resize.mount(event);
//       },
//       ul {
//         li {
//           "Generation: {cx.generation()}"
//         },
//         li {
//           "Size: {size.read().width} x {size.read().height}"
//         }
//       }
//     }
//   }
// }

#[derive(Clone, Copy)]
pub struct ComponentSize {
    pub width: i64,
    pub height: i64,
}

impl Default for ComponentSize {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
        }
    }
}

pub struct OnResize {
    // We keep ownership of that one so that it doesn't get dropped
    // when the constructor ends
    _js_closure: JsClosure<dyn FnMut(JsArray)>,
}
impl OnResize {
    pub fn new(mut callback: impl FnMut(ComponentSize) + 'static) -> Self {
        let _js_closure = JsClosure::<dyn FnMut(JsArray)>::new(move |entries: JsArray| {
            callback(Self::_get_size(entries));
        });

        Self {
            _js_closure: _js_closure,
        }
    }

    pub fn mount(&self, event: Event<MountedData>) {
        let element = event
            .get_raw_element()
            .expect("Failed to get element")
            .downcast_ref::<JsElement>()
            .expect("Not a `JsElement`");

        JsResizeObserver::new(self._js_closure.as_ref().unchecked_ref())
            .expect("Failed to create observer")
            .observe(element);
    }

    fn _get_size(entries: JsArray) -> ComponentSize {
        // TODO
        // We _can_ have several entries here (several elements observed).
        //
        // Using one observer for every element that setup a `onresize` event
        // _may_ save some memory and cpu time.
        //
        // Although it implies checking the obtained size in each entry
        // against the old one, and send the event only if changed.
        // But it seems there are many cases where an observer can trigger while
        // the element size hasn't actually changed.
        //
        // It also implies finding a way to tie the entry to the corresponding
        // Dioxus element.
        //
        // INVESTIGATE
        // https://drafts.csswg.org/resize-observer

        let entry = entries.at(0);
        let entry: JsResizeObserverEntry = entry.dyn_into().expect("Not a `JsResizeObserverEntry`");
        let rect = entry.content_rect();

        ComponentSize {
            width: rect.width() as i64,
            height: rect.height() as i64,
        }
    }
}

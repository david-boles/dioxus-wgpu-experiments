#![allow(non_snake_case)]
use console_log::log;
use dioxus::html::geometry::{PixelsVector, WheelDelta};
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use dioxus::prelude::*;
// use dioxus_helmet::Helmet;
use futures_util::stream::StreamExt;
use js_sys::is_finite;
use js_sys::Math::random;
use lazy_static::lazy_static;
use log::info;
use rand::{distributions::Uniform, Rng};
use std::borrow::BorrowMut;
use std::cell::Cell;
use std::cmp::{max, min};
use std::num::{NonZeroU16, NonZeroU64};
use std::{
    iter,
    ops::{RangeInclusive, RangeToInclusive},
    sync::mpsc::TryRecvError,
};
use web_sys::{HtmlCanvasElement, Text};
use wgpu::{
    util::{DeviceExt, RenderEncoder},
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BufferBinding, ShaderStages,
};
use wgpu::{
    BlendComponent, BlendFactor, BlendOperation, BufferDescriptor, BufferUsages, CompareFunction,
    CompositeAlphaMode, DepthBiasState, DepthStencilState, Extent3d, Operations,
    RenderPassDepthStencilAttachment, StencilState, SurfaceConfiguration, Texture,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    raw_window_handle::{WebCanvasWindowHandle, WebWindowHandle},
    window::WindowBuilder,
};

use wasm_bindgen::prelude::*;

// mod decimate;

mod render;
mod resize;

use render::*;
use resize::*;

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
    // launch the web app
    dioxus_web::launch(App);
}

// create a component that renders a div with the text "Hello, world!"
fn App(cx: Scope) -> Element {
    let my_str = use_state(cx, || "Hello world!".to_owned());

    // let plot = use_state(cx, || None);

    let render_coroutine = use_coroutine(cx, |mut rx: UnboundedReceiver<CanvasEvent>| {
        to_owned![my_str];
        async move { Plot::new(my_str).await.run_event_loop(rx).await }
    });

    let size = use_ref(cx, ComponentSize::default);

    let on_resize = use_state(cx, {
        to_owned![render_coroutine];
        || {
            to_owned![size];
            OnResize::new(move |new_size: ComponentSize| {
                render_coroutine.send(CanvasEvent::Resize(new_size));
                size.set(new_size);
            })
        }
    });

    cx.render(rsx! {
        div { height: "100vh",
            "{my_str}"
            br {}
            "Size: {size.read().width} x {size.read().height}"
            br {}
            div { height: "80%",
                canvas {
                    id: "my-canvas",
                    style: "width: 100%; height: 100%;",
                    image_rendering: "pixelated",
                    background: "#222",
                    onmounted: |event| {
                        on_resize.mount(event);
                    },
                    onwheel: |event: dioxus::core::Event<WheelData>| {
                        if let WheelDelta::Pixels(pixels) = event.delta() {
                            render_coroutine.send(CanvasEvent::Wheel(pixels))
                        }
                        event.stop_propagation()
                    }
                }
            }
        }
    })
}

use std::{cell::Cell, rc::Rc};

use web_sys::WebSocket;
use wgpu::Device;
use ws_stream_wasm::WsMeta;

use super::Scale;

pub struct DataCache {
    device: Rc<Device>,
    target_scale: Cell<Scale>,
}

impl DataCache {
    pub fn new(device: Rc<Device>, target_scale: Scale) -> Self {
        DataCache {
            device,
            target_scale: Cell::new(target_scale),
        }
    }

    pub fn set_target_scale(&self, target_scale: Scale) {
        self.target_scale.set(target_scale)
    }

    pub async fn run_event_loop(&self) {
        let (meta, stream) = WsMeta::connect("ws://localhost:3000/signal", None)
            .await
            .unwrap();

        meta.wrapped().send_with_str("foobar").unwrap();
    }
}

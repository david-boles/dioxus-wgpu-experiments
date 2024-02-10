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

#[derive(Clone, Copy, Debug)]
pub struct AxisScale {
    /// First point's position in some axis in some space
    pub start: f32,
    /// Last point's position in some axis in some space
    pub end: f32,
}

impl AxisScale {
    const NAN: Self = Self {
        start: f32::NAN,
        end: f32::NAN,
    };

    pub fn diff(&self) -> f32 {
        self.end - self.start
    }

    pub fn scale_pct(&mut self, percent: f32) {
        let diff = self.diff() * percent;
        let mid = self.start + (self.diff() / 2.0);
        self.start = mid - (diff / 2.0);
        self.end = mid + (diff / 2.0);
    }

    pub fn shift_pct(&mut self, percent: f32) {
        let diff = self.diff() * percent;
        self.start += diff;
        self.end += diff
    }
}

/// The scale of a set of points in some space
#[derive(Clone, Copy, Debug)]
pub struct Scale {
    pub horizontal: AxisScale,
    pub vertical: AxisScale,
}

impl Scale {
    pub const NAN: Self = Self {
        horizontal: AxisScale::NAN,
        vertical: AxisScale::NAN,
    };

    pub const RASTER: Self = Self {
        horizontal: AxisScale {
            start: -1.0,
            end: 1.0,
        },
        vertical: AxisScale {
            start: -1.0,
            end: 1.0,
        },
    };

    /// WGSL mat3x2f (2x3) to convert from a scale in one space to a scale in another space.
    pub fn transform_matrix(from: Scale, to: Scale) -> [f32; 6] {
        // info!("transform");
        // info!("{from:?}");
        // info!("{to:?}");
        let h_scale = (to.horizontal.end - to.horizontal.start)
            / (from.horizontal.end - from.horizontal.start);
        let v_scale =
            (to.vertical.end - to.vertical.start) / (from.vertical.end - from.vertical.start);
        // info!("{h_scale}");
        // info!("{v_scale}");
        [
            h_scale,
            0.0,
            0.0,
            v_scale,
            to.horizontal.start - (from.horizontal.start * h_scale),
            to.vertical.start - (from.vertical.start * v_scale),
        ]
    }

    pub fn transform_matrix_to(&self, to: Scale) -> [f32; 6] {
        Self::transform_matrix(*self, to)
    }

    pub fn point_scale_to_uniform(
        &self,
        px_width: u32,
        px_height: u32,
        px_margin: u32,
        color: &[f32; 3],
        alpha: f32,
        depth_ind: NonZeroU16,
    ) -> Vec<f32> {
        let px_width = px_width as f32;
        let px_height = px_height as f32;
        let px_margin = px_margin as f32;
        let px_scale = Scale {
            horizontal: AxisScale {
                start: 0.0,
                end: (px_width - 1.0),
            },
            vertical: AxisScale {
                start: (px_height - 1.0),
                end: 0.0,
            },
        };
        let px_scale_with_margin = Scale {
            horizontal: AxisScale {
                start: px_scale.horizontal.start + px_margin,
                end: px_scale.horizontal.end - px_margin,
            },
            vertical: AxisScale {
                start: px_scale.vertical.start - px_margin,
                end: px_scale.vertical.end + px_margin,
            },
        };

        let mut out = Vec::new();
        out.extend_from_slice(&self.transform_matrix_to(px_scale_with_margin));
        out.extend_from_slice(&px_scale.transform_matrix_to(Scale::RASTER));
        out.extend(color.map(|c| c * alpha)); // Premultiply
        out.push(alpha);
        out.push((depth_ind.get() as f32) / (2f32.powi(16)));
        out.push(0.0); // padding

        return out;
    }
}

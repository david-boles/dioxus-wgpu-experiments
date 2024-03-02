#![allow(non_snake_case)]
use console_log::{init, log};
use dioxus::html::geometry::{PixelsVector, WheelDelta};
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use dioxus::prelude::*;
use futures_util::join;
// use dioxus_helmet::Helmet;
use futures_util::stream::StreamExt;
use js_sys::is_finite;
use js_sys::Math::random;
use lazy_static::lazy_static;
use log::{info, log};
use rand::{distributions::Uniform, Rng};
use std::borrow::BorrowMut;
use std::cell::Cell;
use std::cmp::{max, min};
use std::num::{NonZeroU16, NonZeroU64};
use std::rc::Rc;
use std::{
    iter,
    ops::{RangeInclusive, RangeToInclusive},
    sync::mpsc::TryRecvError,
};
use web_sys::{HtmlCanvasElement, Text, Window};
use wgpu::{
    util::{DeviceExt, RenderEncoder},
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BufferBinding, ShaderStages,
};
use wgpu::{
    BindGroup, BlendComponent, BlendFactor, BlendOperation, Buffer, BufferDescriptor, BufferUsages,
    CompareFunction, CompositeAlphaMode, DepthBiasState, DepthStencilState, Device, Extent3d,
    Operations, PipelineLayout, Queue, RenderPassDepthStencilAttachment, RenderPipeline,
    StencilState, Surface, SurfaceConfiguration, Texture, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages,
};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    raw_window_handle::{WebCanvasWindowHandle, WebWindowHandle},
    window::WindowBuilder,
};

use data_cache::*;
use scale::*;

mod data_cache;
mod scale;

use wasm_bindgen::prelude::*;

use crate::ComponentSize;

fn f32_min(a: f32, b: f32) -> f32 {
    match (a.is_finite(), b.is_finite()) {
        (true, true) => {
            if a.lt(&b) {
                a
            } else {
                b
            }
        }
        (true, false) => a,
        (false, true) => b,
        (false, false) => f32::MAX,
    }
}

fn f32_max(a: f32, b: f32) -> f32 {
    match (a.is_finite(), b.is_finite()) {
        (true, true) => {
            if a.gt(&b) {
                a
            } else {
                b
            }
        }
        (true, false) => a,
        (false, true) => b,
        (false, false) => f32::MAX,
    }
}

// --- vertices to draw a 2x2 regtangle centered at 0,0 ---

const OFFSET_VERTICES: &[[f32; 2]] = &[[-1.0, -1.0], [-1.0, 1.0], [1.0, -1.0], [1.0, 1.0]];

const OFFSET_INDICES: &[u16] = &[
    0, 1, 2, // Bottom left
    1, 2, 3, // Top right
];

const offset_buffers_desc: &[wgpu::VertexBufferLayout] = &[wgpu::VertexBufferLayout {
    array_stride: wgpu::VertexFormat::Float32x2.size() as wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[wgpu::VertexAttribute {
        offset: 0,
        shader_location: 0,
        format: wgpu::VertexFormat::Float32x2,
    }],
}];

// --- datapoint to plot ---

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Point(f32, f32);

// const POINTS: &[Point] = &[
//     Point(-0.5, -0.5),
//     Point(0.5, 0.5),
//     Point(0.5, -0.5),
//     Point(-0.5, 0.5),
//     Point(-0.5, -0.45),
// ];

// const POINTS: &[Point] = &[Point(-1.0, 0.0), Point(0.0, 1.0), Point(1.0, 0.0)];

// Degenerate - reverse
// const POINTS: &[Point] = &[
//     Point(0.3, -0.7),
//     Point(0.49, -0.7),
//     Point(0.5, 0.7),
//     Point(0.51, -2.0),
// ];
// Degenerate - continue
// const POINTS: &[Point] = &[Point(0.0, -0.5), Point(0.0, 0.0), Point(0.0, 0.5)];

// const POINTS: &[Point] = &[Point(-1.0, -1.0), Point(1.0, 1.0)];
// const NUM_POINTS: usize = 134217728 / (8);
const NUM_POINTS: usize = 100_000;

lazy_static! {
    static ref POINTS: Vec<Point> = {
        (0..NUM_POINTS)
            .map(|i| {
                Point(
                    1.5 * (((i as f32) / (NUM_POINTS as f32)) - 0.5),
                    rand::thread_rng().sample(Uniform::from(-0.75..0.75)),
                )
            })
            .collect()
    };
    static ref POINTS2: Vec<Point> = {
        (0..NUM_POINTS)
            .map(|i| {
                Point(
                    1.5 * (((i as f32) / (NUM_POINTS as f32)) - 0.5),
                    rand::thread_rng().sample(Uniform::from(-0.75..0.75)),
                )
            })
            .collect()
    };
}

// --- pretty matlab colors ---

const c1: [f32; 3] = [0.0, 0.4470, 0.7410];
const c2: [f32; 3] = [0.8500, 0.3250, 0.0980];
const c3: [f32; 3] = [0.9290, 0.6940, 0.1250];
const c4: [f32; 3] = [0.4940, 0.1840, 0.5560];
const c5: [f32; 3] = [0.4660, 0.6740, 0.1880];
const c6: [f32; 3] = [0.3010, 0.7450, 0.9330];
const c7: [f32; 3] = [0.6350, 0.0780, 0.1840];

// --- events that can be sent to the coroutine to update the plot ---

pub enum CanvasEvent {
    Wheel(PixelsVector),
    Resize(ComponentSize),
}

pub struct Plot {
    device: Rc<Device>,
    surface: Surface,
    config: RefCell<SurfaceConfiguration>,
    queue: Queue,
    current_scale: RefCell<Scale>,
    canvas_outer_size: Cell<ComponentSize>,
    my_str: UseState<String>,
    offset_vertex_buffer: Buffer,
    offset_index_buffer: Buffer,
    uniform_buffer: Buffer,
    uniform_buffer2: Buffer,
    line_bind_group: BindGroup,
    line_bind_group2: BindGroup,
    multisample_texture: RefCell<Texture>,
    depth_texture: RefCell<Texture>,
    segment_render_pipeline: RenderPipeline,
    dot_render_pipeline: RenderPipeline,
    animation_frame_requested_but_render_not_queued: Cell<bool>,
    data_cache: DataCache,
}

impl Plot {
    pub async fn new(my_str: UseState<String>) -> Rc<Plot> {
        my_str.modify(|_| "begun".to_owned());
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("my-canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = unsafe {
            instance
                .create_surface_from_canvas(HtmlCanvasElement::try_from(canvas.clone()).unwrap())
        }
        .unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let mut config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: canvas.width(),
            height: canvas.height(),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: CompositeAlphaMode::PreMultiplied,
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let segment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("SegmentShader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("segmented_line.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let line_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("segment bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0, // TODO 0
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(NonZeroU64::new(80).unwrap()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1, // TODO 0
                    visibility: ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let segment_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Segment Render Pipeline Layout"),
                bind_group_layouts: &[&line_bind_group_layout],
                push_constant_ranges: &[],
            });

        let depth_stencil = Some(DepthStencilState {
            format: TextureFormat::Depth24Plus,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Greater, //TODO?
            bias: Default::default(),
            stencil: Default::default(),
        });

        let segment_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Segment Render Pipeline"),
                layout: Some(&segment_render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &segment_shader,
                    entry_point: "vs_line",
                    buffers: offset_buffers_desc,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &segment_shader,
                    entry_point: "fs_line",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        // blend: Some(wgpu::BlendState {
                        //     color: BlendComponent {
                        //         src_factor: BlendFactor::One,
                        //         dst_factor: BlendFactor::Zero,
                        //         operation: BlendOperation::Max,
                        //     },
                        //     alpha: BlendComponent {
                        //         src_factor: BlendFactor::One,
                        //         dst_factor: BlendFactor::Zero,
                        //         operation: BlendOperation::Add,
                        //     },
                        // }),
                        // blend: Some(wgpu::BlendState {
                        //     color: BlendComponent {
                        //         src_factor: BlendFactor::One,
                        //         dst_factor: BlendFactor::One,
                        //         operation: BlendOperation::Max,
                        //     },
                        //     alpha: BlendComponent {
                        //         src_factor: BlendFactor::One,
                        //         dst_factor: BlendFactor::One,
                        //         operation: BlendOperation::Max,
                        //     },
                        // }),
                        blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                    // or Features::POLYGON_MODE_POINT
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: depth_stencil.clone(),
                multisample: wgpu::MultisampleState {
                    count: 4,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                // If the pipeline will be used with a multiview render pass, this
                // indicates how many array layers the attachments will have.
                multiview: None,
            });

        let dot_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Dot Render Pipeline"),
            layout: Some(&segment_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &segment_shader,
                entry_point: "vs_dot",
                buffers: offset_buffers_desc,
            },
            fragment: Some(wgpu::FragmentState {
                module: &segment_shader,
                entry_point: "fs_dot",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    // blend: Some(wgpu::BlendState {
                    //     color: BlendComponent {
                    //         src_factor: BlendFactor::One,
                    //         dst_factor: BlendFactor::One,
                    //         operation: BlendOperation::Max,
                    //     },
                    //     alpha: BlendComponent {
                    //         src_factor: BlendFactor::One,
                    //         dst_factor: BlendFactor::One,
                    //         operation: BlendOperation::Max,
                    //     },
                    // }),
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil,
            multisample: wgpu::MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });

        let offset_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Offset Vertex Buffer"),
            contents: bytemuck::cast_slice(OFFSET_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let offset_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Offset Index Buffer"),
            contents: bytemuck::cast_slice(OFFSET_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let point_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Point Buffer"),
            contents: bytemuck::cast_slice(&POINTS),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });

        let point_buffer2 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Point Buffer"),
            contents: bytemuck::cast_slice(&POINTS2),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("uniforms"),
            size: 80,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let uniform_buffer2 = device.create_buffer(&BufferDescriptor {
            label: Some("uniforms"),
            size: 80,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let line_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("segment bind group"),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(BufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(BufferBinding {
                        buffer: &point_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
            layout: &line_bind_group_layout,
        });

        let line_bind_group2 = device.create_bind_group(&BindGroupDescriptor {
            label: Some("segment bind group"),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(BufferBinding {
                        buffer: &uniform_buffer2,
                        offset: 0,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(BufferBinding {
                        buffer: &point_buffer2,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
            layout: &line_bind_group_layout,
        });

        let initial_scale = {
            let mut initial_scale =
                POINTS
                    .iter()
                    .chain(POINTS2.iter())
                    .fold(Scale::NAN, |scale, point| Scale {
                        horizontal: AxisScale {
                            start: f32_min(scale.horizontal.start, point.0),
                            end: f32_max(scale.horizontal.end, point.0),
                        },
                        vertical: AxisScale {
                            start: f32_min(scale.vertical.start, point.1),
                            end: f32_max(scale.vertical.end, point.1),
                        },
                    });

            for axis in [&mut initial_scale.horizontal, &mut initial_scale.vertical] {
                if axis.start.is_finite() {
                    if axis.start == axis.end {
                        *axis = AxisScale {
                            start: axis.start - 1.0,
                            end: axis.start + 1.0,
                        }
                    }
                } else {
                    *axis = AxisScale {
                        start: -1.0,
                        end: 1.0,
                    }
                }
            }

            initial_scale
        };
        let current_scale = RefCell::new(initial_scale);

        let mut canvas_outer_size = Cell::new(ComponentSize::default());

        let mut multisample_texture =
            RefCell::new(Self::create_multisample_texture(&device, &config));

        let mut depth_texture = RefCell::new(Self::create_depth_texture(&device, &config));

        let device = Rc::new(device);

        let plot = Plot {
            device: Rc::clone(&device),
            surface,
            config: RefCell::new(config),
            queue,
            current_scale,
            canvas_outer_size,
            my_str,
            offset_vertex_buffer,
            offset_index_buffer,
            uniform_buffer,
            uniform_buffer2,
            line_bind_group,
            line_bind_group2,
            multisample_texture,
            depth_texture,
            segment_render_pipeline,
            dot_render_pipeline,
            animation_frame_requested_but_render_not_queued: Cell::new(false),
            data_cache: DataCache::new(device, initial_scale),
        };

        plot.queue_render().unwrap();

        plot.my_str.modify(|_| "success!".to_owned());

        return Rc::new(plot);
    }

    fn create_multisample_texture(device: &Device, config: &SurfaceConfiguration) -> Texture {
        device.create_texture(&TextureDescriptor {
            label: Some("msaa"),
            size: Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: TextureDimension::D2,
            format: config.format,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
    }

    fn create_depth_texture(device: &Device, config: &SurfaceConfiguration) -> Texture {
        device.create_texture(&TextureDescriptor {
            label: Some("depth"),
            size: Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth24Plus,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
    }

    pub async fn run_event_loop(self: Rc<Self>, mut rx: UnboundedReceiver<CanvasEvent>) {
        join!(
            Plot::run_render_event_loop(Rc::clone(&self), rx),
            self.data_cache.run_event_loop()
        );
    }

    async fn run_render_event_loop(self: Rc<Self>, mut rx: UnboundedReceiver<CanvasEvent>) {
        let window = web_sys::window().unwrap();

        let queue_render_js_closure = {
            let plot = self.clone();
            Closure::<dyn Fn() -> ()>::new(move || plot.queue_render().unwrap()).into_js_value()
        };

        'wait_for_events: loop {
            let mut event = rx.next().await.expect("always some");
            let mut count = 1;
            'process_events: loop {
                match event {
                    CanvasEvent::Resize(size) => {
                        self.canvas_outer_size.set(size);
                        let ratio = web_sys::window().unwrap().device_pixel_ratio();
                        // Consider using devicePixelContentBoxSize conditionally or once Safari supports it.
                        // https://webgpufundamentals.org/webgpu/lessons/webgpu-resizing-the-canvas.html
                        // let ratio = 0.5;
                        self.config.borrow_mut().width = ((size.width as f64) * ratio) as u32;
                        self.config.borrow_mut().height = ((size.height as f64) * ratio) as u32;

                        self.surface.configure(&self.device, &self.config.borrow());

                        let mut multisample_texture = self.multisample_texture.borrow_mut();
                        multisample_texture.destroy();
                        *multisample_texture =
                            Self::create_multisample_texture(&self.device, &self.config.borrow());

                        let mut depth_texture = self.depth_texture.borrow_mut();
                        depth_texture.destroy();
                        *depth_texture =
                            Self::create_depth_texture(&self.device, &self.config.borrow());
                    }
                    CanvasEvent::Wheel(delta) => {
                        // info!("{:?}", delta);
                        let mut current_scale = self.current_scale.borrow_mut();
                        current_scale
                            .horizontal
                            .scale_pct(f32::exp(0.01 * delta.y as f32));
                        current_scale.horizontal.shift_pct(
                            (delta.x as f32) / (self.canvas_outer_size.get().width as f32),
                        );
                    }
                }

                match rx.try_next() {
                    Ok(Some(new_event)) => {
                        event = new_event;
                        count += 1;
                        continue 'process_events;
                    }
                    // Error indicates no event ready for processing
                    Err(_) => {
                        if !self.animation_frame_requested_but_render_not_queued.get() {
                            window
                                .request_animation_frame(queue_render_js_closure.unchecked_ref())
                                .unwrap();
                            self.animation_frame_requested_but_render_not_queued
                                .set(true);
                        }
                        // self.queue_render().unwrap();

                        // log::info!("rendered changes for {count} events!");
                        continue 'wait_for_events;
                    }
                    _ => todo!("component unloaded"),
                }
            }
        }
    }

    fn queue_render(&self) -> Result<(), wgpu::SurfaceError> {
        self.animation_frame_requested_but_render_not_queued
            .set(false);

        let output = self.surface.get_current_texture()?;

        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&self.current_scale.borrow().point_scale_to_uniform(
                self.canvas_outer_size.get().width as u32,
                self.canvas_outer_size.get().height as u32,
                100,
                &c1,
                0.5,
                NonZeroU16::new(1).unwrap(),
            )),
        );

        self.queue.write_buffer(
            &self.uniform_buffer2,
            0,
            bytemuck::cast_slice(&self.current_scale.borrow().point_scale_to_uniform(
                self.canvas_outer_size.get().width as u32,
                self.canvas_outer_size.get().height as u32,
                100,
                &c2,
                0.5,
                NonZeroU16::new(2).unwrap(),
            )),
        );

        // info!("{:?}", POINTS.as_ref());
        // info!("Matrices");
        // info!("{current_scale:?}");
        self.my_str.modify(|_| {
            format!(
                "Transforms for {} x {}",
                self.canvas_outer_size.get().width as u32,
                self.canvas_outer_size.get().height as u32
            )
        });
        // for mat in current_scale.point_scale_to_uniform(
        //     canvas_outer_size.get().width as u32,
        //     canvas_outer_size.get().height as u32,
        //     0,
        // ) {
        //     // info!(
        //     //     "{{{{{}, {}, {}}},{{{}, {}, {}}}}}",
        //     //     mat[0], mat[2], mat[4], mat[1], mat[3], mat[5]
        //     // );
        // }

        let multisample_view = self
            .multisample_texture
            .borrow()
            .create_view(&wgpu::TextureViewDescriptor::default());
        let depth_view = self
            .depth_texture
            .borrow()
            .create_view(&wgpu::TextureViewDescriptor::default());
        let final_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &multisample_view,
                    resolve_target: Some(&final_view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    depth_ops: Some(Operations::default()),
                    stencil_ops: None,
                    view: &depth_view, // TODO
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // for bind_group in [&line_bind_group, &line_bind_group2] {
            for bind_group in [&self.line_bind_group] {
                render_pass.set_bind_group(0, bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.offset_vertex_buffer.slice(..));
                render_pass.set_index_buffer(
                    self.offset_index_buffer.slice(..),
                    wgpu::IndexFormat::Uint16,
                );
                render_pass.set_pipeline(&self.segment_render_pipeline);
                render_pass.draw_indexed(0..6, 0, 0..(u32::try_from(POINTS.len()).unwrap() - 1));
                render_pass.set_pipeline(&self.dot_render_pipeline);
                render_pass.draw_indexed(0..6, 0, 0..u32::try_from(POINTS.len()).unwrap());
            }
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        info!("done");

        Ok(())
    }
}

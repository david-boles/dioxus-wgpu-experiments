#![allow(non_snake_case)]
use console_log::log;
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use dioxus::prelude::*;
// use dioxus_helmet::Helmet;
use futures_util::stream::StreamExt;
use js_sys::is_finite;
use js_sys::Math::random;
use lazy_static::lazy_static;
use log::info;
use rand::{distributions::Uniform, Rng};
use std::cell::Cell;
use std::cmp::{max, min};
use std::num::NonZeroU64;
use std::{
    iter,
    ops::{RangeInclusive, RangeToInclusive},
    sync::mpsc::TryRecvError,
};
use web_sys::HtmlCanvasElement;
use wgpu::{
    util::{DeviceExt, RenderEncoder},
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BufferBinding, ShaderStages,
};
use wgpu::{
    BlendComponent, BlendFactor, BlendOperation, BufferDescriptor, BufferUsages,
    CompositeAlphaMode, Extent3d, SurfaceConfiguration, Texture, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages,
};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    raw_window_handle::{WebCanvasWindowHandle, WebWindowHandle},
    window::WindowBuilder,
};

use wasm_bindgen::prelude::*;

use resize::*;

mod resize;

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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // E
];

const OFFSET_VERTICES: &[[f32; 2]] = &[[-1.0, -1.0], [-1.0, 1.0], [1.0, -1.0], [1.0, 1.0]];

const OFFSET_INDICES: &[u16] = &[
    0, 1, 2, // Bottom left
    1, 2, 3, // Top right
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4, /* padding */ 0];

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
const NUM_POINTS: usize = 20;

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
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

const offset_buffers_desc: &[wgpu::VertexBufferLayout] = &[wgpu::VertexBufferLayout {
    array_stride: wgpu::VertexFormat::Float32x2.size() as wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[wgpu::VertexAttribute {
        offset: 0,
        shader_location: 0,
        format: wgpu::VertexFormat::Float32x2,
    }],
}];

// fn segmented_line_desc() -> wgpu::layout {
//     wgpu::VertexBufferLayout {
//         array_stride: std::mem::size_of::<Point>() as wgpu::BufferAddress,
//         step_mode: wgpu::VertexStepMode::,
//         attributes: &[
//             wgpu::VertexAttribute {
//                 offset: 0,
//                 shader_location: 0,
//                 format: wgpu::VertexFormat::Float32x3,
//             },
//             wgpu::VertexAttribute {
//                 offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
//                 shader_location: 1,
//                 format: wgpu::VertexFormat::Float32x3,
//             },
//         ],
//     }
// }

enum CanvasEvent {
    Resize(ComponentSize),
}

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
    // launch the web app
    dioxus_web::launch(App);
}

// create a component that renders a div with the text "Hello, world!"
fn App(cx: Scope) -> Element {
    let my_str = use_state(cx, || "Hello world!".to_owned());

    let render_coroutine = use_coroutine(cx, |mut rx: UnboundedReceiver<CanvasEvent>| {
        to_owned![my_str];
        async move {
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
                instance.create_surface_from_canvas(
                    HtmlCanvasElement::try_from(canvas.clone()).unwrap(),
                )
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

            let line_bind_group_layout =
                device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("segment bind group layout"),
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0, // TODO 0
                            visibility: ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: Some(NonZeroU64::new(12 * 4).unwrap()),
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

            let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                    // or Features::POLYGON_MODE_POINT
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 4,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                // If the pipeline will be used with a multiview render pass, this
                // indicates how many array layers the attachments will have.
                multiview: None,
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
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 4,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    // If the pipeline will be used with a multiview render pass, this
                    // indicates how many array layers the attachments will have.
                    multiview: None,
                });

            let dot_render_pipeline =
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 4,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    // If the pipeline will be used with a multiview render pass, this
                    // indicates how many array layers the attachments will have.
                    multiview: None,
                });

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            });
            let num_indices = INDICES.len() as u32;

            let offset_vertex_buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Offset Vertex Buffer"),
                    contents: bytemuck::cast_slice(OFFSET_VERTICES),
                    usage: wgpu::BufferUsages::VERTEX,
                });

            let offset_index_buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Offset Index Buffer"),
                    contents: bytemuck::cast_slice(OFFSET_INDICES),
                    usage: wgpu::BufferUsages::INDEX,
                });

            let point_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Point Buffer"),
                contents: bytemuck::cast_slice(&POINTS),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            });

            let uniform_buffer = device.create_buffer(&BufferDescriptor {
                label: Some("uniforms"),
                size: 12 * 4,
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

            #[derive(Clone, Copy, Debug)]
            struct AxisScale {
                /// First point's position in some axis in some space
                start: f32,
                /// Last point's position in some axis in some space
                end: f32,
            }

            impl AxisScale {
                const NAN: Self = Self {
                    start: f32::NAN,
                    end: f32::NAN,
                };
            }

            /// The scale of a set of points in some space
            #[derive(Clone, Copy, Debug)]
            struct Scale {
                horizontal: AxisScale,
                vertical: AxisScale,
            }

            impl Scale {
                const NAN: Self = Self {
                    horizontal: AxisScale::NAN,
                    vertical: AxisScale::NAN,
                };

                const RASTER: Self = Self {
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
                    info!("transform");
                    info!("{from:?}");
                    info!("{to:?}");
                    let h_scale = (to.horizontal.end - to.horizontal.start)
                        / (from.horizontal.end - from.horizontal.start);
                    let v_scale = (to.vertical.end - to.vertical.start)
                        / (from.vertical.end - from.vertical.start);
                    info!("{h_scale}");
                    info!("{v_scale}");
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
                ) -> [[f32; 6]; 2] {
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

                    // {{605.5, 0, 605.5},{0, -445.5, 445.5}}
                    // {{0.0016515277, 0, -1},{0, -0.002244669, 1}}

                    // Matrix computed for -1,-1 to 1, 1
                    // return [
                    //     [605.5, 0.0, 0.0, -445.5, 605.5, 445.5],
                    //     [0.0016515277, 0.0, 0.0, -0.002244669, -1.0, 1.0],
                    // ];

                    // Random matrix and inverse
                    // let det = 915.0;
                    // return [
                    //     [24.0, 51.0, -17.0, 2.0, -6.0, 12.0],
                    //     [
                    //         2.0 / det,
                    //         -51.0 / det,
                    //         17.0 / det,
                    //         24.0 / det,
                    //         -192.0 / det,
                    //         -594.0 / det,
                    //     ],
                    // ];

                    // return [
                    //     [1.0, 0.0, 0.0, 1.0, -1.0, 0.0],
                    //     [1.0, 0.0, 0.0, 1.0, 1.0, 0.0],
                    // ];

                    return [
                        self.transform_matrix_to(px_scale_with_margin),
                        px_scale.transform_matrix_to(Scale::RASTER),
                    ];
                }
            }

            let mut current_scale = {
                let mut initial_scale = POINTS.iter().fold(Scale::NAN, |scale, point| Scale {
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

            let mut canvas_outer_size = Cell::new(ComponentSize::default());

            let mut create_multisample_texture = |config: &SurfaceConfiguration| -> Texture {
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
            };
            let mut multisample_texture = RefCell::new(create_multisample_texture(&config));

            let mut render = || -> Result<(), wgpu::SurfaceError> {
                let output = surface.get_current_texture()?;

                queue.write_buffer(
                    &uniform_buffer,
                    0,
                    bytemuck::cast_slice(&current_scale.point_scale_to_uniform(
                        canvas_outer_size.get().width as u32,
                        canvas_outer_size.get().height as u32,
                        100,
                    )),
                );

                // info!("{:?}", POINTS.as_ref());
                info!("Matrices");
                info!("{current_scale:?}");
                my_str.modify(|_| {
                    format!(
                        "Transforms for {} x {}",
                        canvas_outer_size.get().width as u32,
                        canvas_outer_size.get().height as u32
                    )
                });
                for mat in current_scale.point_scale_to_uniform(
                    canvas_outer_size.get().width as u32,
                    canvas_outer_size.get().height as u32,
                    0,
                ) {
                    info!(
                        "{{{{{}, {}, {}}},{{{}, {}, {}}}}}",
                        mat[0], mat[2], mat[4], mat[1], mat[3], mat[5]
                    );
                }

                let multisample_view = multisample_texture
                    .borrow()
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let final_view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });

                    render_pass.set_pipeline(&render_pipeline);
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..num_indices, 0, 0..1);

                    render_pass.set_bind_group(0, &line_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, offset_vertex_buffer.slice(..));
                    render_pass
                        .set_index_buffer(offset_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.set_pipeline(&segment_render_pipeline);
                    render_pass.draw_indexed(
                        0..6,
                        0,
                        0..(u32::try_from(POINTS.len()).unwrap() - 1),
                    );
                    render_pass.set_pipeline(&dot_render_pipeline);
                    render_pass.draw_indexed(0..6, 0, 0..u32::try_from(POINTS.len()).unwrap());
                }

                queue.submit(iter::once(encoder.finish()));
                output.present();

                Ok(())
            };

            render().unwrap();

            my_str.modify(|_| "success!".to_owned());

            'wait_for_events: loop {
                let mut event = rx.next().await.expect("always some");
                let mut count = 1;
                'process_events: loop {
                    match event {
                        CanvasEvent::Resize(size) => {
                            canvas_outer_size.set(size);
                            let ratio = web_sys::window().unwrap().device_pixel_ratio();
                            // Consider using devicePixelContentBoxSize conditionally or once Safari supports it.
                            // https://webgpufundamentals.org/webgpu/lessons/webgpu-resizing-the-canvas.html
                            // let ratio = 0.1;
                            config.width = ((size.width as f64) * ratio) as u32;
                            config.height = ((size.height as f64) * ratio) as u32;

                            surface.configure(&device, &config);
                            let mut multisample_texture = multisample_texture.borrow_mut();
                            multisample_texture.destroy();
                            *multisample_texture = create_multisample_texture(&config);
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
                            render().unwrap();

                            log::info!("rendered changes for {count} events!");
                            continue 'wait_for_events;
                        }
                        _ => todo!("component unloaded"),
                    }
                }
            }
        }
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
                    }
                }
            }
        }
    })
}

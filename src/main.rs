#![allow(non_snake_case)]
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use dioxus::prelude::*;
// use dioxus_helmet::Helmet;
use futures_util::stream::StreamExt;
use js_sys::Math::random;
use lazy_static::lazy_static;
use rand::{distributions::Uniform, Rng};
use std::{iter, sync::mpsc::TryRecvError};
use web_sys::HtmlCanvasElement;
use wgpu::{
    util::{DeviceExt, RenderEncoder},
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BufferBinding, ShaderStages,
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

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4, /* padding */ 0];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Point(f32, f32);

// Degenerate - reverse
// const POINTS: &[Point] = &[
//     Point(0.49999999999999, -0.7),
//     Point(0.4, 0.7),
//     Point(0.5, -0.7),
// ];
// Degenerate - continue
// const POINTS: &[Point] = &[Point(0.0, -0.5), Point(0.0, 0.0), Point(0.0, 0.5)];
// const NUM_POINTS: usize = 134217728 / (8);
const NUM_POINTS: usize = 10;

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
                present_mode: surface_caps.present_modes[0],
                alpha_mode: surface_caps.alpha_modes[0],
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

            let points_bind_group_layout =
                device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("segment bind group layout"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0, // TODO 0
                        visibility: ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

            let segment_render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Segment Render Pipeline Layout"),
                    bind_group_layouts: &[&points_bind_group_layout],
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
                    count: 1,
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
                        buffers: &[],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &segment_shader,
                        entry_point: "fs_line",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: config.format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleStrip,
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
                        count: 1,
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
                        buffers: &[],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &segment_shader,
                        entry_point: "fs_dot",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: config.format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
                        count: 1,
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

            let point_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Point Buffer"),
                contents: bytemuck::cast_slice(&POINTS),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            });

            let points_bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: Some("segment bind group"),
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(BufferBinding {
                        buffer: &point_buffer,
                        offset: 0,
                        size: None,
                    }),
                }],
                layout: &points_bind_group_layout,
            });

            let render = || -> Result<(), wgpu::SurfaceError> {
                let output = surface.get_current_texture()?;
                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
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

                    render_pass.set_pipeline(&segment_render_pipeline);
                    render_pass.set_bind_group(0, &points_bind_group, &[]);
                    render_pass.draw(0..u32::try_from(2 * POINTS.len()).unwrap(), 0..1);

                    render_pass.set_pipeline(&dot_render_pipeline);
                    render_pass.set_bind_group(0, &points_bind_group, &[]); // TODO
                    render_pass.draw(0..u32::try_from(6 * POINTS.len()).unwrap(), 0..1);
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
                        CanvasEvent::Resize(ComponentSize { width, height }) => {
                            let ratio = web_sys::window().unwrap().device_pixel_ratio();
                            // Consider using devicePixelContentBoxSize conditionally or once Safari supports it.
                            // https://webgpufundamentals.org/webgpu/lessons/webgpu-resizing-the-canvas.html
                            // let ratio = 0.1;
                            config.width = ((width as f64) * ratio) as u32;
                            config.height = ((height as f64) * ratio) as u32;

                            surface.configure(&device, &config);
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
                    onmounted: |event| {
                        on_resize.mount(event);
                    }
                }
            }
        }
    })
}

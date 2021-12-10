use pollster::block_on;
use std::num::NonZeroU32;

const WIDTH: u32 = 1792;
const HEIGHT: u32 = 1024;
const PIXEL_COUNT: u32 = WIDTH * HEIGHT;
const OUTPUT_BUFFER_SIZE: u64 = std::mem::size_of::<u32>() as u64 * PIXEL_COUNT as u64;
const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

const VERTEX_SHADER: &str = r#"

[[stage(vertex)]]
fn vertex_main() {}

[[stage(fragment)]]
fn fragment_main() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
"#;

fn main() {
    env_logger::init();
    let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .expect("No adapter");
    let (device, queue) = block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("tom's render device"),
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
        },
        None,
    ))
    .unwrap();

    let texture = texture(&device);
    let view = texture.create_view(&Default::default());
    let output_buffer = output_buffer(&device);

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("tom's pipeline's layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("tom's vertex shader"),
        source: wgpu::ShaderSource::Wgsl(VERTEX_SHADER.into()),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("tom's pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: "vertex_main",
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            clamp_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: "fragment_main",
            targets: &[wgpu::ColorTargetState {
                format: TEXTURE_FORMAT,
                write_mask: wgpu::ColorWrites::ALL,
                blend: Some(wgpu::BlendState::REPLACE),
            }],
        }),
    });

    let mut webp_encoder = webp_animation::Encoder::new_with_options(
        (WIDTH, HEIGHT),
        webp_animation::EncoderOptions {
            minimize_size: true,
            kmin: 0,
            kmax: 0,
            allow_mixed: false,
            verbose: false,
            color_mode: webp_animation::ColorMode::Rgba,
            encoding_config: None,
        },
    )
    .unwrap();
    let mut timestamp_ms = 0;

    for n in 0..200 {
        timestamp_ms += 100;
        let red_value = (n as f64 / 40.0).sin().abs() * 0.8;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("tom's command encoder"),
        });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("tom's render pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: red_value,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            pass.set_pipeline(&pipeline);
        }

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(
                        NonZeroU32::new(WIDTH * std::mem::size_of::<u32>() as u32).unwrap(),
                    ),
                    rows_per_image: Some(NonZeroU32::new(HEIGHT).unwrap()),
                },
            },
            wgpu::Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
        );

        queue.submit([encoder.finish()]);

        {
            let buffer_slice = output_buffer.slice(..);

            let mapping = buffer_slice.map_async(wgpu::MapMode::Read);
            device.poll(wgpu::Maintain::Wait);
            block_on(mapping).unwrap();

            let data = buffer_slice.get_mapped_range();

            webp_encoder.add_frame(&data, timestamp_ms).unwrap();

            // use image::ImageBuffer;
            // let buffer = ImageBuffer::<image::Rgba<u8>, _>::from_raw(WIDTH, HEIGHT, data).unwrap();
            // buffer.save("image.png").unwrap();
        }

        output_buffer.unmap();
    }

    let webp = webp_encoder.finalize(201 * 100).unwrap();

    std::fs::write("./out.webp", &webp).unwrap();

    println!("Done");
}

fn texture(device: &wgpu::Device) -> wgpu::Texture {
    let texture_desc = wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: TEXTURE_FORMAT,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: None,
    };
    device.create_texture(&texture_desc)
}

fn output_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    let output_buffer_desc = wgpu::BufferDescriptor {
        size: OUTPUT_BUFFER_SIZE,
        usage: wgpu::BufferUsages::COPY_DST
        // this tells wpgu that we want to read this buffer from the cpu
        | wgpu::BufferUsages::MAP_READ,
        label: None,
        mapped_at_creation: false,
    };
    device.create_buffer(&output_buffer_desc)
}

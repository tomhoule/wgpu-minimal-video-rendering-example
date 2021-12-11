use log::*;
use pollster::block_on;
use std::{f64::consts::TAU, fmt, num::NonZeroU32, sync::Arc};

const WIDTH: u32 = 1792;
const HEIGHT: u32 = 1024;
const PIXEL_COUNT: u32 = WIDTH * HEIGHT;
const FRAME_SIZE: u64 = std::mem::size_of::<u32>() as u64 * PIXEL_COUNT as u64;
const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;
const FRAMES_PER_SECOND: u64 = 60;
const TOTAL_SECONDS: u64 = 12;
const TOTAL_FRAMES: u64 = FRAMES_PER_SECOND * TOTAL_SECONDS;

const SHADERS: &str = r#"

[[stage(vertex)]]
fn vertex_main() {}

[[stage(fragment)]]
fn fragment_main() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
"#;

fn main() -> Result<(), Error> {
    dcv_color_primitives::initialize();
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
    let device = Arc::new(device);

    // // Constantly poll the device.
    // std::thread::spawn(move || loop {
    //     device.poll(wgpu::Maintain::Wait);
    // });

    // video
    use dcv_color_primitives::{ColorSpace, ImageFormat, PixelFormat};
    let mut video_file = std::io::BufWriter::new(std::fs::File::create("out.y4m").unwrap());
    let mut video_encoder = y4m::encode(
        WIDTH as usize,
        HEIGHT as usize,
        y4m::Ratio::new(FRAMES_PER_SECOND as usize, 1),
    )
    .with_colorspace(y4m::Colorspace::C444)
    .write_header(&mut video_file)
    .unwrap();
    let mut buf = Vec::new();
    let mut buf2 = Vec::new();
    let mut buf3 = Vec::new();
    let source_format = ImageFormat {
        pixel_format: PixelFormat::Bgra,
        color_space: ColorSpace::Lrgb,
        num_planes: 1,
    };
    let mut sizes = [0, 0, 0];

    dbg!(sizes);

    let target_format = ImageFormat {
        pixel_format: PixelFormat::I444,
        color_space: ColorSpace::Bt601,
        num_planes: 3,
    };

    dcv_color_primitives::get_buffers_size(WIDTH, HEIGHT, &target_format, None, &mut sizes)
        .unwrap();

    for (b, size) in [&mut buf, &mut buf2, &mut buf3]
        .into_iter()
        .zip(sizes.iter())
    {
        b.resize(*size, 0);
    }

    // std::thread::spawn(move || {
    //     output_buffer.destroy();
    // });

    let texture = texture(&device);
    let view = texture.create_view(&Default::default());
    let output_buffer = new_output_buffer(&device);

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("tom's pipeline's layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some("tom's vertex shader"),
        source: wgpu::ShaderSource::Wgsl(SHADERS.into()),
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

    info!("Rendering...");
    for frame_idx in 0..TOTAL_FRAMES {
        debug!("Rendering frame {}/{}", frame_idx + 1, TOTAL_FRAMES);
        let green_value = ((frame_idx as f64 / TOTAL_FRAMES as f64) * TAU * 5.0)
            .sin()
            .abs();
        assert!(green_value >= 0.0);
        assert!(green_value <= 1.0);
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: green_value,
                            b: 0.1,
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
        device.poll(wgpu::Maintain::Poll);

        debug!("Writing frames now");
        {
            let buffer_slice = output_buffer.slice(..);

            let mapping = buffer_slice.map_async(wgpu::MapMode::Read);
            device.poll(wgpu::Maintain::Wait);
            block_on(mapping).unwrap();

            let data = buffer_slice.get_mapped_range();

            debug!("Got mapped range...");

            dcv_color_primitives::convert_image(
                WIDTH,
                HEIGHT,
                &source_format,
                None,
                &[&data[0..FRAME_SIZE as usize]],
                &target_format,
                None,
                &mut [
                    &mut buf[0..sizes[0]],
                    &mut buf2[0..sizes[1]],
                    &mut buf3[0..sizes[2]],
                ],
            )
            .unwrap();

            let frame = y4m::Frame::new(
                [&buf[0..sizes[0]], &buf2[0..sizes[1]], &buf3[0..sizes[2]]],
                None,
            );

            video_encoder.write_frame(&frame).unwrap();
        }
        output_buffer.unmap();
    }

    // sender.send(output_buffer)?;

    info!("Rendering: done");

    drop(video_encoder);
    let video_file = video_file.into_inner().unwrap();
    video_file.sync_all().unwrap();
    drop(video_file);

    info!("Cleanup: done");

    // Don't run on destructors: rely on the process dying to clean up, dropping everything cleanly
    // is slow.
    std::process::exit(0);
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

fn new_output_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    let output_buffer_desc = wgpu::BufferDescriptor {
        size: FRAME_SIZE,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        label: None,
        mapped_at_creation: false,
    };
    device.create_buffer(&output_buffer_desc)
}

struct Error(
    Box<dyn std::error::Error>,
    &'static std::panic::Location<'static>,
);

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BOOM at \n")?;
        fmt::Display::fmt(&self.1, f)?;
        f.write_str("\n")?;
        fmt::Debug::fmt(&self.0, f)?;
        f.write_str("\n---\n")?;
        fmt::Display::fmt(&self.0, f)
    }
}

impl<E: std::error::Error + 'static> From<E> for Error {
    #[track_caller]
    fn from(e: E) -> Error {
        Error(Box::new(e), &std::panic::Location::caller())
    }
}

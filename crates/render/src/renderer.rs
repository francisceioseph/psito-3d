//! The progressive scene renderer.

use psito3d_scene::{MAX_OBJECTS, Scene};
use psito3d_terrain::Heightfield;
use wgpu::util::DeviceExt;

use crate::gpu::{self, GpuObject, ResolveParams};
use crate::{Quality, RESOLVE_WGSL, TRACE_WGSL};

/// The format of the image the renderer produces: 8-bit sRGB, which egui
/// shows directly and the exporter writes as PNG.
pub const OUTPUT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
const ACCUM_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;

/// What can go wrong reading the image back.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// The GPU could not map the readback buffer.
    #[error("could not read the image from the GPU: {0}")]
    Readback(String),
}

/// Two accumulation textures used in turn: one is read (the samples so far)
/// while the other is written (with the new sample added).
struct Accumulation {
    // Kept so the textures live as long as their views, and are easy to inspect.
    _textures: [wgpu::Texture; 2],
    views: [wgpu::TextureView; 2],
    size: (u32, u32),
}

/// Draws a [`Scene`] progressively into an sRGB texture.
///
/// Call [`Self::set_scene`] whenever the scene may have changed (it notices
/// when nothing did), [`Self::set_heightfield`] when the terrain was
/// rebuilt, and [`Self::render`] once per frame. The renderer stops adding
/// samples when it reaches [`Self::target_samples`], so a still image costs
/// nothing.
pub struct SceneRenderer {
    trace: wgpu::RenderPipeline,
    resolve: wgpu::RenderPipeline,
    trace_layout: wgpu::BindGroupLayout,
    resolve_layout: wgpu::BindGroupLayout,
    globals: wgpu::Buffer,
    resolve_params: wgpu::Buffer,
    objects: wgpu::Buffer,
    heightmap: wgpu::Texture,
    heightmap_view: wgpu::TextureView,
    heightmap_size: u32,
    accum: Accumulation,
    output: wgpu::Texture,
    output_view: wgpu::TextureView,
    output_size: (u32, u32),
    /// The packed scene of the samples so far, without the per-sample fields.
    current: Option<(gpu::Globals, Vec<GpuObject>, u32)>,
    quality: Quality,
    exposure: f32,
    filter: u32,
    samples: u32,
    target_samples: u32,
    /// Bumped whenever the output texture is recreated, so a viewer knows
    /// to register the new one.
    generation: u64,
}

fn texture(
    device: &wgpu::Device,
    label: &str,
    size: (u32, u32),
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d { width: size.0.max(1), height: size.1.max(1), depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

fn texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn buffer_entry(binding: u32, ty: wgpu::BufferBindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer { ty, has_dynamic_offset: false, min_binding_size: None },
        count: None,
    }
}

fn pipeline(
    device: &wgpu::Device,
    label: &str,
    source: &str,
    layout: &wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[Some(layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

impl Accumulation {
    fn new(device: &wgpu::Device, size: (u32, u32)) -> Self {
        let usage = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        let textures = [
            texture(device, "accumulation a", size, ACCUM_FORMAT, usage),
            texture(device, "accumulation b", size, ACCUM_FORMAT, usage),
        ];
        let views = [textures[0].create_view(&Default::default()), textures[1].create_view(&Default::default())];
        Self { _textures: textures, views, size }
    }
}

impl SceneRenderer {
    /// The samples a still image gets by default.
    pub const DEFAULT_TARGET_SAMPLES: u32 = 256;

    /// A renderer drawing at `size` pixels, with a flat terrain until
    /// [`Self::set_heightfield`] is called.
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, size: (u32, u32)) -> Self {
        let trace_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("trace"),
            entries: &[
                buffer_entry(0, wgpu::BufferBindingType::Uniform),
                buffer_entry(1, wgpu::BufferBindingType::Storage { read_only: true }),
                texture_entry(2),
                texture_entry(3),
            ],
        });
        let resolve_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("resolve"),
            entries: &[buffer_entry(0, wgpu::BufferBindingType::Uniform), texture_entry(1)],
        });
        let trace = pipeline(device, "trace", TRACE_WGSL, &trace_layout, ACCUM_FORMAT);
        let resolve = pipeline(device, "resolve", RESOLVE_WGSL, &resolve_layout, OUTPUT_FORMAT);

        let globals = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("globals"),
            size: std::mem::size_of::<gpu::Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let resolve_params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("resolve params"),
            size: std::mem::size_of::<ResolveParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let objects = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("objects"),
            size: (std::mem::size_of::<GpuObject>() * MAX_OBJECTS) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let (heightmap, heightmap_view) = Self::upload_heightmap(device, queue, &Heightfield::flat(2, 0.0));
        let size = (size.0.max(1), size.1.max(1));
        let (output, output_view) = Self::make_output(device, size);
        Self {
            trace,
            resolve,
            trace_layout,
            resolve_layout,
            globals,
            resolve_params,
            objects,
            heightmap,
            heightmap_view,
            heightmap_size: 2,
            accum: Accumulation::new(device, Self::traced_size(size, Quality::Final)),
            output,
            output_view,
            output_size: size,
            current: None,
            quality: Quality::Final,
            exposure: 0.0,
            filter: 0,
            samples: 0,
            target_samples: Self::DEFAULT_TARGET_SAMPLES,
            generation: 0,
        }
    }

    fn traced_size(size: (u32, u32), quality: Quality) -> (u32, u32) {
        let s = quality.scale();
        (((size.0 as f32 * s) as u32).max(1), ((size.1 as f32 * s) as u32).max(1))
    }

    fn make_output(device: &wgpu::Device, size: (u32, u32)) -> (wgpu::Texture, wgpu::TextureView) {
        let usage = wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC;
        let output = texture(device, "output", size, OUTPUT_FORMAT, usage);
        let view = output.create_view(&Default::default());
        (output, view)
    }

    fn upload_heightmap(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        field: &Heightfield,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let size = field.size();
        let tex = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("heightmap"),
                size: wgpu::Extent3d { width: size, height: size, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            bytemuck::cast_slice(field.samples()),
        );
        let view = tex.create_view(&Default::default());
        (tex, view)
    }

    /// Replaces the terrain and starts the image over.
    pub fn set_heightfield(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, field: &Heightfield) {
        let (tex, view) = Self::upload_heightmap(device, queue, field);
        self.heightmap.destroy();
        self.heightmap = tex;
        self.heightmap_view = view;
        self.heightmap_size = field.size();
        self.restart();
    }

    /// Changes the output size; the image starts over.
    pub fn resize(&mut self, device: &wgpu::Device, size: (u32, u32)) {
        let size = (size.0.max(1), size.1.max(1));
        if size == self.output_size {
            return;
        }
        self.output.destroy();
        let (output, view) = Self::make_output(device, size);
        self.output = output;
        self.output_view = view;
        self.output_size = size;
        self.generation += 1;
        self.accum = Accumulation::new(device, Self::traced_size(size, self.quality));
        self.current = None;
        self.restart();
    }

    /// Switches between the fast preview and the clean final image.
    pub fn set_quality(&mut self, device: &wgpu::Device, quality: Quality) {
        if quality == self.quality {
            return;
        }
        self.quality = quality;
        self.accum = Accumulation::new(device, Self::traced_size(self.output_size, quality));
        self.current = None;
        self.restart();
    }

    /// Takes the scene to draw. If nothing that shows changed, the samples
    /// so far are kept.
    pub fn set_scene(&mut self, scene: &Scene) {
        let globals = gpu::pack_globals(scene, self.accum.size, self.heightmap_size, self.quality);
        let objects = gpu::pack_objects(scene);
        let exposure = 2f32.powf(scene.sky.exposure);
        let filter = scene.filter.gpu_index();
        let packed = (globals, objects, filter);
        // The exposure and the filter only touch the resolve pass, so a
        // change to them does not throw the samples away.
        self.exposure = exposure;
        self.filter = filter;
        let unchanged = self.current.as_ref().is_some_and(|(g, o, _)| g == &packed.0 && o == &packed.1);
        if !unchanged {
            self.current = Some(packed);
            self.restart();
        }
    }

    /// Throws the samples away; the next frame starts a new image.
    pub fn restart(&mut self) {
        self.samples = 0;
    }

    /// The samples averaged so far.
    pub fn samples(&self) -> u32 {
        self.samples
    }

    /// The samples after which the image is considered done.
    pub fn target_samples(&self) -> u32 {
        self.target_samples
    }

    /// Sets how many samples make a finished image.
    pub fn set_target_samples(&mut self, n: u32) {
        self.target_samples = n.max(1);
    }

    /// The current quality.
    pub fn quality(&self) -> Quality {
        self.quality
    }

    /// Whether the image needs more samples. A preview is done after one,
    /// because the next interaction replaces it anyway.
    pub fn needs_samples(&self) -> bool {
        self.current.is_some()
            && match self.quality {
                Quality::Preview => self.samples == 0,
                Quality::Final => self.samples < self.target_samples,
            }
    }

    /// The image, as a texture view of [`OUTPUT_FORMAT`].
    pub fn output_view(&self) -> &wgpu::TextureView {
        &self.output_view
    }

    /// The image size.
    pub fn output_size(&self) -> (u32, u32) {
        self.output_size
    }

    /// Changes whenever [`Self::output_view`] is a new texture.
    pub fn output_generation(&self) -> u64 {
        self.generation
    }

    /// Adds one sample, if the image needs it, and refreshes the output.
    /// Returns whether it drew anything. Also refreshes the output when only
    /// the exposure or the filter changed.
    pub fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
        let Some((globals, objects, _)) = &self.current else { return false };
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("psito3d") });
        let traced = self.needs_samples();
        if traced {
            let mut g = *globals;
            let (jx, jy) = gpu::jitter(self.samples);
            g.res[2] = jx;
            g.res[3] = jy;
            g.info[0] = self.samples;
            queue.write_buffer(&self.globals, 0, bytemuck::bytes_of(&g));
            if !objects.is_empty() {
                queue.write_buffer(&self.objects, 0, bytemuck::cast_slice(objects));
            }
            let read = (self.samples % 2) as usize;
            let write = 1 - read;
            let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("trace"),
                layout: &self.trace_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: self.globals.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: self.objects.as_entire_binding() },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&self.heightmap_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&self.accum.views[read]),
                    },
                ],
            });
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("trace"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.accum.views[write],
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.trace);
            pass.set_bind_group(0, &bind, &[]);
            pass.draw(0..3, 0..1);
            drop(pass);
            self.samples += 1;
        }

        // The texture written last holds the average.
        // Sample k reads texture k % 2 and writes the other one.
        let latest = if self.samples == 0 { 0 } else { 1 - ((self.samples - 1) % 2) as usize };
        let params = ResolveParams {
            params: [self.exposure, 0.0, 0.0, 0.0],
            sizes: [
                self.accum.size.0 as f32,
                self.accum.size.1 as f32,
                self.output_size.0 as f32,
                self.output_size.1 as f32,
            ],
            filter: [self.filter, 0, 0, 0],
        };
        queue.write_buffer(&self.resolve_params, 0, bytemuck::bytes_of(&params));
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("resolve"),
            layout: &self.resolve_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.resolve_params.as_entire_binding() },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.accum.views[latest]),
                },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("resolve"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.output_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.resolve);
        pass.set_bind_group(0, &bind, &[]);
        pass.draw(0..3, 0..1);
        drop(pass);
        queue.submit([encoder.finish()]);
        traced
    }

    /// Reads the output back as tightly packed RGBA8 (sRGB) rows. Blocks
    /// until the GPU is done.
    pub fn read_output(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<Vec<u8>, RenderError> {
        let (w, h) = self.output_size;
        let unpadded = w * 4;
        let padded = unpadded.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: u64::from(padded * h),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.output,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded), rows_per_image: Some(h) },
            },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
        queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer.map_async(wgpu::MapMode::Read, .., move |r| {
            let _ = tx.send(r);
        });
        device.poll(wgpu::PollType::wait_indefinitely()).map_err(|e| RenderError::Readback(e.to_string()))?;
        rx.recv()
            .map_err(|e| RenderError::Readback(e.to_string()))?
            .map_err(|e| RenderError::Readback(e.to_string()))?;
        let data = buffer.get_mapped_range(..).map_err(|e| RenderError::Readback(e.to_string()))?;
        let mut pixels = Vec::with_capacity((unpadded * h) as usize);
        for row in data.chunks(padded as usize) {
            pixels.extend_from_slice(&row[..unpadded as usize]);
        }
        drop(data);
        buffer.unmap();
        Ok(pixels)
    }

    /// Renders until the image has `samples` samples, then reads it back.
    /// For exporting and for tests.
    pub fn render_to_pixels(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        samples: u32,
    ) -> Result<Vec<u8>, RenderError> {
        self.set_target_samples(samples);
        while self.needs_samples() {
            self.render(device, queue);
            // Keep the queue from growing without bound on slow GPUs.
            if self.samples.is_multiple_of(8) {
                device.poll(wgpu::PollType::wait_indefinitely()).map_err(|e| RenderError::Readback(e.to_string()))?;
            }
        }
        self.render(device, queue);
        self.read_output(device, queue)
    }
}

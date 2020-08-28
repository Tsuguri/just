use just_core::graphics::{
    self,
    hal,
    hal::device::Device,
    mesh::{Mesh, MeshBuilder, Normal, PosNormTex, Position, TexCoord},
    resource::Escape,
    texture::image,
};
use wavefront_obj::obj;

//use super::Backend;
use crate::traits;
use std::collections::HashMap;

pub struct TextureRes<B: hal::Backend> {
    pub texture: graphics::texture::Texture<B>,
    pub desc: graphics::resource::DescriptorSet<B>,
}

pub struct ResourceManager<B: hal::Backend> {
    mesh_names: HashMap<String, usize>,
    meshes: Vec<Mesh<B>>,
    texture_names: HashMap<String, usize>,
    textures: Vec<TextureRes<B>>,
}

impl<B: hal::Backend> Default for ResourceManager<B> {
    fn default() -> Self {
        Self {
            mesh_names: HashMap::new(),
            meshes: vec![],
            texture_names: HashMap::new(),
            textures: vec![],
        }
    }
}

impl<B: hal::Backend> traits::ResourceProvider for ResourceManager<B> {
    fn get_mesh(&self, name: &str) -> Option<traits::MeshId> {
        return self.mesh_names.get(name).copied();
    }
    fn get_texture(&self, name: &str) -> Option<traits::TextureId> {
        return self.texture_names.get(name).copied();
    }
}

impl<B: hal::Backend> traits::ResourceManager<super::Hardware<B>> for ResourceManager<B> {
    type Config = String;

    fn load_resources(&mut self, config: &Self::Config, hardware: &mut super::Hardware<B>) {
        let path = std::path::Path::new(config);
        println!(
            "Loading resources from: {}",
            std::fs::canonicalize(path).unwrap().display()
        );
        let paths = std::fs::read_dir(path)
            .map_err(|_| "counldn't read directory")
            .unwrap();
        for path in paths {
            println!("reading thing: {:?}", path);
            let path = path.unwrap().path();
            let extension = path.extension().unwrap().to_str();
            if extension == Some("obj") {
                self.load_model(hardware, &path);
            } else if extension == Some("png") {
                self.load_texture(hardware, &path);
            }
        }
    }

    fn create(config: &Self::Config, hardware: &mut super::Hardware<B>) -> Self {
        let mut s: Self = Default::default();
        s.load_resources(config, hardware);
        s
    }
}

impl<B: hal::Backend> ResourceManager<B> {
    pub fn get_real_mesh(&self, id: usize) -> &Mesh<B> {
        return &self.meshes[id];
    }

    pub fn get_real_texture(&self, id: usize) -> &TextureRes<B> {
        return &self.textures[id];
    }

    pub fn load_from_obj(
        bytes: &[u8],
    ) -> Result<Vec<(MeshBuilder<'static>, Option<String>)>, failure::Error> {
        let string = std::str::from_utf8(bytes)?;
        let set = obj::parse(string).map_err(|e| {
            failure::format_err!(
                "Error during parsing obj-file at line '{}': {}",
                e.line_number,
                e.message
            )
        })?;
        Self::load_from_data(set)
    }

    fn load_from_data(
        obj_set: obj::ObjSet,
    ) -> Result<Vec<(MeshBuilder<'static>, Option<String>)>, failure::Error> {
        // Takes a list of objects that contain geometries that contain shapes that contain
        // vertex/texture/normal indices into the main list of vertices, and converts to
        // MeshBuilders with Position, Normal, TexCoord.
        let mut objects = vec![];

        for object in obj_set.objects {
            for geometry in &object.geometry {
                let mut builder = MeshBuilder::new();

                let mut indices = Vec::new();

                geometry.shapes.iter().for_each(|shape| {
                    if let obj::Primitive::Triangle(v1, v2, v3) = shape.primitive {
                        indices.push(v1);
                        indices.push(v2);
                        indices.push(v3);
                    }
                });
                // We can't use the vertices directly because we have per face normals and not per vertex normals in most obj files
                // TODO: Compress duplicates and return indices for indexbuffer.
                let positions = indices
                    .iter()
                    .map(|index| {
                        let vertex: obj::Normal = object.vertices[index.0];
                        Position([vertex.x as f32, vertex.y as f32, vertex.z as f32])
                    })
                    .collect::<Vec<_>>();

                let normals = indices
                    .iter()
                    .map(|index| {
                        index
                            .2
                            .map(|i| {
                                let normal: obj::Normal = object.normals[i];
                                Normal([normal.x as f32, normal.y as f32, normal.z as f32])
                            })
                            .unwrap_or(Normal([0.0, 0.0, 0.0]))
                    })
                    .collect::<Vec<_>>();

                let tex_coords = indices
                    .iter()
                    .map(|index| {
                        index
                            .1
                            .map(|i| {
                                let tvertex: obj::TVertex = object.tex_vertices[i];
                                TexCoord([tvertex.u as f32, tvertex.v as f32])
                            })
                            .unwrap_or(TexCoord([0.0, 0.0]))
                    })
                    .collect::<Vec<_>>();

                debug_assert!(&normals.len() == &positions.len());
                debug_assert!(&tex_coords.len() == &positions.len());

                let verts: Vec<_> = positions
                    .into_iter()
                    .zip(normals)
                    .zip(tex_coords)
                    .map(|x| PosNormTex {
                        position: (x.0).0,
                        normal: (x.0).1,
                        tex_coord: x.1,
                    })
                    .collect();

                // builder.set_indices(indices.iter().map(|i| i.0 as u16).collect::<Vec<u16>>());

                builder.add_vertices(verts);
                //builder.add_vertices(normals);
                //builder.add_vertices(tex_coords);

                // TODO: Add Material loading
                objects.push((builder, geometry.material_name.clone()))
            }
        }
        Ok(objects)
    }
    fn load_model(&mut self, hardware: &mut super::Hardware<B>, filename: &std::path::PathBuf) {
        println!("loading model: {:?}", filename);
        let bytes = std::fs::read(filename).unwrap();
        let obj_builder = Self::load_from_obj(&bytes);
        let mut obj_builder = match obj_builder {
            Ok(x) => x,
            Err(y) => {
                println!("Error: {:?}", y);
                return;
            }
        };
        if obj_builder.len() > 1 {
            println!("model {:?} contains more than one object", filename);
            return;
        }
        let model_builder = obj_builder.pop().unwrap();
        let qid = hardware.families.family(hardware.used_family).queue(0).id();
        let model = model_builder.0.build(qid, &hardware.factory).unwrap();

        let id = self.meshes.len();
        let name = filename.file_stem().unwrap().to_str().unwrap();
        let indices = model.len();

        self.meshes.push(model);
        self.mesh_names.insert(name.to_owned(), id);
        println!("{} as: {} with {} indices", id, name, indices);
    }

    fn load_texture(&mut self, hardware: &mut super::Hardware<B>, filename: &std::path::PathBuf) {
        use std::fs::File;
        use std::io::BufReader;
        let image_reader = BufReader::new(File::open(filename).unwrap());

        let texture_builder = image::load_from_image(
            image_reader,
            image::ImageTextureConfig {
                generate_mips: true,
                ..Default::default()
            },
        )
        .unwrap();

        let texture = texture_builder
            .build(
                graphics::factory::ImageState {
                    queue: hardware.families.family(hardware.used_family).queue(0).id(),
                    stage: hal::pso::PipelineStage::FRAGMENT_SHADER,
                    access: hal::image::Access::SHADER_READ,
                    layout: hal::image::Layout::ShaderReadOnlyOptimal,
                },
                &mut hardware.factory,
            )
            .unwrap();

        let id = self.textures.len();
        let name = filename.file_stem().unwrap().to_str().unwrap();

        // set_layout! {
        //     factory,
        //     [1] UniformBuffer hal::pso::ShaderStageFlags::FRAGMENT,
        //     [T::len()] CombinedImageSampler hal::pso::ShaderStageFlags::FRAGMENT
        // },
        let factory = &mut hardware.factory;
        let layout = factory
            .create_descriptor_set_layout(vec![
                hal::pso::DescriptorSetLayoutBinding {
                    binding: 0,
                    ty: hal::pso::DescriptorType::SampledImage,
                    count: 1,
                    stage_flags: hal::pso::ShaderStageFlags::FRAGMENT,
                    immutable_samplers: false,
                },
                hal::pso::DescriptorSetLayoutBinding {
                    binding: 1,
                    ty: hal::pso::DescriptorType::Sampler,
                    count: 1,
                    stage_flags: hal::pso::ShaderStageFlags::FRAGMENT,
                    immutable_samplers: false,
                },
            ])
            .unwrap();
        let descriptor_set = factory
            .create_descriptor_set(Escape::share(layout))
            .unwrap();

        unsafe {
            factory.device().write_descriptor_sets(vec![
                hal::pso::DescriptorSetWrite {
                    set: descriptor_set.raw(),
                    binding: 0,
                    array_offset: 0,
                    descriptors: vec![hal::pso::Descriptor::Image(
                        texture.view().raw(),
                        hal::image::Layout::ShaderReadOnlyOptimal,
                    )],
                },
                hal::pso::DescriptorSetWrite {
                    set: descriptor_set.raw(),
                    binding: 1,
                    array_offset: 0,
                    descriptors: vec![hal::pso::Descriptor::Sampler(texture.sampler().raw())],
                },
            ]);
        }

        self.textures.push(TextureRes {
            texture,
            desc: Escape::unescape(descriptor_set),
        });
        self.texture_names.insert(name.to_owned(), id);
    }
}

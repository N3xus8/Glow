use std::io::{self, BufReader, Cursor};

use wgpu::util::DeviceExt;

use crate::{model, texture::{self, Texture}, utils::rgba_f32_to_u8};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::UnwrapThrowExt;

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let mut origin = location.origin().unwrap();
    if !origin.ends_with("learn-wgpu") {
        origin = format!("{}/learn-wgpu", origin);
    }
    //let base = reqwest::Url::parse(&format!("{}/", origin,)).unwrap();
    let base = reqwest::Url::parse(&format!("{}", origin)).unwrap();
    base.join(file_name).unwrap()
}

pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    #[cfg(target_arch = "wasm32")]
    let txt = {
        let url = format_url(file_name);
        reqwest::get(url).await?.text().await?
    };
    #[cfg(not(target_arch = "wasm32"))]
    let txt = {
        let path = std::path::Path::new(env!("OUT_DIR"))
            .join("res")
            .join(file_name);
        std::fs::read_to_string(path)?
    };
    Ok(txt)
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    let data = {
        let url = format_url(file_name);
        reqwest::get(url).await?.bytes().await?.to_vec()
    };
    #[cfg(not(target_arch = "wasm32"))]
    let data = {
        let path = std::path::Path::new(env!("OUT_DIR"))
            .join("res")
            .join(file_name);
        std::fs::read(path)?
    };

    Ok(data)
}

pub async fn load_texture(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<texture::Texture> {
    texture::Texture::get_texture_from_image(device, queue, file_name).await
}

pub enum ModelFile<'a> {
    Obj(&'a str),
    Gltf(&'a str),
}

pub async fn load_model<'a> (
    file: &ModelFile<'_>,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<model::Model>{
    match file {
            ModelFile::Obj(file_name) => { 
                load_model_obj(
                    file_name,
                    &device,
                    &queue,
                    &layout,
                ).await
            },
            ModelFile::Gltf(file_name) => {
                load_model_gtlf(
                    file_name,
                    &device,
                    &queue,
                    &layout,
                ).await
            },            
        }
    }

pub async fn load_model_obj(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<model::Model> {
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let mat_text = load_string(&p).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let mut materials = Vec::new();
    for m in obj_materials? {
        if let Some(filename) = m.diffuse_texture {
            let diffuse_texture = load_texture(&filename, device, queue).await?;

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    },
                ],
                label: None,
            });

            materials.push(model::Material {
                name: m.name,
                diffuse_texture,
                bind_group,
            })
        }
    }

    let meshes = models
        .into_iter()
        .map(|m| {
            let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| {
                    if m.mesh.normals.is_empty() {
                        model::ModelVertex {
                            position: [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2],
                            ],
                            tex_coords: [
                                m.mesh.texcoords[i * 2],
                                1.0 - m.mesh.texcoords[i * 2 + 1],
                            ],
                            normal: [0.0, 0.0, 0.0],
                        }
                    } else {
                        model::ModelVertex {
                            position: [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2],
                            ],
                            tex_coords: [
                                m.mesh.texcoords[i * 2],
                                1.0 - m.mesh.texcoords[i * 2 + 1],
                            ],
                            normal: [
                                m.mesh.normals[i * 3],
                                m.mesh.normals[i * 3 + 1],
                                m.mesh.normals[i * 3 + 2],
                            ],
                        }
                    }
                })
                .collect::<Vec<_>>();

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file_name)),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            model::Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();

    Ok(model::Model { meshes, materials })
}

use gltf::Gltf ;

pub async fn load_model_gtlf(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<model::Model> {


    let text = load_string(file_name).await?;
    let cursor = Cursor::new(text);
    let reader = BufReader::new(cursor);   

    let gltf = Gltf::from_reader(reader)?;
    let document = gltf.clone().document;
    let blob = gltf.clone().blob;

    let base_path = Some(std::path::Path::new(env!("OUT_DIR"))
                            .join("res/models"));
    
    let buffers = gltf::import_buffers(&document, base_path.as_deref(), blob.clone())?;
    
    let meshes: Vec<model::Mesh> = gltf
        .meshes()
        .flat_map(|mesh| {
            //println!("Mesh #{}", mesh.index());

            mesh.primitives().map(|primitive| {
                //println!("- Primitive #{}", primitive.index());

                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                // --- Vertices (positions required, normals optional)
                let positions = reader.read_positions().expect(" no positions");
                let normals = reader.read_normals();                

                let mut vertices: Vec<model::ModelVertex> = match normals {
                    Some(normals) => {

                        positions
                            .zip(normals)
                            .map(|(position, normal)| model::ModelVertex {
                                position,
                                normal,
                                tex_coords: [0.0; 2],
                            })
                            .collect()
                    }
                    None => positions
                        .map(|position| model::ModelVertex {
                            position,
                            normal: [0.0; 3],
                            tex_coords: [0.0; 2],
                        })
                        .collect(),
                };

                // --- Tex coords (optional)
                if let Some(tex_coords) = reader.read_tex_coords(0) {
                    for (vertex, uv) in vertices.iter_mut().zip(tex_coords.into_f32()) {
                        vertex.tex_coords = uv;
                    }
                }

                // --- Vertex buffer
                let vertex_buffer = device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Vertex Buffer"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    },
                );



                let indices: Vec<u32> = match reader.read_indices() {
                    Some(gltf::mesh::util::ReadIndices::U8(iter)) =>
                        iter.map(|i| i as u32).collect(),
                    Some(gltf::mesh::util::ReadIndices::U16(iter)) =>
                        iter.map(|i| i as u32).collect(),
                    Some(gltf::mesh::util::ReadIndices::U32(iter)) =>
                        iter.collect(),
                    None => panic!("Missing index buffer"),
                };

                let index_buffer = device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Index Buffer"),
                        contents: bytemuck::cast_slice(&indices),
                        usage: wgpu::BufferUsages::INDEX,
                    },
                );

                // --- Final GPU mesh (one per primitive)
                model::Mesh {
                    name: file_name.to_string(),
                    vertex_buffer,
                    index_buffer,
                    num_elements: indices.len() as u32,
                    material: primitive.material().index().unwrap_or(0),
                }
            })
        })
        .collect();


    let mut materials = Vec::new();

    for material in gltf.materials() {
        let name = material.name().unwrap_or("Unnamed").to_string();
        
         

        // Base color texture
        let diffuse_texture = if let Some(base_color_info) = material.pbr_metallic_roughness().base_color_texture() {

            let texture = base_color_info.texture();
            let image = texture.source().source();

            // image: gltf::image::Source
            let diffuse_texture = match image {
                gltf::image::Source::View { view, mime_type: _} => {
                    let start = view.offset();
                    let end = start + view.length();
                    let blob = blob.as_ref().expect("no blob data");
                    let image_bytes = &blob[start..end];
                    
                    println!("Info: texture bin  found");

                    cfg_if::cfg_if! {
                        // embedded buffer
                        if #[cfg(not(target_arch = "wasm32"))]{
                            Texture::load_texture_from_buffer(image_bytes, device, queue).await?
                        } else {
                            Texture::load_texture_from_buffer_web(image_bytes, device, queue).await.unwrap_throw()
                        }
                    }
                },
                gltf::image::Source::Uri { uri, mime_type: _ } => {

                    let path = std::path::Path::new(env!("OUT_DIR"))
                            .join("res")
                            .join(file_name);

                    if path.exists() {
                        println!("Info: texture file found");
                        load_texture(&uri, device, queue).await?
                    } else {
                        println!("No texture attached to this material, applying default texture");
                        Texture::create_black_pink_checker_texture(device, queue)
                    }
                },
            };

            Some(diffuse_texture)
        } else {

            // If there's no base color texture, use the base color factor
            //println!("No base color texture, using base color factor.");
            let color = rgba_f32_to_u8(material.pbr_metallic_roughness().base_color_factor());
            
            // You can create a solid color texture or just use this as a base color for the material
            Some(Texture::create_solid_color_texture(device, queue, color)) // This function could create a 1x1 texture with the color

        };

        //let diffuse_texture: Option<Texture> = None ;
        // We add default texture and bind group if don't exist
        let (diffuse_texture, bind_group) =
        if let Some(diffuse_texture) = diffuse_texture {
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    },
                ],
                label: Some(&format!("{} bind group", name)),
            });
            (diffuse_texture, bind_group)
        } else {
                
                println!("No texture attached to this material, applying default texture");

                // If there's no texture
                let default_texture = Texture::create_black_pink_checker_texture(device, queue);
                let default_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&default_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&default_texture.sampler),
                        },
                    ],
                    label: Some("default material bind group"),
                });
            (default_texture, default_bind_group)
        };



        materials.push(model::Material {
            name,
            diffuse_texture,
            bind_group,
        });

    }

        Ok( 
                model::Model{
                    meshes,
                    materials,
                }
            )

    }
    


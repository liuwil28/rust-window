use std::io::{BufReader, Cursor};
use std::path::{Path, PathBuf};
use std::fs;
use wgpu::util::DeviceExt;

use crate::{model, texture};

fn load_path(filename: &str) -> PathBuf {
    Path::new(env!("OUT_DIR"))
        .join("res")
        .join(filename)
}

fn load_texture(filename: &str, device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<texture::Texture> {
    let data = fs::read(load_path(filename))?;
    Ok(texture::Texture::from_image_bytes(&data, filename, device, queue))
}

pub fn load_model(filename: &str, bind_group_layout: &wgpu::BindGroupLayout, device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<model::Model> {
    let obj_text = fs::read_to_string(load_path(filename))?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| {
            let mat_text = fs::read_to_string(load_path(p.to_str().unwrap())).unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )?;
    let mut obj_materials = obj_materials?;

    if obj_materials.len() == 0 {
        obj_materials.push(tobj::Material {
            name: String::from("missing_material"),
            ..Default::default()
        });
    }

    let mut materials = Vec::new();
    for m in obj_materials {
        // let diffuse_texture = texture::Texture::from_rgba(&m.name, 255, 255, 255, 255, &device, &queue);
        let diffuse_texture = if let Some(texture) = &m.diffuse_texture {
            load_texture(&texture, device, queue)?
        } else {
            let color: Vec<u8> = m.diffuse
                .unwrap_or([0.0, 0.0, 0.0])
                .iter()
                .map(|v| (v * 255.0).floor() as u8)
                .collect();
            texture::Texture::from_rgba(&m.name, color[0], color[1], color[2], 255, &device, &queue)
        };
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
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
            texture: diffuse_texture,
            bind_group,
        })
    }

    let meshes = models
        .into_iter()
        .map(|m| {
            let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| model::ModelVertex {
                    position: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                    texture_coords: if m.mesh.texcoords.len() > 0 {[
                        m.mesh.texcoords[i * 2],
                        1.0 - m.mesh.texcoords[i * 2 + 1]
                    ]} else {
                        [0.0, 0.0]
                    },
                    normal: if m.mesh.normals.len() > 0 {[
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ]} else {
                        [0.0, 0.0, 0.0]
                    },
                })
                .collect::<Vec<_>>();

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", filename)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", filename)),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            model::Mesh {
                name: String::from(filename),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();

    Ok(model::Model { meshes, materials })
}

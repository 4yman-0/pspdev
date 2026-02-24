//! WIP

use bytemuck::cast_slice;
use clap::{Parser, ValueEnum};
use core::mem;
use std::fs::{self, File};
use std::io::{BufWriter, Seek, Write};
use std::path::PathBuf;

mod vertex;
use vertex::*;

#[derive(Parser)]
struct Cli {
    input: PathBuf,
    output: PathBuf,

    #[arg(short, long)]
    texcoords: Option<VertexAttrType>,
    #[arg(short, long)]
    colors: Option<ColorAttrType>,
    #[arg(short, long)]
    normals: Option<VertexAttrType>,
    #[arg(short, long)]
    vertices: Option<VertexAttrType>,

    #[arg(long)]
    use_texcoords: Option<bool>,
    #[arg(long)]
    use_colors: Option<bool>,
    #[arg(long)]
    use_normals: Option<bool>,
}

#[derive(Clone, Copy, ValueEnum)]
enum VertexAttrType {
    F32,
    I8,
    I16,
}

#[derive(Clone, Copy, ValueEnum)]
#[non_exhaustive]
enum ColorAttrType {
    Rgba8,
}

/// Primitive types
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum Primitive {
    /// Single pixel points (1 vertex per primitive)
    Points = 0,
    /// Single pixel lines (2 vertices per primitive)
    Lines = 1,
    /// Single pixel line-strip (2 vertices for the first primitive, 1 for every following)
    LineStrip = 2,
    /// Filled triangles (3 vertices per primitive)
    Triangles = 3,
    /// Filled triangles-strip (3 vertices for the first primitive, 1 for every following)
    TriangleStrip = 4,
    /// Filled triangle-fan (3 vertices for the first primitive, 1 for every following)
    TriangleFan = 5,
    /// Filled blocks (2 vertices per primitive)
    Sprites = 6,
}

fn read_from_stdin() -> String {
    let mut buffer = String::new();
    let stdin = std::io::stdin();
    stdin.read_line(&mut buffer).unwrap();
    buffer
}

fn main() {
    let cli = Cli::parse();
    if std::fs::exists(&cli.output).unwrap() {
        print!("Output file already exists, overwrite? [y/N]: ");
        std::io::stdout().flush().unwrap();
		let input = read_from_stdin();
        if input.to_lowercase().trim() != "y" {
            println!();
            std::io::stdout().flush().unwrap();
            return;
        }
    }

    let mut normals: Vec<u8> = Vec::new();
    let mut colors: Vec<u8> = Vec::new();
    let mut texcoords: Vec<u8> = Vec::new();
    let mut vertices: Vec<u8> = Vec::new();
    let mut indices: Vec<u8> = Vec::new();

    let normals_attr_type = cli.normals.unwrap_or(VertexAttrType::I8);
	let colors_attr_type = cli.colors.unwrap_or(ColorAttrType::Rgba8);
    let texcoords_attr_type = cli.texcoords.unwrap_or(VertexAttrType::I16);
    let vertices_attr_type = cli.vertices.unwrap_or(VertexAttrType::F32);

    let model = modelz::Model3D::load(&cli.input)
    	.unwrap();
    if model.meshes.len() != 1 {
        panic!("Models count != 1");
    }
    let mesh = model.meshes.into_iter().next().unwrap();
    let (mesh_vertices, mesh_indices, mesh_primitive) = (
    	mesh.vertices,
    	mesh.indices.expect("Indices not found"),
    	mesh.mode,
    );
    let _ = mesh;

    let use_normals = mesh_vertices[0].normal.is_some()
     && cli.use_normals.unwrap_or(true);
    println!("normals: {use_normals}");

    let use_colors = mesh_vertices[0].color.is_some()
     && cli.use_colors.unwrap_or(true);
    println!("vertex colors: {use_colors}");

    let use_texcoords = mesh_vertices[0].tex_coord.is_some()
     && cli.use_texcoords.unwrap_or(true);
    println!("texture coords: {use_texcoords}");

    println!(
        "v: {}, i: {}",
        vertices.len() / 3,
        match &mesh_indices {
        	modelz::Indices::U8(i) => i.len(),
        	modelz::Indices::U16(i) => i.len(),
        	modelz::Indices::U32(i) => i.len(),
        },
    );

    let vertex_type = {
        let mut v = VertexType::empty();
        v |= VertexType::TRANSFORM_3D;
        v |= match &mesh_indices {
        	modelz::Indices::U8(_) => VertexType::INDEX_8BIT,
        	modelz::Indices::U16(_)
        	 | modelz::Indices::U32(_) => VertexType::INDEX_16BIT,
        };
        if use_texcoords {
            v |= match texcoords_attr_type {
                VertexAttrType::F32 => VertexType::TEXTURE_32BITF,
                VertexAttrType::I16 => VertexType::TEXTURE_16BIT,
                VertexAttrType::I8 => VertexType::TEXTURE_8BIT,
            };
        }
        if use_colors {
        	v |= match colors_attr_type {
        		ColorAttrType::Rgba8 => VertexType::COLOR_8888,
        		//_ => { panic!("Unsupported color attribute type") },
        	};
        }
        if use_normals {
            v |= match normals_attr_type {
                VertexAttrType::F32 => VertexType::NORMAL_32BITF,
                VertexAttrType::I16 => VertexType::NORMAL_16BIT,
                VertexAttrType::I8 => VertexType::NORMAL_8BIT,
            };
        }
        v |= match vertices_attr_type {
            VertexAttrType::F32 => VertexType::VERTEX_32BITF,
            VertexAttrType::I16 => VertexType::VERTEX_16BIT,
            VertexAttrType::I8 => VertexType::VERTEX_8BIT,
        };
        v
    };

    let (sizes, (vertex_size, paddings)) = {
        let elem_sizes = vt_element_sizes(vertex_type);
        let sizes = vt_sizes(&elem_sizes);
        (sizes, vt_alignments(&elem_sizes, &sizes))
    };
    println!("sizes: {sizes:?}");
    println!("paddings: {paddings:?}");

    for vert in &mesh_vertices {
        match vertices_attr_type {
            VertexAttrType::F32 => {
                vertices.extend(cast_slice(&vert.position));
            }
            VertexAttrType::I16 => {
                let pos = vert_to_i16(&vert.position);
                vertices.extend(cast_slice(&pos));
            }
            VertexAttrType::I8 => {
                let pos = vert_to_i8(&vert.position);
                vertices.extend(cast_slice(&pos));
            }
        }
    }

    if use_normals {
        for vert in &mesh_vertices {
        	let norm = vert.normal.unwrap();
            match normals_attr_type {
                VertexAttrType::F32 => {
                    normals.extend(cast_slice(&norm));
                }
                VertexAttrType::I16 => {
                    let norm = norms_to_i16(&norm);
                    normals.extend(cast_slice(&norm));
                }
                VertexAttrType::I8 => {
                    let norm = norms_to_i8(&norm);
                    normals.extend(cast_slice(&norm));
                }
            }
        }
    }
    
    if use_colors {
        for vert in &mesh_vertices {
        	let color = vert.color.unwrap();
            match colors_attr_type {
				ColorAttrType::Rgba8 => {
					let color = color_to_rgba8(&color);
					colors.extend(color);
				},
            }
        }
    }

    if use_texcoords {
        for vert in &mesh_vertices {
        	let texcoord = vert.tex_coord.unwrap();
            match texcoords_attr_type {
                VertexAttrType::F32 => {
                    texcoords.extend(cast_slice(&texcoord));
                }
                VertexAttrType::I16 => {
                    let texcoord = texcoords_to_i16(&texcoord);
                    texcoords.extend(cast_slice(&texcoord));
                }
                VertexAttrType::I8 => {
                    let texcoord = texcoords_to_i8(&texcoord);
                    texcoords.extend(cast_slice(&texcoord));
                }
            }
        }
    }

	match mesh_indices {
		modelz::Indices::U8(i) => indices = i,
		modelz::Indices::U16(i) => {
			for idx in i {
				let bytes = idx.to_le_bytes();
				indices.extend(bytes);
			}
		},
		modelz::Indices::U32(i) => {
			for idx in i {
				let bytes = (idx as u16).to_le_bytes();
				indices.extend(bytes);
			}			
		},
	}

    let _ = mesh;
    let _ = (vertices_attr_type, normals_attr_type, texcoords_attr_type, colors_attr_type);

    println!(
        "t: {}, c: {}, n: {}, v: {}",
		if use_texcoords {
		    texcoords.len() / sizes[0]
		} else {
		    0
		},
        if use_colors {
            colors.len() / sizes[1]
        } else {
            0
        },
        if use_normals {
            normals.len() / sizes[2]
        } else {
            0
        },
        vertices.len() / sizes[3],
    );

    let _ = fs::remove_file(&cli.output);
    let output_file = File::create(&cli.output).unwrap();
    let mut output_file = BufWriter::new(output_file);

    output_file
        .write_all(&((match mesh_primitive {
        	modelz::RenderMode::TriangleStrip => Primitive::TriangleStrip,
        	modelz::RenderMode::Triangles => Primitive::Triangles,
        	modelz::RenderMode::Lines => Primitive::Lines,
        	modelz::RenderMode::LineStrip => Primitive::LineStrip,
        	_ => {
        		println!("Unknown render mode, pretending it is Triangles");
        		Primitive::Triangles
        	},
        }) as u32).to_le_bytes())
        .unwrap();

    output_file
        .write_all(&vertex_type.bits().to_le_bytes())
        .unwrap();

    let vertices_length = vertices.len() / sizes[3];
    output_file
        .write_all(&((vertex_size * vertices_length) as u32).to_le_bytes())
        .unwrap();
    println!(
        "vertices: {}, {vertices_length}",
        vertex_size * vertices_length
    );

    let indices_size = mem::size_of_val(&indices[..]) as u32;
    output_file.write_all(&indices_size.to_le_bytes()).unwrap();
    println!("indices: {indices_size}, {}", indices.len());

    let zero_array = [0u8; 64];

    for idx in 0..vertices_length {
        if use_texcoords {
            output_file.write_all(&zero_array[..paddings[0]]).unwrap();
            let tex_idx = idx * sizes[0];
            output_file
                .write_all(cast_slice(&texcoords[tex_idx..tex_idx + sizes[0]]))
                .unwrap();
        }
        if use_colors {
            output_file.write_all(&zero_array[..paddings[1]]).unwrap();
            let color_idx = idx * sizes[1];
            output_file
                .write_all(cast_slice(&colors[color_idx..color_idx + sizes[1]]))
                .unwrap();
        }
        if use_normals {
            output_file.write_all(&zero_array[..paddings[2]]).unwrap();
            let norm_idx = idx * sizes[2];
            output_file
                .write_all(cast_slice(&normals[norm_idx..norm_idx + sizes[2]]))
                .unwrap();
        }
        output_file.write_all(&zero_array[..paddings[3]]).unwrap();
        let vert_idx = idx * sizes[3];
        output_file
            .write_all(cast_slice(&vertices[vert_idx..vert_idx + sizes[3]]))
            .unwrap();
    }

    println!("written {} bytes", output_file.stream_position().unwrap());

    let indices_slice: &[u8] = cast_slice(&indices);
    output_file.write_all(indices_slice).unwrap();

    output_file.flush().unwrap();
}

fn vert_to_i16(vert: &[f32]) -> [i16; 3] {
    const COEFF: f32 = i16::MAX as f32;
    [
        (vert[0] * COEFF) as i16,
        (vert[1] * COEFF) as i16,
        (vert[2] * COEFF) as i16,
    ]
}

fn vert_to_i8(vert: &[f32]) -> [i8; 3] {
    const COEFF: f32 = i8::MAX as f32;
    [
        (vert[0] * COEFF) as i8,
        (vert[1] * COEFF) as i8,
        (vert[2] * COEFF) as i8,
    ]
}

fn norms_to_i16(vert: &[f32]) -> [i16; 3] {
    const COEFF: f32 = i16::MAX as f32;
    [
        (vert[0] * COEFF) as i16,
        (vert[1] * COEFF) as i16,
        (vert[2] * COEFF) as i16,
    ]
}

fn norms_to_i8(vert: &[f32]) -> [i8; 3] {
    const COEFF: f32 = i8::MAX as f32;
    [
        (vert[0] * COEFF) as i8,
        (vert[1] * COEFF) as i8,
        (vert[2] * COEFF) as i8,
    ]
}
// TODO: support wrapping
fn texcoords_to_i16(vert: &[f32]) -> [i16; 2] {
    const COEFF: f32 = i16::MAX as f32;
    [
        (vert[0].abs().clamp(0.0, 1.0) * COEFF) as i16,
        (vert[1].abs().clamp(0.0, 1.0) * COEFF) as i16,
    ]
}

fn texcoords_to_i8(vert: &[f32]) -> [i8; 2] {
    const COEFF: f32 = i8::MAX as f32;
    [
        (vert[0].abs().clamp(0.0, 1.0) * COEFF) as i8,
        (vert[1].abs().clamp(0.0, 1.0) * COEFF) as i8,
    ]
}

fn color_to_rgba8(color: &[f32]) -> [u8; 4] {
	const COEFF: f32 = u8::MAX as f32;
	[
		(color[0] * COEFF) as u8,
		(color[1] * COEFF) as u8,
		(color[2] * COEFF) as u8,
		(color[3] * COEFF) as u8,
	]
}

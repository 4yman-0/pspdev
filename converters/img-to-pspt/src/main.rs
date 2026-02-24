//! WIP

mod texture;
use texture::TexturePixelFormat;

use clap::Parser;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use image::{ImageReader};
use bytemuck::cast_slice;

#[derive(clap::ValueEnum, Clone, Copy, Default)]
enum FormatType {
	#[default]
	Abgr8,
	Abgr4,
	Abgr5551,
	Bgr565,
	// TODO: other types
}
#[derive(Parser)]
struct Cli {
    input: PathBuf,
    output: PathBuf,

	#[arg(short, long)]
    format: Option<FormatType>,
}

fn main() {
    let cli = Cli::parse();
    if std::fs::exists(&cli.output).unwrap() {
        print!("Output file already exists, overwrite? [y/N]: ");
        std::io::stdout().flush().unwrap();
        let mut buffer = String::new();
        let stdin = std::io::stdin();
        stdin.read_line(&mut buffer).unwrap();
        if buffer.to_lowercase().trim() != "y" {
            return;
        }
    }

	let image = ImageReader::open(&cli.input)
		.unwrap()
		.decode()
		.unwrap();

	if !image.width().is_power_of_two()
	|| !image.height().is_power_of_two() {
		println!("Width and height should be a power of two");
		return;
	}

	let format = match cli.format.unwrap_or_default() {
		FormatType::Abgr8 => TexturePixelFormat::Psm8888,
		FormatType::Abgr4 => TexturePixelFormat::Psm4444,
		FormatType::Abgr5551 => TexturePixelFormat::Psm5551,
		FormatType::Bgr565 => TexturePixelFormat::Psm5650,
	};

    let _ = fs::remove_file(&cli.output);
    let output_file = File::create(&cli.output).unwrap();
    let mut output_file = BufWriter::new(output_file);

    output_file.write_all(&(format as u32).to_le_bytes()).unwrap();
    output_file.write_all(&u8::from(false).to_le_bytes()).unwrap();

    output_file.write_all(&image.width().to_le_bytes()).unwrap();
    output_file.write_all(&image.height().to_le_bytes()).unwrap();

	// TODO: actually write the texture	
	match format {
		TexturePixelFormat::Psm8888 => {
			let image = image.to_rgba8();
			let image = image
				.pixels()
				.map(|c| abgr8888(c))
				.collect::<Vec<_>>();
			output_file.write_all(cast_slice(&image)).unwrap();
		},
		TexturePixelFormat::Psm5551 => {
			let image = image.to_rgba8();
			let image = image
				.pixels()
				.map(|c| abgr5551(c))
				.collect::<Vec<_>>();
			output_file.write_all(cast_slice(&image)).unwrap();
		},
		TexturePixelFormat::Psm4444 => {
			let image = image.to_rgba8();
			let image = image
				.pixels()
				.map(|c| abgr4444(c))
				.collect::<Vec<_>>();
			output_file.write_all(cast_slice(&image)).unwrap();
		},
		TexturePixelFormat::Psm5650 => {
			let image = image.to_rgb8();
			let image = image
				.pixels()
				.map(|c| bgr565(c))
				.collect::<Vec<_>>();
			output_file.write_all(cast_slice(&image)).unwrap();
		},
		_ => { unimplemented!() },
	}

    output_file.flush().unwrap();
}

fn abgr8888(color: &image::Rgba<u8>) -> u32 {
	((color[3] as u32) << 24)
	 | ((color[2] as u32) << 16)
	 | ((color[1] as u32) << 8)
	 | color[0] as u32
}

fn abgr4444(color: &image::Rgba<u8>) -> u16 {
	let r = color[0] as u16 * 15 / 255;
	let g = color[1] as u16 * 15 / 255;
	let b = color[2] as u16 * 15 / 255;
	let a = color[3] as u16 * 15 / 255;
	(a << 12)
	 | (b << 8)
	 | (g << 4)
	 | r
}

fn abgr5551(color: &image::Rgba<u8>) -> u16 {
	let r = color[0] as u16 * 31 / 255;
	let g = color[1] as u16 * 31 / 255;
	let b = color[2] as u16 * 31 / 255;
	let a = u16::from(color[3] > 128);
	(a << 15)
	 | (b << 10)
	 | (g << 5)
	 | r
}

fn bgr565(color: &image::Rgb<u8>) -> u16 {
	let r = color[0] as u16 * 31 / 255;
	let g = color[1] as u16 * 63 / 255;
	let b = color[2] as u16 * 31 / 255;
	(b << 11)
	 | (g << 5)
	 | r
}

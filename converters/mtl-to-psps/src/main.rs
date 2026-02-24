//! WIP

use clap::Parser;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    input: PathBuf,
    output: PathBuf,
}

fn write_bytes<W: Write>(write: &mut W, value: &[u8]) {
	write.write_all(value).unwrap();
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
    let input_file = File::open(&cli.input).unwrap();
    let input_file = BufReader::new(input_file);

    let mut ambient: Option<u32> = None;
    let mut diffuse: Option<u32> = None;
    let mut specular: Option<u32> = None;
    let mut specular_coeff: Option<f32> = None;
    let mut alpha: Option<f32> = None;

    for input_line in input_file.lines() {
        let input_line = input_line.unwrap();
        let input_line = input_line.trim();
        if input_line.is_empty() {
            continue;
        }
        let parts = input_line.split(" ").collect::<Vec<_>>();
        if parts[0] == "Ka" {
            ambient = Some(color_convert(
                parts[1].parse().unwrap(),
                parts[2].parse().unwrap(),
                parts[3].parse().unwrap(),
            ));
        } else if parts[0] == "Kd" {
            diffuse = Some(color_convert(
                parts[1].parse().unwrap(),
                parts[2].parse().unwrap(),
                parts[3].parse().unwrap(),
            ));
        } else if parts[0] == "Ks" {
            specular = Some(color_convert(
                parts[1].parse().unwrap(),
                parts[2].parse().unwrap(),
                parts[3].parse().unwrap(),
            ));
        } else if parts[0] == "Ns" {
            specular_coeff = Some(parts[1].parse().unwrap());
        } else if parts[0] == "d" {
            alpha = Some(parts[1].parse().unwrap());
        } else if parts[0] == "Tr" {
            alpha = Some(1.0 - parts[1].parse::<f32>().unwrap());
        }
    }

	let use_ambient = ambient.is_some();
	let use_diffuse = diffuse.is_some();
	let use_specular = specular.is_some();
	let use_specular_coeff = specular_coeff.is_some();
	
    let ambient = ambient.unwrap_or(0);
    let diffuse = diffuse.unwrap_or(0);
    let specular = specular.unwrap_or(0);
    let specular_coeff = specular_coeff.unwrap_or(0.0);

	let alpha = (alpha.unwrap_or(1.0) * 255.0).clamp(0.0, 255.0) as u32;

	let use_real_ambient = true;
	let real_ambient = 0xffffff00 | alpha;
	let _ = alpha;

	if use_real_ambient {
		println!("real_ambient: {real_ambient:X}");
	}
	if use_ambient {
		println!("ambient: {ambient:X}");
	}
	if use_diffuse {
		println!("diffuse: {diffuse:X}");
	}
	if use_specular {
		println!("specular: {specular:X}");
	}
	if use_specular_coeff {
		println!("specular_coeff: {specular_coeff}");
	}

    let _ = fs::remove_file(&cli.output);
    let output_file = File::create(&cli.output).unwrap();
    let mut output_file = BufWriter::new(output_file);

    write_bytes(&mut output_file, &u16::from(use_real_ambient).to_le_bytes());
    write_bytes(&mut output_file, &real_ambient.to_le_bytes());
    
    write_bytes(&mut output_file, &u16::from(use_ambient).to_le_bytes());
    write_bytes(&mut output_file, &ambient.to_le_bytes());
    
    write_bytes(&mut output_file, &u16::from(use_diffuse).to_le_bytes());
    write_bytes(&mut output_file, &diffuse.to_le_bytes());
    
    write_bytes(&mut output_file, &u16::from(use_specular).to_le_bytes());
    write_bytes(&mut output_file, &specular.to_le_bytes());
    
    write_bytes(&mut output_file, &u16::from(use_specular_coeff).to_le_bytes());
    write_bytes(
    	&mut output_file,
        &specular_coeff.to_le_bytes(),
	);

    output_file.flush().unwrap();
}

fn color_convert(r: f32, g: f32, b: f32) -> u32 {
    let (r, g, b) = (
        r.clamp(0.0, 1.0) * 255.0,
        g.clamp(0.0, 1.0) * 255.0,
        b.clamp(0.0, 1.0) * 255.0,
    );
    // RGB
    ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | 0xff
}

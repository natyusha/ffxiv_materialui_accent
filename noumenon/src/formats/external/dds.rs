#![allow(dead_code)]

use std::io::{Read, Seek, Write, SeekFrom};
use binrw::BinReaderExt;
use squish::Format as SFormat;

pub trait Dds {
	fn read<T>(reader: &mut T) -> Self where T: Read + Seek;
	fn write<T>(&self, writer: &mut T) where T: Write + Seek;
}

// https://docs.microsoft.com/en-us/windows/win32/direct3ddds/dx-graphics-dds-pguide
// b g r a
// ---------------------------------------- //

#[derive(Copy, Clone)]
pub enum Format {
	Unknown,
	L8,
	A8,
	A4R4G4B4,
	A1R5G5B5,
	A8R8G8B8,
	X8R8G8B8,
	Dxt1,
	Dxt3,
	Dxt5,
	A16B16G16R16
}

impl Format {
	pub fn convert_from(&self, data: &[u8]) -> Option<Vec<u8>> {
		match self {
			Format::L8       => Some(convert_from_l8(data)),
			Format::A8       => Some(convert_from_a8(data)),
			Format::A4R4G4B4 => Some(convert_from_a4r4g4b4(data)),
			Format::A1R5G5B5 => Some(convert_from_a1r5g5b5(data)),
			Format::A8R8G8B8 => Some(Vec::from(data)),
			Format::X8R8G8B8 => Some(convert_from_x8r8g8b8(data)),
			Format::Dxt1     => Some(convert_from_compressed(SFormat::Bc1, data)),
			Format::Dxt3     => Some(convert_from_compressed(SFormat::Bc3, data)),
			Format::Dxt5     => Some(convert_from_compressed(SFormat::Bc5, data)),
			_                => None,
		}
	}
	
	pub fn convert_to(&self, data: &[u8]) -> Option<Vec<u8>> {
		match self {
			Format::L8       => Some(convert_to_l8(data)),
			Format::A8       => Some(convert_to_a8(data)),
			Format::A4R4G4B4 => Some(convert_to_a4r4g4b4(data)),
			Format::A1R5G5B5 => Some(convert_to_a1r5g5b5(data)),
			Format::A8R8G8B8 => Some(Vec::from(data)),
			Format::X8R8G8B8 => Some(convert_to_x8r8g8b8(data)),
			Format::Dxt1     => Some(convert_to_compressed(SFormat::Bc1, data)),
			Format::Dxt3     => Some(convert_to_compressed(SFormat::Bc3, data)),
			Format::Dxt5     => Some(convert_to_compressed(SFormat::Bc5, data)),
			_                => None,
		}
	}
	
	pub fn get<T>(reader: &mut T) -> Format where T: Read + Seek {
		reader.seek(SeekFrom::Start(84)).unwrap();
		let cc: u32 = reader.read_le().unwrap();
		reader.seek(SeekFrom::Start(92)).unwrap();
		let rmask: u32 = reader.read_le().unwrap();
		reader.seek(SeekFrom::Current(8)).unwrap();
		let amask: u32 = reader.read_le().unwrap();
		
		match (cc, rmask, amask) { // eh, good enough
			(0,          0xFF,       0         ) => Format::L8,
			(0,          0,          0xFF      ) => Format::A8,
			(0,          0x0F00,     0xF000    ) => Format::A4R4G4B4,
			(0,          0x7C00,     0x8000    ) => Format::A1R5G5B5,
			(0,          0x00FF0000, 0xFF000000) => Format::A8R8G8B8,
			(0,          0x00FF0000, 0         ) => Format::X8R8G8B8,
			(0x31545844, 0,          0         ) => Format::Dxt1,
			(0x33545844, 0,          0         ) => Format::Dxt3,
			(0x35545844, 0,          0         ) => Format::Dxt5,
			// (113,        0,          0         ) => Format::A16B16G16R16,
			_                                    => Format::Unknown,
		}
	}
}

// ---------------------------------------- //

fn convert_from_l8(data: &[u8]) -> Vec<u8> {
	data
		.chunks_exact(1)
		.flat_map(|p| {
			let v = p[0];
			[v, v, v, 255]
		}).collect::<Vec<u8>>()
}

fn convert_to_l8(data: &[u8]) -> Vec<u8> {
	data
		.chunks_exact(4)
		.flat_map(|p| {
			[p[0]]
		}).collect::<Vec<u8>>()
}

// ---------------------------------------- //

fn convert_from_a8(data: &[u8]) -> Vec<u8> {
	data
		.chunks_exact(1)
		.flat_map(|p| {
			[0, 0, 0, p[0]]
		}).collect::<Vec<u8>>()
}

fn convert_to_a8(data: &[u8]) -> Vec<u8> {
	data
		.chunks_exact(4)
		.flat_map(|p| {
			[p[0]]
		}).collect::<Vec<u8>>()
}

// ---------------------------------------- //

fn convert_from_a4r4g4b4(data: &[u8]) -> Vec<u8> {
	data
		.chunks_exact(2)
		.flat_map(|p| {
			let v = ((p[1] as u16) << 8) + p[0] as u16;
			[
				(v & 0x000F << 4) as u8,
				(v & 0x00F0     ) as u8,
				(v & 0x0F00 >> 4) as u8,
				(v & 0xF000 >> 8) as u8,
			]
		}).collect::<Vec<u8>>()
}

fn convert_to_a4r4g4b4(data: &[u8]) -> Vec<u8> {
	data
		.chunks_exact(4)
		.flat_map(|p| {
			[
				(p[0] >> 4) + p[1],
				(p[2] >> 4) + p[3],
			]
		}).collect::<Vec<u8>>()
}

// ---------------------------------------- //

fn convert_from_a1r5g5b5(data: &[u8]) -> Vec<u8> {
	data
		.chunks_exact(2)
		.flat_map(|p| {
			let v = ((p[1] as u16) << 8) + p[0] as u16;
			[
				(v & 0x001F << 3) as u8,
				(v & 0x03E0 >> 2) as u8,
				(v & 0x7C00 >> 7) as u8,
				(v & 0x8000 >> 8) as u8,
			]
		}).collect::<Vec<u8>>()
}

fn convert_to_a1r5g5b5(data: &[u8]) -> Vec<u8> {
	data
		.chunks_exact(4)
		.flat_map(|p| {
			[
				(p[0] >> 3) + ((p[1] << 2) & 0xE0),
				(p[1] >> 6) + ((p[2] >> 1) & 0x7C) + p[3] & 0x80,
			]
		}).collect::<Vec<u8>>()
}

// ---------------------------------------- //

fn convert_from_x8r8g8b8(data: &[u8]) -> Vec<u8> {
	data
		.chunks_exact(4)
		.flat_map(|p| {
			[p[0], p[1], p[2], 255]
		}).collect::<Vec<u8>>()
}

fn convert_to_x8r8g8b8(data: &[u8]) -> Vec<u8> {
	data
		.chunks_exact(4)
		.flat_map(|p| {
			[p[0], p[1], p[2], 0]
		}).collect::<Vec<u8>>()
}

// ---------------------------------------- //

fn convert_from_compressed(format: SFormat, data: &[u8]) -> Vec<u8> {
	// we just assume width is 4 since it doesnt really matter
	// its all 4x4 block based anyways
	// (if actual width isnt divisible by 4 thats a you issue)
	let width = 4;
	let uncompressed_len = data.len() * format.block_size();
	let height = uncompressed_len / (width * 4);
	let mut output = Vec::with_capacity(uncompressed_len);
	format.decompress(data, width, height, &mut output);
	output
}

fn convert_to_compressed(format: SFormat, data: &[u8]) -> Vec<u8> {
	let width = 4;
	let height = data.len() / (width * 4);
	let mut output = Vec::with_capacity(format.compressed_size(width, height));
	format.compress(data, width, height, squish::Params {
		algorithm: squish::Algorithm::IterativeClusterFit,
		weights: squish::COLOUR_WEIGHTS_UNIFORM,
		weigh_colour_by_alpha: true,
	}, &mut output);
	output
}
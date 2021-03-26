use std::io::{Result, Write};

use core::mem::size_of;

use crate::{BYTES_PER_PIXEL, HEIGHT, WIDTH};

// A simple bmp encoder, should not be used by anything but this project

static DIBHEADER: BmpHeader = BmpHeader::new();
pub static ENCODER: BmpEncoder = BmpEncoder::new();

pub struct BmpEncoder {
    file_header: [u8; 14],
}

impl BmpEncoder {
    pub const fn new() -> Self {
        let byte_offset = size_of::<BmpHeader>() as u32 + 14;

        let file_size = ((WIDTH * HEIGHT * 3) as u32 + byte_offset).to_le_bytes();
        let byte_offset = byte_offset.to_le_bytes();
        let file_header = [
            b'B',
            b'M',
            file_size[0],
            file_size[1],
            file_size[2],
            file_size[3],
            0,
            0,
            0,
            0,
            byte_offset[0],
            byte_offset[1],
            byte_offset[2],
            byte_offset[3],
        ];

        Self { file_header }
    }
    /// Correctly writes a bmp image `pixel_array` to `writer`
    pub fn write_all(&self, writer: &mut impl Write, pixel_array: &[u8]) -> Result<()> {
        writer.write_all(&self.file_header)?;
        DIBHEADER.write_all(writer)?;
        writer.write_all(pixel_array)?;

        Ok(())
    }
}

pub struct BmpHeader {
    // Header size-atleast 40
    bi_size: u32,
    // Image width in pixels
    bi_width: u32,
    /// Image height in pixels
    bi_height: i32,
    // Must be 1
    bi_planes: u16,
    // Bits per pixel (1, 4, 8, 16, 24, or 32)
    bi_bit_count: u16,
    // Compression type (0 = uncompressed)
    bi_compression: u32,
    // Image Size - may be zero for uncompressed images
    bi_size_image: u32,
    // Set to 0
    bi_x_pels_per_meter: u32,
    // Set to 0
    bi_y_pels_per_meter: u32,
    // Set to 0
    bi_clr_used: u32,
    // Set to 0
    bi_clr_important: u32,
}

impl BmpHeader {
    pub const fn new() -> Self {
        Self {
            bi_size: 40,
            bi_width: WIDTH as u32,
            bi_height: -(HEIGHT as i32),
            bi_planes: 1,
            bi_bit_count: (BYTES_PER_PIXEL * 8) as u16,
            bi_compression: 0,
            bi_size_image: (WIDTH * HEIGHT * BYTES_PER_PIXEL) as u32,
            bi_x_pels_per_meter: 0,
            bi_y_pels_per_meter: 0,
            bi_clr_used: 0,
            bi_clr_important: 0,
        }
    }
    /// Correctly writes self to the writer
    fn write_all(&self, writer: &mut impl Write) -> Result<()> {
        writer.write_all(&self.bi_size.to_le_bytes())?;
        writer.write_all(&self.bi_width.to_le_bytes())?;
        writer.write_all(&self.bi_height.to_le_bytes())?;
        writer.write_all(&self.bi_planes.to_le_bytes())?;
        writer.write_all(&self.bi_bit_count.to_le_bytes())?;
        writer.write_all(&self.bi_compression.to_le_bytes())?;
        writer.write_all(&self.bi_size_image.to_le_bytes())?;
        writer.write_all(&self.bi_x_pels_per_meter.to_le_bytes())?;
        writer.write_all(&self.bi_y_pels_per_meter.to_le_bytes())?;
        writer.write_all(&self.bi_clr_used.to_le_bytes())?;
        writer.write_all(&self.bi_clr_important.to_le_bytes())?;

        Ok(())
    }
}

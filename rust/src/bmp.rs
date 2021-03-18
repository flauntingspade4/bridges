use std::io::{Result, Write};

use crate::{HEIGHT, WIDTH};

// A simple bmp encoder, should not be used by anything but this project

const COLOUR_COUNT: usize = 8;

const BITS_PER_PIXEL: u16 = 8;

const REAL_BYTES_PER_PIXEL: usize = 1;

// Blue, green. red, 0
const COLOUR_TABLE: [u8; COLOUR_COUNT * 4] = [
    255, 255, 255, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0,
];

pub struct BmpEncoder<'a> {
    file_header: [u8; 14],
    dib_header: BmpHeader,
    pixel_array: &'a [u8],
}

impl<'a> BmpEncoder<'a> {
    pub const fn new(pixel_array: &'a [u8]) -> Self {
        let byte_offset: u32 =
            core::mem::size_of::<BmpHeader>() as u32 + 14 + COLOUR_COUNT as u32 * 4;

        let file_size: [u8; 4] =
            ((WIDTH * HEIGHT * REAL_BYTES_PER_PIXEL) as u32 + byte_offset).to_le_bytes();

        let byte_offset: [u8; 4] = byte_offset.to_le_bytes();

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
        let dib_header = BmpHeader::new();
        Self {
            file_header,
            dib_header,
            pixel_array,
        }
    }
    pub fn write_all(&mut self, writer: &mut impl Write) -> Result<()> {
        writer.write_all(&self.file_header)?;
        self.dib_header.write_all(writer)?;
        writer.write_all(&COLOUR_TABLE)?;

        for item in self
            .pixel_array
            .chunks(3)
            .map(|c| if c == &[255, 255, 255] { 0 } else { 1 })
        {
            writer.write(&[item])?;
        }

        //writer.write_all(self.pixel_array)?;

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
            bi_bit_count: BITS_PER_PIXEL,
            bi_compression: 0,
            bi_size_image: (WIDTH * HEIGHT * REAL_BYTES_PER_PIXEL) as u32,
            bi_x_pels_per_meter: 0,
            bi_y_pels_per_meter: 0,
            bi_clr_used: COLOUR_COUNT as u32,
            bi_clr_important: 0,
        }
    }
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

use std::collections::HashMap;
use std::io::prelude::*;
use std::sync::Once;
use std::usize;
use std::{fs::File};
use thiserror::Error;
use flate2::read::{ZlibDecoder};

#[derive(Debug, Error)]
pub enum PNGParseError {
    #[error("Invalid file `{0}`.")]
    InvalidFile(&'static str),
    #[error("Parse error: `{0}`.")]
    ParseError(&'static str),
    #[error("End of file error.")]
    EOF,
}

struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

struct Chunk {
    size: u32,
    type_: String,
    data: Vec<u8>,
    crc: u32,
}

#[derive(Default)]
pub struct PNGFile {
    file: Option<File>,
    data: Vec<u8>,
    size: u32,
    width: u32,
    height: u32,
    pallette: Vec<RGB>,
    bit_depth: u8,
    color_type: u8,
    filter_method: u8,
    compression_method: u8,
    interlace_method: u8,
    chunks: HashMap<usize, Chunk>,
}

static mut CRC_TABLE: [u32; 256] = [0; 256];
static CRC_TABLE_INIT: Once = Once::new();

/// Initialize the CRC table for faster computation
fn make_crc_table() {
    // Safety: This is safe because we use Once to ensure single initialization
    // and this is only called in a synchronized context
    unsafe {
        CRC_TABLE_INIT.call_once(|| {
            for n in 0..256 {
                let mut c = n as u32;
                for _ in 0..8 {
                    if c & 1 != 0 {
                        c = 0xedb88320u32 ^ (c >> 1);
                    } else {
                        c = c >> 1;
                    }
                }
                CRC_TABLE[n] = c;
            }
        });
    }
}

/// Update a running CRC with the bytes from the buffer.
/// The CRC should be initialized to all 1's, and the transmitted value
/// is the 1's complement of the final running CRC.
pub fn update_crc(mut crc: u32, buf: &[u8]) -> u32 {
    make_crc_table();

    // Safety: This is safe because we've initialized the table
    // and we're only reading from it
    unsafe {
        for &byte in buf {
            crc = CRC_TABLE[((crc ^ byte as u32) & 0xff) as usize] ^ (crc >> 8);
        }
    }
    crc
}

/// Calculate the CRC for the given buffer
pub fn crc(buf: &[u8]) -> u32 {
    update_crc(0xffffffff, buf) ^ 0xffffffff
}

impl PNGFile {
    pub fn init(filepath: String) -> PNGFile {
        let input_file = File::open(filepath).unwrap();
        let png_file = PNGFile {
            file: Some(input_file),
            ..Default::default()
        };
        png_file
    }

    pub fn parse(self: &mut Self) -> Result<(), PNGParseError> {
        match &mut self.file {
            Some(f) => {
                // verify header
                let mut png_header: [u8; 8] = [0; 8];
                let png_signiture: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
                let _ = f.read(&mut png_header).unwrap_or(0);
                if png_header != png_signiture {
                    return Err(PNGParseError::ParseError(
                        "Invalid png file, wrong signiture.",
                    ));
                }
                let mut data_chunks: Vec<u8> = vec![];
                // reading chuncks
                self.chunks = HashMap::new();
                let mut i: usize = 0;
                while let Ok(chunk) = PNGFile::read_chunk(f) {
                    if chunk.type_ == "IHDR" && i == 0 {
                        let width_buf: [u8; 4] = chunk
                            .data
                            .get(0..4)
                            .unwrap_or(&[0 as u8; 4])
                            .try_into()
                            .unwrap_or([0 as u8; 4]);
                        self.width = u32::from_be_bytes(width_buf);
                        //must verify width else error
                        let height_buf: [u8; 4] = chunk
                            .data
                            .get(4..8)
                            .unwrap_or(&[0 as u8; 4])
                            .try_into()
                            .unwrap_or([0 as u8; 4]);
                        self.height = u32::from_be_bytes(height_buf);
                        //must verify height else error
                        self.bit_depth = *(chunk.data.get(8).unwrap_or(&0));
                        self.color_type = *(chunk.data.get(9).unwrap_or(&0));
                        self.compression_method = *(chunk.data.get(10).unwrap_or(&0));
                        self.filter_method = *(chunk.data.get(11).unwrap_or(&0));
                        self.interlace_method = *(chunk.data.get(12).unwrap_or(&0));
                    } else if chunk.type_ == "IDAT" {
                        let mut data = chunk.data.clone();
                        data_chunks.append(&mut data);
                        assert!(data_chunks.len() > 0);
                    } else if chunk.type_ == "PLTE" {
                        for i in 0..chunk.data.len() {
                            let rgb_bytes = &chunk.data[i..i + 3];
                            let rgb_entry = RGB {
                                r: rgb_bytes[0],
                                g: rgb_bytes[1],
                                b: rgb_bytes[2],
                            };
                            self.pallette.push(rgb_entry);
                        }
                    } else if chunk.type_ == "IEND" {
                        assert!(data_chunks.len() > 0);
                        let mut dec = ZlibDecoder::new(&data_chunks[..]);
                        let mut deflated_data: Vec<u8> = vec![];
                        dec.read_to_end(&mut deflated_data).unwrap();
                        assert!(deflated_data.len() > 0);
                        self.data.append(&mut deflated_data);
                    } else {
                        self.chunks.insert(i, chunk);
                    }
                    i += 1;
                }

                println!("width:{} height:{} bit_depth: {}", self.width, self.height, self.bit_depth);
                println!("data: {}", self.data.len());

                println!("chunks lenght: {}", self.chunks.len());

                for (i, chunk) in self.chunks.iter() {
                    println!(
                        "chunk:\t index: {} size:{} type:{}",
                        i, chunk.size, chunk.type_
                    );
                }

                Ok(())
            }
            None => Ok(()),
        }
    }

    fn read_chunk(file: &mut File) -> Result<Chunk, PNGParseError> {
        let mut chunk_st: [u8; 4] = [0; 4];
        let mut bytes_read = file.read(&mut chunk_st).unwrap();
        //println!("bytes read: {}", bytes_read);
        if bytes_read != chunk_st.len() {
            return Err(PNGParseError::EOF);
        }
        let chunk_size_int = u32::from_be_bytes(chunk_st);
        let mut chunk_type_buf = [0; 4];
        bytes_read = file.read(&mut chunk_type_buf).unwrap();
        //println!("bytes read: {}", bytes_read);
        if bytes_read != chunk_st.len() {
            return Err(PNGParseError::EOF);
        }
        let chunk_type = match std::str::from_utf8(&chunk_type_buf) {
            Ok(s) => s,
            _ => "",
        };
        let mut chunk_data = vec![0; chunk_size_int as usize];
        bytes_read = file.read(&mut chunk_data).unwrap();
        //println!("bytes read: {}", bytes_read);
        if bytes_read != chunk_size_int as usize {
            return Err(PNGParseError::EOF);
        }

        let mut chunk_crc_buf = [0; 4];
        bytes_read = file.read(&mut chunk_crc_buf).unwrap();
        //println!("bytes read: {}", bytes_read);
        if bytes_read != chunk_st.len() {
            return Err(PNGParseError::EOF);
        }
        let chunk_crc = u32::from_be_bytes(chunk_crc_buf);

        let mut crc_buffer: Vec<u8> = Vec::new();
        crc_buffer.append(&mut chunk_type_buf.clone().to_vec());
        crc_buffer.append(&mut chunk_data.clone().to_vec());
        if crc(&crc_buffer) != chunk_crc {
            return Err(PNGParseError::ParseError("Invalid CRC"));
        }
        let chunk = Chunk {
            type_: String::from(chunk_type),
            size: chunk_size_int,
            data: chunk_data,
            crc: chunk_crc,
        };
        Ok(chunk)
    }

}

use clap::Parser;
use thiserror::Error;
use std::collections::HashMap;
use std::io::prelude::*;
use std::usize;
use std::{fs::File, path::Path};

#[derive(Debug, Parser)]
#[command(version, about, long_about=None)]
struct Args {
    #[arg(short, long)]
    input: String,
    #[arg(short, long, default_value_t=String::from("output.png"))]
    output: String
}

#[derive(Debug, Error)]
enum PNGParseError {
    #[error("Invalid file `{0}`.")]
    InvalidFile(&'static str),
    #[error("Parse error: `{0}`.")]
    ParseError(&'static str),
    #[error("End of file error.")]
    EOF
}

struct RGB {
    r: u8,
    g: u8,
    b: u8
}

struct Chunk {
    size: u32,
    type_: String,
    data: Vec<u8>,
    crc: u32
}

#[derive(Default)]
struct PNGFile {
    file: Option<File>,
    data: Vec<Vec<u8>>,
    size: u32,
    width: u32,
    height: u32,
    pallette: Vec<RGB>,
    bit_depth: u8,
    color_type: u8,
    filter_method: u8,
    compression_method: u8,
    interlace_method: u8,
    chunks: HashMap<usize, Chunk>
}

impl PNGFile {
    fn init(filepath: String) -> PNGFile {
        let mut input_file = File::open(filepath).unwrap();
        let png_file = PNGFile{file: Some(input_file), ..Default::default()};
        png_file
    } 

    fn parse(self: &mut Self) -> Result<(), PNGParseError>{
        // verify header
        match &mut self.file {
            Some(f) => {
                let mut png_header: [u8; 8] = [0; 8];
                let png_signiture: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
                let _ = f.read(&mut png_header).unwrap_or(0);
                if png_header != png_signiture {
                    return Err(PNGParseError::ParseError("Invalid png file, wrong signiture."));
                }
                // reading chuncks
                self.chunks = HashMap::new();
                let mut i: usize = 0;
                while let Ok(chunk) = PNGFile::read_chunk(f) {
                    if chunk.type_ == "IHDR" && i==0 {
                        let width_buf: [u8; 4] = chunk.data.get(0..4).unwrap_or(&[0 as u8;4]).try_into().unwrap_or([0 as u8; 4]);
                        self.width = u32::from_be_bytes(width_buf);
                        //must verify width else error
                        let height_buf: [u8; 4] = chunk.data.get(4..8).unwrap_or(&[0 as u8;4]).try_into().unwrap_or([0 as u8; 4]);
                        self.height= u32::from_be_bytes(height_buf);
                        //must verify height else error
                        self.bit_depth = *(chunk.data.get(8).unwrap_or(&0));
                        self.color_type = *(chunk.data.get(9).unwrap_or(&0));
                        self.compression_method = *(chunk.data.get(10).unwrap_or(&0));
                        self.filter_method = *(chunk.data.get(11).unwrap_or(&0));
                        self.interlace_method = *(chunk.data.get(12).unwrap_or(&0));
                    } else if chunk.type_ == "IDAT" {
                        self.data.push(chunk.data);
                    } else {
                        self.chunks.insert(i, chunk);
                    }
                    i +=1;
                }

                println!("width:{} height:{}", self.width, self.height);

                println!("chunks lenght: {}", self.chunks.len());

                for (i, chunk) in self.chunks.iter() {
                    println!("chunk:\t size:{} type:{}", chunk.size, chunk.type_);
                }

                Ok(())
            },
            None => Ok(())
        }
    }

    fn read_chunk(file:&mut File) -> Result<Chunk, PNGParseError> {
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
            Ok(s)=> s,
            _ => ""
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
        let chunk = Chunk {type_:String::from(chunk_type), size : chunk_size_int, data : chunk_data, crc : chunk_crc};
        Ok(chunk)
    }
}

fn main() {
    let args = Args::parse();
    let input_file_path = args.input;
    let file_segs: Vec<&str> = input_file_path.split(".").collect();
    let file_ext = if file_segs.len() >=2 {
        file_segs.get(file_segs.len()-1).unwrap()
    } else {
        ""
    };

    // verify
    if Path::new(&input_file_path).exists() {
        if file_ext!= "png" {
            println!("Unrecognized file format, supported formats are: (png).");
            std::process::exit(1);
        }
    }

    let mut image_file = PNGFile::init(input_file_path);
    let _ = image_file.parse();
}

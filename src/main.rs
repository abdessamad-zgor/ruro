use clap::Parser;
use thiserror::Error;
use std::io::prelude::*;
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
    data: Vec<u8>,
    size: u32,
    width: u32,
    height: u32,
    pallette: Vec<RGB>,
    bit_depth: u8,
    color_type: u8,
    filter_method: u8,
    compression_type: u8,
    interlace_method: u8,
}
//impl Default for PNGFile {
//    fn default() -> Self {
//        PNGFile {
//            width: 1,
//            height: 1,
//            bit_depth: 0,
//            color_type: 0,
//            compression_type: 0,
//            filter_method: 0,
//            interlace_method: 0,
//            pallette: Vec::new(),
//            data: Vec::new(),
//            size: 0
//        }
//    }
//}

impl PNGFile {
    fn parse(filepath: String) -> Result<PNGFile, PNGParseError>{
        // verify header
        let mut input_file = File::open(filepath).unwrap();
        let mut png_header: [u8; 8] = [0; 8];
        let png_signiture: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
        let _ = input_file.read(&mut png_header);
        if png_header != png_signiture {
            return Err(PNGParseError::ParseError("Invalid png file, wrong signiture."));
        }
        // reading chunks
        let mut chunks: Vec<Chunk> = Vec::new();
        let mut chunk_result = PNGFile::read_chunk(input_file);
        loop {
            match chunk_result {
                Ok(chunk) => chunks.push(chunk),
                _ => {break;}
            }
            chunk_result = PNGFile::read_chunk(input_file);
        }

        Ok(PNGFile{file: Some(input_file), ..Default::default()})
    }

    fn read_chunk(&mut file:File) -> Result<Chunk, PNGParseError> {
        let mut chunk_st: [u8; 4] = [0; 4];
        let mut bytes_read = file.read(&mut chunk_st).unwrap();
        if bytes_read != chunk_st.len() {
            return Err(PNGParseError::EOF);
        }
        let chunk_size_int = u32::from_be_bytes(chunk_st);
        let mut chunk_type_buf = [0; 4];
        bytes_read = file.read(&mut chunk_type_buf).unwrap();
        if bytes_read != chunk_st.len() {
            return Err(PNGParseError::EOF);
        }
        let chunk_type = match std::str::from_utf8(&chunk_type_buf) {
            Ok(s)=> s,
            _ => ""
        };
        let mut chunk_data = vec![0; chunk_size_int as usize];
        bytes_read = file.read(&mut chunk_data).unwrap();
        if bytes_read != chunk_st.len() {
            return Err(PNGParseError::EOF);
        }

        let mut chunk_crc_buf = [0; 4];
        bytes_read = file.read(&mut chunk_crc_buf).unwrap();
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

    //let mut chunk_size: [u8; 4] = [0; 4];
    //let _ = input_file.read(&mut chunk_size);
    //let chunk_size_int = u32::from_be_bytes(chunk_size);
    //println!("chunk size: {}", chunk_size_int);
    //if chunk_size_int != 13 {
    //    println!("Invalid header size.");
    //    std::process::exit(1);
    //}

    //let mut chunk_type: [u8; 4] = [0; 4];
    //let _ = input_file.read(&mut chunk_type);
    //let mut chunk_type_str = match std::str::from_utf8(&chunk_type) {
    //    Ok(s) => s,
    //    _ => ""
    //};

    //if chunk_type_str != "IHDR" {
    //    println!("Expected header.");
    //    std::process::exit(1);
    //}

    //let mut image_width_buf: [u8; 4] = [0; 4];
    //let _ = input_file.read(&mut image_width_buf);
    //let image_width = u32::from_be_bytes(image_width_buf);
    //println!("image width: {}", image_width);

    //let mut image_height_buf: [u8; 4] = [0; 4];
    //let _ = input_file.read(&mut image_height_buf);
    //let image_height = u32::from_be_bytes(image_height_buf);
    //println!("image height: {}", image_height);

    //let mut image_bit_depth_buf: [u8; 1] = [0; 1];
    //let _ = input_file.read(&mut image_bit_depth_buf);
    //let image_bit_depth = u8::from_be_bytes(image_bit_depth_buf);
    //println!("image bit depth: {}", image_bit_depth);

    //let mut image_color_type_buf: [u8; 1] = [0; 1];
    //let _ = input_file.read(&mut image_color_type_buf);
    //let image_color_type = u8::from_be_bytes(image_color_type_buf);
    //println!("image color type: {}", image_color_type);

    //let mut image_compression_method_buf: [u8; 1] = [0; 1];
    //let _ = input_file.read(&mut image_compression_method_buf);
    //let image_compression_method = u8::from_be_bytes(image_compression_method_buf);
    //println!("image compression method: {}", image_compression_method);

    //let mut image_filter_method_buf: [u8; 1] = [0; 1];
    //let _ = input_file.read(&mut image_filter_method_buf);
    //let image_filter_method = u8::from_be_bytes(image_filter_method_buf);
    //println!("image filter method: {}", image_filter_method);

    //let mut image_interlace_method_buf: [u8; 1] = [0; 1];
    //let _ = input_file.read(&mut image_interlace_method_buf);
    //let image_interlace_method = u8::from_be_bytes(image_interlace_method_buf);
    //println!("image interlace method: {}", image_interlace_method);

    //chunk_size = [0; 4];
    //let _ = input_file.read(&mut chunk_size);
    //let chunk_crc = u32::from_be_bytes(chunk_size);
    //println!("crc chunk: {}", chunk_crc);

    //chunk_size = [0;4];
    //let _ = input_file.read(&mut chunk_size);
    //let pltdat_size = u32::from_be_bytes(chunk_size);
    //println!("pallete or data size: {}", pltdat_size);

    //chunk_type = [0; 4];
    //let _ = input_file.read(&mut chunk_type);
    //chunk_type_str = match std::str::from_utf8(&chunk_type) {
    //    Ok(s) => s,
    //    _ => ""
    //};

    //println!("header type: {}", chunk_type_str);
}

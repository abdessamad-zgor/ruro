use std::path::Path;

use clap::Parser;

pub mod png;

#[derive(Debug, Parser)]
#[command(version, about, long_about=None)]
struct Args {
    #[arg(short, long)]
    input: String,
    #[arg(short, long, default_value_t=String::from("output.png"))]
    output: String,
}

fn main() {
    let args = Args::parse();
    let input_file_path = args.input;
    let file_segs: Vec<&str> = input_file_path.split(".").collect();
    let file_ext = if file_segs.len() >= 2 {
        file_segs.get(file_segs.len() - 1).unwrap()
    } else {
        ""
    };

    // verify
    if Path::new(&input_file_path).exists() {
        if file_ext != "png" {
            println!("Unrecognized file format, supported formats are: (png).");
            std::process::exit(1);
        }
    }

    let mut image_file = png::PNGFile::init(input_file_path);
    let _ = image_file.parse();
}

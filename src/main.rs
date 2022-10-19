use clap::Parser;

use std::path::Path;
use std::fs::File;
use std::io::BufWriter;

use image;

mod tif;


#[derive(Parser)]
#[command(author="Tom Walters", version, about="Converts an image into MechAssault's texture format (.tif)", long_about = None)]
struct Args {
    /// Source image to convert (png or bmp)
    file: String,

    /// Which game to convert the texture for (ma1 or ma2)
    game: String,

    #[arg(short, long)]
    /// Texture format (dxt1, dxt3, dxt5, etc...)
    format: String
}

fn main() {
    let args = Args::parse();

    assert!(args.game == "ma1" || args.game == "ma2", "Argument 'game' must be either \"ma1\" or \"ma2\"");
    assert!(tif::SUPPORTED_FORMATS.iter().any(|&f| f == args.format), "Texture format must be one of {:?}", tif::SUPPORTED_FORMATS);
    
    let source_path = Path::new(&args.file);
    let source_image = image::open(source_path).expect("File not found");
    
    let tif_filename = source_path.parent().unwrap().join(source_path.file_stem().unwrap()).join(".tif");
    let tif_file = File::create(tif_filename).unwrap();
    let mut tif_writer = BufWriter::new(tif_file);

    if args.game == "ma1" {
        tif::write_tif_file_ma1(&mut tif_writer, &source_image, &args.format);
    }
    else {

    }
}

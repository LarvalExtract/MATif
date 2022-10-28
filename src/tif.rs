use std::fs::File;
use std::io::{BufWriter, Write};
use std::mem::size_of;

use image::GenericImage;
use texpresso::Format;
use texpresso::Algorithm;

pub const SUPPORTED_FORMATS: [&str;8] = [ 
    "argb8888",  
    "rgb565", 
    "argb5551", 
    "argb4444", 
    "a8", 
    "dxt1", 
    "dxt3", 
    "dxt5" 
];

pub fn write_tif_file_ma1(buf: &mut BufWriter<File>, img: &image::DynamicImage, format: &str) {
    let (width , height) = img.dimensions();

    let flags: u32 = match format {
        "argb8888"  => 0x00000000,
        "rgb565"    => 0x00000001,
        "argb4444"  => 0x00000003,
        "l8"        => 0x00000005,
        "la88"      => 0x00000007,
        "dxt1"      => 0x00000100,
        "dxt3"      => 0x00000300,
        "dxt5"      => 0x00000500,
        _           => panic!("Unrecognised format")
    };

    if format == "dxt1" || format == "dxt3" || format == "dxt5" {
        let dxt_fmt: texpresso::Format = match format {
            "dxt1"  => Format::Bc1,
            "dxt3"  => Format::Bc3,
            "dxt5"  => Format::Bc5,
            _       => panic!("Unrecognised DXT format")
        };

        assert!(width % 2 == 0 && height & 2 == 0, "Source image dimensions must be powers of 2 when using dxt1, dxt3, or dxt5");

        let params = texpresso::Params { algorithm: Algorithm::ClusterFit, weights: texpresso::COLOUR_WEIGHTS_PERCEPTUAL, weigh_colour_by_alpha: false };
        let mut dest_pixels = vec![0u8; dxt_fmt.compressed_size(width as usize, height as usize)];

        dxt_fmt.compress(&img.raw_pixels(), width as usize, height as usize, params, &mut dest_pixels);

        buf.write(&dest_pixels);
    }
    else {
        
    }


}

fn write_header_ma1(buf: &mut BufWriter<File>, flags: u32, width: u32, height: u32, mips: u32, size: u32) {

    let total_length: u32 = 0;
    let pic_length: u32 = 0;
    let bits: u32 = 0;

    // Write MGI header
    buf.write(&u32::to_be_bytes(65536));
    buf.write(b"MGIc");
    buf.write(&total_length.to_be_bytes());
    buf.write(&u32::to_be_bytes(1));
    buf.write(b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0");

    // Write PIC chunk
    buf.write(b"PIC ");
    buf.write(b"\0\0\0\0");
    buf.write(&pic_length.to_be_bytes());
    buf.write(&u32::to_be_bytes(9));

    // Write "ver "
    buf.write(b"ver ");
    buf.write(&u32::to_be_bytes(9));
    buf.write(&u8::to_be_bytes(2));

    // Write "flgs"
    buf.write(b"flgs");
    buf.write(&u32::to_be_bytes(12));
    buf.write(&flags.to_be_bytes());

    // Write "wdth"
    buf.write(b"wdth");
    buf.write(&u32::to_be_bytes(12));
    buf.write(&width.to_be_bytes());

    // Write "hgt "
    buf.write(b"hgt ");
    buf.write(&u32::to_be_bytes(12));
    buf.write(&height.to_be_bytes());

    // Write "mips "
    buf.write(b"mips");
    buf.write(&u32::to_be_bytes(12));
    buf.write(&mips.to_be_bytes());

    // Write "size"
    buf.write(b"size");
    buf.write(&u32::to_be_bytes(12));
    buf.write(&size.to_be_bytes());

    // Write "bits"
    buf.write(b"bits");
    buf.write(&bits.to_be_bytes());
}

fn get_bytes_per_pixel(fmt: &str) -> usize {
    match fmt {
        "argb8888"  => size_of::<u32>(),
        "rgb565"    => size_of::<u16>(),
        "argb4444"  => size_of::<u16>(),
        "l8"        => size_of::<u8>(),
        "la88"      => size_of::<u16>(),
        _           => size_of::<u32>()
    }
}

fn pad_width(width: u32) -> u32 {
    if width % 16 == 0 { width } else { width + (16 - width % 16) }
}

fn pixel_to_argb8888(src_pixel: image::Rgba<u8>) -> u32 {
    let r = src_pixel.data[0];
    let g = src_pixel.data[1];
    let b = src_pixel.data[2];
    let a = src_pixel.data[3];

    (a << 24) as u32 |
    (r << 16) as u32 |
    (g << 8) as u32 |
    b as u32
}

fn pixel_to_argb4444(src_pixel: image::Rgba<u8>) -> u16 {
    let r = src_pixel.data[0];
    let g = src_pixel.data[1];
    let b = src_pixel.data[2];
    let a = src_pixel.data[3];

    ((a >> 4) << 12) as u16 |
    ((r >> 4) << 8) as u16 |
    ((g >> 4) << 4) as u16 |
    (b >> 4) as u16
}

fn pixel_to_rgb565(src_pixel: image::Rgba<u8>) -> u16 {
    let r = src_pixel.data[0];
    let g = src_pixel.data[1];
    let b = src_pixel.data[2];
    let a = src_pixel.data[3];

   // ((a >> 7) << 12) as u16 |
    ((r >> 3) << 11) as u16 |
    ((g >> 2) << 6) as u16 |
    (b >> 3) as u16
}

fn pixel_to_l8(src_pixel: image::Luma<u8>) -> u8 {
    src_pixel.data[0]
}

fn pixel_to_la88(src_pixel: image::LumaA<u8>) -> u16 {
    let l = src_pixel.data[0];
    let a = src_pixel.data[1];

    (l << 8) as u16 |
    a as u16
}
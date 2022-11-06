use std::fs::File;
use std::io::{BufWriter, Write};
use std::mem::size_of;

use image::GenericImage;
use texpresso::Format;
use texpresso::Algorithm;

pub const SUPPORTED_FORMATS: [&str;8] = [
    "argb8888",
    "rgb565",  
    "argb4444",
    "l8",      
    "la88",    
    "dxt1",    
    "dxt3",    
    "dxt5"    
];

const FORMAT_CODE_ARGB8888: u32 = 0x00000000;
const FORMAT_CODE_RGB565: u32   = 0x00000001;
const FORMAT_CODE_ARGB4444: u32 = 0x00000003;
const FORMAT_CODE_L8: u32       = 0x00000005;
const FORMAT_CODE_LA88: u32     = 0x00000007;
const FORMAT_CODE_DXT1: u32     = 0x00000100;
const FORMAT_CODE_DXT3: u32     = 0x00000300;
const FORMAT_CODE_DXT5: u32     = 0x00000500;
pub const FORMAT_CODE_INVALID: u32  = 0xFFFFFFFF;

pub fn get_format_code(fmt: &str) -> u32 {
    match fmt {
        "argb8888" => FORMAT_CODE_ARGB8888,
        "rgb565"   => FORMAT_CODE_RGB565,
        "argb4444" => FORMAT_CODE_ARGB4444,
        "l8"       => FORMAT_CODE_L8,
        "la88"     => FORMAT_CODE_LA88,
        "dxt1"     => FORMAT_CODE_DXT1,
        "dxt3"     => FORMAT_CODE_DXT3,
        "dxt5"     => FORMAT_CODE_DXT5,
        _          => FORMAT_CODE_INVALID
    }
}

pub fn write_tif_file(buf: &mut BufWriter<File>, img: &image::DynamicImage, format: &str, game: &str) {
    let (mut width, mut height) = img.dimensions();
    let flags = if game == "ma2" { 0x80000000 } else { 0x00000000 } | get_format_code(format);

    let is_dxt = flags & FORMAT_CODE_DXT1 > 0 || flags & FORMAT_CODE_DXT3 > 0 || flags & FORMAT_CODE_DXT5 > 0;
    if is_dxt {
        assert!(width % 2 == 0 && height & 2 == 0, "Source image dimensions must be powers of 2 when using dxt1, dxt3, or dxt5");

        let dxt_fmt: texpresso::Format = match flags & 0x7FFFFFFF {
            FORMAT_CODE_DXT1  => Format::Bc1,
            FORMAT_CODE_DXT3  => Format::Bc3,
            FORMAT_CODE_DXT5  => Format::Bc5,
            _ => todo!()
        };
        
        let params = texpresso::Params { 
            algorithm: Algorithm::ClusterFit, 
            weights: texpresso::COLOUR_WEIGHTS_UNIFORM, 
            weigh_colour_by_alpha: false 
        };
        
        let mip_levels = ((core::cmp::min(width, height) - 1) >> 1).count_ones();
        let total_size = compute_image_size(width, height, flags, mip_levels);
        let mut pixels: Vec<u8> = Vec::<u8>::new();

        write_header(buf, flags, width, height, mip_levels, total_size, total_size, game == "ma2");

        for mip_level in 0..mip_levels {
            pixels.resize(dxt_fmt.compressed_size(width as usize, height as usize), 0);
            
            println!("Writing mip level {} ({} x {}, {} bytes)", mip_level, width, height, pixels.len());
            
            dxt_fmt.compress(
                &img.resize_exact(width, height, image::FilterType::Triangle).raw_pixels(),
                width as usize, 
                height as usize, 
                params, 
                &mut pixels
            );

            width >>= 1;
            height >>= 1;

            buf.write_all(&pixels).unwrap();
        }

    }
    else {
        let padded_width = pad_width(width);
        let bpp = get_bytes_per_pixel(flags);
        let total_size = padded_width * height * bpp as u32;

        write_header(buf, flags, width, height, 1, total_size, total_size, game == "ma2");

        for y in 0..height {
            for x in 0..padded_width {
                if x < width {
                    match flags & 0x7FFFFFFF {
                        FORMAT_CODE_ARGB8888 => buf.write(&pixel_to_argb8888(img.get_pixel(x, y)).to_le_bytes()).unwrap(),
                        FORMAT_CODE_RGB565   => buf.write(&pixel_to_rgb565(img.get_pixel(x, y)).to_le_bytes()).unwrap(),
                        FORMAT_CODE_ARGB4444 => buf.write(&pixel_to_argb4444(img.get_pixel(x, y)).to_le_bytes()).unwrap(),
                        FORMAT_CODE_L8       => buf.write(&pixel_to_l8(img.get_pixel(x, y)).to_le_bytes()).unwrap(),
                        FORMAT_CODE_LA88     => buf.write(&pixel_to_la88(img.get_pixel(x, y)).to_le_bytes()).unwrap(),
                        _ => todo!()
                    };
                } else {
                    match bpp {
                        1 => buf.write(b"\0").unwrap(),
                        2 => buf.write(b"\0\0").unwrap(),
                        4 => buf.write(b"\0\0\0\0").unwrap(),
                        _ => todo!()
                    };
                }
            }
        }
    }
}

fn write_header(buf: &mut BufWriter<File>, flags: u32, width: u32, height: u32, mips: u32, size: u32, actual_size: u32, is_ma2: bool) {

    let bits = actual_size + 8;
    let mut pic_length = 16 + 9 + (12 * 5) + bits;
    if is_ma2 {
        pic_length += 12 * 2;
    }
    let total_length = pic_length + 32;

    // Write MGI header
    buf.write(&u32::to_le_bytes(65536)).unwrap();
    buf.write(b"MGIc").unwrap();
    buf.write(&total_length.to_le_bytes()).unwrap();
    buf.write(&u32::to_le_bytes(1)).unwrap();
    buf.write(b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0").unwrap();

    // Write PIC chunk
    buf.write(b"PIC ").unwrap();
    buf.write(b"\0\0\0\0").unwrap();
    buf.write(&pic_length.to_le_bytes()).unwrap();
    buf.write(&u32::to_le_bytes(9)).unwrap();

    // Write "ver "
    buf.write(b"ver ").unwrap();
    buf.write(&u32::to_le_bytes(9)).unwrap();
    buf.write(&u8::to_le_bytes(if is_ma2 { 4u8 } else { 2u8 })).unwrap();

    // Write "flgs"
    buf.write(b"flgs").unwrap();
    buf.write(&u32::to_le_bytes(12)).unwrap();
    buf.write(&flags.to_le_bytes()).unwrap();

    // Write "wdth"
    buf.write(b"wdth").unwrap();
    buf.write(&u32::to_le_bytes(12)).unwrap();
    buf.write(&width.to_le_bytes()).unwrap();

    // Write "hgt "
    buf.write(b"hgt ").unwrap();
    buf.write(&u32::to_le_bytes(12)).unwrap();
    buf.write(&height.to_le_bytes()).unwrap();

    // Write "mips "
    buf.write(b"mips").unwrap();
    buf.write(&u32::to_le_bytes(12)).unwrap();
    buf.write(&mips.to_le_bytes()).unwrap();

    // Write "size"
    buf.write(b"size").unwrap();
    buf.write(&u32::to_le_bytes(12)).unwrap();
    buf.write(&size.to_le_bytes()).unwrap();

    if is_ma2 {
        buf.write(b"frms").unwrap();
        buf.write(&u32::to_le_bytes(12)).unwrap();
        buf.write(&u32::to_le_bytes(1)).unwrap();

        buf.write(b"dpth").unwrap();
        buf.write(&u32::to_le_bytes(12)).unwrap();
        buf.write(&u32::to_le_bytes(1)).unwrap();
    }

    // Write "bits"
    buf.write(b"bits").unwrap();
    buf.write(&bits.to_le_bytes()).unwrap();
}

fn get_bytes_per_pixel(fmt_code: u32) -> usize {
    match fmt_code & 0x7FFFFFFF {
        FORMAT_CODE_ARGB8888 => size_of::<u32>(),
        FORMAT_CODE_RGB565   => size_of::<u16>(),
        FORMAT_CODE_ARGB4444 => size_of::<u16>(),
        FORMAT_CODE_L8       => size_of::<u8>(),
        FORMAT_CODE_LA88     => size_of::<u16>(),
        _                         => 1
    }
}

fn pad_width(width: u32) -> u32 {
    if width % 16 == 0 { width } else { width + (16 - width % 16) }
}

fn compute_image_size(mut width: u32, mut height: u32, format: u32, mips: u32) -> u32 {
    let mut size: u32 = 0;

    for _ in 0..mips {
        size += match format & 0x7FFFFFFF {
            FORMAT_CODE_DXT1 => texpresso::Format::compressed_size(texpresso::Format::Bc1, width as usize, height as usize),
            FORMAT_CODE_DXT3 => texpresso::Format::compressed_size(texpresso::Format::Bc3, width as usize, height as usize),
            FORMAT_CODE_DXT5 => texpresso::Format::compressed_size(texpresso::Format::Bc5, width as usize, height as usize),
            _ => get_bytes_per_pixel(format) * pad_width(width) as usize * height as usize
        } as u32;
        
        width >>= 1;
        height >>= 1;
    }

    return size;
}

fn pixel_to_argb8888(src_pixel: image::Rgba<u8>) -> u32 {
    let r = src_pixel.data[0] as u32;
    let g = src_pixel.data[1] as u32;
    let b = src_pixel.data[2] as u32;
    let a = src_pixel.data[3] as u32;

    a << 24 | 
    r << 16 | 
    g << 8  | 
    b
}

fn pixel_to_argb4444(src_pixel: image::Rgba<u8>) -> u16 {
    let r = src_pixel.data[0] as u16;
    let g = src_pixel.data[1] as u16;
    let b = src_pixel.data[2] as u16;
    let a = src_pixel.data[3] as u16;

    (a >> 4) << 12 |
    (r >> 4) << 8  |
    (g >> 4) << 4  |
    (b >> 4) 
}

fn pixel_to_rgb565(src_pixel: image::Rgba<u8>) -> u16 {
    let r = src_pixel.data[0] as u16;
    let g = src_pixel.data[1] as u16;
    let b = src_pixel.data[2] as u16;
    //let a = src_pixel.data[3];

   // ((a >> 7) << 12) as u16 |
    (r >> 3) << 11 |
    (g >> 2) << 5  |
    (b >> 3) 
}

fn pixel_to_l8(src_pixel: image::Rgba<u8>) -> u8 {
    src_pixel.data[0]
}

fn pixel_to_la88(src_pixel: image::Rgba<u8>) -> u16 {
    let l = src_pixel.data[0] as u16;
    let a = src_pixel.data[1] as u16;

    l << 8 |
    a 
}
use std::fs::File;
use std::io::{BufWriter, Write};

use image::GenericImage;
use texpresso::Format;

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

pub fn write_tif_file_ma1(buf: &BufWriter<File>, img: &image::DynamicImage, format: &str) {
    let (width, height) = img.dimensions();

    let mut flags: u32 = 0;

    let dxt_fmt: texpresso::Format;
    let mut is_dxt = false;

    if format == "argb8888" {

    }
    else if format == "rgb565" {
        flags |= 0x00000001;
    }
    else if format == "argb4444" {
        flags |= 0x00000003;
    }
    else if format == "dxt1" {
        assert!(width % 2 == 0 && height % 2 == 0);
        flags |= 0x00000100;
        dxt_fmt = Format::Bc1;
        is_dxt = true;
    } 
    else if format == "dxt3" {
        assert!(width % 2 == 0 && height % 2 == 0);
        flags |= 0x00000300;
        dxt_fmt = Format::Bc3;
        is_dxt = true;
    }
    else if format == "dxt5" {
        assert!(width % 2 == 0 && height % 2 == 0);
        flags |= 0x00000500;
        dxt_fmt = Format::Bc5;
        is_dxt = true;
    }

    if is_dxt {

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
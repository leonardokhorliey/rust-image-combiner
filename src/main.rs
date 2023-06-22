mod args;
use std::{io::{BufReader}, fs::File};

use args::Args;
use image::{DynamicImage, ImageFormat, io::Reader, GenericImageView, imageops::FilterType::Triangle, save_buffer_with_format, ImageError};

#[derive(Debug)]
enum ImageErrors {
    DifferentImageFormats,
    BufferTooSmall,
    UnableToSaveImage(ImageError),
    UnableToReadFile(String),
    UnableToReadFormat,
    UnableToDecodeImage(String)
}

struct FloatingImage {
    width: u32,
    height: u32,
    data: Vec<u8>,
    name: String
}

impl FloatingImage {
    fn new(width: u32, height: u32, name: String) -> Self {

        Self {
            width,
            height,
            data: Vec::with_capacity((width * height * 4).try_into().unwrap()),
            name
        }
    }

    fn set_data(&mut self, new_data: Vec<u8>) -> Result<(), ImageErrors> {
        match new_data.len() > self.data.capacity() {
            true => Err(ImageErrors::BufferTooSmall),
            false => {
                self.data = new_data;
                Ok(())
            }
        }
    }
}

fn main() -> Result<(), ImageErrors> {
    let args = Args::new();
    let (image_1, image_format_1) = find_image_from_path(args.image_1)?;
    let (image_2, image_format_2) = find_image_from_path(args.image_2)?;

    match image_format_1 == image_format_2 {
        true => Err(ImageErrors::DifferentImageFormats),
        false => {
            let (image_1, image_2) = standardise_size(image_1, image_2);
            let mut output = FloatingImage::new(image_1.width(), image_1.height(), args.output);

            output.set_data(combine_images(image_1, image_2))?;

            match save_buffer_with_format(output.name
                , &output.data
                , output.width
                , output.height
                , image::ColorType::Rgba8
                , image_format_2
            ) {
                Ok(_) => {},
                Err(e) => return Err(ImageErrors::UnableToSaveImage(e))
            }
            Ok(())
        }
    }

    
    
    // println!("Hello, world!");
}

fn find_image_from_path(path: String) -> Result<(DynamicImage, ImageFormat), ImageErrors> {

    let image_reader: Reader<BufReader<File>> = Reader::open(path.clone()).unwrap();
    let image_format: ImageFormat = image_reader.format().unwrap();
    let image: DynamicImage = image_reader.decode().unwrap();

    Ok((image, image_format))
}

fn get_smallest_dimension(dim_1: (u32, u32), dim_2: (u32, u32)) -> (u32, u32) {
    let pix_1 = dim_1.0 * dim_1.1;
    let pix_2 = dim_2.0 * dim_2.1;
    match pix_1 > pix_2 {
        true => dim_2,
        false => dim_1
    }
}

fn standardise_size(image_1: DynamicImage, image_2: DynamicImage) -> (DynamicImage, DynamicImage) {
    let (width, height) = get_smallest_dimension(image_1.dimensions(), image_2.dimensions());

    match image_1.dimensions() == (width, height) {
        true => (image_1, image_2.resize_exact(width, height, Triangle)),
        false => (image_1.resize_exact(width, height, Triangle), image_2),
    }
}


fn combine_images(image_1: DynamicImage, image_2: DynamicImage) -> Vec<u8> {

    let vec_1 = image_1.to_rgba8().into_vec();
    let vec_2 = image_2.to_rgba8().into_vec();

    // alternating the pixels
    let data_length = vec_1.len();
    let mut combined_image_data = vec![1u8; vec_1.len()];

    let mut i = 0;
    loop {
        match i % 8 {
            0 => combined_image_data.splice(i..=i + 3, get_inserting_rgba(&vec_1, i, i + 3)),
            _ =>  combined_image_data.splice(i..=i + 3, get_inserting_rgba(&vec_2, i, i + 3)),
        };

        i += 4;
        if i >= data_length { break;}
        
    }

    combined_image_data
}

fn get_inserting_rgba(vec: &Vec<u8>, start: usize, end: usize) -> Vec<u8> {
    let mut rgba = Vec::new();

    for i in start..=end {
        let val = match vec.get(i) {
            Some(d) => *d,
            None => 0
        };

        rgba.push(val);
    }
    rgba
}

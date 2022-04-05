use anyhow::Result;
use gzp::{deflate::Gzip, Compression, ZBuilder};
use rayon::prelude::*;
use std::io::prelude::*;
use image::Rgba;

type R = Rgba<u8>;

#[repr(u8)]
enum Colors {
    Burdgundy(R),
    DarkRed(R),
    Red(R),
    Orange(R),
    Yellow(R),
    PaleYellow(R),
    DarkGreen(R),
    Green(R),
    LightGreen(R),
    DarkTeal(R),
    Teal(R),
    LightTeal(R),
    DarkBlue(R),
    Blue(R),
}

fn main() -> Result<()> {
    let mut args = Vec::new();
    for arg in std::env::args().skip(1) {
        args.push(arg.clone());
    }
    let mut paths = Vec::new();
    for arg in &args {
        paths.push(std::path::Path::new(arg));
    }
    paths.sort();

    eprintln!("starting processing...");
    let mut encoder = ZBuilder::<Gzip, _>::new()
        .compression_level(Compression::best())
        .num_threads(0)
        .from_writer(std::fs::File::create("data.bin")?);
    for chunk in paths.par_iter().chunks(500).collect::<Vec<_>>() {
        eprintln!("processing...");
        let data: Vec<u8> = chunk
            .par_iter()
            .map(|path| {
                eprintln!("{}", path.display());
                image::open(path)
            })
            .filter_map(Result::ok)
            .map(|image| {
                image
                    .to_rgba8()
                    .enumerate_pixels()
                    .map(|(_, _, pixel)| pixel.0)
                    .flatten()
                    .collect::<Vec<u8>>()
            })
            .flatten()
            .collect();

        eprintln!("writing...");
        encoder.write(&data)?;
    }

    encoder.finish()?;

    eprintln!("done.");
    Ok(())
}

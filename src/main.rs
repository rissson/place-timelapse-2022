use anyhow::anyhow;
use anyhow::Result;
use parking_lot::Mutex;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

fn main() -> Result<()> {
    let mut args = Vec::new();
    for arg in std::env::args().skip(1) {
        args.push(arg.clone());
    }
    let mut paths = Vec::new();
    for arg in &args {
        paths.push(std::path::Path::new(arg));
    }

    let db: Arc<Mutex<HashMap<i64, [Option<_>; 4]>>> =
        Arc::new(Mutex::new(HashMap::new()));

    eprintln!("importing...");
    paths
        .par_iter()
        .try_for_each(|path| -> Result<()> {
            let filename =
                match path.file_name().unwrap().to_os_string().into_string() {
                    Ok(filename) => filename,
                    Err(_) => return Err(anyhow!("unrecognized file name")),
                };
            eprintln!("{}", &filename);
            let mut split = filename.split('-');
            let canvas_id: usize = split.next().unwrap().parse()?;
            if canvas_id > 3 {
                return Err(anyhow!("bollocks"));
            }
            let timestamp: i64 =
                split.next().unwrap().split('.').next().unwrap().parse()?;

            let image = image::open(path)?;

            if db.lock().contains_key(&timestamp) {
                db.lock().get_mut(&timestamp).unwrap()[canvas_id] =
                    Some(image);
            } else {
                let mut canvases = [None, None, None, None];
                canvases[canvas_id] = Some(image);
                db.lock().insert(timestamp, canvases);
            }
            Ok(())
        })?;

    eprintln!("processing...");
    db.lock().par_iter().try_for_each(|(timestamp, canvases)| -> Result<()> {
        let mut image = image::RgbaImage::new(2000, 2000);
        for i in 0..4 {
            let offset = match i {
                0 => (0, 0),
                1 => (1000, 0),
                2 => (0, 1000),
                3 => (1000, 1000),
                _ => (100_000, 100_000),
            };
            if let Some(canvas) = &canvases[i] {
                image::imageops::replace(&mut image, canvas, offset.0, offset.1);
            }
        }

        image.save(format!("{timestamp}.png"))?;

        Ok(())
    })?;

    eprintln!("done.");
    Ok(())
}

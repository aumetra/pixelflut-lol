use image::{GenericImageView, Pixel};
use itertools::Itertools;
use postcard::ser_flavors::io::WriteFlavor;
use serde::{Serializer, ser::SerializeSeq};
use std::{
    borrow::Cow,
    fs::{self, File},
    io::{BufWriter, Write},
    path::PathBuf,
};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(argh::FromArgs)]
/// process a directory of frames to be used by riptide
struct Args {
    #[argh(positional)]
    /// path to the directory with image files
    path: PathBuf,

    #[argh(option)]
    /// path to output file
    output: PathBuf,
}

fn read_frame(entry_path: PathBuf) -> anyhow::Result<riptide_common::Frame<'static>> {
    let image = image::open(entry_path)?;

    let mut frame_data = Vec::new();
    for (_y, x_lane) in &image.pixels().chunk_by(|(_x, y, _pixel)| *y) {
        let mut x_acc = Vec::new();
        for (_x, _y, pixel) in x_lane {
            let [r, g, b] = pixel.to_rgb().0;
            let hex_repr = format!("{r:02x}{g:02x}{b:02x}");

            x_acc.push(riptide_common::Pixel {
                r,
                g,
                b,
                hex_repr: Cow::Owned(hex_repr.into_bytes()),
            });
        }

        frame_data.push(Cow::Owned(x_acc));
    }

    Ok(riptide_common::Frame {
        data: Cow::Owned(frame_data),
    })
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args: Args = argh::from_env();

    let files = fs::read_dir(args.path)?.collect_vec();

    let file = File::create(args.output)?;
    let mut file = BufWriter::new(file);

    let mut ser = postcard::Serializer {
        output: WriteFlavor::new(&mut file),
    };

    let mut seq_ser = ser.serialize_seq(Some(files.len()))?;
    for (idx, entry) in files.into_iter().enumerate() {
        let entry = entry?;
        println!("serializing frame {idx}");

        let frame = read_frame(entry.path())?;
        seq_ser.serialize_element(&frame)?;
    }
    seq_ser.end()?;

    file.flush()?;

    Ok(())
}

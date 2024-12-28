use image::{GenericImageView, Pixel};
use itertools::Itertools;
use rkyv::{
    Archive, Serialize,
    api::serialize_using,
    rancor::Strategy,
    ser::{allocator::Arena, sharing::Share, writer::IoWriter},
    vec::{ArchivedVec, VecResolver},
    with::{ArchiveWith, SerializeWith},
};
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
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

fn read_frame(entry_path: &Path) -> anyhow::Result<riptide_common::Frame> {
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
                hex_repr: hex_repr.into_bytes(),
            });
        }

        frame_data.push(x_acc);
    }

    Ok(riptide_common::Frame { data: frame_data })
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args: Args = argh::from_env();

    let files: Vec<PathBuf> = fs::read_dir(args.path)?
        .map_ok(|entry| entry.path())
        .try_collect()?;

    let file = File::create(args.output)?;
    let mut file = BufWriter::new(file);

    let mut arena = Arena::new();
    let mut serializer = rkyv::ser::Serializer {
        writer: IoWriter::new(&mut file),
        allocator: arena.acquire(),
        sharing: Share::new(),
    };
    let serializer = Strategy::<_, rkyv::rancor::Error>::wrap(&mut serializer);

    struct FileSerializer;

    impl ArchiveWith<Vec<PathBuf>> for FileSerializer {
        type Archived = ArchivedVec<riptide_common::Frame>;
        type Resolver = VecResolver;

        fn resolve_with(
            field: &Vec<PathBuf>,
            resolver: Self::Resolver,
            out: rkyv::Place<Self::Archived>,
        ) {
            ArchivedVec::resolve_from_len(field.len(), resolver, out)
        }
    }

    impl<S> SerializeWith<Vec<PathBuf>, S> for FileSerializer
    where
        S: rkyv::rancor::Fallible,
        S: rkyv::ser::Writer<S::Error> + rkyv::ser::Allocator,
    {
        fn serialize_with(
            field: &Vec<PathBuf>,
            serializer: &mut S,
        ) -> Result<Self::Resolver, <S as rkyv::rancor::Fallible>::Error> {
            ArchivedVec::serialize_from_iter(
                field.iter().map(|entry| read_frame(entry).unwrap()),
                serializer,
            )
        }
    }

    #[derive(Archive, Serialize)]
    #[repr(transparent)]
    struct ArchiveThing(#[rkyv(with = FileSerializer)] Vec<PathBuf>);
    serialize_using(&ArchiveThing(files), serializer)?;

    file.flush()?;

    Ok(())
}

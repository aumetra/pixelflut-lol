#[macro_use]
extern crate tracing;

use monoio::{
    io::{AsyncWriteRent, AsyncWriteRentExt},
    net::TcpStream,
};
use std::{
    fs::File,
    mem,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    str,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

const WHERE_TO: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(151, 217, 2, 166)), 1337);

macro_rules! attempt {
    ($io:expr) => {{
        let (result, buf) = { $io };
        result?;
        buf
    }};
}

#[inline]
fn encode_dec(
    num: impl lexical::ToLexicalWithOptions<Options = lexical::WriteIntegerOptions>,
) -> Vec<u8> {
    static DEC_FMT: u128 = lexical::NumberFormatBuilder::decimal();
    static OPTS: lexical::WriteIntegerOptions = lexical::WriteIntegerOptions::new();

    lexical::to_string_with_options::<_, DEC_FMT>(num, &OPTS).into_bytes()
}

async fn release_the_kraken(
    conn: &mut TcpStream,
    frame: &riptide_common::Frame<'_>,
) -> anyhow::Result<()> {
    info!("sending frame..");
    for (y_pos, y_lane) in frame.data.iter().enumerate() {
        for (x_pos, pixel) in y_lane.iter().enumerate() {
            attempt!(conn.write_all(b"PX ").await);

            let x_str = encode_dec(x_pos);
            attempt!(conn.write_all(x_str).await);
            attempt!(conn.write_all(b" ").await);

            let y_str = encode_dec(y_pos);
            attempt!(conn.write_all(y_str).await);
            attempt!(conn.write_all(b" ").await);

            // encode pixel as hex
            attempt!(conn.write_all(pixel.hex_repr.to_vec()).await);
            attempt!(conn.write_all(b"\n").await);
        }
    }

    conn.flush().await?;

    Ok(())
}

async fn build_conn_pool(addr: SocketAddr, num_conn: usize) -> anyhow::Result<Vec<TcpStream>> {
    let mut conn = Vec::new();
    for idx in 0..num_conn {
        info!("building conn {idx}");
        let stream = TcpStream::connect(addr).await?;
        stream.set_nodelay(true)?;
        conn.push(stream);
    }

    Ok(conn)
}

#[derive(argh::FromArgs)]
/// Pixelflut client to play cool videos :P
struct Args {
    #[argh(option)]
    /// framerate of the video
    framerate: f32,

    #[argh(option, default = "50")]
    /// amount of parallel connections open to flood
    num_conn: usize,

    #[argh(option, default = "WHERE_TO")]
    /// address of the pixelflut server
    addr: SocketAddr,

    #[argh(option)]
    /// file containing the frame data
    data: PathBuf,
}

#[monoio::main(enable_timer = true)]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args: Args = argh::from_env();

    info!("loading data..");
    let data_file = File::open(args.data)?;
    let data = unsafe { memmap2::Mmap::map(&data_file)? };
    let frames: Vec<riptide_common::FrameRef<'_>> = postcard::from_bytes(&data)?;
    let frames = unsafe {
        mem::transmute::<
            &[riptide_common::FrameRef<'_>],
            &'static [riptide_common::FrameRef<'static>],
        >(&frames)
    };

    info!("building conn pool");
    let pool = build_conn_pool(args.addr, args.num_conn).await?;

    let sleep_duration = Duration::from_secs_f32(1.0 / args.framerate);
    let mut frame_ctr = 0;
    let current_frame = Arc::new(AtomicUsize::new(&frames[frame_ctr] as *const _ as usize));

    info!("spawning streams");
    for mut stream in pool {
        let current_frame = Arc::clone(&current_frame);
        
        monoio::time::sleep(Duration::from_millis(2)).await;
        
        monoio::spawn(async move {
            loop {
                let frame = current_frame.load(Ordering::Acquire);
                let frame = unsafe { &*(frame as *const _) };

                if let Err(error) = release_the_kraken(&mut stream, frame).await {
                    error!(?error, "sending failed :((");
                }
            }
        });
    }

    loop {
        frame_ctr += 1;
        frame_ctr %= frames.len();

        monoio::time::sleep(sleep_duration).await;

        current_frame.store(&frames[frame_ctr] as *const _ as usize, Ordering::Release);
    }
}

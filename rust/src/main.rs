#![warn(clippy::pedantic, clippy::nursery)]

use std::{
    convert::TryInto,
    io::{BufRead, BufReader, BufWriter, Read, Seek, Write},
    num::TryFromIntError,
    time::Instant,
};

mod bmp;
mod setup;

use setup::{get_fps_counts_displacment, setup_ffmpeg, Config};

use bmp::ENCODER;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;
const BYTES_PER_PIXEL: usize = 3;

/// The file that'll be written to with the video
const OUTPUT_FILE: &str = "../output/output.avi";
/// The default input file-can be rewritten by the first argument passed at the
/// command line
const DEFAULT_INPUT_FILE: &str = "../input/input.csv";

/// The pixel array that's written to, and actually used for images
static mut PIXEL_ARRAY: &mut [u8] = &mut [255; WIDTH * HEIGHT * BYTES_PER_PIXEL];

/// A pixel array that's written to once at the beginning of the program,
/// and countains the axis drawn the first time
static mut DRAWN_AXIS: &mut [u8] = &mut [255; WIDTH * HEIGHT * BYTES_PER_PIXEL];

use plotters::{
    prelude::{BitMapBackend, ChartBuilder, Circle, IntoDrawingArea},
    style::{Color, RED},
};

/// Sets up the chart-must be identical for both when the axis
/// are being drawn initially, and when the points are being drawn.
/// This is a macro instead of a function to avoid generics messing
/// it up, were plotters to change-only breaking changes could affect
/// this macro
#[macro_export]
macro_rules! setup_chart {
    ($chart:ident, $y_min:ident, $y_max:ident) => {{
        $chart
            .y_label_area_size(50)
            .x_label_area_size(25)
            .build_cartesian_2d(-0.1..0.1, $y_min..$y_max)
            .unwrap()
    }};
}

fn main() {
    start();
}

/// Returns the amount of time it takes to encode, and to draw
fn draw_image_to_writer(
    rdr: BufReader<impl Read>,
    mut writer: BufWriter<impl Write>,
    config: Config,
) -> Result<(u128, u128), TryFromIntError> {
    // FFMPEG doesn't automatically remove the target destination
    if std::fs::remove_file(OUTPUT_FILE).is_ok() {
        println!("Removed \"{}\"", OUTPUT_FILE)
    }

    let Config {
        point_count,
        span_count,
        displacements,
        y_max,
        y_min,
        ..
    } = config;

    let mut enc_time = 0;
    let mut draw_time = 0;

    // Skip the first line, which is just headers
    for points in rdr.lines().skip(1) {
        let points = points.expect("The input file should contain only vaild UTF-8");

        // Skip the first point, which is the time, and unimportant
        let points = points.split(',').skip(1);

        let start_draw = Instant::now();

        // Set up the window we're drawing to
        let window = BitMapBackend::with_buffer(
            unsafe { &mut PIXEL_ARRAY },
            (WIDTH.try_into()?, HEIGHT.try_into()?),
        )
        .into_drawing_area();

        // Split the window into the correct amount of drawing areas
        let windows = window.split_evenly((3, point_count / 3));

        
        let mut window_index = 0;
        // A `first` variable must exist, as 0 % 15 == 0, but we
        // don't want to take 10 off the window index at the beginning
        let mut first = true;

        // Draw all sthe circles on their respective windows
        for (done, point) in points.enumerate() {
            // Some basic maths done here, so the point is drawn to the correct
            // window-this assumes the same ordering as the dataset initially
            // given to us, and if different, will have to be adjusted
            if done % 15 == 0 && !first {
                window_index -= 10 * span_count;
            } else if done % 5 == 0 && !first {
                window_index += 5 * (span_count - 1);
            }

            let mut chart = ChartBuilder::on(&windows[window_index]);

            let chart = setup_chart!(chart, y_min, y_max);

            let point = match point.trim().parse::<f64>() {
                Ok(t) => t,
                Err(e) => panic!("Error {} whilst parsing {}", e, point),
            };

            let circle = Circle::new((0., point - displacements[done]), 5, RED.filled());
            chart.plotting_area().draw(&circle).unwrap();

            window_index += 1;
            first = false;
        }

        draw_time += start_draw.elapsed().as_millis();

        let start_enc = Instant::now();

        // Write the frame to the encoder, which in turn writes it to FFMPEG, with
        // the bmp headers required
        ENCODER
            .write_all(&mut writer, unsafe { &mut PIXEL_ARRAY })
            .unwrap();

        let start_draw_2 = Instant::now();

        // Copy the empty axis over to the current frame, leaving it empty for the next frame
        unsafe { &mut PIXEL_ARRAY }.copy_from_slice(unsafe { &mut DRAWN_AXIS });

        draw_time += start_draw_2.elapsed().as_millis();
        enc_time += start_enc.elapsed().as_millis();
    }

    Ok((enc_time, draw_time))
}

/// The function that's called in main, and the only function
/// that should be exposed to a python caller
#[no_mangle]
pub extern "C" fn start() {
    let start = Instant::now();

    let mut args = std::env::args().skip(1);

    let mut rdr = std::fs::File::open(args.next().as_deref().unwrap_or(DEFAULT_INPUT_FILE))
        .expect("The input and default path don't yield a file");

    let (fps, point_count, span_count, displacements) = {
        let rdr = BufReader::new(&rdr);

        get_fps_counts_displacment(rdr)
    };

    rdr.seek(std::io::SeekFrom::Start(0)).unwrap();

    let config = setup::setup(&mut rdr, fps, point_count, span_count, displacements)
        .expect("Either HEIGHT or WIDTH is too large");

    // Reset the reading for the rdr, so the csv parsing is correct
    rdr.seek(std::io::SeekFrom::Start(0)).unwrap();

    // writer is the stdin we're writing to, for ffmpeg
    let writer = setup_ffmpeg(config.fps / 10);

    println!("Took {}ms to get to here", start.elapsed().as_millis());

    let (enc_time, draw_time) =
        draw_image_to_writer(BufReader::new(rdr), BufWriter::new(writer), config)
            .expect("Either HEIGHT or WIDTH is too large");

    let elapsed = start.elapsed().as_millis();

    println!(
        "Took {}ms overall, {}ms to encode, {}ms to draw",
        elapsed, enc_time, draw_time,
    );
}

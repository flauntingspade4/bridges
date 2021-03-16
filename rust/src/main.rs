#![warn(clippy::pedantic, clippy::nursery)]
#![deny(clippy::erasing_op)]

use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Seek, Write},
    process::{ChildStdin, Command, Stdio},
    time::Instant,
};

mod bmp;

use bmp::BmpEncoder;

const WIDTH: usize = 3000;
const HEIGHT: usize = 1080;

const OUTPUT_FILE: &str = "../output/output.avi";
const DEFAULT_INPUT_FILE: &str = "../input/input.csv";

static mut PIXEL_ARRAY: &mut [u8] = &mut [255; WIDTH * HEIGHT * 3];

use csv::Reader;

use plotters::{
    prelude::{BitMapBackend, ChartBuilder, Circle, IntoDrawingArea},
    style::{Color, RED},
};

fn start(mut rdr: Reader<File>, mut writer: impl Write) {
    let start = Instant::now();

    if std::fs::remove_file(OUTPUT_FILE).is_ok() {
        println!("Removed \"{}\"", OUTPUT_FILE)
    }

    let mut enc_time = 0;
    let mut draw_time = 0;

    println!("Took {}ms to get to here", start.elapsed().as_millis());

    for points in rdr.records().take(500) {
        let points = points.unwrap();

        // SAFETY: As the application is exclusively single-threaded,
        // the static mut will only be accessed by one thread at once
        unsafe {
            {
                let start_draw = Instant::now();

                let window =
                    BitMapBackend::with_buffer(&mut PIXEL_ARRAY, (WIDTH as u32, HEIGHT as u32))
                        .into_drawing_area();

                // Split the window into the correct amount of drawing areas
                let windows = window.split_evenly((3, 10));

                // Draw all the circles on their respective windows
                for (point, window) in points.iter().skip(1).zip(windows.iter()) {
                    let mut chart = ChartBuilder::on(window)
                        //.margin(10)
                        .y_label_area_size(50)
                        .x_label_area_size(25)
                        .build_cartesian_2d(-1.0..1.0, -15.0..15.0)
                        .unwrap();

                    chart
                        .configure_mesh()
                        .disable_mesh()
                        .y_labels(15)
                        .draw()
                        .unwrap();

                    let point = point.parse::<f64>().unwrap();
                    let circle = Circle::new((0., point), 5, RED.filled());
                    chart.plotting_area().draw(&circle).unwrap();
                }
                draw_time += start_draw.elapsed().as_millis();
            }

            let start_enc = Instant::now();

            let mut encoder = BmpEncoder::new(PIXEL_ARRAY);

            encoder.write_all(&mut writer).unwrap();

            PIXEL_ARRAY.iter_mut().for_each(|a| *a = 255);

            enc_time += start_enc.elapsed().as_millis();
        }
    }

    let elapsed = start.elapsed().as_millis();

    println!(
        "Took {}ms overall, {}ms to encode, {}ms to draw",
        elapsed, enc_time, draw_time,
    );
}

pub fn main() {
    let mut args = std::env::args().skip(1);

    let mut file =
        std::fs::File::open(args.next().as_deref().unwrap_or(DEFAULT_INPUT_FILE)).unwrap();

    // Get the desired fps to view the video as if it were real-time
    let fps = get_frame_rate(&file);

    // Reset the reading for the file, so the csv parsing is correct
    file.seek(std::io::SeekFrom::Start(0)).unwrap();

    // rdr is the excel file we're reading from
    let rdr = csv::Reader::from_reader(file);

    // writer is the stdin we're writing to, for ffmpeg
    let writer = ffmpeg_stuff(fps / 10);

    start(rdr, writer);
}

fn ffmpeg_stuff(fps: usize) -> BufWriter<ChildStdin> {
    let mut ffmpeg = Command::new("ffmpeg")
        .args(&[
            "-framerate",
            &format!("{}", fps),
            "-hide_banner",
            "-nostats",
            "-i",
            "pipe:0",
            OUTPUT_FILE,
        ])
        .stdin(Stdio::piped())
        .spawn()
        .expect("failed to execute process");

    let ffmpeg = ffmpeg.stdin.take().unwrap();
    BufWriter::new(ffmpeg)
}

fn get_frame_rate(rdr: &File) -> usize {
    let mut fps = 0;

    let rdr = BufReader::new(rdr);

    for line in rdr.lines().skip(1) {
        let line = line.unwrap();

        let time = line
            .split(',')
            .next()
            .expect("Time must be the first paramater")
            .parse::<f64>()
            .unwrap();

        if time >= 1.0 {
            break;
        }
        fps += 1;
    }

    fps
}

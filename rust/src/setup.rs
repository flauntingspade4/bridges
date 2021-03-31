use std::{
    convert::TryInto,
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
    num::TryFromIntError,
    process::{ChildStdin, Command, Stdio},
};

use plotters::prelude::{ChartBuilder, IntoDrawingArea};

/// A configuration struct used to determine the video's output
pub struct Config {
    /// The output the video should run at
    pub fps: usize,
    /// The number of points in the data set
    pub point_count: usize,
    /// The number of spans in the data set
    pub span_count: usize,
    /// The displacement of each point-used to
    /// normalise the points
    pub displacements: Vec<f64>,
    /// The minimum y value that must be drawn
    pub y_min: f64,
    /// The maximum y value that must be drawn
    pub y_max: f64,
}

/// Setup the ffmpeg writer with the specified fps, returning the
/// `ChildStdin` of the thread started
pub fn setup_ffmpeg(fps: usize) -> ChildStdin {
    let ffmpeg = Command::new("ffmpeg")
        .args(&[
            "-framerate",
            &format!("{}", fps),
            "-hide_banner",
            "-nostats",
            "-i",
            "pipe:0",
            crate::OUTPUT_FILE,
        ])
        .stdin(Stdio::piped())
        .spawn()
        .expect("failed to execute process");

    match ffmpeg.stdin {
        Some(t) => t,
        None => unreachable!(),
    }
}

/// Returns the general config, and sets `DRAWN_AXIS`.
/// # Example
/// In `input.csv`:
/// ```csv
/// Time,Displacement 1,Displacement 2,Displacement 3,Displacement 4,Displacement 5,Displacement 6,Displacement 7,Displacement 8,Displacement 9,Displacement 10,Displacement 11,Displacement 12
/// 0,0,0,0,0,0,-1.2722422,-1.788168,-2.943386,-3.397,-3.247458,-1.0217578,-3.46305
/// 0.004992,0,0,0,0,0,-1.26975,-1.786918,-2.948372,-3.393262,-3.252442,-1.0267422
/// 0.009984,0,0,0,0,0,-1.2734882,-1.7856718,-2.945878,-3.394508,-3.251196,-1.0192656
/// ```
///
/// In `main.rs`
/// ```rust
/// const WIDTH: usize = 1920;
/// const HEIGHT: usize = 1080;
///
/// let file = std::fs::open("input.csv").unwrap();
/// let (fps, point_count, span_count, displacements) = {
///     let rdr = BufReader::new(&rdr);
///
///     get_fps_counts_displacment(rdr)
/// };
///
/// rdr.seek(std::io::SeekFrom::Start(0)).unwrap();
///
/// let config = setup::setup(&mut rdr, fps, point_count, span_count, displacements)
///     .expect("Either HEIGHT or WIDTH is too large");
/// ```
pub fn setup(
    rdr: &mut File,
    fps: usize,
    point_count: usize,
    span_count: usize,
    displacements: Vec<f64>,
) -> Result<Config, TryFromIntError> {
    let (y_min, y_max) = {
        rdr.seek(SeekFrom::Start(0)).unwrap();

        let rdr = BufReader::new(rdr);

        get_max_min(rdr, &displacements)
    };

    println!("{} min {} max", y_min, y_max);

    let window = plotters::prelude::BitMapBackend::with_buffer(
        unsafe { &mut crate::PIXEL_ARRAY },
        (crate::WIDTH.try_into()?, crate::HEIGHT.try_into()?),
    )
    .into_drawing_area();

    let windows = window.split_evenly((3, point_count / 3));

    for window in &windows {
        let mut chart = ChartBuilder::on(window);

        let mut chart = crate::setup_chart!(chart, y_min, y_max);

        chart
            .configure_mesh()
            .x_labels(1)
            .y_labels(15)
            .draw()
            .unwrap();
    }

    // At this point PIXEL_ARRAY contains the default axis,
    // so we must set DRAWN_AXIS to a clone of this.
    // SAFETY: DRAWN_AXIS is only borrowed mutably once, here,
    // where no other threads should be active
    unsafe {
        crate::DRAWN_AXIS.copy_from_slice(crate::PIXEL_ARRAY);
    }

    Ok(Config {
        fps,
        point_count,
        span_count,
        displacements,
        y_min,
        y_max,
    })
}

/// Returns the fps, the number of points (Sensors) in the data set,
/// the number of spans, and the displacements in the data set
pub fn get_fps_counts_displacment(rdr: BufReader<impl Read>) -> (usize, usize, usize, Vec<f64>) {
    let mut fps = 0;

    let buf_rdr = BufReader::new(rdr);
    let mut lines = buf_rdr.lines().skip(1);

    let point_count = lines.next().unwrap().unwrap().split(',').count() - 1;
    let span_count = point_count / 15;

    let mut displacements = Vec::with_capacity(point_count);
    displacements.resize(point_count, 1_000_000.);

    let mut last = Vec::with_capacity(point_count);
    last.resize(point_count, 0.);

    for line in lines {
        let line = line.expect("The input file should contain only vaild UTF-8");

        let mut line = line.split(',');

        let time = match line
            .next()
            .expect("Time must be the first paramater")
            .parse::<f64>()
        {
            Ok(t) => t,
            Err(e) => panic!("Error {} whilst parsing {:?}", e, line),
        };

        if time < 1.0 {
            fps += 1;
        }

        for (i, point) in line.enumerate() {
            match point.parse() {
                Ok(t) => {
                    if (t - last[i]) < displacements[i] {
                        displacements[i] = t;
                    }
                    last[i] = t;
                }
                Err(e) => panic!("Error {} whilst parsing {}", e, point),
            }
        }
    }

    (fps, point_count, span_count, displacements)
}

/// Gets the minimum and maximum bounds of the data set, with a given
/// displacements
pub fn get_max_min(rdr: BufReader<impl Read>, displacements: &[f64]) -> (f64, f64) {
    let mut y_min = f64::MAX;
    let mut y_max = f64::MIN;

    // Skip the first line, as it's just headers
    for line in rdr.lines().skip(1) {
        let line = line.unwrap();

        // Skip the first part, as it's the time, which is unneeded
        let line = line.split(',').skip(1);

        for (i, point) in line.enumerate() {
            match point.parse::<f64>() {
                Ok(t) => {
                    // Adjust to t's displacement
                    let t = t - displacements[i];
                    if t > y_max {
                        y_max = t;
                    } else if t < y_min {
                        y_min = t
                    }
                }
                Err(e) => panic!("Error {} whilst parsing {}", e, point),
            }
        }
    }

    (y_min, y_max)
}

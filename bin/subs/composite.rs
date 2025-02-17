use crate::subs::runnable::RunnableSubcommand;
use async_trait::async_trait;
use clap::Parser;
use mars_raw_utils::{composite, prelude::*};
use sciimg::{drawable::*, prelude::*, quaternion::Quaternion};
use std::process;

pb_create_spinner!();

#[derive(Parser)]
#[command(author, version, about = "Create composite mosaic", long_about = None)]
pub struct Composite {
    #[arg(long, short, help = "Input images", num_args = 1..)]
    input_files: Vec<std::path::PathBuf>,

    #[arg(long, short, help = "Output image")]
    output: std::path::PathBuf,

    #[arg(long, short, help = "Anaglyph mode")]
    anaglyph: bool,

    #[arg(long, short = 'r', help = "Azimuth rotation")]
    azimuth: Option<f64>,
}
#[async_trait]
impl RunnableSubcommand for Composite {
    async fn run(&self) {
        pb_set_print!();

        print::print_experimental();

        let in_files: Vec<String> = self
            .input_files
            .iter()
            .map(|s| String::from(s.as_os_str().to_str().unwrap()))
            .collect();

        let output = self.output.as_os_str().to_str().unwrap();

        let azimuth_rotation: f64 = self.azimuth.unwrap_or(0.0);

        let quat = Quaternion::from_pitch_roll_yaw(0.0, 0.0, azimuth_rotation.to_radians());

        let map_context = composite::determine_map_context(&in_files, &quat);
        vprintln!("Map Context: {:?}", map_context);
        vprintln!(
            "FOV Vertical: {}",
            map_context.top_lat - map_context.bottom_lat
        );
        vprintln!(
            "FOV Horizontal: {}",
            map_context.right_lon - map_context.left_lon
        );

        if map_context.width == 0 {
            eprintln!("Output expected to have zero width. Cannot continue with that. Exiting...");
            pb_done_with_error!();
            process::exit(1);
        } else if map_context.height == 0 {
            eprintln!("Output expected to have zero height. Cannot continue with that. Exiting...");
            pb_done_with_error!();
            process::exit(1);
        }

        let mut map = Image::create_masked(map_context.width, map_context.height, true);

        let first_image = MarsImage::open(in_files[0].to_owned(), Instrument::M20MastcamZLeft);
        let initial_origin = if let Some(model) = composite::get_cahvor(&first_image) {
            model.c()
        } else {
            eprintln!("Cannot determine initial camera origin");
            pb_done_with_error!();
            process::exit(2);
        };

        for in_file in in_files.iter() {
            if path::file_exists(in_file) {
                vprintln!("Processing File: {}", in_file);
                composite::process_file(
                    in_file,
                    &map_context,
                    &mut map,
                    self.anaglyph,
                    &quat,
                    &initial_origin,
                );
            } else {
                eprintln!("File not found: {}", in_file);
                pb_done_with_error!();
                process::exit(1);
            }
        }

        map.save(output);

        pb_done!();
    }
}

use crate::subs::runnable::RunnableSubcommand;
use async_trait::async_trait;
use clap::Parser;
use mars_raw_utils::prelude::*;
use sciimg::{drawable::*, prelude::*, vector::Vector};
use std::process;

pb_create_spinner!();

#[derive(Parser)]
#[command(author, version, about = "Generate anaglyph from stereo pair", long_about = None)]
pub struct Anaglyph {
    #[arg(long, short, help = "Left image")]
    left: std::path::PathBuf,

    #[arg(long, short, help = "Right image")]
    right: std::path::PathBuf,

    #[arg(long, short, help = "Output image")]
    output: std::path::PathBuf,

    #[arg(long, short, help = "Monochrome color (before converting to red/blue)")]
    mono: bool,
}
#[async_trait]
impl RunnableSubcommand for Anaglyph {
    async fn run(&self) {
        pb_set_print!();
        print::print_experimental();

        let left_image_path = String::from(self.left.as_os_str().to_str().unwrap());
        let right_image_path = String::from(self.right.as_os_str().to_str().unwrap());
        let out_file_path = self.output.as_os_str().to_str().unwrap();

        if !path::file_exists(&left_image_path) {
            eprintln!("Error: File not found (left eye): {}", left_image_path);
            pb_done_with_error!();
            process::exit(1);
        }

        if !path::file_exists(&right_image_path) {
            eprintln!("Error: File not found (right eye): {}", right_image_path);
            pb_done_with_error!();
            process::exit(1);
        }

        if !path::parent_exists_and_writable(out_file_path) {
            eprintln!(
                "Error: Output file directory not found or is not writable: {}",
                out_file_path
            );
            pb_done_with_error!();
            process::exit(1);
        }

        let mut left_img = MarsImage::open(left_image_path, Instrument::M20MastcamZLeft);
        let mut right_img = MarsImage::open(right_image_path, Instrument::M20MastcamZRight);

        if self.mono {
            vprintln!("Converting input images to monochrome...");
            left_img.to_mono();
            right_img.to_mono();
        }

        let left_cahv = if let Some(left_md) = &left_img.metadata {
            if left_md.camera_model_component_list.is_valid() {
                left_md.camera_model_component_list.clone()
            } else {
                pb_done_with_error!();
                process::exit(2);
            }
        } else {
            pb_done_with_error!();
            process::exit(1);
        };

        let right_cahv = if let Some(right_md) = &right_img.metadata {
            if right_md.camera_model_component_list.is_valid() {
                right_md.camera_model_component_list.clone()
            } else {
                pb_done_with_error!();
                process::exit(2);
            }
        } else {
            pb_done_with_error!();
            process::exit(1);
        };

        let ground = Vector::new(0.0, 0.0, 1.84566);

        let mut map = Image::create(left_img.image.width, left_img.image.height);
        let output_model = left_cahv
            .linearize(
                left_img.image.width,
                left_img.image.height,
                left_img.image.width,
                left_img.image.height,
            )
            .unwrap();

        anaglyph::process_image(
            &right_img,
            &mut map,
            &right_cahv,
            &output_model,
            &ground,
            Eye::Right,
        );
        anaglyph::process_image(
            &left_img,
            &mut map,
            &left_cahv,
            &output_model,
            &ground,
            Eye::Left,
        );

        map.save(out_file_path);
        pb_done!();
    }
}

use crate::{
    calibfile, calibrate::*, calprofile::CalProfile, decompanding, enums, enums::Instrument,
    marsimage::MarsImage, memcache::load_image, util, vprintln,
};

use anyhow::Result;
use sciimg::{image::Image, path, prelude::ImageBuffer};

#[derive(Copy, Clone)]
pub struct M20EECam {}

// Converts an image mask with values 0-255 to 0, 1
fn create_adjusted_mask(buffer: &ImageBuffer) -> ImageBuffer {
    let mut adjusted = buffer.clone();
    (0..adjusted.buffer.len()).for_each(|i| {
        if adjusted.buffer[i] > 200.0 {
            // We're bias towards darkening here to account for interpolated pixel values
            // following an image resize (scale_factor>1). I'd rather grow the dark area (zeros)
            // then grow the light area.
            adjusted.buffer[i] = 1.0;
        } else {
            adjusted.buffer[i] = 0.0;
        }
    });
    adjusted
}

impl Calibration for M20EECam {
    fn accepts_instrument(&self, instrument: Instrument) -> bool {
        matches!(
            instrument,
            Instrument::M20FrontHazLeft
                | Instrument::M20FrontHazRight
                | Instrument::M20NavcamLeft
                | Instrument::M20NavcamRight
                | Instrument::M20RearHazLeft
                | Instrument::M20RearHazRight
        )
    }

    fn process_file(
        &self,
        input_file: &str,
        cal_context: &CalProfile,
        only_new: bool,
    ) -> Result<CompleteContext> {
        let out_file = util::append_file_name(input_file, cal_context.filename_suffix.as_str());
        if path::file_exists(&out_file) && only_new {
            vprintln!("Output file exists, skipping. ({})", out_file);
            return cal_warn(cal_context, &out_file);
        }

        let mut instrument = enums::Instrument::M20NavcamRight;

        // Attempt to figure out camera from file name
        if util::filename_char_at_pos(input_file, 0) == 'N' {
            // NAVCAMS
            if util::filename_char_at_pos(input_file, 1) == 'L' {
                // Left
                instrument = enums::Instrument::M20NavcamLeft;
            } else {
                // Assume Right
                instrument = enums::Instrument::M20NavcamRight;
            }
        } else if util::filename_char_at_pos(input_file, 0) == 'F' {
            // FHAZ
            if util::filename_char_at_pos(input_file, 1) == 'L' {
                // Left
                instrument = enums::Instrument::M20FrontHazLeft;
            } else {
                // Assume Right
                instrument = enums::Instrument::M20FrontHazRight;
            }
        } else if util::filename_char_at_pos(input_file, 0) == 'R' {
            // RHAZ
            if util::filename_char_at_pos(input_file, 1) == 'L' {
                // Left
                instrument = enums::Instrument::M20RearHazLeft;
            } else {
                // Assume Right
                instrument = enums::Instrument::M20RearHazRight;
            }
        }

        let mut raw = MarsImage::open(String::from(input_file), instrument);

        // Apply destretching based off histogram gaps
        //vprintln!("Destretching...");
        //raw.destretch_image();

        let data_max = if cal_context.apply_ilt {
            vprintln!("Decompanding...");
            let lut = decompanding::get_ilt_for_instrument(instrument).unwrap();
            raw.decompand(&lut);
            lut.max() as f32
        } else {
            255.0
        };

        // Looks like 'ECM' in the name seems to indicate that it still have the bayer pattern
        if raw.image.is_grayscale() {
            vprintln!("Debayering...");
            raw.debayer_with_method(cal_context.debayer_method);
        }

        vprintln!("Flatfielding...");
        let scale_factor = if let Some(md) = raw.metadata.clone() {
            md.scale_factor
        } else {
            1
        };
        let scale_factor_str = format!("sf{}", scale_factor);

        // let mut flat = flatfield::load_flat(instrument).unwrap();
        let mut flat = match calibfile::get_calibration_file_for_instrument(
            instrument,
            enums::CalFileType::FlatField,
        ) {
            Ok(s) => {
                let flat_file_path = s.replace("-scalefactor-", scale_factor_str.as_str());
                vprintln!(
                    "Flat file path for scale factor {}: {}",
                    scale_factor,
                    flat_file_path
                );
                load_image(&flat_file_path).unwrap()
            }
            Err(why) => {
                vprintln!(
                    "Flat file not determined for instrument {:?}: {:?}",
                    instrument,
                    why
                );
                Image::new_empty().unwrap()
            }
        };

        vprintln!("Loading image mask");
        let mut mask = match calibfile::get_calibration_file_for_instrument(
            instrument,
            enums::CalFileType::Mask,
        ) {
            Ok(s) => {
                let mask_file_path = s.replace("-scalefactor-", scale_factor_str.as_str());
                vprintln!(
                    "Mask file path for scale factor {}: {}",
                    scale_factor,
                    mask_file_path
                );
                load_image(&mask_file_path).unwrap()
            }
            Err(why) => {
                vprintln!(
                    "Flat file not determined for instrument {:?}: {:?}",
                    instrument,
                    why
                );
                Image::new_empty().unwrap()
            }
        };

        if let Some(md) = raw.metadata.clone() {
            if let Some(rect) = &md.subframe_rect {
                flat.crop(
                    (rect[0] as usize - 1) / scale_factor as usize,
                    (rect[1] as usize - 1) / scale_factor as usize,
                    (rect[2] as usize) / scale_factor as usize,
                    (rect[3] as usize) / scale_factor as usize,
                );

                if !mask.is_empty() {
                    mask.crop(
                        (rect[0] as usize - 1) / scale_factor as usize,
                        (rect[1] as usize - 1) / scale_factor as usize,
                        (rect[2] as usize) / scale_factor as usize,
                        (rect[3] as usize) / scale_factor as usize,
                    );
                }

                vprintln!("Flat cropped to {}x{}", flat.width, flat.height);
            }
        }

        if !mask.is_empty() {
            let mask_adjusted = create_adjusted_mask(mask.get_band(0));
            raw.image
                .set_band(&raw.image.get_band(0).multiply(&mask_adjusted).unwrap(), 0);
            raw.image
                .set_band(&raw.image.get_band(1).multiply(&mask_adjusted).unwrap(), 1);
            raw.image
                .set_band(&raw.image.get_band(2).multiply(&mask_adjusted).unwrap(), 2);
        }
        raw.apply_flat(&flat);

        if !raw.image.is_grayscale() {
            vprintln!("Applying color weights...");
            raw.apply_weight(
                cal_context.red_scalar,
                cal_context.green_scalar,
                cal_context.blue_scalar,
            );
        }

        if cal_context.decorrelate_color {
            vprintln!("Normalizing with decorrelated colors...");
            raw.image.normalize_to_16bit_decorrelated();
        } else {
            vprintln!("Normalizing with correlated colors...");
            raw.image.normalize_to_16bit_with_max(data_max);
        }

        // Trim off border pixels
        if let Some(mut md) = raw.metadata.clone() {
            if md.scale_factor == 1 {
                if let Some(rect) = &md.subframe_rect {
                    //rect[0] += 1.0;
                    let new_rect = vec![rect[0] + 2.0, rect[1] + 2.0, rect[2] - 2.0, rect[3] - 2.0];
                    md.subframe_rect = Some(new_rect);
                }
            }
            raw.metadata = Some(md);
            let crop_to_width = raw.image.width - 4;
            let crop_to_height = raw.image.height - 4;
            raw.crop(2, 2, crop_to_width, crop_to_height);
        }
        //
        //

        vprintln!("Writing to disk...");
        raw.save(&out_file);

        cal_ok(cal_context, &out_file)
    }
}

use bytes::Bytes;
use eyre::ContextCompat;
use image::math::Rect;
use image::{DynamicImage, Luma};
use rawler::RawlerError;

pub struct ImgWithMetadata {
    pub image: DynamicImage,
    pub crop_area: Rect,
    pub exposure_time: Option<f64>,
}

impl ImgWithMetadata {
    pub fn from_data(data: Bytes) -> eyre::Result<Self> {
        let mut raw_file = rawler::RawFile::new("", std::io::Cursor::new(data.clone()));
        match rawler::get_decoder(&mut raw_file) {
            Ok(decoder) => {
                let exposure_time = decoder
                    .raw_metadata(&mut raw_file, Default::default())?
                    .exif
                    .exposure_time
                    .map(|r| f64::from(r.n) / f64::from(r.d));
                let raw_image = decoder.raw_image(&mut raw_file, Default::default(), false)?;
                let cfa = raw_image.cfa.to_string();
                eyre::ensure!(cfa == "RGGB", "Unsupported Bayer pattern: {cfa}");
                let width = raw_image.width as u32;
                let height = raw_image.height as u32;
                Ok(ImgWithMetadata {
                    image: match raw_image.data {
                        rawler::RawImageData::Float(_) => {
                            // See https://github.com/image-rs/image/issues/1940.
                            // Might use our own enum if it turns out that any popular DSLRs use this format
                            // (I'm not aware of any though).
                            eyre::bail!("Floating-point raw files are unsupported")
                        }
                        rawler::RawImageData::Integer(img_data) => {
                            image::ImageBuffer::<Luma<u16>, _>::from_vec(width, height, img_data)
                                .context("couldn't match dimensions to raw data")?
                                .into()
                        }
                    },
                    crop_area: match raw_image.crop_area {
                        Some(crop_area) => Rect {
                            x: crop_area.x() as u32,
                            y: crop_area.y() as u32,
                            width: crop_area.width() as u32,
                            height: crop_area.height() as u32,
                        },
                        None => Rect {
                            x: 0,
                            y: 0,
                            width,
                            height,
                        },
                    },
                    exposure_time,
                })
            }
            Err(RawlerError::Unsupported { .. }) => {
                let image = image::load_from_memory(&data)?;

                let exposure_time = match exif::Reader::new()
                    .read_from_container(&mut std::io::Cursor::new(data))
                {
                    Ok(exif) => exif
                        .get_field(exif::Tag::ExposureTime, exif::In::PRIMARY)
                        .map(|field| match &field.value {
                            exif::Value::Rational(rational) if rational.len() == 1 => {
                                Ok(rational[0].to_f64())
                            }
                            v => eyre::bail!("Invalid field type for exposure time: {v:?}"),
                        })
                        .transpose()?,
                    Err(exif::Error::NotFound(_)) => None,
                    Err(err) => return Err(err.into()),
                };

                Ok(ImgWithMetadata {
                    crop_area: Rect {
                        x: 0,
                        y: 0,
                        width: image.width(),
                        height: image.height(),
                    },
                    image,
                    exposure_time,
                })
            }
            Err(err) => Err(err.into()),
        }
    }
}

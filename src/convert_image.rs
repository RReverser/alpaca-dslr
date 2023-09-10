use ascom_alpaca::api::ImageArray;
use image::{DynamicImage, ImageBuffer, Pixel};

fn convert_image_buffer<P: Pixel>(img: ImageBuffer<P, Vec<P::Subpixel>>) -> ImageArray
where
    ImageArray: From<ndarray::Array3<P::Subpixel>>,
{
    let flat_samples = img.into_flat_samples();
    let mut arr = ndarray::Array::from_shape_vec(
        (
            flat_samples.layout.height as usize,
            flat_samples.layout.width as usize,
            flat_samples.layout.channels.into(),
        ),
        flat_samples.samples,
    )
    .expect("shape mismatch when creating image array");

    // From image layout (y * x * c) to algebraic matrix layout (x * y * c).
    arr.swap_axes(0, 1);

    arr.into()
}

pub(crate) fn convert_dynamic_image(img: DynamicImage) -> eyre::Result<ImageArray> {
    Ok(match img {
        DynamicImage::ImageLuma8(img) => convert_image_buffer(img),
        DynamicImage::ImageLuma16(img) => convert_image_buffer(img),
        DynamicImage::ImageRgb8(img) => convert_image_buffer(img),
        DynamicImage::ImageRgb16(img) => convert_image_buffer(img),
        _ => eyre::bail!("unsupported image colour format"),
    })
}

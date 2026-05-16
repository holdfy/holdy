//! Geração de imagem PNG do QR Code a partir do payload PIX (EMV / copia-e-cola).

use image::{ImageBuffer, ImageFormat, Rgb};
use qrcode::types::QrError;
use qrcode::QrCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QrCodeError {
    #[error("QR encode: {0}")]
    Encode(#[from] QrError),
    #[error("PNG encode: {0}")]
    Png(#[from] image::ImageError),
}

/// Renderiza o texto do PIX (BR Code) como PNG em memória.
pub fn pix_qr_png_bytes(payload: &str) -> Result<Vec<u8>, QrCodeError> {
    let code = QrCode::new(payload.as_bytes())?;
    let image: ImageBuffer<Rgb<u8>, Vec<u8>> =
        code.render::<Rgb<u8>>().min_dimensions(512, 512).build();
    let dyn_img = image::DynamicImage::ImageRgb8(image);
    let mut buf = Vec::new();
    dyn_img.write_to(&mut std::io::Cursor::new(&mut buf), ImageFormat::Png)?;
    Ok(buf)
}

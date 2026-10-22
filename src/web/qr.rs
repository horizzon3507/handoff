use qrcode::render::svg;
use qrcode::QrCode;

pub fn render_qr_svg(value: &str) -> Result<String, String> {
    let code =
        QrCode::new(value.as_bytes()).map_err(|_| "The room link is too long for a QR code.")?;
    Ok(code
        .render::<svg::Color>()
        .min_dimensions(240, 240)
        .dark_color(svg::Color("#20251e"))
        .light_color(svg::Color("#f4f1e8"))
        .build())
}

//! The brand mark at the left of the title bar, where Photoshop shows its "Ps" tile. Evora build
//! (patch E1): the upstream app icon is replaced by Lumbra's neutral mark (`assets/evora/`, an
//! indigo tile with a half-lit disc). The PNG carries the icon's rounded corners; it is decoded
//! once per context into a mipmapped texture, so it stays crisp at the title bar's size on any
//! display scale.

use egui::{Color32, Context, Id, Rect, TextureHandle, TextureOptions, Ui, pos2};
use evophoto_codecs::{ChannelLayout, SampleType};

/// 128 px: sharp at 20 pt on a 3× display, and the mipmaps keep it clean at 1×.
const ICON_PNG: &[u8] = include_bytes!("../../../assets/evora/lumbra-mark-128.png");

/// The green channel of the mark's indigo tile, the channel where indigo and white stand furthest apart.
const TILE_GREEN: f32 = 70.0;

/// EVORA-LUM-115: the mark's tile in the member's colour. Its pixels are indigo, white or between the two; the
/// green channel says how far each stands from white, and that share of the member's colour replaces it.
fn tinted(rgba: &[u8], accent: Color32) -> Vec<u8> {
    rgba.chunks_exact(4)
        .flat_map(|px| {
            let share = ((255.0 - f32::from(px[1])) / (255.0 - TILE_GREEN)).clamp(0.0, 1.0);
            let channel = |c: u8| (255.0 + (f32::from(c) - 255.0) * share).round() as u8;
            [channel(accent.r()), channel(accent.g()), channel(accent.b()), px[3]]
        })
        .collect()
}

/// The icon's pixels (one transparent pixel if it couldn't be decoded, which a test rules out).
fn decode() -> egui::ColorImage {
    let rgba = evophoto_codecs::decode(ICON_PNG).ok().map(|img| img.convert(ChannelLayout::Rgba, SampleType::U8));
    match rgba {
        Some(img) if img.data().len() == img.width() as usize * img.height() as usize * 4 => {
            let size = [img.width() as usize, img.height() as usize];
            let accent = crate::theme::host_accent();
            if accent == crate::theme::EVORA_ACCENT {
                egui::ColorImage::from_rgba_unmultiplied(size, img.data())
            } else {
                egui::ColorImage::from_rgba_unmultiplied(size, &tinted(img.data(), accent))
            }
        }
        _ => egui::ColorImage::filled([1, 1], Color32::TRANSPARENT),
    }
}

/// The icon texture, uploaded on first use and kept in the context.
fn texture(ctx: &Context) -> TextureHandle {
    let id = Id::new(("evophoto-brand-mark", crate::theme::host_accent()));
    if let Some(tex) = ctx.data(|d| d.get_temp::<TextureHandle>(id)) {
        return tex;
    }
    let options = TextureOptions { mipmap_mode: Some(egui::TextureFilter::Linear), ..TextureOptions::LINEAR };
    let tex = ctx.load_texture("evophoto-brand-mark", decode(), options);
    ctx.data_mut(|d| d.insert_temp(id, tex.clone()));
    tex
}

/// Where the mark was painted this frame (tests and screenshots).
fn rect_id() -> Id {
    Id::new("evophoto-brand-mark-rect")
}

/// Paint the mark into `r` (square).
pub fn paint_mark(ui: &Ui, r: Rect) {
    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    ui.painter().image(texture(ui.ctx()).id(), r, uv, Color32::WHITE);
    ui.ctx().data_mut(|d| d.insert_temp(rect_id(), r));
}

/// The rect the mark was last painted into.
#[cfg(test)]
pub fn mark_rect(ctx: &Context) -> Option<Rect> {
    ctx.data(|d| d.get_temp(rect_id()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_icon_decodes_to_a_rounded_128_px_tile() {
        let img = decode();
        assert_eq!(img.size, [128, 128]);
        // Rounded corners: the corner pixel is transparent, the centre opaque.
        assert_eq!(img.pixels[0].a(), 0);
        assert_eq!(img.pixels[64 * 128 + 64].a(), 255);
    }

    #[test]
    fn the_title_bar_paints_the_icon_from_one_cached_texture() {
        let ctx = Context::default();
        crate::EvoPhotoApp::setup_context(&ctx, crate::theme::ThemeKind::ALL[0]);
        let mut app = crate::EvoPhotoApp::new(evophoto_engine::Session::new(), crate::Services::default());
        // The system title bar hides the in-app mark (the OS draws its own icon), so paint the
        // app-drawn bar here.
        app.custom_titlebar = true;
        let mut frame = || {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| crate::panels::title_bar(&mut app, ui));
            let uploads = out.textures_delta.set.values().flat_map(|d| d.iter()).filter(|d| d.image.size() == [128, 128]).count();
            out.textures_delta.clear();
            uploads
        };
        assert_eq!(frame(), 1, "uploaded on first use");
        assert_eq!(frame(), 0, "then reused");
        assert!(mark_rect(&ctx).is_some());
    }
}

use std::sync::LazyLock;

use gtk::glib;
use relm4::gtk::gdk::Texture;
use relm4::gtk::gdk_pixbuf::Pixbuf;
use relm4::gtk::gio::{Cancellable, MemoryInputStream};
use relm4::prelude::*;

/// embedded logo as paintable texture
///
/// The bytes from PNG are included during build time and shipped
/// within the executable.
/// Inspired from from:
/// https://github.com/Relm4/Relm4/blob/d6b68a9e1887fe3b61c3700935c06b97ec054409/examples/embedded_logo.rs#L15
fn embedded_image(bytes: &[u8]) -> Texture {
    let g_bytes = glib::Bytes::from(&bytes.to_vec());
    let stream = MemoryInputStream::from_bytes(&g_bytes);
    let pixbuf = Pixbuf::from_stream(&stream, Cancellable::NONE).unwrap();
    let _ = Texture::for_pixbuf(&pixbuf);
    Texture::from_bytes(&g_bytes).unwrap()
}

static LOGO_SVG_BYTES: &[u8] = include_bytes!("../assets/logo.svg");
pub static LOGO_SVG: LazyLock<Texture> = LazyLock::new(|| embedded_image(LOGO_SVG_BYTES));

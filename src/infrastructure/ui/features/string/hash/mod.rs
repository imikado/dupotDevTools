use gtk::glib;
use gtk::prelude::*;
use sha2::Digest;

use crate::domain::contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::timestamp;

pub const DESCRIPTION: &str = "Generate cryptographic hash digests";

const ALGORITHMS: [&str; 8] = [
    "md5", "sha1", "sha224", "sha256", "sha384", "sha512", "sha3_256", "blake2b",
];
const DEFAULT_ALGORITHM: u32 = 3;

pub struct Feature;

fn hex<D: Digest>(data: &[u8]) -> String {
    D::digest(data).iter().map(|b| format!("{b:02x}")).collect()
}

fn hash(algorithm: &str, data: &[u8]) -> String {
    match algorithm {
        "md5" => hex::<md5::Md5>(data),
        "sha1" => hex::<sha1::Sha1>(data),
        "sha224" => hex::<sha2::Sha224>(data),
        "sha256" => hex::<sha2::Sha256>(data),
        "sha384" => hex::<sha2::Sha384>(data),
        "sha512" => hex::<sha2::Sha512>(data),
        "sha3_256" => hex::<sha3::Sha3_256>(data),
        "blake2b" => hex::<blake2::Blake2b512>(data),
        _ => unreachable!("unknown hash algorithm {algorithm}"),
    }
}

fn text_view(editable: bool) -> gtk::TextView {
    let tv = gtk::TextView::new();
    tv.set_monospace(true);
    tv.set_editable(editable);
    tv.set_wrap_mode(gtk::WrapMode::WordChar);
    tv.set_left_margin(8);
    tv.set_right_margin(8);
    tv.set_top_margin(8);
    tv.set_bottom_margin(8);
    tv
}

fn section(label: &str, view: &gtk::TextView, height: i32) -> (gtk::Label, gtk::ScrolledWindow) {
    let lbl = gtk::Label::builder().label(label).xalign(0.0).build();
    lbl.add_css_class("heading");
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_size_request(-1, height);
    scroll.add_css_class("card");
    scroll.set_child(Some(view));
    (lbl, scroll)
}

fn set_status(status: &gtk::Label, message: &str) {
    status.remove_css_class("error");
    status.add_css_class("dim-label");
    status.set_text(message);
}

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 12);

        let input_tv = text_view(true);
        let (input_lbl, input_scroll) = section("Input", &input_tv, 150);

        let algo_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        algo_box.set_halign(gtk::Align::Center);
        let dropdown = gtk::DropDown::from_strings(&ALGORITHMS);
        dropdown.set_selected(DEFAULT_ALGORITHM);
        algo_box.append(&gtk::Label::new(Some("Algorithm:")));
        algo_box.append(&dropdown);

        let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        btn_box.set_halign(gtk::Align::Center);
        let hash_btn = gtk::Button::with_label("Hash");
        hash_btn.add_css_class("suggested-action");
        let copy_btn = gtk::Button::with_label("Copy hash");
        let clear_btn = gtk::Button::with_label("Clear");
        btn_box.append(&hash_btn);
        btn_box.append(&copy_btn);
        btn_box.append(&clear_btn);

        let output_tv = text_view(false);
        let (output_lbl, output_scroll) = section("Hash", &output_tv, 80);

        let status = gtk::Label::new(None);
        status.add_css_class("dim-label");
        status.set_halign(gtk::Align::Start);

        hash_btn.connect_clicked(glib::clone!(
            #[weak]
            input_tv,
            #[weak]
            output_tv,
            #[weak]
            dropdown,
            #[weak]
            status,
            move |_| {
                let buf = input_tv.buffer();
                let text = buf.text(&buf.start_iter(), &buf.end_iter(), false);
                let algo = ALGORITHMS[dropdown.selected() as usize];
                output_tv.buffer().set_text(&hash(algo, text.as_bytes()));
                set_status(
                    &status,
                    &format!("{} at {}", algo.to_uppercase(), timestamp()),
                );
            }
        ));

        copy_btn.connect_clicked(glib::clone!(
            #[weak]
            output_tv,
            #[weak]
            status,
            move |_| {
                let buf = output_tv.buffer();
                if buf.char_count() == 0 {
                    return;
                }
                buf.select_range(&buf.start_iter(), &buf.end_iter());
                buf.copy_clipboard(&output_tv.clipboard());
                buf.select_range(&buf.end_iter(), &buf.end_iter());
                set_status(&status, &format!("Copied to clipboard at {}", timestamp()));
            }
        ));

        clear_btn.connect_clicked(glib::clone!(
            #[weak]
            input_tv,
            #[weak]
            output_tv,
            #[weak]
            status,
            move |_| {
                input_tv.buffer().set_text("");
                output_tv.buffer().set_text("");
                set_status(&status, "");
            }
        ));

        root.append(&input_lbl);
        root.append(&input_scroll);
        root.append(&algo_box);
        root.append(&btn_box);
        root.append(&output_lbl);
        root.append(&output_scroll);
        root.append(&status);
        root.upcast()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn known_digests() {
        assert_eq!(
            super::hash("md5", b"abc"),
            "900150983cd24fb0d6963f7d28e17f72"
        );
        assert_eq!(
            super::hash("sha256", b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(super::hash("sha3_256", b"").len(), 64);
        assert_eq!(super::hash("blake2b", b"").len(), 128);
    }
}

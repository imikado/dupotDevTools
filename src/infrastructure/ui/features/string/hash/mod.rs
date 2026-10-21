use gtk::prelude::*;
use sha2::Digest;

use crate::domain::contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub const TITLE: &str = "Hash Generator";
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

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            output_label: "Hash",
            output_height: 120,
            ..Default::default()
        });

        let dropdown = gtk::DropDown::from_strings(&ALGORITHMS);
        dropdown.set_selected(DEFAULT_ALGORITHM);
        dropdown.set_tooltip_text(Some("Algorithm"));
        let algo_lbl = gtk::Label::new(Some("Algorithm"));
        algo_lbl.add_css_class("dim-label");
        tool.add_extra_widget(&algo_lbl);
        tool.add_extra_widget(&dropdown);

        tool.add_button("Hash", move |t| {
            let algo = ALGORITHMS[dropdown.selected() as usize];
            t.report(Ok(hash(algo, t.input().as_bytes())));
        });
        tool.into_widget()
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

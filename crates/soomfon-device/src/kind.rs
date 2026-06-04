//! Static registry of supported devices.
//!
//! Soomfon hardware is an OEM rebrand of the Mirabox / Ajazz stream controller
//! family, so a single table covers every model. The per-model data mirrors the
//! community `opendeck-soomfon` plugin, re-expressed here as our own table.

use mirajazz::device::DeviceQuery;
use mirajazz::types::{ImageFormat, ImageMirroring, ImageMode, ImageRotation};

/// HID usage page exposed by the vendor (control) interface of every model.
const USAGE_PAGE: u16 = 65440; // 0xFFA0
const USAGE_ID: u16 = 1;

const AJAZZ_VID: u16 = 0x0300;
const N3_VID: u16 = 0x6602;
const MIRABOX_VID: u16 = 0x6603;
const SOOMFON_VID: u16 = 0x1500;

const AKP03_PID: u16 = 0x1001;
const AKP03E_PID: u16 = 0x1002;
const AKP03R_PID: u16 = 0x1003;
const AKP03E_REV2_PID: u16 = 0x3002;
const N3_PID: u16 = 0x1002;
const N3EN_PID: u16 = 0x1003;
const SOOMFON_SE_PID: u16 = 0x3001;

/// Every model in this family is a 3x3 grid: six LCD keys plus three plain keys.
const KEY_COUNT: u8 = 9;
/// The first six keys (the top two rows) are LCD screens; the bottom row is
/// plain push buttons with nothing to draw on.
const LCD_KEY_COUNT: u8 = 6;
/// ...with three rotary encoders / knobs.
const ENCODER_COUNT: u8 = 3;
/// Side length, in pixels, of each square LCD key.
const KEY_IMAGE_DIM: u32 = 60;

/// A specific, supported device model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Akp03,
    Akp03E,
    Akp03R,
    Akp03Erev2,
    N3,
    N3En,
    SoomfonSe,
}

/// Every supported model, in query order.
pub const ALL: [Kind; 7] = [
    Kind::Akp03,
    Kind::Akp03E,
    Kind::Akp03R,
    Kind::Akp03Erev2,
    Kind::N3,
    Kind::N3En,
    Kind::SoomfonSe,
];

impl Kind {
    /// Resolve a model from its USB vendor/product ids, if supported.
    pub const fn from_ids(vid: u16, pid: u16) -> Option<Self> {
        match (vid, pid) {
            (AJAZZ_VID, AKP03_PID) => Some(Self::Akp03),
            (AJAZZ_VID, AKP03E_PID) => Some(Self::Akp03E),
            (AJAZZ_VID, AKP03R_PID) => Some(Self::Akp03R),
            (AJAZZ_VID, AKP03E_REV2_PID) => Some(Self::Akp03Erev2),
            (N3_VID, N3_PID) => Some(Self::N3),
            (MIRABOX_VID, N3EN_PID) => Some(Self::N3En),
            (SOOMFON_VID, SOOMFON_SE_PID) => Some(Self::SoomfonSe),
            _ => None,
        }
    }

    const fn ids(self) -> (u16, u16) {
        match self {
            Self::Akp03 => (AJAZZ_VID, AKP03_PID),
            Self::Akp03E => (AJAZZ_VID, AKP03E_PID),
            Self::Akp03R => (AJAZZ_VID, AKP03R_PID),
            Self::Akp03Erev2 => (AJAZZ_VID, AKP03E_REV2_PID),
            Self::N3 => (N3_VID, N3_PID),
            Self::N3En => (MIRABOX_VID, N3EN_PID),
            Self::SoomfonSe => (SOOMFON_VID, SOOMFON_SE_PID),
        }
    }

    /// Human-readable model name. The USB stack only reports generic OEM
    /// strings, so we label devices ourselves.
    pub const fn model(self) -> &'static str {
        match self {
            Self::Akp03 => "Ajazz AKP03",
            Self::Akp03E => "Ajazz AKP03E",
            Self::Akp03R => "Ajazz AKP03R",
            Self::Akp03Erev2 => "Ajazz AKP03E (rev. 2)",
            Self::N3 => "Mirabox N3",
            Self::N3En => "Mirabox N3EN",
            Self::SoomfonSe => "Soomfon Stream Controller SE",
        }
    }

    /// Number of physical keys.
    pub const fn key_count(self) -> u8 {
        KEY_COUNT
    }

    /// Number of rotary encoders / knobs.
    pub const fn encoder_count(self) -> u8 {
        ENCODER_COUNT
    }

    /// Whether the six top keys are individually addressable LCD screens.
    pub const fn has_key_screens(self) -> bool {
        true
    }

    /// Number of keys that are backed by an LCD screen. These are always the
    /// first keys (row-major), so a key index is drawable iff it is less than
    /// this count.
    pub const fn lcd_key_count(self) -> u8 {
        LCD_KEY_COUNT
    }

    /// Pixel dimensions `(width, height)` of a single LCD key.
    pub const fn key_image_size(self) -> (u32, u32) {
        (KEY_IMAGE_DIM, KEY_IMAGE_DIM)
    }

    /// The framing the firmware expects for a key image: a 60x60 JPEG. Some
    /// models wire their panels rotated, so the bitmap is pre-rotated to land
    /// upright on the physical screen.
    pub fn image_format(self) -> ImageFormat {
        let rotation = match self {
            Self::Akp03Erev2 | Self::N3En | Self::SoomfonSe => ImageRotation::Rot90,
            Self::Akp03 | Self::Akp03E | Self::Akp03R | Self::N3 => ImageRotation::Rot0,
        };

        ImageFormat {
            mode: ImageMode::JPEG,
            size: (KEY_IMAGE_DIM as usize, KEY_IMAGE_DIM as usize),
            rotation,
            mirror: ImageMirroring::None,
        }
    }

    /// Whether the device emits separate press *and* release reports rather than
    /// a single toggle. The input layer needs this to report key-up correctly.
    pub const fn supports_both_states(self) -> bool {
        matches!(self, Self::N3En | Self::Akp03Erev2 | Self::SoomfonSe)
    }

    /// Whether the device speaks the v2 framing of the OEM protocol. Every model
    /// supported so far does.
    pub const fn is_v2(self) -> bool {
        true
    }

    /// The vendor-interface query that matches this model during enumeration.
    pub fn query(self) -> DeviceQuery {
        let (vid, pid) = self.ids();
        DeviceQuery::new(USAGE_PAGE, USAGE_ID, vid, pid)
    }
}

/// Build the HID queries that match every supported model.
pub(crate) fn queries() -> Vec<DeviceQuery> {
    ALL.iter().map(|kind| kind.query()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_the_users_soomfon_se() {
        assert_eq!(Kind::from_ids(0x1500, 0x3001), Some(Kind::SoomfonSe));
    }

    #[test]
    fn unknown_ids_are_rejected() {
        assert_eq!(Kind::from_ids(0x1234, 0x5678), None);
    }

    #[test]
    fn from_ids_round_trips_with_ids() {
        for kind in ALL {
            let (vid, pid) = kind.ids();
            assert_eq!(Kind::from_ids(vid, pid), Some(kind));
        }
    }

    #[test]
    fn se_reports_both_states() {
        assert!(Kind::SoomfonSe.supports_both_states());
        assert!(!Kind::Akp03.supports_both_states());
    }

    #[test]
    fn one_query_per_model() {
        assert_eq!(queries().len(), ALL.len());
    }

    #[test]
    fn six_of_the_nine_keys_have_screens() {
        assert_eq!(Kind::SoomfonSe.lcd_key_count(), 6);
        assert!(Kind::SoomfonSe.lcd_key_count() < Kind::SoomfonSe.key_count());
    }

    #[test]
    fn se_panel_image_is_rotated_jpeg() {
        let fmt = Kind::SoomfonSe.image_format();
        assert!(matches!(fmt.mode, ImageMode::JPEG));
        assert_eq!(fmt.size, (60, 60));
        assert!(matches!(fmt.rotation, ImageRotation::Rot90));
    }

    #[test]
    fn plain_jpeg_models_are_not_rotated() {
        assert!(matches!(
            Kind::Akp03.image_format().rotation,
            ImageRotation::Rot0
        ));
    }
}

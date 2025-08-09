use crate::prelude::*;

use core::fmt::Write;

use qrcode::{
    render::{Canvas, Pixel},
    types::Color as ModuleColor,
    QrCode,
};

use crate::utils::CCStr;

#[derive(Debug, Clone, PartialEq, Eq)]
struct QRCodeSvg {
    width: u32,
    path: String,
}
impl Canvas for QRCodeSvg {
    type Pixel = DummyPixel;

    type Image = Self;

    fn new(width: u32, _height: u32, _dark_pixel: Self::Pixel, _light_pixel: Self::Pixel) -> Self {
        Self {
            width,
            path: String::new(),
        }
    }

    fn draw_dark_pixel(&mut self, _x: u32, _y: u32) {
        unreachable!("Never called")
    }
    fn draw_dark_rect(&mut self, left: u32, top: u32, _width: u32, _height: u32) {
        write!(self.path, "M{left} {top}h1v1H{left}Z").unwrap();
    }

    fn into_image(self) -> Self::Image {
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DummyPixel;
impl Pixel for DummyPixel {
    type Image = QRCodeSvg;
    type Canvas = QRCodeSvg;

    fn default_color(color: ModuleColor) -> Self {
        color.select(Self, Self)
    }
    fn default_unit_size() -> (u32, u32) {
        (1, 1)
    }
}

#[component]
pub fn QRCode(data: CCStr) -> Element {
    let qr_code = QrCode::new(data.as_bytes())
        .unwrap()
        .render::<DummyPixel>()
        .quiet_zone(false)
        .build();

    let width = qr_code.width;

    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            view_box: "0 0 {width} {width}",
            path { fill: "currentcolor", d: qr_code.path.as_str() }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UIQRCode(CCStr);
impl LoadedElement for UIQRCode {
    type Loader = SkeletonLoader;
    #[inline(always)]
    fn element<M: LoadedComponentInputMapper>(self, _m: M) -> Element {
        rsx! {
            div { class: "w-80",
                QRCode { data: self.0 }
            }
        }
    }

    fn place_holder() -> Self {
        Self::from("BITCOIN:TB1PRDSU09F3PEYZK8YRUPYW2VN73ZN7KJDA9VC3CEJFHAKLJH8C2XCSUC6ZKQ")
    }
}
impl<T> From<T> for UIQRCode
where
    CCStr: From<T>,
{
    fn from(value: T) -> Self {
        Self(CCStr::from(value))
    }
}

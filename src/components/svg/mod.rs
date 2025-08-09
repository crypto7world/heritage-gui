use crate::prelude::*;

mod svgs;
pub use svgs::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(unused)]
pub enum SvgSize {
    Full,
    Custom(&'static str),
    Size2,
    Size3,
    Size4,
    Size5,
    #[default]
    Size6,
    Size7,
    Size8,
    Size9,
    Size10,
}
impl SvgSize {
    fn class(self) -> &'static str {
        match self {
            Self::Full => "size-full",
            Self::Custom(s) => s,
            Self::Size2 => "size-2",
            Self::Size3 => "size-3",
            Self::Size4 => "size-4",
            Self::Size5 => "size-5",
            Self::Size6 => "size-6",
            Self::Size7 => "size-7",
            Self::Size8 => "size-8",
            Self::Size9 => "size-9",
            Self::Size10 => "size-10",
        }
    }
}

pub trait DrawableSvg: 'static {
    fn path() -> &'static str;

    fn view_box() -> &'static str {
        "0 0 24 24"
    }
}

#[doc = "Properties for the [`Svg`] component."]
#[allow(missing_docs)]
#[derive(Props, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub struct DrawSvgProps {
    pub base_class: Option<&'static str>,
    pub size: Option<SvgSize>,
}

/// # Props
/// *For details, see the [props struct definition](SvgProps).*
/// - [`size`](SvgProps::size) : `Option<SvgSize>`
#[allow(non_snake_case)]
pub fn DrawSvg<S: DrawableSvg>(DrawSvgProps { base_class, size }: DrawSvgProps) -> Element {
    let base_class = base_class.unwrap_or("fill-current");
    let size_class = size.unwrap_or_default().class();
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            view_box: S::view_box(),
            class: "{base_class} {size_class}",
            path { d: S::path() }
        }
    }
}

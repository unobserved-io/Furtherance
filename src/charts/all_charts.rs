use plotters::style::{
    full_palette::{BLACK, WHITE},
    RGBColor,
};

pub fn light_dark_color() -> RGBColor {
    match dark_light::detect() {
        Ok(mode) => match mode {
            dark_light::Mode::Light | dark_light::Mode::Unspecified => BLACK,
            dark_light::Mode::Dark => WHITE,
        },
        Err(_) => BLACK,
    }
}

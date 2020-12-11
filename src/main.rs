use gdk::prelude::*;
use gdk::WindowTypeHint;
use gdk_pixbuf::{Colorspace, Pixbuf};
use gio::prelude::*;
use glib::idle_add_local;
use gtk::{Application, ApplicationWindow, Image};
use gtk::prelude::*;
use num::Complex;
use palette::{Gradient, LinSrgb};
use rayon::prelude::*;

struct Color { red: u8, grn: u8, blu: u8 }

const WIDTH: i32 = 960;
const HEIGHT: i32 = 640;
const BLACK: Color = Color { red: 0, grn: 0, blu: 0 };

fn to_lin_srgb(rgb: u32) -> LinSrgb {
    return LinSrgb::new((rgb >> 16 & 255) as f32, (rgb >> 8 & 255) as f32, (rgb & 255) as f32);
}

fn generate_colors() -> Vec<Color> {
    let colors: Vec<Color> = Gradient::new(vec![
        to_lin_srgb(0x0AFC84), to_lin_srgb(0x3264F0), to_lin_srgb(0xE63C14),
        to_lin_srgb(0xE6AA00), to_lin_srgb(0xAFAF0A), to_lin_srgb(0x5A0032),
        to_lin_srgb(0xB45A78), to_lin_srgb(0xFF1428), to_lin_srgb(0x1E46C8),
        to_lin_srgb(0x0AFC84)
    ]).take(256).map(|rgb| {
        Color { red: rgb.red as u8, grn: rgb.green as u8, blu: rgb.blue as u8 }
    }).collect();
    colors
}

fn evaluate_point(iter: i32, a: f64, b: f64) -> i32 {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut i = 0;
    while i < iter {
        let z = x * x;
        let w = y * y;
        if z + w >= 4.0 { return i; }
        x = 2.0 * x * y + a;
        y = w - z + b;
        i += 1;
    }
    i
}

fn generate_fractal(width: usize, upper_left: Complex<f64>, pixels: &mut [i16], max_iterations: i32, dw: f64, dh: f64) {
    let bands: Vec<(usize, &mut [i16])> = pixels.chunks_mut(width).enumerate().collect();
    bands.into_par_iter().for_each(|(i, band)| {
        let a = upper_left.im - dh * i as f64;
        for col in 0..width {
            let result = evaluate_point(max_iterations, a, upper_left.re + dw * col as f64);
            band[col] = if result < max_iterations { (result & 255) as i16 } else { -1 };
        }
    });
}

unsafe fn update_pixels(pix_buf: &Pixbuf, colors: &Vec<Color>, pixels: &[i16]) {
    let dest = pix_buf.get_pixels();
    for mem in 0..pixels.len() {
        let index = pixels[mem];
        let color = if index < 0 { &BLACK } else { &colors[index as usize] };
        dest[mem * 3 + 0] = color.red;
        dest[mem * 3 + 1] = color.grn;
        dest[mem * 3 + 2] = color.blu;
    }
}

fn fractal_zoom(pix_buf: Pixbuf, image: Image, window: ApplicationWindow) {
    let colors = generate_colors();
    let size = (WIDTH as usize, HEIGHT as usize);
    let mut minimum = Complex { re: -2.25, im: -1.0 };
    let mut maximum = Complex { re: 0.75, im: 1.0 };
    let mut pixels: [i16; (WIDTH * HEIGHT) as usize] = [0; (WIDTH * HEIGHT) as usize];
    let mut max_iterations = 1;

    idle_add_local(move || unsafe {
        minimum.re += (-0.743643887037151 - minimum.re) / 50.0;
        minimum.im += (0.131825904205330 - minimum.im) / 50.0;
        maximum.re += (-0.743643887037151 - maximum.re) / 50.0;
        maximum.im += (0.131825904205330 - maximum.im) / 50.0;
        max_iterations += 2;

        let dw = (maximum.re - minimum.re) / size.0 as f64;
        let dh = (minimum.im - maximum.im) / size.1 as f64;
        generate_fractal(size.0, minimum, &mut pixels, max_iterations, dw, dh);

        update_pixels(&pix_buf, &colors, &pixels);
        image.set_from_pixbuf(Option::Some(&pix_buf));

        return Continue(window.is_visible());
    });
}

fn main() {
    let application = Application::new(
        Some("com.zynaps.rust.mandelbrot-zoom"),
        Default::default()).expect("failed to initialize GTK application"
    );

    application.connect_activate(|app| {
        let window = ApplicationWindow::new(app);
        window.set_title("Mandelbrot Zoom");
        window.set_default_size(WIDTH, HEIGHT);
        window.set_resizable(false);
        window.set_type_hint(WindowTypeHint::Dialog);
        window.move_((gdk::Screen::width() - WIDTH) / 2, (gdk::Screen::height() - HEIGHT) / 2);

        let pix = Pixbuf::new(Colorspace::Rgb, false, 8, WIDTH, HEIGHT).expect("failed to create pixel buffer");
        let img = Image::from_pixbuf(Option::Some(&pix));

        window.add(&img);
        window.show_all();

        fractal_zoom(pix, img, window);
    });

    application.run(&[]);
}

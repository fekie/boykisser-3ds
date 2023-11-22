/// Bitmap Graphics example.
///
/// This example uses the CPU to render a simple bitmap image to the screen.
use ctru::prelude::*;
use ctru::services::gfx::{Flush, Screen, Swap};

/// Ferris image taken from <https://rustacean.net> and scaled down to 320x240px.
/// To regenerate the data, you will need to install `imagemagick` and run this
/// command from the `examples` directory:
///
/// ```sh
/// magick assets/ferris.png -channel-fx "red<=>blue" -rotate 90 assets/ferris.rgb
/// ```
///
/// This creates an image appropriate for the default frame buffer format of
/// [`Bgr8`](ctru::services::gspgpu::FramebufferFormat::Bgr8)
/// and rotates the image 90° to account for the portrait mode screen.
static IMAGE: &[u8] = include_bytes!("../assets/boykisser.rgb");

fn flipped_image_over_y_axis() -> Vec<u8> {
    // What we do is take the rows (which are the columns as the image is rotated 90 degrees)
    // and reverse them.

    // We take split the pixels into chunks of 240, and then reverse the chunk order.
    let mut column_buffer: Vec<u8> = Vec::with_capacity(240 * 3);

    let mut columns: Vec<Vec<u8>> = Vec::with_capacity(240 * 3);

    for pixel in IMAGE.chunks(3) {
        column_buffer.extend_from_slice(pixel);

        if column_buffer.len() == 240 * 3 {
            columns.push(column_buffer);
            column_buffer = Vec::with_capacity(240 * 3);
        }
    }

    let reconstructed_image: Vec<u8> = columns.into_iter().rev().flatten().collect();

    reconstructed_image
}

fn main() {
    ctru::use_panic_handler();

    let gfx = Gfx::new().expect("Couldn't obtain GFX controller");
    let mut hid = Hid::new().expect("Couldn't obtain HID controller");
    let apt = Apt::new().expect("Couldn't obtain APT controller");
    let _console = Console::new(gfx.top_screen.borrow_mut());

    println!("\x1b[21;4HPress A to flip the image over the x-axis.");
    println!("\x1b[22;4HPress B to flip the image over the y-axis.");
    println!("\x1b[29;16HPress Start to exit");

    let mut bottom_screen = gfx.bottom_screen.borrow_mut();

    // We don't need double buffering in this example.
    // In this way we can draw our image only once on screen.
    bottom_screen.set_double_buffering(false);
    // Swapping buffers commits the change from the line above.
    bottom_screen.swap_buffers();

    // 3 bytes per pixel, we just want to reverse the pixels but not individual bytes
    let flipped_image_over_x_axis: Vec<_> = IMAGE.chunks(3).rev().flatten().copied().collect(); // We start with the original image.

    // We now flip the image over the y-axis.
    let flipped_image_over_y_axis: Vec<_> = flipped_image_over_y_axis();

    let mut image_bytes = IMAGE;

    // We assume the image is the correct size already, so we drop width + height.
    let frame_buffer = bottom_screen.raw_framebuffer();

    // We copy the image to the framebuffer.
    unsafe {
        frame_buffer
            .ptr
            .copy_from(image_bytes.as_ptr(), image_bytes.len());
    }

    while apt.main_loop() {
        hid.scan_input();

        if hid.keys_down().contains(KeyPad::START) {
            break;
        }

        if hid.keys_down().contains(KeyPad::A) {
            image_bytes = if std::ptr::eq(image_bytes, IMAGE) {
                &flipped_image_over_y_axis[..]
            } else {
                IMAGE
            };

            let frame_buffer = bottom_screen.raw_framebuffer();

            // We render the newly switched image to the framebuffer.
            unsafe {
                frame_buffer
                    .ptr
                    .copy_from(image_bytes.as_ptr(), image_bytes.len());
            }
        }

        // Flush framebuffers. Since we're not using double buffering,
        // this will render the pixels immediately
        bottom_screen.flush_buffers();

        gfx.wait_for_vblank();
    }
}

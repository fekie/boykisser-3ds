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

enum ImageState {
    Original,
    FlippedVertically,
    FlippedHorizontally,
    FlippedVerticallyAndHorizontally,
}

// fn set_bottom_screen_image(image_state: &ImageState, bottom_screen: &mut dyn Screen) {}

/// If the A button is pressed, we change the picture state.
/// If the picture state is in the original state, we change it to the FlippedVetically
/// state and vice versa. If the picture state is in the FlippedHorizontally state, we
/// change it to the FlippedVerticallyAndHorizontally state and vice versa.
fn keypad_handle_if_a_pressed(hid: &Hid, image_state: &mut ImageState) {
    // If the A button is not pressed, we don't do anything.
    if !hid.keys_down().contains(KeyPad::A) {
        return;
    }

    // If the A button is pressed, we change the picture state.
    *image_state = match image_state {
        ImageState::Original => ImageState::FlippedVertically,
        ImageState::FlippedVertically => ImageState::Original,
        ImageState::FlippedHorizontally => ImageState::FlippedVerticallyAndHorizontally,
        ImageState::FlippedVerticallyAndHorizontally => ImageState::FlippedHorizontally,
    };
}

/// If the B button is pressed, we change the picture state.
/// If the picture state is in the original state, we change it to the FlippedHorizontally
/// state and vice versa. If the picture state is in the FlippedVertically state, we
/// change it to the FlippedVerticallyAndHorizontally state and vice versa.
fn keypad_handle_if_b_pressed(hid: &Hid, image_state: &mut ImageState) {
    // If the B button is not pressed, we don't do anything.
    if !hid.keys_down().contains(KeyPad::B) {
        return;
    }

    // If the B button is pressed, we change the picture state.
    *image_state = match image_state {
        ImageState::Original => ImageState::FlippedHorizontally,
        ImageState::FlippedVertically => ImageState::FlippedVerticallyAndHorizontally,
        ImageState::FlippedHorizontally => ImageState::Original,
        ImageState::FlippedVerticallyAndHorizontally => ImageState::FlippedVertically,
    };
}

/// Converts the image state to a reference to the image bytes.
fn image_state_to_image_bytes<'a>(
    image_state: &'a ImageState,
    image_flipped_vertically: &'a [u8],
    image_flipped_horizontally: &'a [u8],
    image_flipped_vertically_and_horizontally: &'a [u8],
) -> &'a [u8] {
    match image_state {
        ImageState::Original => IMAGE,
        ImageState::FlippedVertically => &image_flipped_vertically[..],
        ImageState::FlippedHorizontally => &image_flipped_horizontally[..],
        ImageState::FlippedVerticallyAndHorizontally => {
            &image_flipped_vertically_and_horizontally[..]
        }
    }
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

    let image_flipped_vertically: Vec<u8> = IMAGE.chunks(3).rev().flatten().copied().collect(); // We start with the original image.

    // What we do is take the rows (which are the columns as the image is rotated 90 degrees)
    // and reverse them.
    let image_flipped_horizontally: Vec<u8> =
        IMAGE.chunks(240 * 3).rev().flatten().copied().collect();

    let image_flipped_vertically_and_horizontally: Vec<u8> = image_flipped_horizontally
        .chunks(3)
        .rev()
        .flatten()
        .copied()
        .collect();

    let mut image_bytes = IMAGE;

    let mut image_state = ImageState::Original;

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

        keypad_handle_if_a_pressed(&hid, &mut image_state);
        keypad_handle_if_b_pressed(&hid, &mut image_state);

        let image_bytes = image_state_to_image_bytes(
            &image_state,
            &image_flipped_vertically,
            &image_flipped_horizontally,
            &image_flipped_vertically_and_horizontally,
        );

        let frame_buffer = bottom_screen.raw_framebuffer();

        // We render the newly switched image to the framebuffer.
        unsafe {
            frame_buffer
                .ptr
                .copy_from(image_bytes.as_ptr(), image_bytes.len());
        }

        /* if hid.keys_down().contains(KeyPad::A) {
            image_bytes = if std::ptr::eq(image_bytes, IMAGE) {
                &flipped_image_over_x_axis[..]
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

        if hid.keys_down().contains(KeyPad::B) {
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
        } */

        // Flush framebuffers. Since we're not using double buffering,
        // this will render the pixels immediately
        bottom_screen.flush_buffers();

        gfx.wait_for_vblank();
    }
}

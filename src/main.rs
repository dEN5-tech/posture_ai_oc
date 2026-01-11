#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hides console in release mode

/// Posture AI - Main application entry point with system tray integration
///
/// This application uses a camera and MoveNet model to detect posture
/// and provide visual feedback when bad posture is detected.

use anyhow::Result;
use image::imageops::FilterType;
use minifb::{Key, Window, WindowOptions};
use ndarray::Array4;
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};
use ort::{
    session::{builder::GraphOptimizationLevel, Session},
    value::Value,
};

// Tray & Menu Dependencies
use tray_icon::{
    menu::{Menu, MenuItem, MenuEvent},
    TrayIconBuilder, Icon,
};

// Windows API Dependencies
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::*;

use posture_ai_oc::{blur_overlay::BlurOverlay, canvas::Canvas, config};

// Simple text drawing function for debug display
fn draw_text(canvas: &mut Canvas, text: &str, x: i32, y: i32, color: u32) {
    // Very basic text rendering - in a real app, use a proper font renderer
    for (i, _) in text.chars().enumerate() {
        // This is a placeholder - would need proper font rendering for production
        // For now, just draw a small rectangle for each character
        let char_x = x + i as i32 * 8;
        for dy in 0..8 {
            for dx in 0..6 {
                canvas.plot(char_x + dx, y + dy, color);
            }
        }
    }
}

fn main() -> Result<()> {
    // 1. Initialize the Overlay (Hidden at start)
    let mut overlay = BlurOverlay::new()?;

    // 2. Setup System Tray
    let tray_menu = Menu::new();
    let toggle_item = MenuItem::new("Show/Hide Debug Window", true, None);
    let quit_item = MenuItem::new("Quit Posture AI", true, None);
    tray_menu.append(&toggle_item)?;
    tray_menu.append(&quit_item)?;

    // Create a simple green icon 32x32
    let icon_rgba = vec![0u8; 32 * 32 * 4].into_iter().enumerate().map(|(i, _)| {
        if i % 4 == 1 { 255 } else if i % 4 == 3 { 255 } else { 0 } // Green, Alpha 255
    }).collect::<Vec<u8>>();

    let tray_icon_obj = Icon::from_rgba(icon_rgba, 32, 32)?;
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Posture AI Running")
        .with_icon(tray_icon_obj)
        .build()?;

    // 3. Load AI & Camera
    println!("Loading MoveNet Thunder...");
    let mut model = Session::builder()?
        .with_optimization_level(GraphOptimizationLevel::Level3)?
        .commit_from_file("movenet_singlepose_thunder.onnx")?;

    println!("Opening Camera...");
    let index = CameraIndex::Index(0);
    let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
    let mut camera = Camera::new(index, requested)?;
    camera.open_stream()?;

    // 4. Create Debug Window
    let mut window = Window::new(
        "Posture AI - Monitor (Check System Tray)",
        config::WIDTH,
        config::HEIGHT,
        WindowOptions::default(),
    )?;
    window.set_target_fps(30);

    // Get Raw Handle to allow Hiding/Showing via Windows API
    let raw_window_handle = window.get_window_handle();
    let debug_hwnd = unsafe { std::mem::transmute::<_, HWND>(raw_window_handle) };

    let mut good_posture_baseline: Option<f32> = None;
    let mut buffer: Vec<u32> = vec![0; config::WIDTH * config::HEIGHT];
    let mut bad_posture_counter = 0;

    // Window Visibility State
    let mut is_debug_visible = true;

    println!("Running... Minimize to tray using the Tray Icon.");

    // MAIN LOOP
    // Note: We check `overlay.hwnd` validity because Minifb might close,
    // but we want to keep running if hidden.
    loop {
        // --- A. Handle Tray Events ---
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == quit_item.id() {
                println!("Quitting...");
                break;
            } else if event.id == toggle_item.id() {
                is_debug_visible = !is_debug_visible;
                unsafe {
                    if is_debug_visible {
                        ShowWindow(debug_hwnd, SW_SHOW);
                    } else {
                        ShowWindow(debug_hwnd, SW_HIDE);
                    }
                }
            }
        }

        // Also quit if Debug Window is open and ESC is pressed
        if is_debug_visible && !window.is_open() {
            // If user clicked X on the window, we treat it as Hide (Minimize to tray)
            // instead of Quit, to keep the service running.
            println!("Window closed by user - Minimizing to tray.");
            is_debug_visible = false;
            // Minifb destroys the window on close, so we can't just 'Hide' it if it's already destroyed.
            // Limitation: Minifb doesn't support 'Minimize to Tray' natively well.
            // Workaround: We break here if window is destroyed.
            // To make it truly persistent requires a different GUI crate (winit).
            // For now: ESC or Menu->Quit exits. Closing window exits.
            break;
        }

        // --- B. AI Logic (Always Runs) ---
        let frame_buffer = camera.frame()?;
        let raw_frame = frame_buffer.decode_image::<RgbFormat>()?;

        // Apply camera rotation if needed (fixes upside-down cameras)
        let processed_frame = match config::CAMERA_ROTATION_DEGREES {
            180 => image::imageops::rotate180(&raw_frame),
            90 => image::imageops::rotate90(&raw_frame),
            270 => image::imageops::rotate270(&raw_frame),
            _ => raw_frame, // 0 degrees or any other value = no rotation
        };

        let model_input_img = image::imageops::resize(&processed_frame, config::MOVENET_SIZE, config::MOVENET_SIZE, FilterType::Triangle);

        let mut input_array = Array4::<i32>::zeros((1, config::MOVENET_SIZE as usize, config::MOVENET_SIZE as usize, 3));
        for (x, y, pixel) in model_input_img.enumerate_pixels() {
            let [r, g, b] = pixel.0;
            input_array[[0, y as usize, x as usize, 0]] = r as i32;
            input_array[[0, y as usize, x as usize, 1]] = g as i32;
            input_array[[0, y as usize, x as usize, 2]] = b as i32;
        }

        let input_value = Value::from_array(input_array)?;
        let outputs = model.run(ort::inputs![input_value])?;
        let (_, data_slice) = outputs["output_0"].try_extract_tensor::<f32>()?;

        // Logic
        let kp_idx = 2;
        let base_idx = kp_idx * 3;
        let mut current_eye_y: Option<f32> = None;

        if data_slice.len() > base_idx + 2 {
            let raw_y = data_slice[base_idx];
            let score = data_slice[base_idx + 2];
            if score > 0.3 {
                current_eye_y = Some(raw_y * config::HEIGHT as f32);
            }
        }

        // Posture Check - Only trigger when slouching down (positive delta)
        let mut is_currently_bad = false;
        if let Some(curr_y) = current_eye_y {
            if good_posture_baseline.is_none() {
                good_posture_baseline = Some(curr_y);
            }
            if let Some(baseline) = good_posture_baseline {
                let delta = curr_y - baseline;
                // Only trigger when slouching down (positive delta)
                if delta > config::GOOD_POSTURE_DEVIATION {
                    is_currently_bad = true;
                }
            }
        }

        if is_currently_bad { bad_posture_counter += 1; } else { bad_posture_counter = 0; }

        if bad_posture_counter > config::DEBOUNCE_FRAMES {
            overlay.set_target_visible(true);
        } else {
            overlay.set_target_visible(false);
        }
        overlay.update();

        // --- C. Reset Key ---
        // Only works if window is focused
        if is_debug_visible && window.is_key_down(Key::R) {
            good_posture_baseline = None;
            bad_posture_counter = 0;
            println!("Posture Reset!");
        }

        // --- D. Update Debug Window (Only if visible) ---
        if is_debug_visible {
            let display_img = image::imageops::resize(&processed_frame, config::WIDTH as u32, config::HEIGHT as u32, FilterType::Triangle);

            for (i, pixel) in display_img.pixels().enumerate() {
                let [r, g, b] = pixel.0;
                buffer[i] = posture_ai_oc::canvas::from_u8_rgb(r, g, b);
            }

            if let (Some(curr_y), Some(baseline)) = (current_eye_y, good_posture_baseline) {
                let mut canvas = Canvas { buffer: &mut buffer, width: config::WIDTH, height: config::HEIGHT };

                // Draw baseline (white line)
                canvas.draw_line(0, baseline as i32, config::WIDTH as i32, baseline as i32, 0xFFFFFFFF);

                // Draw current position with color coding
                let delta = curr_y - baseline;
                let color = if delta > config::GOOD_POSTURE_DEVIATION {
                    // Red: Bad posture (slouching)
                    0xFFFF0000
                } else if delta > 0.0 {
                    // Yellow: Approaching bad posture
                    0xFFFFFF00
                } else {
                    // Green: Good posture
                    0xFF00FF00
                };

                canvas.draw_line(0, curr_y as i32, config::WIDTH as i32, curr_y as i32, color);

                // Draw threshold boundaries
                let good_upper_bound = baseline + config::GOOD_POSTURE_DEVIATION;
                let good_lower_bound = baseline - config::GOOD_POSTURE_DEVIATION;

                // Draw threshold lines (semi-transparent)
                canvas.draw_line(0, good_upper_bound as i32, config::WIDTH as i32, good_upper_bound as i32, 0x80FFFFFF);
                canvas.draw_line(0, good_lower_bound as i32, config::WIDTH as i32, good_lower_bound as i32, 0x80FFFFFF);

                // Draw status text
                if bad_posture_counter > config::DEBOUNCE_FRAMES {
                    draw_text(&mut canvas, "BAD POSTURE", 10, 10, 0xFFFF0000);
                    draw_text(&mut canvas, &format!("Delta: {:.1}px", delta), 10, 30, 0xFFFFFFFF);
                } else {
                    draw_text(&mut canvas, "Good Posture", 10, 10, 0xFF00FF00);
                    draw_text(&mut canvas, &format!("Delta: {:.1}px", delta), 10, 30, 0xFFFFFFFF);
                }
            }

            window.update_with_buffer(&buffer, config::WIDTH, config::HEIGHT)?;
        } else {
            // Important: We must still update the window pump even if hidden/not drawing
            // to keep the application responsive to OS messages.
            window.update();
        }
    }

    Ok(())
}

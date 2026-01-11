# Posture AI - Open Camera

![Posture AI Logo](https://dl.dropboxusercontent.com/scl/fi/zsanv1h36iuaxtwkoqzxd/Generated_Image_January_11__2026_-_6_34PM-removebg-preview.png?rlkey=erqh1sb0f2xe4kz875xp9xe1b&dl=0) <!-- Replace with actual logo -->

A real-time posture detection application that uses computer vision to monitor your posture and provide visual feedback when you slouch.

## 📋 Overview

Posture AI is a desktop application that runs in the background and uses your webcam to detect when you're slouching. When bad posture is detected, it shows a visual overlay to remind you to sit up straight.

## 🚀 Features

- **Real-time Posture Detection**: Uses MoveNet Thunder model for accurate pose estimation
- **System Tray Integration**: Runs in the background with easy access via system tray
- **Visual Feedback**: Shows a blur overlay when bad posture is detected
- **Debug Window**: Optional debug window to see posture analysis in real-time
- **Configurable**: Adjustable sensitivity and detection parameters

## 📸 Screenshots

<!-- Add screenshots here -->
![Debug Window](https://dl.dropboxusercontent.com/scl/fi/h70wpj1ruf7az8yevqmkk/posture_ai_oc_wzP72xcYBU.png?rlkey=wry2kkvfdrr88fs808mlohvpx&dl=0) <!-- Replace with actual screenshot -->
![System Tray](https://dl.dropboxusercontent.com/scl/fi/9xlshikhhy8pdki6xh1ph/UFY5cznJe9.png?rlkey=h9fd4yauqoqqtxhdpb1lu1vnx&dl=0) <!-- Replace with actual screenshot -->

## 🛠️ Requirements

- Windows 10/11 (currently Windows-only due to system tray implementation)
- Webcam
- Rust 1.60+ (for building from source)

## 📦 Installation

### Pre-built Binaries

Download the latest release from the [Releases page](https://github.com/dEN5-tech/posture_ai_oc/releases).

### From Source

```bash
# Clone the repository
git clone https://github.com/dEN5-tech/posture_ai_oc.git
cd posture_ai_oc

# Build the application
cargo build --release

# Run the application
cargo run --release
```

## 🎯 Usage

1. Launch the application
2. The app will run in the background with a system tray icon
3. When you slouch, a visual overlay will appear to remind you
4. Use the system tray menu to:
   - Show/hide the debug window
   - Quit the application

### Keyboard Shortcuts

- **R**: Reset posture baseline (when debug window is focused)

## 🔧 Configuration

Edit the configuration in `src/config.rs`:

```rust
// Camera settings
pub const WIDTH: usize = 640;
pub const HEIGHT: usize = 480;

// AI Model settings
pub const MOVENET_SIZE: u32 = 192;

// Posture detection settings
pub const GOOD_POSTURE_DEVIATION: f32 = 20.0; // Pixels from baseline
pub const DEBOUNCE_FRAMES: i32 = 15; // Frames before triggering overlay

// Camera rotation (0, 90, 180, 270 degrees)
pub const CAMERA_ROTATION_DEGREES: u32 = 0;
```

## 📂 Project Structure

```
posture_ai_oc/
├── Cargo.toml          # Rust dependencies and configuration
├── build.rs            # Build script
├── movenet_singlepose_thunder.onnx  # AI model
├── src/
│   ├── main.rs         # Main application entry point
│   ├── lib.rs          # Library module
│   ├── config.rs       # Configuration constants
│   ├── canvas.rs       # Canvas drawing utilities
│   ├── blur_overlay.rs # Blur overlay implementation
│   └── ...             # Other modules
└── README.md           # This file
```

## 🔬 How It Works

1. **Camera Capture**: The application captures video frames from your webcam
2. **Pose Estimation**: Uses MoveNet Thunder model to detect key points in your body
3. **Posture Analysis**: Tracks the position of your eyes relative to a baseline
4. **Feedback**: Shows visual overlay when you slouch below the threshold
5. **System Tray**: Provides easy access to controls without interrupting your workflow

## 📊 Technical Details

- **AI Model**: MoveNet SinglePose Thunder (ONNX format)
- **Camera**: Uses nokhwa for cross-platform camera access
- **GUI**: minifb for simple window rendering
- **System Tray**: tray-icon for background operation
- **Performance**: Optimized with ONNX Runtime and GPU acceleration

## 🤝 Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Commit your changes: `git commit -am 'Add some feature'`
4. Push to the branch: `git push origin feature/your-feature`
5. Create a pull request

## 🐛 Issues

Found a bug? Please [open an issue](https://github.com/dEN5-tech/posture_ai_oc/issues) with:

- Description of the problem
- Steps to reproduce
- Your system information (OS, webcam model, etc.)
- Screenshots if applicable

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Google for the MoveNet model
- ONNX Runtime team for the inference engine
- All the Rust crate authors for their amazing libraries

## 🔮 Future Plans

- [ ] Multi-platform support (macOS, Linux)
- [ ] Customizable overlay styles
- [ ] Posture statistics and history
- [ ] Multiple posture detection modes
- [ ] Audio notifications
- [ ] Integration with health apps

---

**Posture AI** - Because your health matters! 💙

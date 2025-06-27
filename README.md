# OP2MapViewerRust

A minimal, cross-platform prototype of the OP2MapViewer application written in Rust using [`eframe`](https://github.com/emilk/egui/tree/master/crates/eframe) (egui) for the GUI.

## Features

- Open and display map images (PNG, JPG, BMP) in a desktop window.
- Simple, modern GUI.
- Cross-platform (Windows, Linux, macOS).
- **Note:** This prototype currently loads standard image files as a placeholder for Outpost 2 map rendering. Actual `.map` file parsing and rendering will be implemented in future versions.

## Usage

1. **Build and Run:**

   ```sh
   cargo run --release
   ```

2. **Open a Map Image:**
   - Use the "Open Map Image..." menu to select and display a PNG, JPG, or BMP file.

3. **Quit:**
   - Use the "Quit" menu option to close the application.

## Dependencies

- [eframe/egui](https://crates.io/crates/eframe) - GUI framework
- [image](https://crates.io/crates/image) - Image loading and processing
- [rfd](https://crates.io/crates/rfd) - Native file dialogs

## Roadmap

- [ ] Parse and render Outpost 2 `.map` files
- [ ] Display tile and cell type overlays
- [ ] Export map as JPG or JSON
- [ ] Add zoom, pan, and grid overlay features
- [ ] Undo/redo for cell type editing

## License

MIT

---

**This project is a work in progress.**  
For questions or contributions, please open an issue or pull request!
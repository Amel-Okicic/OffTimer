use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const ICON_SIZES: [u32; 7] = [16, 24, 32, 48, 64, 128, 256];
    const RUNTIME_ICON_SIZE: u32 = 256;

    let svg_path = PathBuf::from("icons/OffTimerIcon.svg");
    println!("cargo:rerun-if-changed={}", svg_path.display());

    let svg_data = fs::read(&svg_path)?;
    let tree = resvg::usvg::Tree::from_data(
        &svg_data,
        &resvg::usvg::Options::default(),
    )?;

    let out_dir = PathBuf::from(
        env::var_os("OUT_DIR").ok_or("Cargo did not provide OUT_DIR")?,
    );
    let source_size = tree.size();
    let scale_base = source_size.width().max(source_size.height());

    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

    for size in ICON_SIZES {
        let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size)
            .ok_or("Could not allocate the application icon image")?;
        let scale = size as f32 / scale_base;

        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::from_scale(scale, scale),
            &mut pixmap.as_mut(),
        );

        // tiny-skia stores premultiplied alpha; egui and ICO expect straight RGBA.
        let rgba = pixmap.take_demultiplied();

        if size == RUNTIME_ICON_SIZE {
            fs::write(out_dir.join("offtimer_icon.rgba"), &rgba)?;
        }

        let image = ico::IconImage::from_rgba_data(size, size, rgba);
        icon_dir.add_entry(ico::IconDirEntry::encode(&image)?);
    }

    let ico_path = out_dir.join("offtimer.ico");
    icon_dir.write(fs::File::create(&ico_path)?)?;

    #[cfg(windows)]
    {
        let mut resources = winres::WindowsResource::new();
        resources.set_icon(
            ico_path
                .to_str()
                .ok_or("The generated icon path is not valid UTF-8")?,
        );
        resources.compile()?;
    }

    Ok(())
}

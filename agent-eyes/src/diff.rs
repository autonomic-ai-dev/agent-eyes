use anyhow::Result;
use image::GenericImageView;
use std::path::Path;

pub fn pixel_diff(reference: &Path, comparison: &Path, output: &Path) -> Result<()> {
    let ref_img = image::open(reference)?;
    let comp_img = image::open(comparison)?;

    let (rw, rh) = ref_img.dimensions();
    let (cw, ch) = comp_img.dimensions();

    let mut diff_img = image::RgbaImage::new(rw.max(cw), rh.max(ch));
    let mut diff_pixels = 0u64;
    let total_pixels = (rw.max(cw) * rh.max(ch)) as u64;

    for y in 0..rh.max(ch) {
        for x in 0..rw.max(cw) {
            let ref_px = ref_img.get_pixel(x.min(rw - 1), y.min(rh - 1));
            let comp_px = comp_img.get_pixel(x.min(cw - 1), y.min(ch - 1));

            if ref_px != comp_px {
                diff_pixels += 1;
                diff_img.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
            } else {
                diff_img.put_pixel(x, y, ref_px);
            }
        }
    }

    diff_img.save(output)?;

    let pct = if total_pixels > 0 {
        (diff_pixels as f64 / total_pixels as f64) * 100.0
    } else {
        0.0
    };

    let result = serde_json::json!({
        "diff_pixels": diff_pixels,
        "total_pixels": total_pixels,
        "diff_percent": format!("{:.2}", pct),
        "dimensions": {
            "reference": format!("{}x{}", rw, rh),
            "comparison": format!("{}x{}", cw, ch),
        },
        "diff_output": output.display().to_string(),
    });
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;

    fn write_test_image(width: u32, height: u32, color: [u8; 4], path: &Path) {
        let mut img = RgbaImage::new(width, height);
        for pixel in img.pixels_mut() {
            *pixel = image::Rgba(color);
        }
        img.save(path).unwrap();
    }

    #[test]
    fn test_identical_images() {
        let dir = std::env::temp_dir().join("agent_eyes_test");
        std::fs::create_dir_all(&dir).unwrap();
        let ref_path = dir.join("identical_ref.png");
        let comp_path = dir.join("identical_comp.png");
        let out_path = dir.join("identical_out.png");

        write_test_image(4, 4, [128, 128, 128, 255], &ref_path);
        std::fs::copy(&ref_path, &comp_path).unwrap();

        assert!(pixel_diff(&ref_path, &comp_path, &out_path).is_ok());
        assert!(out_path.exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_different_images() {
        let dir = std::env::temp_dir().join("agent_eyes_test");
        std::fs::create_dir_all(&dir).unwrap();
        let ref_path = dir.join("diff_ref.png");
        let comp_path = dir.join("diff_comp.png");
        let out_path = dir.join("diff_out.png");

        write_test_image(4, 4, [255, 0, 0, 255], &ref_path);
        write_test_image(4, 4, [0, 0, 255, 255], &comp_path);

        assert!(pixel_diff(&ref_path, &comp_path, &out_path).is_ok());

        let diff_img = image::open(&out_path).unwrap();
        assert_eq!(diff_img.dimensions(), (4, 4));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_different_dimensions() {
        let dir = std::env::temp_dir().join("agent_eyes_test");
        std::fs::create_dir_all(&dir).unwrap();
        let ref_path = dir.join("dims_ref.png");
        let comp_path = dir.join("dims_comp.png");
        let out_path = dir.join("dims_out.png");

        write_test_image(2, 4, [100, 100, 100, 255], &ref_path);
        write_test_image(4, 2, [100, 100, 100, 255], &comp_path);

        assert!(pixel_diff(&ref_path, &comp_path, &out_path).is_ok());

        let diff_img = image::open(&out_path).unwrap();
        assert_eq!(diff_img.dimensions(), (4, 4));

        let _ = std::fs::remove_dir_all(&dir);
    }
}

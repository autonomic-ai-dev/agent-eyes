//! pHash perceptual diff + ROI gate before VLM caption (Phase 9 P1).

use anyhow::Result;
use image::{DynamicImage, GenericImageView, Rgba};

pub const PHASH_HAMMING_SKIP: u32 = 6;

#[must_use]
pub fn phash64(img: &DynamicImage) -> u64 {
    let gray = img.to_luma8();
    let (w, h) = gray.dimensions();
    let mut samples = Vec::with_capacity(64);
    for y in 0..8 {
        for x in 0..8 {
            let sx = x * w / 8;
            let sy = y * h / 8;
            samples.push(gray.get_pixel(sx, sy).0[0] as u32);
        }
    }
    let avg = samples.iter().sum::<u32>() / 64;
    let mut hash = 0u64;
    for (i, v) in samples.iter().enumerate() {
        if *v >= avg {
            hash |= 1u64 << i;
        }
    }
    hash
}

#[must_use]
pub fn hamming(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CascadeDecision {
    SkipVlmReuseCaption,
    RunVlmOnRoi,
    RunVlmFullFrame,
}

#[must_use]
pub fn decide_cascade(prev_hash: Option<u64>, current_hash: u64, roi_changed: bool) -> CascadeDecision {
    if let Some(prev) = prev_hash {
        if hamming(prev, current_hash) <= PHASH_HAMMING_SKIP && !roi_changed {
            return CascadeDecision::SkipVlmReuseCaption;
        }
    }
    if roi_changed {
        CascadeDecision::RunVlmOnRoi
    } else {
        CascadeDecision::RunVlmFullFrame
    }
}

pub fn roi_changed(prev: &DynamicImage, current: &DynamicImage, threshold_pct: f64) -> Result<bool> {
    let (w, h) = prev.dimensions();
    let cur = current.resize_exact(w, h, image::imageops::FilterType::Nearest);
    let mut diff = 0u64;
    let total = (w as u64) * (h as u64);
    for y in 0..h {
        for x in 0..w {
            let a = prev.get_pixel(x, y);
            let b = cur.get_pixel(x, y);
            if pixel_delta(a, b) > 12 {
                diff += 1;
            }
        }
    }
    let pct = (diff as f64 / total as f64) * 100.0;
    Ok(pct >= threshold_pct)
}

fn pixel_delta(a: Rgba<u8>, b: Rgba<u8>) -> u16 {
    a.0.iter()
        .zip(b.0.iter())
        .map(|(x, y)| (*x as i16 - *y as i16).unsigned_abs() as u16)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;

    #[test]
    fn identical_images_share_phash() {
        let img = RgbaImage::from_pixel(32, 32, Rgba([10, 20, 30, 255]));
        let dyn_img = DynamicImage::ImageRgba8(img.clone());
        let h1 = phash64(&dyn_img);
        let h2 = phash64(&DynamicImage::ImageRgba8(img));
        assert_eq!(h1, h2);
        assert_eq!(decide_cascade(Some(h1), h2, false), CascadeDecision::SkipVlmReuseCaption);
    }
}

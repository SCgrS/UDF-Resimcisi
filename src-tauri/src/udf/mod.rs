//! UDF üretimi. **Saf modül**: dosya sistemi yok, Tauri bağımlılığı yok, `cargo test` ile
//! doğrudan sınanır.
//!
//! İşin özü: `.udf` dosyasını kendimiz yazdığımız için UDE'nin resmi belgeye alırken
//! uyguladığı yeniden örnekleme (≈72 DPI'a indirme) hiç çalışmaz. Bitmap ne ise o gömülür;
//! sayfaya sığdırma yalnızca `width`/`height` **punto** öznitelikleriyle yapılır.

pub mod model;
pub mod serialize;
pub mod zip;

use anyhow::{bail, Result};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;

use model::{Alignment, Block, Document, ImageRun, PageFormat, Paragraph, Run};

/// 1 santimetre kaç punto eder.
pub const PT_PER_CM: f64 = 28.3465;

/// Belgeye girecek tek bir resim: baytlar olduğu gibi gömülür.
#[derive(Debug, Clone)]
pub struct ImageSpec {
    /// Resmin ham baytları (PNG/JPEG). Yeniden kodlanmaz.
    pub bytes: Vec<u8>,
    pub px_w: u32,
    pub px_h: u32,
}

/// Görüntüleme boyutu kuralı.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sizing {
    /// Sayfaya sığdır (varsayılan): büyükse küçültülür, küçükse büyütülmez.
    FitPage,
    /// %100: UDE'nin kabulüyle 1 piksel = 1 punto.
    Actual,
    /// Sabit genişlik (cm); yükseklik en-boy oranından gelir.
    WidthCm(f64),
}

#[derive(Debug, Clone)]
pub struct BuildOptions {
    /// Her resim ayrı sayfada (aralara `<page-break>` konur).
    pub separate_pages: bool,
    pub sizing: Sizing,
    pub page: PageFormat,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            separate_pages: true,
            sizing: Sizing::FitPage,
            page: PageFormat::default(),
        }
    }
}

/// Bir resmin punto cinsinden görüntüleme boyutunu hesaplar.
///
/// UDE'nin kabulü: 1 piksel = 1 punto. `FitPage`'te ölçek `k = min(1, Gk/w, Yk/h)` —
/// `k = 1` durumunda resim doğal boyutunda kalır, **büyütülmez**.
pub fn goruntuleme_boyutu(px_w: u32, px_h: u32, sizing: Sizing, page: &PageFormat) -> (f64, f64) {
    let w = px_w.max(1) as f64;
    let h = px_h.max(1) as f64;
    match sizing {
        Sizing::Actual => (w, h),
        Sizing::FitPage => {
            let k = 1.0_f64
                .min(page.usable_width() / w)
                .min(page.usable_height() / h);
            (w * k, h * k)
        }
        Sizing::WidthCm(cm) => {
            let hedef_w = (cm * PT_PER_CM).max(1.0);
            let k = hedef_w / w;
            (hedef_w, h * k)
        }
    }
}

/// Resimlerden `.udf` dosya baytlarını üretir.
pub fn build_udf(images: &[ImageSpec], opts: &BuildOptions) -> Result<Vec<u8>> {
    if images.is_empty() {
        bail!("Belgeye koyacak resim yok.");
    }
    let doc = belge_kur(images, opts);
    let xml = serialize::serialize(&doc);
    zip::paketle(&xml)
}

/// Resimlerden belge modelini kurar (test edilebilir ara adım).
pub fn belge_kur(images: &[ImageSpec], opts: &BuildOptions) -> Document {
    let mut body: Vec<Block> = Vec::with_capacity(images.len() * 2);

    for (i, img) in images.iter().enumerate() {
        if i > 0 && opts.separate_pages {
            body.push(Block::PageBreak);
        }
        let (w, h) = goruntuleme_boyutu(img.px_w, img.px_h, opts.sizing, &opts.page);
        body.push(Block::Paragraph(Paragraph {
            alignment: Alignment::Center,
            runs: vec![Run::Image(ImageRun {
                data_b64: B64.encode(&img.bytes),
                width: w,
                height: h,
            })],
            ..Default::default()
        }));
    }

    Document {
        pages: opts.page.clone(),
        body,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn spec(w: u32, h: u32) -> ImageSpec {
        ImageSpec {
            bytes: vec![1, 2, 3],
            px_w: w,
            px_h: h,
        }
    }

    fn content_xml(bytes: &[u8]) -> String {
        let mut zip = ::zip::ZipArchive::new(std::io::Cursor::new(bytes.to_vec())).unwrap();
        let mut f = zip.by_index(0).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        s
    }

    #[test]
    fn sayfaya_sigdir_genislige_gore() {
        let pf = PageFormat::default();
        let (w, h) = goruntuleme_boyutu(3000, 2000, Sizing::FitPage, &pf);
        assert!((w - 524.41).abs() < 0.01, "{w}");
        assert!((h - 349.61).abs() < 0.01, "{h}");
    }

    #[test]
    fn sayfaya_sigdir_yukseklige_gore() {
        // Dar ve uzun görsel: sınırlayan kenar yükseklik.
        let pf = PageFormat::default();
        let (w, h) = goruntuleme_boyutu(1000, 4000, Sizing::FitPage, &pf);
        assert!((h - 773.55).abs() < 0.01, "{h}");
        assert!((w - 193.39).abs() < 0.01, "{w}");
    }

    #[test]
    fn kucuk_gorsel_buyutulmez() {
        let pf = PageFormat::default();
        let (w, h) = goruntuleme_boyutu(200, 150, Sizing::FitPage, &pf);
        assert_eq!((w, h), (200.0, 150.0));
    }

    #[test]
    fn yuzde_yuz_bire_bir_punto() {
        let pf = PageFormat::default();
        assert_eq!(
            goruntuleme_boyutu(3000, 2000, Sizing::Actual, &pf),
            (3000.0, 2000.0)
        );
    }

    #[test]
    fn cm_genislik_orani_korur() {
        let pf = PageFormat::default();
        let (w, h) = goruntuleme_boyutu(2000, 1000, Sizing::WidthCm(10.0), &pf);
        assert!((w - 283.465).abs() < 0.001, "{w}");
        assert!((h - 141.73).abs() < 0.01, "{h}");
    }

    #[test]
    fn resim_baytlari_degistirilmeden_base64_olur() {
        let bytes = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        let doc = belge_kur(
            &[ImageSpec {
                bytes: bytes.clone(),
                px_w: 100,
                px_h: 100,
            }],
            &BuildOptions::default(),
        );
        let Block::Paragraph(p) = &doc.body[0] else {
            panic!("paragraf bekleniyordu");
        };
        let Run::Image(img) = &p.runs[0] else {
            panic!("resim bekleniyordu");
        };
        assert_eq!(B64.decode(&img.data_b64).unwrap(), bytes);
        assert!(!img.data_b64.contains('\n'), "base64'te satır sonu olmamalı");
        assert!(!img.data_b64.starts_with("data:"), "data-URI öneki olmamalı");
    }

    #[test]
    fn ayri_sayfa_kapaliyken_page_break_yok() {
        let opts = BuildOptions {
            separate_pages: false,
            ..Default::default()
        };
        let udf = build_udf(&[spec(100, 100), spec(100, 100)], &opts).unwrap();
        let xml = content_xml(&udf);
        assert!(!xml.contains("<page-break>"), "{xml}");
        assert_eq!(xml.matches("<image ").count(), 2);
    }

    #[test]
    fn ayri_sayfa_acikken_aralarda_page_break_var() {
        let opts = BuildOptions {
            separate_pages: true,
            ..Default::default()
        };
        let udf = build_udf(&[spec(100, 100), spec(100, 100), spec(100, 100)], &opts).unwrap();
        let xml = content_xml(&udf);
        assert_eq!(xml.matches("<page-break>").count(), 2, "n resim → n-1 sayfa sonu");
    }

    #[test]
    fn resimsiz_belge_hata_verir() {
        assert!(build_udf(&[], &BuildOptions::default()).is_err());
    }
}

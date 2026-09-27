//! UDF üretimi. **Saf modül**: dosya sistemi yok, Tauri bağımlılığı yok, `cargo test` ile
//! doğrudan sınanır.
//!
//! İşin özü: `.udf` dosyasını kendimiz yazdığımız için UDE'nin resmi belgeye alırken
//! uyguladığı yeniden örnekleme (≈72 DPI'a indirme) hiç çalışmaz. Bitmap ne ise o gömülür;
//! sayfaya sığdırma yalnızca `width`/`height` **punto** öznitelikleriyle yapılır.

pub mod model;
pub mod serialize;
pub mod zip;

use std::io::{self, Cursor, Seek, SeekFrom, Write};
use std::sync::Arc;

use anyhow::{bail, Result};

use model::{Alignment, Block, Document, ImageRun, PageFormat, Paragraph, Run};

/// 1 santimetre kaç punto eder.
pub const PT_PER_CM: f64 = 28.3465;

/// Belgeye girecek tek bir resim: baytlar olduğu gibi gömülür.
#[derive(Debug, Clone)]
pub struct ImageSpec {
    /// Resmin ham baytları (PNG/JPEG). Yeniden kodlanmaz. Paylaşımlıdır: önbellekten belgeye
    /// geçerken kopyalanmaz.
    pub bytes: Arc<[u8]>,
    pub px_w: u32,
    pub px_h: u32,
}

/// Yazılanı tutmadan yalnızca boyutunu sayan yazıcı. ZIP yazıcısı yerel başlığı sonradan
/// doldurmak için geri sardığından konum ile uzunluk ayrı izlenir.
#[derive(Debug, Default)]
pub struct Olcer {
    konum: u64,
    /// Yazılan en uzak konum: çıktı bir dosya olsaydı boyutu bu olurdu.
    pub uzunluk: u64,
}

impl Write for Olcer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.konum += buf.len() as u64;
        self.uzunluk = self.uzunluk.max(self.konum);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Seek for Olcer {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let yeni = match pos {
            SeekFrom::Start(n) => Some(n),
            SeekFrom::End(n) => self.uzunluk.checked_add_signed(n),
            SeekFrom::Current(n) => self.konum.checked_add_signed(n),
        };
        self.konum = yeni.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "geçersiz konum"))?;
        Ok(self.konum)
    }
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
    /// Açıkken her resim ayrı sayfada (aralara `<page-break>` konur).
    /// Kapalıyken resimler arasına boş bir paragraf — yani bir "enter" — konur.
    pub separate_pages: bool,
    pub sizing: Sizing,
    pub page: PageFormat,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            separate_pages: false,
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

/// Resimlerden `.udf` dosyasını `hedef`e yazar. XML ve base64 metni bellekte bütün olarak
/// kurulmaz; sıkıştırıcıya parça parça akar.
pub fn udf_yaz<W: Write + Seek>(images: &[ImageSpec], opts: &BuildOptions, hedef: W) -> Result<W> {
    if images.is_empty() {
        bail!("Belgeye koyacak resim yok.");
    }
    let doc = belge_kur(images, opts);
    zip::paketle(hedef, |w| serialize::yaz(&doc, w))
}

/// Resimlerden `.udf` dosya baytlarını üretir.
pub fn build_udf(images: &[ImageSpec], opts: &BuildOptions) -> Result<Vec<u8>> {
    Ok(udf_yaz(images, opts, Cursor::new(Vec::new()))?.into_inner())
}

/// Üretilecek `.udf` dosyasının bayt sayısı. Belge gerçekten kurulup sıkıştırılır (tahmin
/// değil) ama çıktı tutulmaz, yalnızca sayılır.
pub fn udf_boyutu(images: &[ImageSpec], opts: &BuildOptions) -> Result<u64> {
    Ok(udf_yaz(images, opts, Olcer::default())?.uzunluk)
}

/// Resimlerden belge modelini kurar (test edilebilir ara adım).
pub fn belge_kur(images: &[ImageSpec], opts: &BuildOptions) -> Document {
    let mut body: Vec<Block> = Vec::with_capacity(images.len() * 2);

    for (i, img) in images.iter().enumerate() {
        if i > 0 {
            if opts.separate_pages {
                body.push(Block::PageBreak);
            } else {
                // Ayrı sayfa istenmiyorsa resimler arasına bir "enter" (boş paragraf).
                body.push(Block::Paragraph(Paragraph::default()));
            }
        }
        let (w, h) = goruntuleme_boyutu(img.px_w, img.px_h, opts.sizing, &opts.page);
        body.push(Block::Paragraph(Paragraph {
            alignment: Alignment::Center,
            runs: vec![Run::Image(ImageRun {
                data: img.bytes.clone(),
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
            bytes: vec![1, 2, 3].into(),
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
        use base64::engine::general_purpose::STANDARD as B64;
        use base64::Engine;

        let bytes = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        let udf = build_udf(
            &[ImageSpec {
                bytes: bytes.clone().into(),
                px_w: 100,
                px_h: 100,
            }],
            &BuildOptions::default(),
        )
        .unwrap();
        let xml = content_xml(&udf);
        let s = xml.find("imageData=\"").unwrap() + 11;
        let e = xml[s..].find('"').unwrap() + s;
        let b64 = &xml[s..e];
        assert_eq!(B64.decode(b64).unwrap(), bytes);
        assert!(!b64.contains('\n'), "base64'te satır sonu olmamalı");
        assert!(!b64.starts_with("data:"), "data-URI öneki olmamalı");
    }

    #[test]
    fn olculen_boyut_uretilen_dosyayla_ayni() {
        // Sıkışmayan (gürültü) ve sıkışan veri; tek ve iki resim; ayrı sayfa açık ve kapalı.
        let gurultu: Vec<u8> = (0..300_000u32)
            .map(|i| (i.wrapping_mul(2_654_435_761) >> 13) as u8)
            .collect();
        let resimler = [
            ImageSpec {
                bytes: gurultu.into(),
                px_w: 3000,
                px_h: 2000,
            },
            ImageSpec {
                bytes: vec![7; 100_000].into(),
                px_w: 800,
                px_h: 600,
            },
        ];
        for separate_pages in [false, true] {
            let opts = BuildOptions {
                separate_pages,
                ..Default::default()
            };
            for n in 1..=resimler.len() {
                let udf = build_udf(&resimler[..n], &opts).unwrap();
                assert_eq!(
                    udf_boyutu(&resimler[..n], &opts).unwrap(),
                    udf.len() as u64,
                    "{n} resim, ayrı sayfa: {separate_pages}"
                );
            }
        }
    }

    #[test]
    fn ayri_sayfa_kapaliyken_page_break_yerine_bos_paragraf() {
        let opts = BuildOptions {
            separate_pages: false,
            ..Default::default()
        };
        let udf = build_udf(&[spec(100, 100), spec(100, 100)], &opts).unwrap();
        let xml = content_xml(&udf);
        assert!(!xml.contains("<page-break>"), "{xml}");
        assert_eq!(xml.matches("<image ").count(), 2);
        // Aradaki "enter": resim taşımayan, uzunluğu 2 olan bir boş paragraf.
        assert_eq!(
            xml.matches("length=\"2\" family=\"Times New Roman\" size=\"10\"").count(),
            1,
            "{xml}"
        );
    }

    #[test]
    fn ayni_resim_iki_kez_eklenirse_ikisi_de_belgeye_girer() {
        let ayni = ImageSpec {
            bytes: vec![9, 9, 9, 9].into(),
            px_w: 800,
            px_h: 600,
        };
        let doc = belge_kur(&[ayni.clone(), ayni.clone()], &BuildOptions::default());
        let resim_sayisi = doc
            .body
            .iter()
            .filter(|b| {
                matches!(b, Block::Paragraph(p) if p.runs.iter().any(|r| matches!(r, Run::Image(_))))
            })
            .count();
        assert_eq!(resim_sayisi, 2, "aynı resim iki kez eklenmişse ikisi de yer almalı");
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

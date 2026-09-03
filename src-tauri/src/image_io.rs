//! Dosya / panodan resim okuma, EXIF yön düzeltmesi ve (istendiğinde) küçültme.
//!
//! Kural: **çözünürlüğe dokunulmaz.** Girdi baytları, yalnızca zorunlu olduğunda
//! (EXIF döndürme, desteklenmeyen biçim, kullanıcının açık küçültme isteği) yeniden kodlanır.

use std::io::Cursor;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use image::{DynamicImage, ImageFormat};
use serde::{Deserialize, Serialize};

use crate::udf::model::PageFormat;
use crate::udf::{goruntuleme_boyutu, ImageSpec, Sizing};

/// Yüklenmiş bir resim ve kullanıcıya gösterilecek bilgileri.
#[derive(Debug, Clone)]
pub struct YuklenenResim {
    pub spec: ImageSpec,
    /// Gömülü baytların biçimi ("PNG" / "JPEG").
    pub bicim: String,
    /// Girdi baytlarına hiç dokunulmadıysa true.
    pub orijinal_korundu: bool,
    /// EXIF nedeniyle döndürüldüyse true.
    pub dondurul: bool,
}

/// Kullanıcının seçtiği kalite basamağı.
///
/// **Ölçek ayarı değildir.** Resmin belgedeki görüntüleme boyutu (cm) her basamakta aynı
/// kalır; değişen tek şey o alanın içine kaç piksel düştüğüdür. UDE 1 pikseli 1 punto
/// saydığı için "punto başına 1 piksel" tam olarak UDE'nin kendi `Ekle → Resim` çıktısına
/// denk gelir (3000×2000 px görsel → 524×349 px). Çarpanlar bunun katıdır.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Kalite {
    /// Hiç dokunma: baytlar olduğu gibi gömülür (varsayılan).
    #[serde(rename = "orijinal")]
    Orijinal,
    #[serde(rename = "buyuk")]
    Buyuk,
    #[serde(rename = "orta")]
    Orta,
    /// UYAP editörünün kendi resim ekleme kalitesiyle eş değer.
    #[serde(rename = "kucuk")]
    Kucuk,
}

impl Kalite {
    /// Punto başına düşecek piksel sayısı. `Orijinal` için yeniden örnekleme yok.
    fn carpan(self) -> Option<f64> {
        match self {
            Kalite::Orijinal => None,
            Kalite::Buyuk => Some(3.0),
            Kalite::Orta => Some(2.0),
            Kalite::Kucuk => Some(1.0),
        }
    }

    /// Yeniden kodlarken kullanılacak JPEG kalitesi.
    fn jpeg_kalitesi(self) -> u8 {
        match self {
            Kalite::Orijinal | Kalite::Buyuk => 90,
            Kalite::Orta => 82,
            Kalite::Kucuk => 75,
        }
    }
}

/// UDE'nin sorunsuz gösterdiği gömme biçimleri. (§6.4 ile ölçüldü: JPEG de kabul ediliyor.)
fn gomulebilir(fmt: ImageFormat) -> bool {
    matches!(fmt, ImageFormat::Png | ImageFormat::Jpeg)
}

/// Diskteki bir resmi belgeye gömülmeye hazır hâle getirir.
pub fn dosyadan(path: &Path) -> Result<YuklenenResim> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("Dosya okunamadı: {}", path.display()))?;
    baytlardan(bytes)
}

/// Ham baytlardan resim hazırlar (sürükle-bırak ve panodan alma ortak yolu).
pub fn baytlardan(bytes: Vec<u8>) -> Result<YuklenenResim> {
    let fmt = image::guess_format(&bytes)
        .map_err(|_| anyhow!("Bu dosya tanınan bir resim değil (PNG, JPEG, WEBP, BMP, TIFF, GIF)."))?;

    let yon = exif_yonu(&bytes);
    let dondurme_gerekli = yon.map(|o| o != 1).unwrap_or(false);

    if gomulebilir(fmt) && !dondurme_gerekli {
        // En sık yol: tek bir piksel bile değişmeden gömülür.
        let (w, h) = olcu(&bytes)?;
        return Ok(YuklenenResim {
            spec: ImageSpec {
                bytes,
                px_w: w,
                px_h: h,
            },
            bicim: if fmt == ImageFormat::Png { "PNG" } else { "JPEG" }.to_string(),
            orijinal_korundu: true,
            dondurul: false,
        });
    }

    // Çözmek gerekiyor: ya yön düzeltmesi ya da UDE'nin bilmediği bir biçim.
    let mut img = image::load_from_memory(&bytes).context("Resim çözülemedi")?;
    if let Some(o) = yon {
        img = exif_uygula(img, o);
    }
    let (w, h) = (img.width(), img.height());
    let png = png_kodla(&img)?;

    Ok(YuklenenResim {
        spec: ImageSpec {
            bytes: png,
            px_w: w,
            px_h: h,
        },
        bicim: "PNG".to_string(),
        orijinal_korundu: false,
        dondurul: dondurme_gerekli,
    })
}

/// Yalnızca başlığı okuyarak piksel ölçüsü (12 MP fotoğrafı tam çözmemek için).
pub fn olcu(bytes: &[u8]) -> Result<(u32, u32)> {
    let reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .context("Resim biçimi anlaşılamadı")?;
    reader.into_dimensions().context("Resim ölçüsü okunamadı")
}

/// EXIF `Orientation` değeri (1..8). Yoksa `None`.
fn exif_yonu(bytes: &[u8]) -> Option<u16> {
    let mut cur = Cursor::new(bytes);
    let exif = exif::Reader::new().read_from_container(&mut cur).ok()?;
    let f = exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)?;
    f.value.get_uint(0).map(|v| v as u16)
}

/// EXIF yön değerini piksel verisine uygular (telefon fotoğrafları yan/ters görünmesin).
fn exif_uygula(img: DynamicImage, yon: u16) -> DynamicImage {
    match yon {
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => img.rotate90().fliph(),
        6 => img.rotate90(),
        7 => img.rotate270().fliph(),
        8 => img.rotate270(),
        _ => img,
    }
}

fn png_kodla(img: &DynamicImage) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
        .context("PNG kodlanamadı")?;
    Ok(buf)
}


/// Resmi seçilen kalite basamağına indirger.
///
/// Görüntüleme boyutu `FitPage` ile hesaplandığı ve hedef piksel ölçüsü o boyutun **üstünde**
/// tutulduğu için resim sayfada aynı yeri kaplamayı sürdürür; yalnızca içindeki ayrıntı azalır.
/// Zaten hedeften daha az pikseli olan resimlere dokunulmaz (büyütmek kaliteyi artırmaz,
/// dosyayı şişirir).
pub fn kaliteye_indir(spec: &ImageSpec, kalite: Kalite, sayfa: &PageFormat) -> Result<ImageSpec> {
    let Some(carpan) = kalite.carpan() else {
        return Ok(spec.clone());
    };
    let (gorunen_w, gorunen_h) = goruntuleme_boyutu(spec.px_w, spec.px_h, Sizing::FitPage, sayfa);
    // +1 px: yuvarlama yüzünden hedef, görüntüleme boyutunun altına düşüp resmi
    // sayfada küçültmesin.
    let hedef_w = (gorunen_w * carpan).ceil() as u32 + 1;
    let hedef_h = (gorunen_h * carpan).ceil() as u32 + 1;
    if hedef_w >= spec.px_w || hedef_h >= spec.px_h {
        return Ok(spec.clone());
    }

    let img = image::load_from_memory(&spec.bytes).context("Resim çözülemedi")?;
    let kucuk = img.resize(hedef_w, hedef_h, image::imageops::FilterType::Lanczos3);
    let (w, h) = (kucuk.width(), kucuk.height());
    let bytes = en_kucuk_kodlama(&kucuk, kalite)?;
    Ok(ImageSpec {
        bytes,
        px_w: w,
        px_h: h,
    })
}

/// PNG ve JPEG'i deneyip küçük olanı seçer. Saydamlık varsa JPEG hiç denenmez.
fn en_kucuk_kodlama(img: &DynamicImage, kalite: Kalite) -> Result<Vec<u8>> {
    let png = png_kodla(img)?;
    if saydam_mi(img) {
        return Ok(png);
    }
    let jpeg = jpeg_kodla(img, kalite.jpeg_kalitesi())?;
    Ok(if jpeg.len() < png.len() { jpeg } else { png })
}

/// Alfa kanalı **kullanılıyor** mu (yalnızca var olması yetmez; ekran görüntüleri hep
/// RGBA'dır ama tamamen mattır).
fn saydam_mi(img: &DynamicImage) -> bool {
    if !img.color().has_alpha() {
        return false;
    }
    img.to_rgba8().pixels().any(|p| p.0[3] != 255)
}

fn jpeg_kodla(img: &DynamicImage, kalite: u8) -> Result<Vec<u8>> {
    let rgb = img.to_rgb8();
    let mut buf = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, kalite)
        .encode(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            image::ExtendedColorType::Rgb8,
        )
        .context("JPEG kodlanamadı")?;
    Ok(buf)
}

/// Panoda gömülebilecek bir resim var mı? (Sağ tık menüsündeki "Yapıştır" için.)
pub fn panoda_resim_var() -> bool {
    arboard::Clipboard::new()
        .map(|mut p| p.get_image().is_ok())
        .unwrap_or(false)
}

/// Panodaki resmi PNG olarak alır (ekran görüntüsünü tek tuşla UDF yapmak için).
pub fn panodan() -> Result<YuklenenResim> {
    let mut pano = arboard::Clipboard::new().context("Pano açılamadı")?;
    let img = match pano.get_image() {
        Ok(i) => i,
        Err(_) => bail!("Panoda resim yok. Önce bir ekran görüntüsü alın (Win+Shift+S)."),
    };
    let w = img.width as u32;
    let h = img.height as u32;
    let buf = image::RgbaImage::from_raw(w, h, img.bytes.into_owned())
        .ok_or_else(|| anyhow!("Panodaki resim çözülemedi"))?;
    let png = png_kodla(&DynamicImage::ImageRgba8(buf))?;

    Ok(YuklenenResim {
        spec: ImageSpec {
            bytes: png,
            px_w: w,
            px_h: h,
        },
        bicim: "PNG".to_string(),
        orijinal_korundu: false,
        dondurul: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2x2 kırmızı PNG (elle kodlanmış, harici dosya gerekmiyor).
    fn ornek_png() -> Vec<u8> {
        let img = DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            4,
            3,
            image::Rgba([200, 30, 30, 255]),
        ));
        png_kodla(&img).unwrap()
    }

    #[test]
    fn png_baytlari_aynen_korunur() {
        let png = ornek_png();
        let y = baytlardan(png.clone()).unwrap();
        assert!(y.orijinal_korundu);
        assert_eq!(y.spec.bytes, png, "PNG girdisi bit-bit korunmalı");
        assert_eq!((y.spec.px_w, y.spec.px_h), (4, 3));
        assert_eq!(y.bicim, "PNG");
    }

    #[test]
    fn olcu_basliktan_okunur() {
        let png = ornek_png();
        assert_eq!(olcu(&png).unwrap(), (4, 3));
    }

    #[test]
    fn resim_olmayan_bayt_reddedilir() {
        let hata = baytlardan(b"bu bir resim degil".to_vec()).unwrap_err();
        assert!(hata.to_string().contains("tanınan bir resim değil"), "{hata}");
    }

    /// Verilen JPEG'in başına `Orientation` taşıyan bir EXIF (APP1) bölümü ekler.
    /// Telefon fotoğraflarının yaptığı şeyin en küçük hâli.
    fn exif_ekle(jpeg: &[u8], yon: u16) -> Vec<u8> {
        let mut tiff = Vec::new();
        tiff.extend_from_slice(b"II"); // küçük endian
        tiff.extend_from_slice(&42u16.to_le_bytes());
        tiff.extend_from_slice(&8u32.to_le_bytes()); // IFD0 offset
        tiff.extend_from_slice(&1u16.to_le_bytes()); // giriş sayısı
        tiff.extend_from_slice(&0x0112u16.to_le_bytes()); // Orientation
        tiff.extend_from_slice(&3u16.to_le_bytes()); // tür: SHORT
        tiff.extend_from_slice(&1u32.to_le_bytes()); // adet
        tiff.extend_from_slice(&(yon as u32).to_le_bytes()); // değer
        tiff.extend_from_slice(&0u32.to_le_bytes()); // sonraki IFD yok

        let mut app1 = Vec::new();
        app1.extend_from_slice(b"Exif\0\0");
        app1.extend_from_slice(&tiff);

        let mut out = Vec::new();
        out.extend_from_slice(&[0xFF, 0xD8]); // SOI
        out.extend_from_slice(&[0xFF, 0xE1]); // APP1
        out.extend_from_slice(&((app1.len() + 2) as u16).to_be_bytes());
        out.extend_from_slice(&app1);
        out.extend_from_slice(&jpeg[2..]); // özgün JPEG'in SOI'sinden sonrası
        out
    }

    fn ornek_jpeg(w: u32, h: u32) -> Vec<u8> {
        let img = DynamicImage::ImageRgb8(image::RgbImage::from_pixel(w, h, image::Rgb([9, 9, 9])));
        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg)
            .unwrap();
        buf
    }

    #[test]
    fn exif_dondurulmus_fotograf_duzeltilir() {
        let jpeg = exif_ekle(&ornek_jpeg(40, 20), 6); // 6 = 90° saat yönünde
        let y = baytlardan(jpeg).unwrap();

        assert!(y.dondurul, "EXIF yönü 6 için döndürme uygulanmalı");
        assert!(!y.orijinal_korundu, "döndürme yeniden kodlama gerektirir");
        assert_eq!(y.bicim, "PNG", "döndürülen resim kayıpsız PNG olarak gömülür");
        assert_eq!(
            (y.spec.px_w, y.spec.px_h),
            (20, 40),
            "90° döndürmede en ve boy yer değiştirir"
        );
    }

    #[test]
    fn exif_yonu_1_ise_baytlara_dokunulmaz() {
        let jpeg = exif_ekle(&ornek_jpeg(40, 20), 1);
        let y = baytlardan(jpeg.clone()).unwrap();

        assert!(!y.dondurul);
        assert!(y.orijinal_korundu, "yön 1: yeniden kodlama yok");
        assert_eq!(y.spec.bytes, jpeg);
        assert_eq!((y.spec.px_w, y.spec.px_h), (40, 20));
    }

    /// Sayfaya sığmayan, gerçekçi (düz renk olmayan) bir görsel.
    fn genis_gorsel(w: u32, h: u32) -> ImageSpec {
        let img = DynamicImage::ImageRgb8(image::RgbImage::from_fn(w, h, |x, y| {
            image::Rgb([(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8])
        }));
        let bytes = png_kodla(&img).unwrap();
        ImageSpec {
            bytes,
            px_w: w,
            px_h: h,
        }
    }

    #[test]
    fn orijinal_kalite_baytlara_dokunmaz() {
        let spec = genis_gorsel(1500, 1000);
        let sonuc = kaliteye_indir(&spec, Kalite::Orijinal, &PageFormat::default()).unwrap();
        assert_eq!(sonuc.bytes, spec.bytes);
        assert_eq!((sonuc.px_w, sonuc.px_h), (1500, 1000));
    }

    #[test]
    fn dusuk_kalite_sayfadaki_yeri_degistirmez() {
        let sayfa = PageFormat::default();
        // En yüksek basamağın (punto başına 3 piksel) bile altına düşmesi için yeterince büyük.
        let spec = genis_gorsel(2400, 1600);
        let (once_w, once_h) = goruntuleme_boyutu(spec.px_w, spec.px_h, Sizing::FitPage, &sayfa);

        for kalite in [Kalite::Buyuk, Kalite::Orta, Kalite::Kucuk] {
            let k = kaliteye_indir(&spec, kalite, &sayfa).unwrap();
            let (sonra_w, sonra_h) = goruntuleme_boyutu(k.px_w, k.px_h, Sizing::FitPage, &sayfa);
            assert!(
                (once_w - sonra_w).abs() < 1.0 && (once_h - sonra_h).abs() < 1.0,
                "{kalite:?}: görüntüleme boyutu {once_w}×{once_h} → {sonra_w}×{sonra_h}"
            );
        }
    }

    #[test]
    fn kalite_basamaklari_gittikce_kucultur() {
        let sayfa = PageFormat::default();
        let spec = genis_gorsel(2400, 1600);
        let buyuk = kaliteye_indir(&spec, Kalite::Buyuk, &sayfa).unwrap();
        let orta = kaliteye_indir(&spec, Kalite::Orta, &sayfa).unwrap();
        let kucuk = kaliteye_indir(&spec, Kalite::Kucuk, &sayfa).unwrap();

        assert!(buyuk.px_w > orta.px_w && orta.px_w > kucuk.px_w);
        assert!(spec.bytes.len() > buyuk.bytes.len(), "büyük, orijinalden küçük olmalı");
        assert!(buyuk.bytes.len() > orta.bytes.len());
        assert!(orta.bytes.len() > kucuk.bytes.len());
    }

    #[test]
    fn kucuk_kalite_ude_ile_ayni_yogunlukta() {
        // Ölçüm: UDE "Ekle → Resim" 3000×2000 px görseli 524×349 px'e indiriyor.
        // "Küçük Boyut" punto başına 1 piksel demek olduğu için aynı yere çıkmalı.
        let spec = genis_gorsel(1500, 1000);
        let k = kaliteye_indir(&spec, Kalite::Kucuk, &PageFormat::default()).unwrap();
        assert!(
            (k.px_w as i64 - 524).abs() <= 3 && (k.px_h as i64 - 349).abs() <= 3,
            "{}×{} bekleniyordu ≈524×349",
            k.px_w,
            k.px_h
        );
    }

    #[test]
    fn sayfaya_zaten_sigan_kucuk_resim_buyutulmez() {
        let spec = genis_gorsel(200, 150);
        let k = kaliteye_indir(&spec, Kalite::Kucuk, &PageFormat::default()).unwrap();
        assert_eq!((k.px_w, k.px_h), (200, 150), "büyütme yapılmamalı");
        assert_eq!(k.bytes, spec.bytes, "dokunulmamış olmalı");
    }

    #[test]
    fn saydam_resim_jpegle_bozulmaz() {
        let img = DynamicImage::ImageRgba8(image::RgbaImage::from_fn(1500, 1000, |x, y| {
            image::Rgba([(x % 256) as u8, (y % 256) as u8, 40, if x < 10 { 0 } else { 255 }])
        }));
        let spec = ImageSpec {
            bytes: png_kodla(&img).unwrap(),
            px_w: 1500,
            px_h: 1000,
        };
        let k = kaliteye_indir(&spec, Kalite::Kucuk, &PageFormat::default()).unwrap();
        assert_eq!(
            image::guess_format(&k.bytes).unwrap(),
            ImageFormat::Png,
            "saydamlık varsa PNG kalmalı"
        );
    }

    #[test]
    fn exif_yon_donusumleri() {
        let img = DynamicImage::ImageRgb8(image::RgbImage::from_pixel(4, 2, image::Rgb([1, 2, 3])));
        assert_eq!((exif_uygula(img.clone(), 6).width()), 2, "90° döndürme en-boyu takas eder");
        assert_eq!((exif_uygula(img.clone(), 1).width()), 4, "yön 1 = dokunma");
        assert_eq!((exif_uygula(img, 3).width()), 4, "180° en-boyu korur");
    }
}

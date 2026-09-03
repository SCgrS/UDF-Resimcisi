//! Dosya / panodan resim okuma, EXIF yön düzeltmesi ve (istendiğinde) küçültme.
//!
//! Kural: **çözünürlüğe dokunulmaz.** Girdi baytları, yalnızca zorunlu olduğunda
//! (EXIF döndürme, desteklenmeyen biçim, kullanıcının açık küçültme isteği) yeniden kodlanır.

use std::io::Cursor;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use image::{DynamicImage, ImageFormat};

use crate::udf::ImageSpec;

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

/// 300 DPI'lık A4 genişliğine (≈2200 px uzun kenar) küçültür. Yalnızca kullanıcı
/// 10 MB uyarısında "küçült" dediğinde çağrılır; varsayılan akışta asla çalışmaz.
pub const HEDEF_UZUN_KENAR_300DPI: u32 = 2200;

pub fn kucult(spec: &ImageSpec, hedef_uzun_kenar: u32) -> Result<ImageSpec> {
    let uzun = spec.px_w.max(spec.px_h);
    if uzun <= hedef_uzun_kenar {
        return Ok(spec.clone());
    }
    let img = image::load_from_memory(&spec.bytes).context("Küçültme için resim çözülemedi")?;
    let k = hedef_uzun_kenar as f64 / uzun as f64;
    let nw = ((spec.px_w as f64 * k).round() as u32).max(1);
    let nh = ((spec.px_h as f64 * k).round() as u32).max(1);
    let kucuk = img.resize_exact(nw, nh, image::imageops::FilterType::Lanczos3);

    // Küçültülmüş sürümde JPEG çok daha küçük dosya verir; UDE JPEG'i kabul ediyor (§6.4).
    let mut buf = Vec::new();
    let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 88);
    enc.encode_image(&kucuk.to_rgb8())
        .context("JPEG kodlanamadı")?;

    Ok(ImageSpec {
        bytes: buf,
        px_w: nw,
        px_h: nh,
    })
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

    #[test]
    fn kucultme_uzun_kenari_hedefe_indirir() {
        let img = DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            3000,
            2000,
            image::Rgb([10, 20, 30]),
        ));
        let spec = ImageSpec {
            bytes: png_kodla(&img).unwrap(),
            px_w: 3000,
            px_h: 2000,
        };
        let k = kucult(&spec, 2200).unwrap();
        assert_eq!(k.px_w, 2200);
        assert_eq!(k.px_h, 1467);
        assert!(k.bytes.len() < spec.bytes.len());
    }

    #[test]
    fn zaten_kucukse_dokunulmaz() {
        let png = ornek_png();
        let spec = ImageSpec {
            bytes: png.clone(),
            px_w: 4,
            px_h: 3,
        };
        let k = kucult(&spec, 2200).unwrap();
        assert_eq!(k.bytes, png);
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

    #[test]
    fn exif_yon_donusumleri() {
        let img = DynamicImage::ImageRgb8(image::RgbImage::from_pixel(4, 2, image::Rgb([1, 2, 3])));
        assert_eq!((exif_uygula(img.clone(), 6).width()), 2, "90° döndürme en-boyu takas eder");
        assert_eq!((exif_uygula(img.clone(), 1).width()), 4, "yön 1 = dokunma");
        assert_eq!((exif_uygula(img, 3).width()), 4, "180° en-boyu korur");
    }
}

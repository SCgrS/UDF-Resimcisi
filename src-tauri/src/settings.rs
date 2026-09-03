//! Kullanıcı ayarları. Tek kaynak Rust tarafındadır; arayüz komutlarla okur/yazar.
//! Dosya: `%APPDATA%\UDF Resimcisi\ayarlar.json`.

use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Arayüz teması.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tema {
    /// İşletim sisteminin ayarını izle.
    #[serde(rename = "sistem")]
    Sistem,
    #[serde(rename = "acik")]
    Acik,
    #[serde(rename = "koyu")]
    Koyu,
}

/// Ayar dosyasının biçim sürümü. Varsayılanı değişen bir alan olduğunda artırılır; daha eski
/// dosyalar okunurken o alan yeni varsayılana çekilir.
pub const AYAR_SURUMU: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Ayarlar {
    /// Birden fazla resim tek belgeye girdiğinde her resim ayrı sayfada olsun mu.
    /// Kapalıyken resimler arasına bir boş paragraf ("enter") konur.
    pub ayri_sayfa: bool,
    pub cikti_klasoru: String,
    pub tema: Tema,
    /// Yazarken her zaman `AYAR_SURUMU` olur; arayüzün göndermesi gerekmez.
    ///
    /// Alan düzeyinde `default` şart: kap düzeyindeki `#[serde(default)]` eksik alanları
    /// `Ayarlar::default()`ten doldurur ve bu da güncel sürümü verirdi — o zaman eski dosyalar
    /// güncel sanılır, taşıma hiç çalışmazdı.
    #[serde(default)]
    pub ayar_surumu: u32,
}

impl Default for Ayarlar {
    fn default() -> Self {
        Self {
            ayri_sayfa: false,
            cikti_klasoru: varsayilan_cikti_klasoru().to_string_lossy().to_string(),
            tema: Tema::Sistem,
            ayar_surumu: AYAR_SURUMU,
        }
    }
}

/// `%USERPROFILE%\Documents\UDF Resimcisi`
pub fn varsayilan_cikti_klasoru() -> PathBuf {
    let ev = std::env::var("USERPROFILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    ev.join("Documents").join("UDF Resimcisi")
}

fn ayar_dosyasi() -> PathBuf {
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| varsayilan_cikti_klasoru());
    base.join("UDF Resimcisi").join("ayarlar.json")
}

pub fn yukle() -> Ayarlar {
    let p = ayar_dosyasi();
    let okunan: Ayarlar = match std::fs::read_to_string(&p) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => Ayarlar::default(),
    };
    let a = tasi(okunan);
    kaldiriciya_bildir(&a);
    a
}

/// Kaldırıcı "verileri sil" işaretlendiğinde nereyi temizleyeceğini bilsin diye kaydetme
/// klasörünü kayıt defterine not eder. Windows dışında bir şey yapmaz.
fn kaldiriciya_bildir(a: &Ayarlar) {
    #[cfg(windows)]
    crate::kayit::cikti_klasorunu_yaz(&a.cikti_klasoru);
    #[cfg(not(windows))]
    let _ = a;
}

/// Eski biçimli ayarları güncel varsayılanlara taşır.
fn tasi(mut a: Ayarlar) -> Ayarlar {
    if a.ayar_surumu < 2 {
        // 2. sürümde "her resim ayrı sayfada" varsayılanı kapalıya döndü; eski dosyalardaki
        // değer kullanıcının bilinçli seçimi değil, eski varsayılandı.
        a.ayri_sayfa = false;
    }
    a.ayar_surumu = AYAR_SURUMU;
    a
}

pub fn kaydet(a: &Ayarlar) -> Result<()> {
    let p = ayar_dosyasi();
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("Ayar klasörü oluşturulamadı: {}", dir.display()))?;
    }
    let mut yazilacak = a.clone();
    yazilacak.ayar_surumu = AYAR_SURUMU;
    let s = serde_json::to_string_pretty(&yazilacak)?;
    std::fs::write(&p, s).with_context(|| format!("Ayarlar yazılamadı: {}", p.display()))?;
    kaldiriciya_bildir(&yazilacak);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varsayilanlar() {
        let a = Ayarlar::default();
        assert!(!a.ayri_sayfa, "ayrı sayfa varsayılan olarak kapalı");
        assert_eq!(a.tema, Tema::Sistem);
        assert!(a.cikti_klasoru.ends_with("UDF Resimcisi"));
    }

    #[test]
    fn json_gidis_donus() {
        let a = Ayarlar {
            tema: Tema::Koyu,
            ayri_sayfa: true,
            ..Default::default()
        };
        let s = serde_json::to_string(&a).unwrap();
        assert!(s.contains("\"koyu\""), "{s}");
        let geri: Ayarlar = serde_json::from_str(&s).unwrap();
        assert_eq!(geri.tema, Tema::Koyu);
        assert!(geri.ayri_sayfa);
    }

    #[test]
    fn eski_ayar_dosyasi_okunabiliyor() {
        // 1.0/1.1'de kip, otomasyon, boyut, uyari_9mb alanları vardı; kaldırıldılar.
        // Eski dosya hâlâ okunmalı, bilinmeyen alanlar yok sayılmalı.
        let a: Ayarlar = serde_json::from_str(
            "{\"kip\":\"A\",\"otomasyon\":true,\"uyari_9mb\":true,\"boyut\":{\"tur\":\"cm\",\"deger\":12},\"ayri_sayfa\":true}",
        )
        .unwrap();
        assert_eq!(a.tema, Tema::Sistem, "eksik alanlar varsayılanla dolmalı");
        assert_eq!(a.ayar_surumu, 0, "eski dosyada sürüm alanı yok");
    }

    #[test]
    fn eski_dosyada_ayri_sayfa_yeni_varsayilana_cekilir() {
        let eski = Ayarlar {
            ayri_sayfa: true,
            ayar_surumu: 0,
            ..Default::default()
        };
        assert!(!tasi(eski).ayri_sayfa, "1.x'ten gelen açık değer kapatılmalı");
    }

    #[test]
    fn guncel_dosyada_kullanici_secimi_korunur() {
        let guncel = Ayarlar {
            ayri_sayfa: true,
            ayar_surumu: AYAR_SURUMU,
            ..Default::default()
        };
        assert!(tasi(guncel).ayri_sayfa, "kullanıcının kendi seçimi bozulmamalı");
    }
}

//! Kullanıcı ayarları. Tek kaynak Rust tarafındadır; arayüz komutlarla okur/yazar.
//! Dosya: `%APPDATA%\UDF Resimcisi\ayarlar.json`.

use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Çalışma kipi (§4.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kip {
    /// Panoya kopyala (UDE üzerinden).
    #[serde(rename = "A")]
    PanoyaKopyala,
    /// UDF'yi aç.
    #[serde(rename = "B")]
    UdfyiAc,
}

/// Görüntüleme boyutu tercihi.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "tur", content = "deger")]
pub enum BoyutTercihi {
    #[serde(rename = "sigdir")]
    SayfayaSigdir,
    #[serde(rename = "yuzde100")]
    YuzdeYuz,
    #[serde(rename = "cm")]
    GenislikCm(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Ayarlar {
    pub kip: Kip,
    /// Ctrl+A / Ctrl+C otomatik gönderilsin mi.
    pub otomasyon: bool,
    /// Her resim ayrı sayfada.
    pub ayri_sayfa: bool,
    pub boyut: BoyutTercihi,
    pub cikti_klasoru: String,
    /// 9 MB üstü çıktıda uyar.
    pub uyari_9mb: bool,
    /// İşlem sonrası pencereyi küçült.
    pub islem_sonrasi_kucult: bool,
}

impl Default for Ayarlar {
    fn default() -> Self {
        Self {
            kip: Kip::PanoyaKopyala,
            otomasyon: true,
            ayri_sayfa: true,
            boyut: BoyutTercihi::SayfayaSigdir,
            cikti_klasoru: varsayilan_cikti_klasoru()
                .to_string_lossy()
                .to_string(),
            uyari_9mb: true,
            islem_sonrasi_kucult: false,
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
    match std::fs::read_to_string(&p) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => Ayarlar::default(),
    }
}

pub fn kaydet(a: &Ayarlar) -> Result<()> {
    let p = ayar_dosyasi();
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("Ayar klasörü oluşturulamadı: {}", dir.display()))?;
    }
    let s = serde_json::to_string_pretty(a)?;
    std::fs::write(&p, s).with_context(|| format!("Ayarlar yazılamadı: {}", p.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varsayilanlar_spec_ile_uyusuyor() {
        let a = Ayarlar::default();
        assert_eq!(a.kip, Kip::PanoyaKopyala);
        assert!(a.otomasyon);
        assert!(a.ayri_sayfa);
        assert_eq!(a.boyut, BoyutTercihi::SayfayaSigdir);
        assert!(a.uyari_9mb);
        assert!(!a.islem_sonrasi_kucult);
        assert!(a.cikti_klasoru.ends_with("UDF Resimcisi"));
    }

    #[test]
    fn json_gidis_donus() {
        let a = Ayarlar {
            kip: Kip::UdfyiAc,
            boyut: BoyutTercihi::GenislikCm(12.5),
            ..Default::default()
        };
        let s = serde_json::to_string(&a).unwrap();
        assert!(s.contains("\"B\""), "{s}");
        assert!(s.contains("\"cm\""), "{s}");
        let geri: Ayarlar = serde_json::from_str(&s).unwrap();
        assert_eq!(geri.kip, Kip::UdfyiAc);
        assert_eq!(geri.boyut, BoyutTercihi::GenislikCm(12.5));
    }

    #[test]
    fn bozuk_json_varsayilana_duser() {
        let a: Ayarlar = serde_json::from_str("{\"kip\":\"A\"}").unwrap();
        assert!(a.otomasyon, "eksik alanlar varsayılanla dolmalı");
    }
}

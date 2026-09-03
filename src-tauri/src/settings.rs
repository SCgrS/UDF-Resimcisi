//! Kullanıcı ayarları. Tek kaynak Rust tarafındadır; arayüz komutlarla okur/yazar.
//! Dosya: `%APPDATA%\UDF Resimcisi\ayarlar.json`.

use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

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
    /// Birden fazla resim tek belgeye girdiğinde her resim ayrı sayfada olsun mu.
    pub ayri_sayfa: bool,
    pub boyut: BoyutTercihi,
    pub cikti_klasoru: String,
    /// 9 MB üstü çıktıda uyar.
    pub uyari_9mb: bool,
}

impl Default for Ayarlar {
    fn default() -> Self {
        Self {
            ayri_sayfa: true,
            boyut: BoyutTercihi::SayfayaSigdir,
            cikti_klasoru: varsayilan_cikti_klasoru()
                .to_string_lossy()
                .to_string(),
            uyari_9mb: true,
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
        assert!(a.ayri_sayfa);
        assert_eq!(a.boyut, BoyutTercihi::SayfayaSigdir);
        assert!(a.uyari_9mb);
        assert!(a.cikti_klasoru.ends_with("UDF Resimcisi"));
    }

    #[test]
    fn json_gidis_donus() {
        let a = Ayarlar {
            boyut: BoyutTercihi::GenislikCm(12.5),
            ..Default::default()
        };
        let s = serde_json::to_string(&a).unwrap();
        assert!(s.contains("\"cm\""), "{s}");
        let geri: Ayarlar = serde_json::from_str(&s).unwrap();
        assert_eq!(geri.boyut, BoyutTercihi::GenislikCm(12.5));
    }

    #[test]
    fn eski_ayar_dosyasi_okunabiliyor() {
        // 1.0 sürümünde kip/otomasyon alanları vardı; kaldırıldılar ama eski dosya
        // hâlâ okunmalı ve eksik alanlar varsayılanla dolmalı.
        let a: Ayarlar =
            serde_json::from_str("{\"kip\":\"A\",\"otomasyon\":true,\"ayri_sayfa\":false}")
                .unwrap();
        assert!(!a.ayri_sayfa);
        assert!(a.uyari_9mb, "eksik alanlar varsayılanla dolmalı");
    }
}

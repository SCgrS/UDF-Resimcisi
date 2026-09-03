//! Kaldırıcının okuyabildiği tek kayıt defteri izi — yalnızca Windows.
//!
//! Neden var: Tauri'nin NSIS kaldırıcısı "verileri sil" işaretlendiğinde sadece
//! `%APPDATA%\<paket kimliği>` ve `%LOCALAPPDATA%\<paket kimliği>` klasörlerini siler.
//! Bu uygulamanın ayarları `%APPDATA%\UDF Resimcisi` altında, ürettiği belgeler ise
//! kullanıcının seçtiği kaydetme klasöründe durur; ikisi de o iki klasöre girmez, bu yüzden
//! kutu işaretlense bile yerinde kalıyorlardı.
//!
//! Kaldırıcı `ayarlar.json`'u ayrıştıramayacağı için kaydetme klasörünün güncel yolunu
//! `HKCU\Software\UDF Resimcisi\CiktiKlasoru` değerine yazıyoruz; `nsis/hooks.nsh` oradan
//! okuyup temizliyor. Yazma başarısız olursa sessizce geçilir: bu değer uygulamanın
//! çalışması için gerekli değil.

#![cfg(windows)]

use windows::core::{HSTRING, PCWSTR};
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_WRITE,
    REG_OPTION_NON_VOLATILE, REG_SZ,
};

const ANAHTAR: &str = r"Software\UDF Resimcisi";
const DEGER: &str = "CiktiKlasoru";

/// Kaydetme klasörünün yolunu kaldırıcı için not eder.
pub fn cikti_klasorunu_yaz(yol: &str) {
    unsafe {
        let alt = HSTRING::from(ANAHTAR);
        let mut anahtar = HKEY::default();
        let sonuc = RegCreateKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(alt.as_ptr()),
            0,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut anahtar,
            None,
        );
        if sonuc.is_err() {
            return;
        }
        let ad = HSTRING::from(DEGER);
        let veri: Vec<u16> = yol.encode_utf16().chain(std::iter::once(0)).collect();
        let baytlar = std::slice::from_raw_parts(veri.as_ptr() as *const u8, veri.len() * 2);
        let _ = RegSetValueExW(anahtar, PCWSTR(ad.as_ptr()), 0, REG_SZ, Some(baytlar));
        let _ = RegCloseKey(anahtar);
    }
}

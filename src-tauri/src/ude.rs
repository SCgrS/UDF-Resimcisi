//! UYAP Doküman Editörü (UDE) ile etkileşim — yalnızca Windows.
//!
//! Uygulama UDE **olmadan da** çalışır: belge her hâlükârda üretilip diske yazılır. UDE yalnızca
//! üretilen belgeyi açmak için kullanılır; kurulu değilse dosya yine kaydedilir ve kullanıcıya
//! nereye kaydedildiği söylenir.
//!
//! Ölçülmüş davranış: `.udf` dosya ilişkisi UDE'yi
//! `getNewWPInstance EDITOR_TYPE_DOCUMENT "%1" "%~s1"` argümanlarıyla çağırır. Bu jetonlar
//! olmadan UDE hiçbir şey açmadan çıkar; bu yüzden önce `ShellExecuteW` denenir, doğrudan
//! çalıştırmaya ancak dosya ilişkisi yoksa düşülür.

#![cfg(windows)]

use std::path::Path;

use anyhow::{bail, Result};
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Registry::{RegGetValueW, HKEY_CLASSES_ROOT, RRF_RT_REG_SZ};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

/// UDE bu bilgisayarda kurulu mu? (`.udf` dosya ilişkisi ya da kayıt defterindeki komut.)
pub fn kurulu_mu() -> bool {
    ude_exe_yolu().is_some()
}

/// `.udf` dosyasını UDE'de açar.
pub fn belgeyi_ac(path: &Path) -> Result<()> {
    let file = HSTRING::from(path.as_os_str());
    let op = HSTRING::from("open");
    let sonuc = unsafe {
        ShellExecuteW(
            HWND::default(),
            PCWSTR(op.as_ptr()),
            PCWSTR(file.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    // ShellExecuteW 32'den büyük bir değer döndürürse başarılı.
    if sonuc.0 as usize > 32 {
        return Ok(());
    }

    // İlişki yok: kayıt defterinden UDE'yi bulup doğrudan çalıştır.
    let Some(exe) = ude_exe_yolu() else {
        bail!("UYAP Doküman Editörü bu bilgisayarda bulunamadı.");
    };
    std::process::Command::new(exe)
        .args([
            "getNewWPInstance",
            "EDITOR_TYPE_DOCUMENT",
            &path.to_string_lossy(),
        ])
        .spawn()?;
    Ok(())
}

/// Kayıt defterinden `.udf` açma komutundaki exe yolunu çıkarır.
pub fn ude_exe_yolu() -> Option<String> {
    let prog_id = reg_oku(".udf", "")?;
    let komut = reg_oku(&format!("{prog_id}\\shell\\open\\command"), "")?;
    // Komut: "C:\...\Uyap Doküman Editörü.exe" "getNewWPInstance" ... — ilk tırnaklı parça.
    let s = komut.trim();
    if let Some(rest) = s.strip_prefix('"') {
        rest.split('"').next().map(|x| x.to_string())
    } else {
        s.split_whitespace().next().map(|x| x.to_string())
    }
}

fn reg_oku(alt_anahtar: &str, deger: &str) -> Option<String> {
    let key = HSTRING::from(alt_anahtar);
    let val = HSTRING::from(deger);
    let mut boyut: u32 = 0;
    unsafe {
        RegGetValueW(
            HKEY_CLASSES_ROOT,
            PCWSTR(key.as_ptr()),
            PCWSTR(val.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            None,
            Some(&mut boyut),
        )
        .ok()
        .ok()?;
        let mut buf = vec![0u16; (boyut as usize / 2) + 1];
        RegGetValueW(
            HKEY_CLASSES_ROOT,
            PCWSTR(key.as_ptr()),
            PCWSTR(val.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            Some(buf.as_mut_ptr() as *mut _),
            Some(&mut boyut),
        )
        .ok()
        .ok()?;
        let son = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        Some(String::from_utf16_lossy(&buf[..son]))
    }
}

/// Dosyayı Explorer'da seçili olarak gösterir.
pub fn klasorde_goster(path: &Path) {
    let _ = std::process::Command::new("explorer")
        .arg(format!("/select,{}", path.display()))
        .spawn();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kayit_defterinden_ude_yolu_okunabiliyorsa_exe_ile_biter() {
        // UDE kurulu olmayan makinede None dönmesi normaldir; kurulu ise yol .exe olmalı.
        if let Some(p) = ude_exe_yolu() {
            assert!(
                p.to_lowercase().ends_with(".exe"),
                "beklenmeyen komut ayrıştırması: {p}"
            );
        }
    }

    #[test]
    fn kurulu_mu_ile_yol_tutarli() {
        assert_eq!(kurulu_mu(), ude_exe_yolu().is_some());
    }
}

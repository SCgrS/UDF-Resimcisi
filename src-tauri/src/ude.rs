//! UYAP Doküman Editörü (UDE) ile etkileşim — yalnızca Windows.
//!
//! Ölçülmüş davranışlar (bkz. `DOGRULAMA.md`):
//! * `.udf` dosya ilişkisi UDE'yi `getNewWPInstance EDITOR_TYPE_DOCUMENT "%1" "%~s1"`
//!   argümanlarıyla çağırır. Bu jetonlar olmadan UDE hiçbir şey açmadan çıkar; bu yüzden
//!   önce `ShellExecuteW` denenir, doğrudan çalıştırmaya ancak ilişki yoksa düşülür.
//! * Soğuk açılış 15-25 sn sürebilir.
//! * **Tuşlar, önce belge tuvaline tıklanmadan editöre ulaşmıyor** (yalnız
//!   `SetForegroundWindow` yetmiyor); bu yüzden `kopyala_panoya` önce sayfaya tıklar.
//! * Pano gerçekten değişti mi `GetClipboardSequenceNumber` ile doğrulanır — değişmediyse
//!   "kopyalandı" denmez, kılavuz kipine düşülür.

#![cfg(windows)]

use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;
use windows::Win32::System::Registry::{
    RegGetValueW, HKEY_CLASSES_ROOT, RRF_RT_REG_SZ,
};
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, SetFocus, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT,
    KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEINPUT, MOUSE_EVENT_FLAGS,
    VIRTUAL_KEY, VK_A,
    VK_C, VK_CONTROL,
};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowRect, GetWindowTextW,
    GetWindowThreadProcessId, IsIconic, IsWindowVisible, SetCursorPos, SetForegroundWindow,
    ShowWindow, SW_RESTORE, SW_SHOWNORMAL,
};

/// Pencere aranırken beklenecek üst sınır. UDE soğuk açılışı 15-25 sn sürebiliyor.
const PENCERE_BEKLEME: Duration = Duration::from_secs(30);
const YOKLAMA_ARALIGI: Duration = Duration::from_millis(250);

/// `.udf` dosyasını UDE'de açar.
///
/// Önce kabuk ilişkisi (`ShellExecuteW`) denenir: UYAP kurulumunun kendi komut satırını
/// (gerekli `getNewWPInstance` jetonuyla) kullandığı için en güvenilir yol budur.
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
        bail!(
            "UYAP Doküman Editörü bulunamadı. Kurulu değilse Kip B'yi (\"UDF'yi aç\") kullanın \
             ya da dosyayı elle açın."
        );
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

// ---------------------------------------------------------------------------
// Pencere bulma
// ---------------------------------------------------------------------------

struct Arama {
    desen: String,
    bulunan: HWND,
}

unsafe extern "system" fn enum_geri(hwnd: HWND, lparam: LPARAM) -> windows::Win32::Foundation::BOOL {
    let arama = &mut *(lparam.0 as *mut Arama);
    if !IsWindowVisible(hwnd).as_bool() {
        return true.into();
    }
    let mut buf = [0u16; 512];
    let n = GetWindowTextW(hwnd, &mut buf);
    if n > 0 {
        let baslik = String::from_utf16_lossy(&buf[..n as usize]).to_lowercase();
        if baslik.contains(&arama.desen) {
            arama.bulunan = hwnd;
            return false.into();
        }
    }
    true.into()
}

/// Başlığında `desen` geçen görünür pencereyi bulur (küçük/büyük harf duyarsız).
pub fn pencere_bul(desen: &str) -> Option<HWND> {
    let mut arama = Arama {
        desen: desen.to_lowercase(),
        bulunan: HWND::default(),
    };
    unsafe {
        let _ = EnumWindows(Some(enum_geri), LPARAM(&mut arama as *mut _ as isize));
    }
    if arama.bulunan.0.is_null() {
        None
    } else {
        Some(arama.bulunan)
    }
}

/// Pencereyi belirli bir süre boyunca yoklayarak bekler.
pub fn pencere_bekle(desen: &str, sure: Duration) -> Option<HWND> {
    let bitis = Instant::now() + sure;
    loop {
        if let Some(h) = pencere_bul(desen) {
            return Some(h);
        }
        if Instant::now() >= bitis {
            return None;
        }
        sleep(YOKLAMA_ARALIGI);
    }
}

/// Pencereyi öne getirir. Windows'un foreground kilidi yüzünden düz `SetForegroundWindow`
/// çoğu zaman yetmez; `AttachThreadInput` kalıbı gerekir.
pub fn one_getir(hwnd: HWND) -> bool {
    unsafe {
        let fg = GetWindowThreadProcessId(GetForegroundWindow(), None);
        let hedef = GetWindowThreadProcessId(hwnd, None);
        let ben = GetCurrentThreadId();
        let _ = AttachThreadInput(ben, fg, true);
        let _ = AttachThreadInput(ben, hedef, true);
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        let ok = SetForegroundWindow(hwnd).as_bool();
        let _ = SetFocus(hwnd);
        let _ = AttachThreadInput(ben, hedef, false);
        let _ = AttachThreadInput(ben, fg, false);
        ok
    }
}

// ---------------------------------------------------------------------------
// Girdi gönderme
// ---------------------------------------------------------------------------

fn fare(bayrak: MOUSE_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dwFlags: bayrak,
                ..Default::default()
            },
        },
    }
}

fn tikla(x: i32, y: i32) {
    unsafe {
        let _ = SetCursorPos(x, y);
        sleep(Duration::from_millis(120));
        let girdi = [fare(MOUSEEVENTF_LEFTDOWN), fare(MOUSEEVENTF_LEFTUP)];
        SendInput(&girdi, std::mem::size_of::<INPUT>() as i32);
    }
}

fn tus(vk: VIRTUAL_KEY, kalkti: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                dwFlags: if kalkti {
                    KEYEVENTF_KEYUP
                } else {
                    KEYBD_EVENT_FLAGS(0)
                },
                ..Default::default()
            },
        },
    }
}

fn ctrl_tus(vk: VIRTUAL_KEY) {
    unsafe {
        let girdi = [
            tus(VK_CONTROL, false),
            tus(vk, false),
            tus(vk, true),
            tus(VK_CONTROL, true),
        ];
        SendInput(&girdi, std::mem::size_of::<INPUT>() as i32);
    }
}

/// Otomasyon sonucu — arayüz buna göre ya "panoda" der ya kılavuz gösterir.
#[derive(Debug, Clone, PartialEq)]
pub enum KopyalamaSonucu {
    /// Pano sıra numarası değişti: resim gerçekten panoda.
    Panoda,
    /// Pencere bulundu ama pano değişmedi.
    PanoDegismedi,
    /// Verilen sürede UDE penceresi bulunamadı.
    PencereYok,
}

/// UDE'de açık belgeyi seçip panoya kopyalar ve panonun **gerçekten** değiştiğini doğrular.
///
/// `baslik_deseni` genellikle dosyanın uzantısız adıdır.
pub fn kopyala_panoya(baslik_deseni: &str) -> KopyalamaSonucu {
    let Some(hwnd) = pencere_bekle(baslik_deseni, PENCERE_BEKLEME) else {
        return KopyalamaSonucu::PencereYok;
    };
    one_getir(hwnd);
    sleep(Duration::from_millis(600));

    // Tuşlar tuvale ancak sayfaya tıklandıktan sonra ulaşıyor (ölçüldü).
    let mut r = RECT::default();
    if unsafe { GetWindowRect(hwnd, &mut r) }.is_ok() {
        let x = r.left + (r.right - r.left) / 2;
        let y = r.top + (r.bottom - r.top) * 6 / 10;
        tikla(x, y);
        sleep(Duration::from_millis(350));
    }

    let once = unsafe { GetClipboardSequenceNumber() };
    ctrl_tus(VK_A);
    sleep(Duration::from_millis(250));
    ctrl_tus(VK_C);
    // Java Swing'in olayı işlemesi için bekle.
    sleep(Duration::from_millis(700));
    let sonra = unsafe { GetClipboardSequenceNumber() };

    if once == sonra {
        KopyalamaSonucu::PanoDegismedi
    } else {
        KopyalamaSonucu::Panoda
    }
}

/// Dosyayı Explorer'da seçili olarak gösterir (UDE bulunamadığında yedek yol).
pub fn klasorde_goster(path: &Path) {
    let _ = std::process::Command::new("explorer")
        .arg(format!("/select,{}", path.display()))
        .spawn();
}

// WPARAM kullanılmıyor ama windows crate'inin bazı sürümlerinde import uyarısı olmasın diye:
#[allow(dead_code)]
fn _wparam_kullan() -> WPARAM {
    WPARAM(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn olmayan_pencere_bulunmaz() {
        assert!(pencere_bul("bu-baslikta-bir-pencere-yok-12345").is_none());
    }

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
    fn kisa_beklemede_bulunamayan_pencere_none() {
        let t = Instant::now();
        let sonuc = pencere_bekle("yok-boyle-bir-pencere-987", Duration::from_millis(400));
        assert!(sonuc.is_none());
        assert!(t.elapsed() < Duration::from_secs(3));
    }
}

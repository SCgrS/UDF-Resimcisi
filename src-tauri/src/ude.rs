//! UYAP Doküman Editörü (UDE) ile etkileşim — yalnızca Windows.
//!
//! Ölçülmüş davranışlar (bkz. `DOGRULAMA.md`):
//! * `.udf` dosya ilişkisi UDE'yi `getNewWPInstance EDITOR_TYPE_DOCUMENT "%1" "%~s1"`
//!   argümanlarıyla çağırır. Bu jetonlar olmadan UDE hiçbir şey açmadan çıkar; bu yüzden
//!   önce `ShellExecuteW` denenir, doğrudan çalıştırmaya ancak ilişki yoksa düşülür.
//! * Soğuk açılış 15-25 sn sürebilir.
//! * Taze açılan belgede tuval zaten odaklı: **tıklamaya gerek yok**, `AttachThreadInput` +
//!   `SetForegroundWindow` kalıbı yeterli (ölçüldü).
//! * Pencereyi **yerinde saydamlaştırmak** (WS_EX_LAYERED, alpha 0) tuş almasını engellemiyor:
//!   kopyalama kullanıcıya hiç gösterilmeden yapılabiliyor. Pencereyi ekran dışına *taşımak*
//!   ise UDE'nin `~/.uki/tercihler.xml` içindeki kayıtlı konumunu kalıcı bozuyor — yapılmamalı.
//! * Pano gerçekten değişti mi `GetClipboardSequenceNumber` ile doğrulanır — değişmediyse
//!   "kopyalandı" denmez.

#![cfg(windows)]

use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, WPARAM};
use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;
use windows::Win32::System::Registry::{
    RegGetValueW, HKEY_CLASSES_ROOT, RRF_RT_REG_SZ,
};
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, SetFocus, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_A, VK_C, VK_CONTROL,
};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetForegroundWindow, GetWindowLongW, GetWindowTextW,
    GetWindowThreadProcessId, IsIconic, IsWindowVisible, PostMessageW, SetForegroundWindow,
    SetLayeredWindowAttributes, SetWindowLongW, ShowWindow, GWL_EXSTYLE, LWA_ALPHA, SW_RESTORE,
    SW_SHOWNORMAL, WM_CLOSE, WS_EX_LAYERED,
};

/// Pencere aranırken beklenecek üst sınır. UDE soğuk açılışı 15-25 sn sürebiliyor.
const PENCERE_BEKLEME: Duration = Duration::from_secs(30);
const YOKLAMA_ARALIGI: Duration = Duration::from_millis(250);
/// Gizli kopyalamada pencereyi belirir belirmez yakalamak için hızlı yoklama.
const HIZLI_YOKLAMA: Duration = Duration::from_millis(20);

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
            "UYAP Doküman Editörü bulunamadı. Belge üretildi ama açmak ve panoya kopyalamak \
             için UDE'nin kurulu olması gerekiyor."
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

/// UDE'nin açılış görselinin pencere sınıfı. Soğuk açılışta ~140 ms'de beliriyor; kullanıcıya
/// hiç göstermemek için bunu da görünmez yapmak gerekiyor.
pub const SPLASH_SINIFI: &str = "JavaSplash";
/// Java Swing ana pencere sınıfı (belge pencereleri bu sınıfta).
pub const BELGE_SINIFI: &str = "SunAwtFrame";

struct Arama {
    desen: String,
    /// Boşsa sınıf denetlenmez.
    sinif: String,
    bulunan: HWND,
}

fn sinif_adi(hwnd: HWND) -> String {
    let mut buf = [0u16; 256];
    let n = unsafe { GetClassNameW(hwnd, &mut buf) };
    if n <= 0 {
        String::new()
    } else {
        String::from_utf16_lossy(&buf[..n as usize])
    }
}

unsafe extern "system" fn enum_geri(hwnd: HWND, lparam: LPARAM) -> windows::Win32::Foundation::BOOL {
    let arama = &mut *(lparam.0 as *mut Arama);
    if !IsWindowVisible(hwnd).as_bool() {
        return true.into();
    }
    if !arama.sinif.is_empty() && sinif_adi(hwnd) != arama.sinif {
        return true.into();
    }
    if arama.desen.is_empty() {
        arama.bulunan = hwnd;
        return false.into();
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

/// Sınıfı verilen ilk görünür pencere (başlığa bakmaz).
pub fn pencere_bul_sinif(sinif: &str) -> Option<HWND> {
    ara(String::new(), sinif.to_string())
}

fn ara(desen: String, sinif: String) -> Option<HWND> {
    let mut arama = Arama {
        desen,
        sinif,
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

/// Başlığında `desen` geçen görünür pencereyi bulur (küçük/büyük harf duyarsız).
pub fn pencere_bul(desen: &str) -> Option<HWND> {
    ara(desen.to_lowercase(), String::new())
}

/// Başlığında `desen` geçen **belge** penceresi (sınıf denetimli — açılış görselini seçmez).
pub fn belge_penceresi_bul(desen: &str) -> Option<HWND> {
    ara(desen.to_lowercase(), BELGE_SINIFI.to_string())
}

/// Pencereyi **yerinde** görünmez yapar (katmanlı pencere, saydamlık 0).
///
/// Neden taşımak yerine saydamlaştırma: UDE pencere konumunu `~/.uki/tercihler.xml` içindeki
/// `win_posx` / `win_posy` alanlarına **kalıcı** yazıyor. Pencereyi ekran dışına taşımak bu
/// değeri bozuyor; sonrasında kullanıcının normal UDE açılışı da ekran dışında oluyor
/// (ölçülerek görüldü). Saydamlaştırma pencere geometrisine hiç dokunmadığı için güvenli.
pub fn gorunmez_yap(hwnd: HWND) {
    unsafe {
        let ex = GetWindowLongW(hwnd, GWL_EXSTYLE);
        SetWindowLongW(hwnd, GWL_EXSTYLE, ex | WS_EX_LAYERED.0 as i32);
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 0, LWA_ALPHA);
    }
}

/// Pencereye kapanma isteği gönderir (kaydedilmemiş değişiklik yoksa sessizce kapanır).
pub fn kapat(hwnd: HWND) {
    unsafe {
        let _ = PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
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

/// Belgeyi UDE'de **görünmeden** açıp içeriğini panoya alır ve pencereyi kapatır.
///
/// Nasıl çalışıyor (hepsi ölçülerek doğrulandı):
/// 1. Dosya kabuk ilişkisiyle açılır.
/// 2. UDE'nin pencereleri belirir belirmez (açılış görseli ~140 ms, belge penceresi ~2 sn)
///    yerinde görünmez yapılır — kullanıcı hiçbir şey görmez, pencere konumu değişmez.
/// 3. Pencere odaklanır; **tıklamaya gerek yok**, taze açılan belgede tuval zaten odaklıdır.
/// 4. `Ctrl+A` + `Ctrl+C` gönderilir, panonun gerçekten değiştiği
///    `GetClipboardSequenceNumber` ile doğrulanır.
/// 5. Pencere kapatılır ve odak kullanıcının önceki penceresine geri verilir.
///
/// Dürüstlük kuralı: pano sıra numarası değişmediyse "kopyalandı" denmez.
pub fn kopyala_gorunmeden(path: &Path) -> KopyalamaSonucu {
    let onceki_fg = unsafe { GetForegroundWindow() };
    // Pencere başlığı "... - ad.udf (tam yol)" biçiminde; eşleştirmeyi **dosya adı** üzerinden
    // yapıyoruz. Tam yol kullanılamaz: `TEMP` 8.3 kısa biçimde olabiliyor
    // (`C:\Users\ARAHIN~1\...`) ama UDE başlıkta uzun biçimi gösteriyor
    // (`C:\Users\Çağrı Şahin\...`) — ölçüldü, eşleşme tutmuyordu.
    //
    // Kullanıcının benzer adlı kendi belgesini yakalamamak için çağıran taraf bu dosyaya
    // **benzersiz** bir ad verir (bkz. `commands::panoya_kopyala`).
    let desen = path
        .file_name()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    if belgeyi_ac(path).is_err() {
        return KopyalamaSonucu::PencereYok;
    }

    // Açılış görseli ve belge penceresi belirir belirmez görünmez yapılır.
    let bitis = Instant::now() + PENCERE_BEKLEME;
    let hwnd = loop {
        if let Some(splash) = pencere_bul_sinif(SPLASH_SINIFI) {
            gorunmez_yap(splash);
        }
        if let Some(h) = belge_penceresi_bul(&desen) {
            gorunmez_yap(h);
            break h;
        }
        if Instant::now() >= bitis {
            return KopyalamaSonucu::PencereYok;
        }
        sleep(HIZLI_YOKLAMA);
    };

    // Görünmez pencere de odak alabiliyor; SendInput odaklı pencereye gider.
    one_getir(hwnd);
    sleep(Duration::from_millis(450));

    let once = unsafe { GetClipboardSequenceNumber() };
    ctrl_tus(VK_A);
    sleep(Duration::from_millis(250));
    ctrl_tus(VK_C);
    // Java Swing'in olayı işlemesi için bekle.
    sleep(Duration::from_millis(700));
    let sonra = unsafe { GetClipboardSequenceNumber() };

    kapat(hwnd);
    if !onceki_fg.0.is_null() {
        one_getir(onceki_fg);
    }

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

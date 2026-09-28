//! UYAP Doküman Editörü (UDE) ile etkileşim — yalnızca Windows.
//!
//! Uygulama UDE **olmadan çalışmaz**: belge üretilir üretilmez UDE'de açılır, kullanıcı oradan
//! kopyalayıp dilekçesine yapıştırır. UDE bulunamazsa arayüz üretme düğmesini hiç açmaz.
//!
//! **UDE'yi bulmak.** `.udf` dosya ilişkisine (`HKCR\.udf`) tek başına güvenilmez: kullanıcının
//! kendi bölümündeki `HKCU\Software\Classes\.udf` değeri — "Birlikte aç" ile seçilmiş ya da
//! kaldırılmış bir programdan kalmış — kurucunun makine geneline yazdığı ilişkiyi gizler. UDE çift
//! tıklayınca yine açılır (Windows kullanıcının seçtiği programa bakar), ama `HKCR\.udf` başka bir
//! yeri gösterir. 1.7.1'de "UDE kurulu olduğu hâlde bulunamadı" hatası buydu. Sırayla bakılır:
//! 1. UDE'nin açma komutunu taşıyan ProgID kayıtları: kullanıcının Windows'ta seçtiği program,
//!    `.udf`'nin birleşik/makine/kullanıcı kökündeki değeri ve UDE kurucusunun bilinen ProgID
//!    adları. Komut UDE'ye özgü `getNewWPInstance` jetonunu taşımalı, exe diskte bulunmalı.
//! 2. Kurulu programlar listesi (Uninstall) → kurulum klasörü.
//! 3. Varsayılan kurulum klasörü (`C:\Uyap\Uyap Kelime Islemci`).
//! 4. Kullanıcının "UDE'nin yerini göster" ile seçtiği exe (ayarlarda).
//!
//! **Açmak.** 1. yoldan bulunduysa belge o kaydın açma komutuyla açılır (`ShellExecuteExW` +
//! `SEE_MASK_CLASSKEY`): çift tıklamanın aynısıdır ama `.udf` için seçilmiş varsayılan programa
//! değil UDE'nin kaydına bakar. Öbür yollarda exe `getNewWPInstance EDITOR_TYPE_DOCUMENT
//! "<belge>"` ile doğrudan çalıştırılır; bu jetonlar olmadan UDE hiçbir şey açmadan çıkar.
//! Kayıtlı komuttaki `"%~s1"` kabukta açılmıyor, UDE'ye olduğu gibi gidiyor: üç argüman yeterli
//! (ölçüm DOGRULAMA.md'de, 1.7.2).

#![cfg(windows)]

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use windows::core::{HSTRING, PCWSTR, PWSTR};
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::{
    RegCloseKey, RegEnumKeyExW, RegGetValueW, RegOpenKeyExW, HKEY, HKEY_CLASSES_ROOT,
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY,
    REG_SAM_FLAGS, RRF_RT_REG_SZ,
};
use windows::Win32::UI::Shell::{
    ShellExecuteExW, SEE_MASK_CLASSKEY, SEE_MASK_FLAG_NO_UI, SEE_MASK_NOASYNC, SHELLEXECUTEINFOW,
};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

/// UDE kurucusunun `.udf` için yazdığı ProgID (ASCII yazımı eski sürümlerden kalma).
const BILINEN_PROGIDLER: [&str; 2] = [
    "Adalet Bakanlığı.Uyap Kelime İşlemci",
    "Adalet Bakanligi.Uyap Kelime Islemci",
];

/// Kullanıcının Windows'ta `.udf` için seçtiği program bu anahtarın altında durur.
const DOSYA_ILISKISI: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.udf";

/// Kurulu programlar listesi.
const KALDIRMA: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall";

/// UDE'nin exe adları, 64 bit önce.
const EXE_ADLARI: [&str; 2] = ["Uyap Doküman Editörü.exe", "Uyap Doküman Editörü x86.exe"];

/// Bulunan UDE ve belgenin nasıl açılacağı.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ude {
    pub exe: PathBuf,
    acma: Acma,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Acma {
    /// Bu ProgID kaydının açma komutuyla, kabuk üzerinden.
    Kayit { kok: Kok, prog_id: String },
    /// Exe doğrudan, UDE'nin jetonlarıyla.
    Dogrudan,
}

/// Sınıf kayıtlarının kökü. Açarken komutun okunduğu kaydın aynısı kullanılsın diye saklanır.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kok {
    /// `HKCR`: kullanıcı ve makine kayıtlarının birleşimi (kullanıcınınki önce gelir).
    Birlesik,
    /// `HKLM\Software\Classes`: kurucunun yazdığı.
    Makine,
    /// `HKCU\Software\Classes`.
    Kullanici,
}

impl Kok {
    const HEPSI: [Kok; 3] = [Kok::Birlesik, Kok::Makine, Kok::Kullanici];

    fn hkey(self) -> HKEY {
        match self {
            Kok::Birlesik => HKEY_CLASSES_ROOT,
            Kok::Makine => HKEY_LOCAL_MACHINE,
            Kok::Kullanici => HKEY_CURRENT_USER,
        }
    }

    /// `alt` sınıf kaydının bu kökteki yolu.
    fn yol(self, alt: &str) -> String {
        match self {
            Kok::Birlesik => alt.to_string(),
            Kok::Makine | Kok::Kullanici => format!(r"Software\Classes\{alt}"),
        }
    }
}

/// UDE'yi bulur (sıra modül açıklamasında). `elle`: kullanıcının ayarlarda gösterdiği exe.
pub fn bul(elle: Option<&Path>) -> Option<Ude> {
    kayitlardan_bul()
        .or_else(kurulumdan_bul)
        .or_else(varsayilan_klasorden_bul)
        .or_else(|| {
            elle.filter(|p| gecerli_exe_mi(p)).map(|p| Ude {
                exe: p.to_path_buf(),
                acma: Acma::Dogrudan,
            })
        })
}

/// Kullanıcının gösterdiği dosya UDE'nin exe'si olabilir mi? (Diskte var ve adı uyuyor.)
pub fn gecerli_exe_mi(p: &Path) -> bool {
    ude_exe_mi(p) && p.is_file()
}

/// Dosya adı UDE'nin exe'sine benziyor mu? Aynı klasördeki Şablon Editörü sayılmaz.
fn ude_exe_mi(p: &Path) -> bool {
    let ad = p
        .file_name()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    ad.ends_with(".exe") && ad.contains("uyap") && (ad.contains("doküman") || ad.contains("dokuman"))
}

// ---------------------------------------------------------------------------
// 1. ProgID kayıtları
// ---------------------------------------------------------------------------

/// Bir ProgID kaydının açma komutu.
#[derive(Debug)]
struct KomutKaydi {
    kok: Kok,
    prog_id: String,
    komut: String,
}

fn kayitlardan_bul() -> Option<Ude> {
    let mut kayitlar = Vec::new();
    for prog_id in prog_id_adaylari() {
        for kok in Kok::HEPSI {
            let yol = kok.yol(&format!(r"{prog_id}\shell\open\command"));
            if let Some(komut) = reg_oku(kok.hkey(), &yol, "") {
                kayitlar.push(KomutKaydi {
                    kok,
                    prog_id: prog_id.clone(),
                    komut,
                });
            }
        }
    }
    kayitlardan_sec(&kayitlar, |p| p.is_file())
}

/// `.udf` ile ilişkili olabilecek ProgID adları, en güvenilirden başlayarak.
fn prog_id_adaylari() -> Vec<String> {
    let mut adlar = Vec::new();
    // Windows'ta seçilmiş varsayılan program (Windows 11'in yeni yeri önce).
    for (alt, deger) in [(r"\UserChoiceLatest\ProgId", "ProgId"), (r"\UserChoice", "ProgId")] {
        adlar.extend(reg_oku(HKEY_CURRENT_USER, &format!("{DOSYA_ILISKISI}{alt}"), deger));
    }
    for kok in Kok::HEPSI {
        adlar.extend(reg_oku(kok.hkey(), &kok.yol(".udf"), ""));
    }
    adlar.extend(BILINEN_PROGIDLER.iter().map(|s| s.to_string()));
    tekillestir(adlar)
}

/// Boş ve (büyük/küçük harf farkı gözetmeden) yinelenen adları atar; sıra korunur.
fn tekillestir(adlar: Vec<String>) -> Vec<String> {
    let mut gorulen = HashSet::new();
    adlar
        .into_iter()
        .filter(|a| !a.trim().is_empty() && gorulen.insert(a.to_lowercase()))
        .collect()
}

/// Komutu UDE'nin kendi komutu olan ilk kayıt. Yoksa exe'si UDE olan ama UDE'nin jetonlarını
/// taşımayan bir kayıt ("Birlikte aç" ile seçilmiş exe): o kayıtla açılırsa UDE hiçbir şey
/// açmadan çıkar, bu yüzden exe alınır ve doğrudan açılır.
fn kayitlardan_sec(kayitlar: &[KomutKaydi], var_mi: impl Fn(&Path) -> bool) -> Option<Ude> {
    let mut dogrudan = None;
    for k in kayitlar {
        let Some(exe) = komuttan_exe(&k.komut) else {
            continue;
        };
        if !var_mi(&exe) {
            continue;
        }
        if komut_ude_mi(&k.komut) {
            return Some(Ude {
                exe,
                acma: Acma::Kayit {
                    kok: k.kok,
                    prog_id: k.prog_id.clone(),
                },
            });
        }
        if dogrudan.is_none() && ude_exe_mi(&exe) {
            dogrudan = Some(Ude {
                exe,
                acma: Acma::Dogrudan,
            });
        }
    }
    dogrudan
}

/// UDE'nin kaydettiği komut mu? (`getNewWPInstance` yalnızca UDE'nin jetonu.)
fn komut_ude_mi(komut: &str) -> bool {
    komut.to_lowercase().contains("getnewwpinstance")
}

/// Komuttaki exe: `"C:\...\Uyap Doküman Editörü.exe" "getNewWPInstance" ...` → ilk tırnaklı parça.
fn komuttan_exe(komut: &str) -> Option<PathBuf> {
    let s = komut.trim();
    let exe = match s.strip_prefix('"') {
        Some(kalan) => kalan.split('"').next(),
        None => s.split_whitespace().next(),
    }?;
    if exe.is_empty() {
        None
    } else {
        Some(PathBuf::from(exe))
    }
}

// ---------------------------------------------------------------------------
// 2. Kurulu programlar listesi, 3. varsayılan klasör
// ---------------------------------------------------------------------------

fn kurulumdan_bul() -> Option<Ude> {
    let gorunumler = [
        (HKEY_LOCAL_MACHINE, KEY_WOW64_64KEY),
        (HKEY_LOCAL_MACHINE, KEY_WOW64_32KEY),
        (HKEY_CURRENT_USER, KEY_WOW64_64KEY),
    ];
    for (kok, gorunum) in gorunumler {
        let Some(liste) = AcikAnahtar::ac(kok, KALDIRMA, KEY_READ | gorunum) else {
            continue;
        };
        for alt in liste.alt_anahtarlar() {
            let ad = reg_oku(liste.0, &alt, "DisplayName").unwrap_or_default();
            if !kaldirma_adi_ude_mi(&ad) {
                continue;
            }
            let klasor = reg_oku(liste.0, &alt, "InstallLocation").unwrap_or_default();
            if let Some(exe) = klasorde_ude(Path::new(klasor.trim())) {
                return Some(Ude {
                    exe,
                    acma: Acma::Dogrudan,
                });
            }
        }
    }
    None
}

/// Kurulu programlar listesinde UDE'nin adı "Uyap Kelime İşlemci"; UYAP e-imza vb. sayılmaz.
fn kaldirma_adi_ude_mi(ad: &str) -> bool {
    let ad = ad.to_lowercase();
    ad.starts_with("uyap")
        && (ad.contains("kelime") || ad.contains("doküman") || ad.contains("dokuman"))
}

/// Klasördeki UDE exe'si: önce bilinen adlar, sonra adı uyan herhangi bir exe (64 bit önce).
fn klasorde_ude(klasor: &Path) -> Option<PathBuf> {
    if klasor.as_os_str().is_empty() {
        return None;
    }
    if let Some(p) = EXE_ADLARI.iter().map(|ad| klasor.join(ad)).find(|p| p.is_file()) {
        return Some(p);
    }
    let mut bulunanlar: Vec<PathBuf> = std::fs::read_dir(klasor)
        .ok()?
        .filter_map(|g| g.ok().map(|g| g.path()))
        .filter(|p| gecerli_exe_mi(p))
        .collect();
    // 32 bit kopya ("… x86.exe") sona.
    bulunanlar.sort_by_key(|p| p.to_string_lossy().to_lowercase().contains("x86"));
    bulunanlar.into_iter().next()
}

fn varsayilan_klasorden_bul() -> Option<Ude> {
    let surucu = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string());
    let exe = klasorde_ude(&PathBuf::from(format!(r"{surucu}\Uyap\Uyap Kelime Islemci")))?;
    Some(Ude {
        exe,
        acma: Acma::Dogrudan,
    })
}

// ---------------------------------------------------------------------------
// Açmak
// ---------------------------------------------------------------------------

/// `.udf` dosyasını UDE'de açar.
pub fn belgeyi_ac(ude: &Ude, belge: &Path) -> Result<()> {
    match &ude.acma {
        Acma::Kayit { kok, prog_id } => kayitla_ac(*kok, prog_id, belge).or_else(|e| {
            // Kayıt beklenmedik biçimde açılamazsa exe'yi doğrudan dene.
            dogrudan_ac(&ude.exe, belge)
                .map_err(|e2| anyhow!("{e}; doğrudan çalıştırma da olmadı: {e2}"))
        }),
        Acma::Dogrudan => dogrudan_ac(&ude.exe, belge),
    }
}

/// Çift tıklamanın aynısı, ama `.udf` için seçilmiş varsayılan programa değil bu kayda göre.
fn kayitla_ac(kok: Kok, prog_id: &str, belge: &Path) -> Result<()> {
    let anahtar = AcikAnahtar::ac(kok.hkey(), &kok.yol(prog_id), KEY_READ)
        .ok_or_else(|| anyhow!("UDE'nin kaydı açılamadı ({prog_id})"))?;
    let fiil = HSTRING::from("open");
    let dosya = HSTRING::from(belge.as_os_str());
    let mut bilgi = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        // NOASYNC: çağıran iş parçacığı hemen işini bitirebilir. FLAG_NO_UI: hatayı biz söyleriz.
        fMask: SEE_MASK_CLASSKEY | SEE_MASK_NOASYNC | SEE_MASK_FLAG_NO_UI,
        lpVerb: PCWSTR(fiil.as_ptr()),
        lpFile: PCWSTR(dosya.as_ptr()),
        nShow: SW_SHOWNORMAL.0,
        hkeyClass: anahtar.0,
        ..Default::default()
    };
    unsafe { ShellExecuteExW(&mut bilgi) }?;
    Ok(())
}

fn dogrudan_ac(exe: &Path, belge: &Path) -> Result<()> {
    std::process::Command::new(exe)
        .arg("getNewWPInstance")
        .arg("EDITOR_TYPE_DOCUMENT")
        .arg(belge)
        .spawn()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Kayıt defteri
// ---------------------------------------------------------------------------

/// Açık kayıt anahtarı; düşünce kapanır.
struct AcikAnahtar(HKEY);

impl AcikAnahtar {
    fn ac(kok: HKEY, yol: &str, erisim: REG_SAM_FLAGS) -> Option<Self> {
        let yol = HSTRING::from(yol);
        let mut h = HKEY::default();
        let r = unsafe { RegOpenKeyExW(kok, PCWSTR(yol.as_ptr()), 0, erisim, &mut h) };
        if r == ERROR_SUCCESS {
            Some(Self(h))
        } else {
            None
        }
    }

    fn alt_anahtarlar(&self) -> Vec<String> {
        let mut adlar = Vec::new();
        for i in 0u32.. {
            // Anahtar adı en çok 255 karakter.
            let mut ad = [0u16; 256];
            let mut uzunluk = ad.len() as u32;
            let r = unsafe {
                RegEnumKeyExW(
                    self.0,
                    i,
                    PWSTR(ad.as_mut_ptr()),
                    &mut uzunluk,
                    None,
                    PWSTR::null(),
                    None,
                    None,
                )
            };
            if r != ERROR_SUCCESS {
                break;
            }
            adlar.push(String::from_utf16_lossy(&ad[..uzunluk as usize]));
        }
        adlar
    }
}

impl Drop for AcikAnahtar {
    fn drop(&mut self) {
        unsafe {
            let _ = RegCloseKey(self.0);
        }
    }
}

/// Metin değeri okur; `deger` boşsa anahtarın varsayılan değeri. `REG_EXPAND_SZ` açılmış gelir
/// (`RRF_RT_REG_SZ` onu da kabul ediyor, ölçüldü).
fn reg_oku(kok: HKEY, alt_anahtar: &str, deger: &str) -> Option<String> {
    let key = HSTRING::from(alt_anahtar);
    let val = HSTRING::from(deger);
    let mut boyut: u32 = 0;
    unsafe {
        RegGetValueW(
            kok,
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
        let mut kapasite = (buf.len() * 2) as u32;
        RegGetValueW(
            kok,
            PCWSTR(key.as_ptr()),
            PCWSTR(val.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            Some(buf.as_mut_ptr() as *mut _),
            Some(&mut kapasite),
        )
        .ok()
        .ok()?;
        let son = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        Some(String::from_utf16_lossy(&buf[..son]))
    }
}

// ---------------------------------------------------------------------------
// Gezgin
// ---------------------------------------------------------------------------

/// Dosyayı Explorer'da seçili olarak gösterir.
pub fn klasorde_goster(path: &Path) {
    let _ = std::process::Command::new("explorer")
        .arg(format!("/select,{}", path.display()))
        .spawn();
}

/// Klasörü Explorer'da açar.
pub fn klasoru_ac(path: &Path) -> Result<()> {
    std::process::Command::new("explorer").arg(path).spawn()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const UDE_EXE: &str = r"C:\Uyap\Uyap Kelime Islemci\Uyap Doküman Editörü.exe";
    /// UDE 5.4.20 kurucusunun yazdığı komut (kayıt defterinden, olduğu gibi).
    const UDE_KOMUTU: &str = r#""C:\Uyap\Uyap Kelime Islemci\Uyap Doküman Editörü.exe" "getNewWPInstance" "EDITOR_TYPE_DOCUMENT" "%1" "%~s1""#;
    /// Ofis bilgisayarında görülen: kullanıcı kökündeki `.udf` → silinmiş bir exe.
    const BOZUK_KOMUT: &str = r#""C:\Users\x\Desktop\kopya.exe" "%1""#;

    fn kayit(kok: Kok, prog_id: &str, komut: &str) -> KomutKaydi {
        KomutKaydi {
            kok,
            prog_id: prog_id.into(),
            komut: komut.into(),
        }
    }

    #[test]
    fn komuttan_exe_ayiklanir() {
        assert_eq!(komuttan_exe(UDE_KOMUTU), Some(PathBuf::from(UDE_EXE)));
        assert_eq!(
            komuttan_exe(r"C:\Windows\notepad.exe %1"),
            Some(PathBuf::from(r"C:\Windows\notepad.exe"))
        );
        assert_eq!(komuttan_exe("   "), None);
        assert_eq!(komuttan_exe(r#""" "%1""#), None);
    }

    /// Sahadaki hata: `HKCR\.udf` kullanıcı kökündeki bozuk ilişkiyi gösteriyor, UDE'nin kaydı
    /// makine kökünde duruyor. UDE bulunmalı ve kendi kaydıyla açılmalı.
    #[test]
    fn kullanicidaki_bozuk_iliski_udeyi_gizlemez() {
        let kayitlar = [
            kayit(Kok::Birlesik, "udf_auto_file", BOZUK_KOMUT),
            kayit(Kok::Makine, BILINEN_PROGIDLER[0], UDE_KOMUTU),
        ];
        let ude = kayitlardan_sec(&kayitlar, |p| p == Path::new(UDE_EXE)).unwrap();
        assert_eq!(ude.exe, PathBuf::from(UDE_EXE));
        assert_eq!(
            ude.acma,
            Acma::Kayit {
                kok: Kok::Makine,
                prog_id: BILINEN_PROGIDLER[0].into()
            }
        );
    }

    #[test]
    fn diskte_olmayan_exe_sayilmaz() {
        let kayitlar = [kayit(Kok::Makine, BILINEN_PROGIDLER[0], UDE_KOMUTU)];
        assert_eq!(kayitlardan_sec(&kayitlar, |_| false), None);
    }

    #[test]
    fn baska_programin_kaydi_ude_sayilmaz() {
        let kayitlar = [kayit(Kok::Kullanici, "udf_auto_file", BOZUK_KOMUT)];
        assert_eq!(kayitlardan_sec(&kayitlar, |_| true), None);
    }

    /// "Birlikte aç" ile seçilmiş exe'nin kaydı jetonsuzdur: exe alınır, doğrudan açılır.
    #[test]
    fn jetonsuz_kayittan_exe_dogrudan_acilir() {
        let komut = format!(r#""{UDE_EXE}" "%1""#);
        let kayitlar = [kayit(Kok::Birlesik, r"Applications\Uyap Doküman Editörü.exe", &komut)];
        let ude = kayitlardan_sec(&kayitlar, |_| true).unwrap();
        assert_eq!(ude.exe, PathBuf::from(UDE_EXE));
        assert_eq!(ude.acma, Acma::Dogrudan);
    }

    #[test]
    fn jetonlu_kayit_jetonsuzdan_once_gelir() {
        let jetonsuz = format!(r#""{UDE_EXE}" "%1""#);
        let kayitlar = [
            kayit(Kok::Birlesik, r"Applications\Uyap Doküman Editörü.exe", &jetonsuz),
            kayit(Kok::Makine, BILINEN_PROGIDLER[0], UDE_KOMUTU),
        ];
        let ude = kayitlardan_sec(&kayitlar, |_| true).unwrap();
        assert!(matches!(ude.acma, Acma::Kayit { kok: Kok::Makine, .. }));
    }

    #[test]
    fn ad_tanima() {
        assert!(ude_exe_mi(Path::new(UDE_EXE)));
        assert!(ude_exe_mi(Path::new(r"C:\x\UYAP DOKÜMAN EDİTÖRÜ X86.EXE")));
        assert!(!ude_exe_mi(Path::new(r"C:\Uyap\Uyap Kelime Islemci\Uyap Sablon Editörü.exe")));
        assert!(!ude_exe_mi(Path::new(r"C:\Uyap\Uyap Kelime Islemci\editor_lib.jar")));
        assert!(kaldirma_adi_ude_mi("Uyap Kelime İşlemci"));
        assert!(!kaldirma_adi_ude_mi("UYAP e-imza"));
        assert!(!kaldirma_adi_ude_mi("Uyap Avukat Bilgi Sistemi"));
    }

    #[test]
    fn prog_id_adaylari_tekillestirilir() {
        let v = tekillestir(vec![
            "A".into(),
            String::new(),
            "a".into(),
            " ".into(),
            "B".into(),
        ]);
        assert_eq!(v, vec!["A".to_string(), "B".to_string()]);
    }

    #[test]
    fn klasorde_64_bit_exe_once_gelir() {
        let dir = std::env::temp_dir().join(format!("udfres-ude-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Uyap Sablon Editörü.exe"), b"").unwrap();
        assert_eq!(klasorde_ude(&dir), None, "Şablon Editörü UDE sayılmaz");

        // Bilinen adla değil de başka yazımla duran exe de bulunur.
        std::fs::write(dir.join("UYAP Dokuman Editoru x86.exe"), b"").unwrap();
        std::fs::write(dir.join("UYAP Dokuman Editoru.exe"), b"").unwrap();
        assert_eq!(klasorde_ude(&dir), Some(dir.join("UYAP Dokuman Editoru.exe")));

        std::fs::write(dir.join("Uyap Doküman Editörü.exe"), b"").unwrap();
        assert_eq!(klasorde_ude(&dir), Some(dir.join("Uyap Doküman Editörü.exe")));
        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(klasorde_ude(Path::new("")), None);
    }

    #[test]
    fn kayit_defteri_okunur() {
        let urun = reg_oku(
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
            "ProductName",
        );
        assert!(urun.is_some_and(|s| !s.is_empty()));
        let liste = AcikAnahtar::ac(HKEY_LOCAL_MACHINE, KALDIRMA, KEY_READ | KEY_WOW64_64KEY)
            .expect("kurulu programlar listesi açılmalı");
        assert!(!liste.alt_anahtarlar().is_empty());
    }

    /// Gerçek makine: UDE kurulu olmayan makinede (CI) `None` normaldir; bulunursa exe diskte olmalı.
    #[test]
    fn bulunan_exe_diskte() {
        if let Some(ude) = bul(None) {
            assert!(ude.exe.is_file(), "{ude:?}");
        }
    }
}

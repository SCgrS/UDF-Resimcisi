//! Arayüzün çağırdığı uçlar. Ağır iş (resim çözme, UDE bekleme) bloklayan iş parçacığında
//! çalışır; pencere donmaz.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::image_io::{self, YuklenenResim};
use crate::settings::{self, Ayarlar, BoyutTercihi};
use crate::udf::{self, BuildOptions, ImageSpec, Sizing, PT_PER_CM};

/// 10 MB UYAP sınırının altında bıraktığımız uyarı eşiği.
pub const UYARI_ESIGI: u64 = 9 * 1024 * 1024;

/// Oturum boyunca yüklü resimler. Arayüz bunlara indeksle atıf yapar; megabaytlar
/// arayüz katmanına hiç geçmez.
#[derive(Default)]
pub struct Oturum {
    pub resimler: Mutex<Vec<Kayit>>,
}

pub struct Kayit {
    pub ad: String,
    pub resim: YuklenenResim,
}

#[derive(Debug, Serialize)]
pub struct ResimBilgi {
    pub indeks: usize,
    pub ad: String,
    pub px_w: u32,
    pub px_h: u32,
    pub bicim: String,
    pub orijinal_korundu: bool,
    pub dondurul: bool,
    pub bayt: usize,
    /// Küçük önizleme (data URI). Yalnızca arayüzde göstermek için üretilir; belgeye girmez.
    pub onizleme: String,
}

#[derive(Debug, Serialize)]
pub struct OlcuBilgi {
    pub indeks: usize,
    pub genislik_cm: f64,
    pub yukseklik_cm: f64,
    pub sayfayi_asiyor: bool,
}

#[derive(Debug, Serialize)]
pub struct UretimSonucu {
    pub yol: String,
    pub boyut_bayt: u64,
    /// 9 MB eşiği aşıldıysa arayüz küçültme teklifi gösterir.
    pub buyuk: bool,
}

#[derive(Debug, Serialize)]
pub struct KipSonucu {
    /// "panoda" | "pano-degismedi" | "pencere-yok" | "acildi" | "otomasyon-kapali"
    pub durum: String,
    pub mesaj: String,
}

fn hata<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

// ---------------------------------------------------------------------------
// Ayarlar
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn ayarlari_getir() -> Ayarlar {
    settings::yukle()
}

#[tauri::command]
pub fn ayarlari_kaydet(ayarlar: Ayarlar) -> Result<(), String> {
    settings::kaydet(&ayarlar).map_err(hata)
}

// ---------------------------------------------------------------------------
// Resim yükleme
// ---------------------------------------------------------------------------

/// Yükleme sonucu: okunabilenler ve okunamayanlar birlikte döner — okunamayan bir dosya
/// sessizce yutulmaz, arayüz adını söyler.
#[derive(Debug, Serialize)]
pub struct YuklemeSonucu {
    pub resimler: Vec<ResimBilgi>,
    pub hatalar: Vec<String>,
}

#[tauri::command]
pub async fn resimleri_yukle(
    yollar: Vec<String>,
    oturum: State<'_, Oturum>,
) -> Result<YuklemeSonucu, String> {
    let yuklenenler = tauri::async_runtime::spawn_blocking(move || {
        let mut cikti: Vec<Result<Kayit, String>> = Vec::new();
        for y in yollar {
            let p = PathBuf::from(&y);
            let ad = p
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "resim".to_string());
            match image_io::dosyadan(&p) {
                Ok(r) => cikti.push(Ok(Kayit { ad, resim: r })),
                Err(e) => cikti.push(Err(format!("{ad}: {e}"))),
            }
        }
        cikti
    })
    .await
    .map_err(hata)?;

    let mut hatalar = Vec::new();
    {
        let mut liste = oturum.resimler.lock().map_err(hata)?;
        for k in yuklenenler {
            match k {
                Ok(kayit) => liste.push(kayit),
                Err(e) => hatalar.push(e),
            }
        }
    }
    Ok(YuklemeSonucu {
        resimler: liste_bilgisi(&oturum)?,
        hatalar,
    })
}

#[tauri::command]
pub async fn panodan_al(oturum: State<'_, Oturum>) -> Result<Vec<ResimBilgi>, String> {
    let r = tauri::async_runtime::spawn_blocking(image_io::panodan)
        .await
        .map_err(hata)?
        .map_err(hata)?;
    {
        let mut liste = oturum.resimler.lock().map_err(hata)?;
        let n = liste.len() + 1;
        liste.push(Kayit {
            ad: format!("pano-{n}.png"),
            resim: r,
        });
    }
    liste_bilgisi(&oturum)
}

#[tauri::command]
pub fn resmi_cikar(indeks: usize, oturum: State<'_, Oturum>) -> Result<Vec<ResimBilgi>, String> {
    {
        let mut liste = oturum.resimler.lock().map_err(hata)?;
        if indeks < liste.len() {
            liste.remove(indeks);
        }
    }
    liste_bilgisi(&oturum)
}

#[tauri::command]
pub fn listeyi_temizle(oturum: State<'_, Oturum>) -> Result<Vec<ResimBilgi>, String> {
    oturum.resimler.lock().map_err(hata)?.clear();
    liste_bilgisi(&oturum)
}

#[tauri::command]
pub fn liste_getir(oturum: State<'_, Oturum>) -> Result<Vec<ResimBilgi>, String> {
    liste_bilgisi(&oturum)
}

fn liste_bilgisi(oturum: &State<'_, Oturum>) -> Result<Vec<ResimBilgi>, String> {
    let liste = oturum.resimler.lock().map_err(hata)?;
    Ok(liste
        .iter()
        .enumerate()
        .map(|(i, k)| ResimBilgi {
            indeks: i,
            ad: k.ad.clone(),
            px_w: k.resim.spec.px_w,
            px_h: k.resim.spec.px_h,
            bicim: k.resim.bicim.clone(),
            orijinal_korundu: k.resim.orijinal_korundu,
            dondurul: k.resim.dondurul,
            bayt: k.resim.spec.bytes.len(),
            onizleme: onizleme_uret(&k.resim.spec),
        })
        .collect())
}

/// Arayüz için küçük önizleme üretir. Başarısız olursa boş dize döner (önizleme yoksa
/// kart yine de sayılarla gösterilir).
fn onizleme_uret(spec: &ImageSpec) -> String {
    use base64::engine::general_purpose::STANDARD as B64;
    use base64::Engine;

    let Ok(img) = image::load_from_memory(&spec.bytes) else {
        return String::new();
    };
    let k = img.thumbnail(280, 280);
    let mut buf = Vec::new();
    if k.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .is_err()
    {
        return String::new();
    }
    format!("data:image/png;base64,{}", B64.encode(&buf))
}

// ---------------------------------------------------------------------------
// Ölçü önizlemesi
// ---------------------------------------------------------------------------

fn sizing_of(b: BoyutTercihi) -> Sizing {
    match b {
        BoyutTercihi::SayfayaSigdir => Sizing::FitPage,
        BoyutTercihi::YuzdeYuz => Sizing::Actual,
        BoyutTercihi::GenislikCm(cm) => Sizing::WidthCm(cm),
    }
}

#[tauri::command]
pub fn olculeri_hesapla(
    boyut: BoyutTercihi,
    oturum: State<'_, Oturum>,
) -> Result<Vec<OlcuBilgi>, String> {
    let liste = oturum.resimler.lock().map_err(hata)?;
    let page = udf::model::PageFormat::default();
    Ok(liste
        .iter()
        .enumerate()
        .map(|(i, k)| {
            let (w, h) =
                udf::goruntuleme_boyutu(k.resim.spec.px_w, k.resim.spec.px_h, sizing_of(boyut), &page);
            OlcuBilgi {
                indeks: i,
                genislik_cm: w / PT_PER_CM,
                yukseklik_cm: h / PT_PER_CM,
                sayfayi_asiyor: w > page.usable_width() + 0.01
                    || h > page.usable_height() + 0.01,
            }
        })
        .collect())
}

// ---------------------------------------------------------------------------
// UDF üretimi
// ---------------------------------------------------------------------------

/// Çakışma varsa " (2)", " (3)" … ekler.
pub fn benzersiz_yol(klasor: &Path, govde: &str) -> PathBuf {
    let ilk = klasor.join(format!("{govde}.udf"));
    if !ilk.exists() {
        return ilk;
    }
    for n in 2..1000 {
        let y = klasor.join(format!("{govde} ({n}).udf"));
        if !y.exists() {
            return y;
        }
    }
    klasor.join(format!("{govde}-{}.udf", chrono::Local::now().format("%H%M%S")))
}

/// Dosya adı gövdesi: tek resimde kaynak adı, çoklu resimde ilk dosyanın adı;
/// pano kaynaklıysa tarih damgası.
fn govde_sec(liste: &[Kayit]) -> String {
    // Panodan gelen resmin kaynak adı yok: tarih damgası kullan.
    if liste[0].ad.starts_with("pano-") {
        return format!("resimler-{}", chrono::Local::now().format("%Y%m%d-%H%M"));
    }
    Path::new(&liste[0].ad)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "resim".to_string())
}

fn temiz_dosya_adi(s: &str) -> String {
    s.chars()
        .map(|c| if r#"\/:*?"<>|"#.contains(c) { '_' } else { c })
        .collect::<String>()
        .trim()
        .to_string()
}

#[derive(Debug, Deserialize)]
pub struct UretimIstegi {
    pub ayri_sayfa: bool,
    pub boyut: BoyutTercihi,
    pub cikti_klasoru: String,
    /// true ise resimler 300 DPI'ya (≈2200 px uzun kenar) küçültülerek gömülür.
    pub kucult: bool,
}

#[tauri::command]
pub async fn udf_uret(
    istek: UretimIstegi,
    oturum: State<'_, Oturum>,
) -> Result<UretimSonucu, String> {
    let (specler, govde) = {
        let liste = oturum.resimler.lock().map_err(hata)?;
        if liste.is_empty() {
            return Err("Önce bir resim ekleyin.".to_string());
        }
        (
            liste.iter().map(|k| k.resim.spec.clone()).collect::<Vec<_>>(),
            govde_sec(&liste),
        )
    };

    let sonuc = tauri::async_runtime::spawn_blocking(move || -> Result<UretimSonucu, String> {
        let specler = if istek.kucult {
            specler
                .iter()
                .map(|s| image_io::kucult(s, image_io::HEDEF_UZUN_KENAR_300DPI))
                .collect::<Result<Vec<_>, _>>()
                .map_err(hata)?
        } else {
            specler
        };

        let opts = BuildOptions {
            separate_pages: istek.ayri_sayfa,
            sizing: sizing_of(istek.boyut),
            page: udf::model::PageFormat::default(),
        };
        let bytes = udf::build_udf(&specler, &opts).map_err(hata)?;

        let klasor = PathBuf::from(&istek.cikti_klasoru);
        std::fs::create_dir_all(&klasor)
            .map_err(|e| format!("Çıktı klasörü oluşturulamadı ({}): {e}", klasor.display()))?;
        let yol = benzersiz_yol(&klasor, &temiz_dosya_adi(&govde));
        std::fs::write(&yol, &bytes)
            .map_err(|e| format!("Dosya yazılamadı ({}): {e}", yol.display()))?;

        let boyut = bytes.len() as u64;
        Ok(UretimSonucu {
            yol: yol.to_string_lossy().to_string(),
            boyut_bayt: boyut,
            buyuk: boyut > UYARI_ESIGI,
        })
    })
    .await
    .map_err(hata)??;

    Ok(sonuc)
}

// ---------------------------------------------------------------------------
// Kip A / Kip B
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn kip_a_calistir(yol: String, otomasyon: bool) -> Result<KipSonucu, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let p = PathBuf::from(&yol);
        #[cfg(windows)]
        {
            if let Err(e) = crate::ude::belgeyi_ac(&p) {
                crate::ude::klasorde_goster(&p);
                return Err(e.to_string());
            }
            if !otomasyon {
                return Ok(KipSonucu {
                    durum: "otomasyon-kapali".into(),
                    mesaj: "Açılan pencerede Ctrl+A → Ctrl+C yapın, sonra dilekçenizde Ctrl+V."
                        .into(),
                });
            }
            let desen = p
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            match crate::ude::kopyala_panoya(&desen) {
                crate::ude::KopyalamaSonucu::Panoda => Ok(KipSonucu {
                    durum: "panoda".into(),
                    mesaj: "Kaliteli resim panoda — UDF'nize Ctrl+V yapın.".into(),
                }),
                crate::ude::KopyalamaSonucu::PanoDegismedi => Ok(KipSonucu {
                    durum: "pano-degismedi".into(),
                    mesaj: "Kopyalama doğrulanamadı. Açılan pencerede Ctrl+A → Ctrl+C yapın, \
                            sonra dilekçenizde Ctrl+V."
                        .into(),
                }),
                crate::ude::KopyalamaSonucu::PencereYok => Ok(KipSonucu {
                    durum: "pencere-yok".into(),
                    mesaj: "UDE penceresi 30 sn içinde bulunamadı — açılınca Ctrl+A, Ctrl+C yapın."
                        .into(),
                }),
            }
        }
        #[cfg(not(windows))]
        {
            let _ = otomasyon;
            Err("Kip A yalnızca Windows'ta çalışır.".to_string())
        }
    })
    .await
    .map_err(hata)?
}

#[tauri::command]
pub async fn kip_b_calistir(yol: String) -> Result<KipSonucu, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let p = PathBuf::from(&yol);
        #[cfg(windows)]
        {
            crate::ude::belgeyi_ac(&p).map_err(|e| e.to_string())?;
        }
        Ok(KipSonucu {
            durum: "acildi".into(),
            mesaj: format!("Belge açıldı ve şuraya kaydedildi: {}", p.display()),
        })
    })
    .await
    .map_err(hata)?
}

#[tauri::command]
pub fn klasorde_goster(yol: String) {
    #[cfg(windows)]
    crate::ude::klasorde_goster(Path::new(&yol));
    #[cfg(not(windows))]
    let _ = yol;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dosya_adi_yasak_karakterleri_temizler() {
        assert_eq!(temiz_dosya_adi("a:b*c?d"), "a_b_c_d");
        assert_eq!(temiz_dosya_adi("normal ad"), "normal ad");
    }

    #[test]
    fn benzersiz_yol_cakismada_numaralandirir() {
        let dir = std::env::temp_dir().join(format!("udfres-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let ilk = benzersiz_yol(&dir, "belge");
        assert!(ilk.ends_with("belge.udf"));
        std::fs::write(&ilk, b"x").unwrap();
        let ikinci = benzersiz_yol(&dir, "belge");
        assert!(ikinci.ends_with("belge (2).udf"), "{ikinci:?}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn uyari_esigi_10mb_sinirinin_altinda() {
        const _: () = assert!(UYARI_ESIGI < 10 * 1024 * 1024);
    }
}

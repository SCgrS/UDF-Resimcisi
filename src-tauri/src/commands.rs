//! Arayüzün çağırdığı uçlar. Ağır iş (resim çözme, belge üretimi, ağ) bloklayan iş
//! parçacığında çalışır; pencere donmaz.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;
use tauri::State;

use crate::image_io::{self, Kalite, YuklenenResim};
use crate::settings::{self, Ayarlar};
use crate::udf::{self, BuildOptions, ImageSpec, Sizing, PT_PER_CM};

/// Oturum boyunca yüklü resimler. Arayüz bunlara indeksle atıf yapar; megabaytlar
/// arayüz katmanına hiç geçmez.
#[derive(Default)]
pub struct Oturum {
    pub resimler: Mutex<Vec<Kayit>>,
}

pub struct Kayit {
    pub ad: String,
    pub resim: YuklenenResim,
    /// Kalite basamağı başına bir kez üretilen indirgenmiş sürüm. Boyut satırı her
    /// seçim değişiminde yeniden hesaplandığı için aynı resmi tekrar tekrar ölçeklemeyelim.
    onbellek: HashMap<Kalite, ImageSpec>,
    /// Basamak başına, UDE'ye yapıştırıldığında kaplayacağı yer (PNG kestirimi, bayt).
    yapistirma_onbellek: HashMap<Kalite, usize>,
}

impl Kayit {
    fn yeni(ad: String, resim: YuklenenResim) -> Self {
        Self {
            ad,
            resim,
            onbellek: HashMap::new(),
            yapistirma_onbellek: HashMap::new(),
        }
    }
}

/// Boyut satırının iki sayısı.
#[derive(Debug, Serialize)]
pub struct BoyutBilgisi {
    /// Üretilecek `.udf` dosyası (gerçekten kurulup ölçülür).
    pub dosya: u64,
    /// Belge UDE'de açılıp dilekçeye yapıştırıldığında resimlerin kaplayacağı yer (yaklaşık;
    /// UDE her resmi PNG olarak yeniden kodlar).
    pub yapistirma: u64,
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
    pub genislik_cm: f64,
    pub yukseklik_cm: f64,
    /// Küçük önizleme (data URI). Yalnızca arayüzde göstermek için üretilir; belgeye girmez.
    pub onizleme: String,
}

#[derive(Debug, Serialize)]
pub struct YuklemeSonucu {
    pub resimler: Vec<ResimBilgi>,
    pub hatalar: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct UretimSonucu {
    pub yol: String,
    pub boyut_bayt: u64,
}

const UDE_YOK: &str = "UYAP Doküman Editörü bu bilgisayarda bulunamadı. Uygulama onsuz çalışmaz: \
                       önce UDE'yi kurun, sonra uygulamayı yeniden açın.";

fn hata<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

/// Görüntüleme boyutu artık seçenek değil: resim tam çözünürlükle gömülür, sayfaya sığacak
/// şekilde yerleştirilir. (Bitmap'e dokunulmaz; yalnızca `width`/`height` punto değerleri
/// sayfaya göre hesaplanır.)
const BOYUT: Sizing = Sizing::FitPage;

fn secenekler(ayri_sayfa: bool) -> BuildOptions {
    BuildOptions {
        separate_pages: ayri_sayfa,
        sizing: BOYUT,
        page: udf::model::PageFormat::default(),
    }
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

/// Uygulamanın sürümü (ayarlar penceresinde gösterilir).
#[tauri::command]
pub fn surum() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Arayüz, UDE yoksa üretme düğmesini hiç açmasın diye.
#[tauri::command]
pub fn ude_kurulu_mu() -> bool {
    #[cfg(windows)]
    {
        crate::ude::kurulu_mu()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

// ---------------------------------------------------------------------------
// Resim listesi
// ---------------------------------------------------------------------------

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
                Ok(r) => cikti.push(Ok(Kayit::yeni(ad, r))),
                Err(e) => cikti.push(Err(format!("{ad}: {e}"))),
            }
        }
        cikti
    })
    .await
    .map_err(hata)?;

    let mut hatalar = Vec::new();
    {
        // Aynı dosya birden çok kez eklenebilir: liste sıraya göre büyür, tekilleştirme yok.
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
        liste.push(Kayit::yeni(format!("pano-{n}.png"), r));
    }
    liste_bilgisi(&oturum)
}

/// Sağ tık menüsündeki "Yapıştır" öğesi soluk mu olsun?
#[tauri::command]
pub async fn panoda_resim_var_mi() -> bool {
    tauri::async_runtime::spawn_blocking(image_io::panoda_resim_var)
        .await
        .unwrap_or(false)
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
    let page = udf::model::PageFormat::default();
    Ok(liste
        .iter()
        .enumerate()
        .map(|(i, k)| {
            let (w, h) = udf::goruntuleme_boyutu(k.resim.spec.px_w, k.resim.spec.px_h, BOYUT, &page);
            ResimBilgi {
                indeks: i,
                ad: k.ad.clone(),
                px_w: k.resim.spec.px_w,
                px_h: k.resim.spec.px_h,
                bicim: k.resim.bicim.clone(),
                orijinal_korundu: k.resim.orijinal_korundu,
                dondurul: k.resim.dondurul,
                bayt: k.resim.spec.bytes.len(),
                genislik_cm: w / PT_PER_CM,
                yukseklik_cm: h / PT_PER_CM,
                onizleme: onizleme_uret(&k.resim.spec),
            }
        })
        .collect())
}

/// Arayüz için küçük önizleme üretir. Başarısız olursa boş dize döner.
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
// Belge üretimi
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

fn belge_uret(
    ayri_sayfa: bool,
    kalite: Kalite,
    oturum: &State<'_, Oturum>,
) -> Result<(Vec<u8>, String), String> {
    let (specler, govde) = {
        let sayfa = udf::model::PageFormat::default();
        let mut liste = oturum.resimler.lock().map_err(hata)?;
        if liste.is_empty() {
            return Err("Önce bir resim ekleyin.".to_string());
        }
        let govde = govde_sec(&liste);
        let mut specler = Vec::with_capacity(liste.len());
        for k in liste.iter_mut() {
            if kalite == Kalite::Orijinal {
                specler.push(k.resim.spec.clone());
                continue;
            }
            if let Some(hazir) = k.onbellek.get(&kalite) {
                specler.push(hazir.clone());
                continue;
            }
            let indirilmis = image_io::kaliteye_indir(&k.resim.spec, kalite, &sayfa).map_err(hata)?;
            k.onbellek.insert(kalite, indirilmis.clone());
            specler.push(indirilmis);
        }
        (specler, govde)
    };
    let bytes = udf::build_udf(&specler, &secenekler(ayri_sayfa)).map_err(hata)?;
    Ok((bytes, temiz_dosya_adi(&govde)))
}

/// Listedeki resimlerin, seçili basamakta UDE'ye yapıştırıldığında kaplayacağı yer.
/// `belge_uret` hemen önce çağrıldığı için indirgenmiş sürümler önbellekte hazırdır.
fn yapistirma_toplami(kalite: Kalite, oturum: &State<'_, Oturum>) -> Result<u64, String> {
    let mut liste = oturum.resimler.lock().map_err(hata)?;
    let mut toplam = 0u64;
    for k in liste.iter_mut() {
        if let Some(b) = k.yapistirma_onbellek.get(&kalite) {
            toplam += *b as u64;
            continue;
        }
        let b = {
            let spec = if kalite == Kalite::Orijinal {
                &k.resim.spec
            } else {
                k.onbellek.get(&kalite).unwrap_or(&k.resim.spec)
            };
            image_io::yapistirma_boyutu(spec).map_err(hata)?
        };
        k.yapistirma_onbellek.insert(kalite, b);
        toplam += b as u64;
    }
    Ok(toplam)
}

/// Boyut satırı: üretilecek `.udf` dosyasının boyutu (gerçekten belge kurup ölçer — tahmin
/// değil) ve dilekçeye yapıştırıldığında kaplayacağı yer (yaklaşık).
#[tauri::command]
pub async fn belge_boyutu(
    ayri_sayfa: bool,
    kalite: Kalite,
    oturum: State<'_, Oturum>,
) -> Result<BoyutBilgisi, String> {
    let bos = oturum.resimler.lock().map_err(hata)?.is_empty();
    if bos {
        return Ok(BoyutBilgisi {
            dosya: 0,
            yapistirma: 0,
        });
    }
    let (bytes, _) = belge_uret(ayri_sayfa, kalite, &oturum)?;
    let yapistirma = yapistirma_toplami(kalite, &oturum)?;
    Ok(BoyutBilgisi {
        dosya: bytes.len() as u64,
        yapistirma,
    })
}

/// Belgeyi kaydetme klasörüne yazar ve UDE'de açar. UDE yoksa hiç başlamaz.
#[tauri::command]
pub async fn udfde_ac(
    ayri_sayfa: bool,
    kalite: Kalite,
    cikti_klasoru: String,
    oturum: State<'_, Oturum>,
) -> Result<UretimSonucu, String> {
    if !ude_kurulu_mu() {
        return Err(UDE_YOK.to_string());
    }
    let (bytes, govde) = belge_uret(ayri_sayfa, kalite, &oturum)?;
    let klasor = PathBuf::from(&cikti_klasoru);

    tauri::async_runtime::spawn_blocking(move || -> Result<UretimSonucu, String> {
        std::fs::create_dir_all(&klasor)
            .map_err(|e| format!("Kaydetme klasörü oluşturulamadı ({}): {e}", klasor.display()))?;
        let yol = benzersiz_yol(&klasor, &govde);
        std::fs::write(&yol, &bytes)
            .map_err(|e| format!("Dosya yazılamadı ({}): {e}", yol.display()))?;

        #[cfg(windows)]
        crate::ude::belgeyi_ac(&yol).map_err(|e| {
            format!(
                "Belge kaydedildi ({}) ama UYAP Doküman Editörü açılamadı: {e}",
                yol.display()
            )
        })?;

        Ok(UretimSonucu {
            yol: yol.to_string_lossy().to_string(),
            boyut_bayt: bytes.len() as u64,
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

/// "Klasörü aç" düğmesi: kaydetme klasörünü Gezgin'de açar, yoksa önce oluşturur.
#[tauri::command]
pub fn klasoru_ac(klasor: String) -> Result<(), String> {
    let p = PathBuf::from(&klasor);
    std::fs::create_dir_all(&p)
        .map_err(|e| format!("Klasör oluşturulamadı ({}): {e}", p.display()))?;
    #[cfg(windows)]
    crate::ude::klasoru_ac(&p).map_err(hata)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Güncelleme denetimi
// ---------------------------------------------------------------------------

/// Sürüm bilgisinin çekildiği adres. Depo herkese açık değilse bu adres 404 döner ve
/// denetim dürüstçe "bilgi alınamadı" der.
const SURUM_ADRESI: &str = "https://api.github.com/repos/SCgrS/UDF-Resimcisi/releases/latest";
/// Kurulum dosyasının sürümden bağımsız adı (bkz. depo README).
const KURULUM_DOSYASI: &str = "UDF-Resimcisi-kurulum.exe";

/// Sınama için adres ortam değişkeniyle bir yerel sunucuya yönlendirilebilir.
fn surum_adresi() -> String {
    std::env::var("UDF_RESIMCISI_SURUM_ADRESI").unwrap_or_else(|_| SURUM_ADRESI.to_string())
}

#[derive(Debug, Serialize)]
pub struct GuncellemeSonucu {
    /// "guncel" | "yeni-surum-var" | "ulasilamadi"
    pub durum: String,
    pub mesaj: String,
    pub bu_surum: String,
    pub yeni_surum: String,
    /// Kurulum dosyasının indirme adresi (yalnız "yeni-surum-var" durumunda dolu).
    pub indirme_adresi: String,
}

fn surum_parcala(s: &str) -> (u32, u32, u32) {
    let t = s.trim_start_matches('v');
    let mut p = t.split('.').map(|x| x.parse::<u32>().unwrap_or(0));
    (
        p.next().unwrap_or(0),
        p.next().unwrap_or(0),
        p.next().unwrap_or(0),
    )
}

#[tauri::command]
pub async fn guncelleme_denetle() -> Result<GuncellemeSonucu, String> {
    let bu = env!("CARGO_PKG_VERSION").to_string();

    tauri::async_runtime::spawn_blocking(move || {
        // Hata türü büyük olduğu için hemen metne çevriliyor (clippy: result_large_err).
        let yanit: Result<String, String> = ureq::get(&surum_adresi())
            .set("User-Agent", "UDF-Resimcisi")
            .set("Accept", "application/vnd.github+json")
            .timeout(std::time::Duration::from_secs(15))
            .call()
            .map_err(|e| e.to_string())
            .and_then(|r| r.into_string().map_err(|e| e.to_string()));

        let govde = match yanit {
            Ok(s) => s,
            Err(e) => {
                return GuncellemeSonucu {
                    durum: "ulasilamadi".into(),
                    mesaj: format!(
                        "Sürüm bilgisi alınamadı; internet bağlantınızı denetleyin. ({e})"
                    ),
                    bu_surum: bu.clone(),
                    yeni_surum: String::new(),
                    indirme_adresi: String::new(),
                }
            }
        };

        let json: serde_json::Value = match serde_json::from_str(&govde) {
            Ok(v) => v,
            Err(e) => {
                return GuncellemeSonucu {
                    durum: "ulasilamadi".into(),
                    mesaj: format!("Sürüm bilgisi okunamadı: {e}"),
                    bu_surum: bu.clone(),
                    yeni_surum: String::new(),
                    indirme_adresi: String::new(),
                }
            }
        };

        let etiket = json["tag_name"].as_str().unwrap_or_default().to_string();
        let adres = json["assets"]
            .as_array()
            .and_then(|a| {
                a.iter()
                    .find(|x| x["name"].as_str() == Some(KURULUM_DOSYASI))
                    .and_then(|x| x["browser_download_url"].as_str())
            })
            .unwrap_or_default()
            .to_string();

        if etiket.is_empty() {
            return GuncellemeSonucu {
                durum: "ulasilamadi".into(),
                mesaj: "Yayımlanmış bir sürüm bulunamadı.".into(),
                bu_surum: bu.clone(),
                yeni_surum: String::new(),
                indirme_adresi: String::new(),
            };
        }

        if surum_parcala(&etiket) > surum_parcala(&bu) {
            GuncellemeSonucu {
                durum: "yeni-surum-var".into(),
                mesaj: format!("Yeni sürüm var: {etiket}"),
                bu_surum: bu.clone(),
                yeni_surum: etiket,
                indirme_adresi: adres,
            }
        } else {
            GuncellemeSonucu {
                durum: "guncel".into(),
                mesaj: format!("En son sürümü kullanıyorsunuz ({bu})."),
                bu_surum: bu.clone(),
                yeni_surum: etiket,
                indirme_adresi: String::new(),
            }
        }
    })
    .await
    .map_err(hata)
}

/// Kurulum dosyasını indirir, sessiz ilerleme penceresiyle çalıştırır ve uygulamayı kapatır;
/// kurucu bitince (`/R`) uygulamayı yeniden açar. Yalnızca kullanıcı isteğiyle çağrılır.
#[tauri::command]
pub async fn guncellemeyi_kur(
    app: tauri::AppHandle,
    indirme_adresi: String,
) -> Result<String, String> {
    if indirme_adresi.is_empty() {
        return Err("İndirme adresi yok.".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
        let yanit = ureq::get(&indirme_adresi)
            .set("User-Agent", "UDF-Resimcisi")
            .timeout(std::time::Duration::from_secs(120))
            .call()
            .map_err(|e| format!("İndirilemedi: {e}"))?;

        let mut veri = Vec::new();
        std::io::copy(&mut yanit.into_reader(), &mut veri)
            .map_err(|e| format!("İndirme yarıda kesildi: {e}"))?;

        let klasor = std::env::temp_dir().join("UDF Resimcisi");
        std::fs::create_dir_all(&klasor).map_err(hata)?;
        let yol = klasor.join(KURULUM_DOSYASI);
        std::fs::write(&yol, &veri).map_err(|e| format!("Kaydedilemedi: {e}"))?;

        // /P: yalnızca ilerleme penceresi, soru sormaz; /R: bitince uygulamayı yeniden aç.
        // Kurucu çalışan uygulamayı kendisi de kapatır; biz yine de dosya kilidi kalmasın diye
        // kısa bir gecikmeyle çıkıyoruz (arayüz bu arada "kuruluyor" mesajını gösterir).
        std::process::Command::new(&yol)
            .args(["/P", "/R"])
            .spawn()
            .map_err(|e| format!("Kurulum başlatılamadı: {e}"))?;
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(1500));
            app.exit(0);
        });
        Ok(yol.to_string_lossy().to_string())
    })
    .await
    .map_err(hata)?
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
    fn surum_karsilastirma() {
        assert!(surum_parcala("v1.2.0") > surum_parcala("1.1.9"));
        assert!(surum_parcala("v1.1.0") == surum_parcala("1.1.0"));
        assert!(surum_parcala("v0.9.0") < surum_parcala("1.0.0"));
        assert_eq!(surum_parcala("bozuk"), (0, 0, 0));
    }
}

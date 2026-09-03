//! UDF Resimcisi — resimden tam çözünürlüklü UDF belgesi üretir.
//!
//! Neden gerekli: UYAP Doküman Editörü resmi belgeye alırken bitmap'i yeniden örnekliyor
//! (ölçüldü: 3000×2000 px görsel "Ekle → Resim" yolundan 524×349 px olarak çıkıyor).
//! Bu uygulama `.udf` dosyasını kendisi yazdığı için o kod hiç çalışmaz.

pub mod commands;
pub mod image_io;
pub mod settings;
pub mod udf;

#[cfg(windows)]
pub mod kayit;
#[cfg(windows)]
pub mod ude;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(commands::Oturum::default())
        .invoke_handler(tauri::generate_handler![
            commands::ayarlari_getir,
            commands::ayarlari_kaydet,
            commands::resimleri_yukle,
            commands::panodan_al,
            commands::panoda_resim_var_mi,
            commands::resmi_cikar,
            commands::listeyi_temizle,
            commands::liste_getir,
            commands::belge_boyutu,
            commands::udfde_ac,
            commands::klasorde_goster,
            commands::ude_kurulu_mu,
            commands::surum,
            commands::guncelleme_denetle,
            commands::guncellemeyi_kur,
        ])
        .run(tauri::generate_context!())
        .expect("UDF Resimcisi başlatılamadı");
}

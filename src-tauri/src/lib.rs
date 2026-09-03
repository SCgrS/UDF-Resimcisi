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
pub mod ude;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .manage(commands::Oturum::default())
        .invoke_handler(tauri::generate_handler![
            commands::ayarlari_getir,
            commands::ayarlari_kaydet,
            commands::resimleri_yukle,
            commands::panodan_al,
            commands::resmi_cikar,
            commands::listeyi_temizle,
            commands::liste_getir,
            commands::olculeri_hesapla,
            commands::udf_uret,
            commands::kip_a_calistir,
            commands::kip_b_calistir,
            commands::klasorde_goster,
        ])
        .run(tauri::generate_context!())
        .expect("UDF Resimcisi başlatılamadı");
}

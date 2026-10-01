// Sürümde konsol penceresi açılmasın.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use udf_resimcisi_lib::uretilenler;

fn main() {
    // Kaldırıcı "Uygulama verilerini sil" işaretlenince uygulamayı bununla çağırır
    // (nsis/hooks.nsh): pencere açılmadan yalnızca uygulamanın ürettiği, değişmemiş belgeler silinir.
    if std::env::args_os().any(|a| a.to_str() == Some(uretilenler::KALDIRMA_ANAHTARI)) {
        uretilenler::temizle();
        return;
    }
    udf_resimcisi_lib::run()
}

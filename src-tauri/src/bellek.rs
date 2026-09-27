//! Pencere arka plandayken WebView2'nin belleğini kısar.
//!
//! Ölçüm (1.7.0, boşta): uygulamanın toplam belleğinin %95'i WebView2 süreçlerinde. WebView2'ye
//! `MemoryUsageTargetLevel = Low` denince çalışma kümesini kırpar, önbelleklerini boşaltır;
//! betikler çalışmayı sürdürür, arka plandaki pencereye sürükle-bırak da çalışır. Pencere simge
//! durumuna küçültülünce hemen, odağı kaybedip `ARKA_PLAN_GECIKMESI` boyunca geri almazsa düşük
//! düzeye geçilir; odak geri gelince normale dönülür (WebView2 kendiliğinden dönmez).

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tauri::{WebviewWindow, WindowEvent};
// Sürümler Tauri'nin (wry) kullandıklarıyla aynı olmalı; `Cargo.toml`'daki nota bakın.
use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2_19, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW,
    COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
};
use windows_core::Interface;

/// UDE'ye geçip hemen geri dönen kullanıcıyı yavaşlatmamak için beklenen süre.
const ARKA_PLAN_GECIKMESI: Duration = Duration::from_secs(30);

/// Her odak değişiminde artar; bekleyen "düşük düzeye geç" işi, beklerken odak değiştiyse
/// hiçbir şey yapmaz.
static KUSAK: AtomicU64 = AtomicU64::new(0);

pub fn izle(pencere: &WebviewWindow) {
    let p = pencere.clone();
    pencere.on_window_event(move |olay| match olay {
        WindowEvent::Focused(true) => {
            KUSAK.fetch_add(1, Ordering::SeqCst);
            duzey(&p, false);
        }
        WindowEvent::Focused(false) => {
            let kusak = KUSAK.fetch_add(1, Ordering::SeqCst) + 1;
            let p = p.clone();
            std::thread::spawn(move || {
                std::thread::sleep(ARKA_PLAN_GECIKMESI);
                if KUSAK.load(Ordering::SeqCst) == kusak {
                    duzey(&p, true);
                }
            });
        }
        WindowEvent::Resized(_) if p.is_minimized().unwrap_or(false) => {
            KUSAK.fetch_add(1, Ordering::SeqCst);
            duzey(&p, true);
        }
        _ => {}
    });
}

fn duzey(pencere: &WebviewWindow, dusuk: bool) {
    let hedef = if dusuk {
        COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW
    } else {
        COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL
    };
    let _ = pencere.with_webview(move |w| unsafe {
        // Arayüz ICoreWebView2_19 (WebView2 1.0.2210) öncesi çalışma zamanlarında yok; o zaman
        // hiçbir şey yapılmaz.
        if let Ok(c) = w.controller().CoreWebView2().and_then(|c| c.cast::<ICoreWebView2_19>()) {
            let _ = c.SetMemoryUsageTargetLevel(hedef);
        }
    });
}

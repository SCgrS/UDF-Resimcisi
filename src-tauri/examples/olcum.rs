//! Ölçüm aracı: bir resmin her kalite basamağında belgeye kaç piksel ve kaç baytla
//! gireceğini, üretilecek `.udf` dosyasının boyutunu ve UDE'nin resmi PNG olarak yeniden
//! kodlaması hâlinde (dilekçeye yapıştırınca) ne kadar yer tutacağını ölçer.
//!
//! Uygulamanın parçası değildir; `DOGRULAMA.md` ve README'deki sayılar buradan çıkar.
//!
//! ```bash
//! cd src-tauri
//! cargo run --release --example olcum -- ../testdata/telefon-12mp.jpg [--yaz <klasör>]
//! ```
//!
//! `--yaz` verilirse her basamak için `.udf` dosyası ve belgeye giren bitmap (`.png`/`.jpg`)
//! o klasöre yazılır; UDE'de açıp gözle karşılaştırmak için.

use std::path::{Path, PathBuf};

use udf_resimcisi_lib::image_io::{self, Kalite};
use udf_resimcisi_lib::udf::{self, BuildOptions, ImageSpec, PT_PER_CM};

fn kb(b: usize) -> String {
    if b < 1024 * 1024 {
        format!("{} KB", (b as f64 / 1024.0).round() as usize)
    } else {
        format!("{:.2} MB", b as f64 / 1024.0 / 1024.0)
    }
}

/// UDE'ye yapıştırıldığında kaplayacağı yer (uygulamanın kendi kestirimi).
fn png_boyutu(spec: &ImageSpec) -> usize {
    image_io::yapistirma_boyutu(spec).unwrap_or(0)
}

fn bicim(bytes: &[u8]) -> &'static str {
    match image::guess_format(bytes) {
        Ok(image::ImageFormat::Png) => "PNG",
        Ok(image::ImageFormat::Jpeg) => "JPEG",
        _ => "?",
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("kullanım: olcum <resim> [--yaz <klasör>]");
        std::process::exit(2);
    }
    let yol = PathBuf::from(&args[0]);
    let yaz: Option<PathBuf> = args
        .iter()
        .position(|a| a == "--yaz")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from);

    let r = match image_io::dosyadan(&yol) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("okunamadı: {e:#}");
            std::process::exit(1);
        }
    };
    let sayfa = udf::model::PageFormat::default();
    let (gw, gh) = udf::goruntuleme_boyutu(r.spec.px_w, r.spec.px_h, udf::Sizing::FitPage, &sayfa);
    let govde = yol
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "resim".into());

    println!(
        "Girdi: {} — {} × {} px, {} ({}), sayfada {:.1} × {:.1} cm",
        yol.display(),
        r.spec.px_w,
        r.spec.px_h,
        kb(r.spec.bytes.len()),
        r.bicim,
        gw / PT_PER_CM,
        gh / PT_PER_CM
    );
    println!();
    println!(
        "| {:<14} | {:<12} | {:<6} | {:>10} | {:>10} | {:>16} |",
        "Basamak", "Bitmap (px)", "Biçim", "Bitmap", ".udf", "Yapıştırınca ≈"
    );
    println!("|{}|{}|{}|{}|{}|{}|", "-".repeat(16), "-".repeat(14), "-".repeat(8), "-".repeat(12), "-".repeat(12), "-".repeat(18));

    for (ad, slug, kalite) in [
        ("Orijinal", "orijinal", Kalite::Orijinal),
        ("İdeal", "ideal", Kalite::Ideal),
        ("Orta", "orta", Kalite::Orta),
        ("Küçük", "kucuk", Kalite::Kucuk),
    ] {
        let spec = image_io::kaliteye_indir(&r.spec, kalite, &sayfa).expect("indirgeme");
        let udf_bytes = udf::build_udf(std::slice::from_ref(&spec), &BuildOptions::default())
            .expect("udf");
        let png = png_boyutu(&spec);
        println!(
            "| {:<14} | {:<12} | {:<6} | {:>10} | {:>10} | {:>16} |",
            ad,
            format!("{} × {}", spec.px_w, spec.px_h),
            bicim(&spec.bytes),
            kb(spec.bytes.len()),
            kb(udf_bytes.len()),
            kb(png)
        );
        if let Some(k) = &yaz {
            std::fs::create_dir_all(k).expect("klasör");
            let uz = if bicim(&spec.bytes) == "JPEG" { "jpg" } else { "png" };
            std::fs::write(k.join(format!("{govde}-{slug}.udf")), &udf_bytes).expect("yaz");
            std::fs::write(k.join(format!("{govde}-{slug}.{uz}")), &spec.bytes).expect("yaz");
        }
    }
    if let Some(k) = &yaz {
        println!();
        println!("Dosyalar yazıldı: {}", Path::new(k).display());
    }
}

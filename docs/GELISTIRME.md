# Geliştirme

## Kaynak koddan derleme

Rust (MSVC), Visual Studio Build Tools (C++) ve Node.js gerekir. WebView2, Windows 10/11'de zaten kuruludur.

```bash
npm install
npm run tauri build
```

Kurulum paketleri `src-tauri/target/release/bundle/` altına (MSI ve NSIS), taşınabilir sürüm
`src-tauri/target/release/udf-resimcisi.exe` olarak çıkar.

## Ölçüm ve yardımcı araçlar

README'deki sayıları üreten ölçüm aracı [`src-tauri/examples/olcum.rs`](../src-tauri/examples/olcum.rs). Öteki yardımcılar
[tools/README.md](../tools/README.md) dosyasında anlatılıyor. Ölçümler ve yöntemleri [DOGRULAMA.md](../DOGRULAMA.md)
dosyasında.

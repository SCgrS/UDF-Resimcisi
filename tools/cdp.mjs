// WebView2 hata ayıklama kapısından (--remote-debugging-port) uygulama sayfasında bir ifade
// çalıştırır ve sonucunu JSON olarak yazar. bellek-olcumu.ps1 kullanır.
//
//   node tools/cdp.mjs <port> <ifade-dosyası>
import { readFileSync } from "node:fs";

const [port, dosya] = process.argv.slice(2);
const ifade = readFileSync(dosya, "utf8");

const uygulamaSayfasi = (h) => h.type === "page" && /^https?:\/\/tauri\.localhost/.test(h.url);
let hedefler = [];
for (let i = 0; i < 150; i++) {
  try {
    hedefler = await (await fetch(`http://127.0.0.1:${port}/json`)).json();
    if (hedefler.some(uygulamaSayfasi)) break;
  } catch {
    // Kapı henüz açılmadı.
  }
  await new Promise((r) => setTimeout(r, 200));
}
const sayfa = hedefler.find(uygulamaSayfasi);
if (!sayfa) {
  console.log(JSON.stringify({ hata: "uygulama sayfası yok", hedefler: hedefler.map((h) => [h.type, h.url]) }));
  process.exit(1);
}

const ws = new WebSocket(sayfa.webSocketDebuggerUrl);
await new Promise((r, j) => {
  ws.onopen = r;
  ws.onerror = j;
});
ws.onmessage = (m) => {
  const v = JSON.parse(m.data);
  if (v.id !== 1) return;
  if (v.result?.exceptionDetails) {
    console.log(JSON.stringify({ hata: v.result.exceptionDetails }));
    process.exitCode = 1;
  } else {
    console.log(JSON.stringify(v.result?.result?.value ?? v));
  }
  ws.close();
};
ws.send(
  JSON.stringify({
    id: 1,
    method: "Runtime.evaluate",
    params: { expression: ifade, awaitPromise: true, returnByValue: true },
  })
);

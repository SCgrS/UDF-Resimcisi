// UDF Resimcisi — arayüz mantığı.
// Ağır işlerin hepsi Rust tarafında; burada yalnız durum yönetimi ve geri bildirim var.

const { invoke } = window.__TAURI__.core;
const dialog = window.__TAURI__.dialog;
const opener = window.__TAURI__.opener;
const bildirim = window.__TAURI__.notification;
const webview = window.__TAURI__.webview;

const $ = (id) => document.getElementById(id);

const el = {
  birakma: $("birakma-alani"),
  dosyaSec: $("dosya-sec"),
  panodanAl: $("panodan-al"),
  listeBolumu: $("liste-bolumu"),
  liste: $("liste"),
  sayac: $("sayac"),
  temizle: $("listeyi-temizle"),
  ayriSayfa: $("ayri-sayfa"),
  boyutCm: $("boyut-cm"),
  ciktiKlasoru: $("cikti-klasoru"),
  klasorSec: $("klasor-sec"),
  uyari9mb: $("uyari-9mb"),
  toplu: $("toplu"),
  hepsiniKopyala: $("hepsini-kopyala"),
  hepsiniAc: $("hepsini-ac"),
  durum: $("durum"),
  kilavuz: $("kilavuz"),
  kilavuzBaslik: $("kilavuz-baslik"),
  kilavuzMetin: $("kilavuz-metin"),
  kilavuzKlasor: $("kilavuz-klasor"),
  kilavuzKapat: $("kilavuz-kapat"),
  buyukUyari: $("buyuk-uyari"),
  buyukMetin: $("buyuk-metin"),
  kucultUret: $("kucult-uret"),
  buyukKapat: $("buyuk-kapat"),
};

let resimler = [];
let sonUretilenYol = null;
let mesgul = false;
/// Küçültme teklifi kabul edilirse aynı işi tekrarlamak için son isteğin özeti.
let sonIslem = null;

// ---------------------------------------------------------------------------
// Yardımcılar
// ---------------------------------------------------------------------------

function mb(bayt) {
  if (bayt < 1024) return `${bayt} B`;
  if (bayt < 1024 * 1024) return `${(bayt / 1024).toFixed(0)} KB`;
  return `${(bayt / 1024 / 1024).toFixed(1)} MB`;
}

function durum(metin, sinif = "") {
  el.durum.textContent = metin;
  el.durum.className = `durum ${sinif}`;
}

function boyutTercihi() {
  const secili = document.querySelector('input[name="boyut"]:checked').value;
  if (secili === "cm") {
    return { tur: "cm", deger: parseFloat(el.boyutCm.value) || 15 };
  }
  return { tur: secili };
}

function istekKur(kucult) {
  return {
    ayri_sayfa: el.ayriSayfa.checked,
    boyut: boyutTercihi(),
    cikti_klasoru: el.ciktiKlasoru.value,
    kucult,
  };
}

function mesgulYap(evet, metin) {
  mesgul = evet;
  document.querySelectorAll("button").forEach((b) => {
    if (b.dataset.hepDurmasin !== "1") b.disabled = evet;
  });
  if (metin) durum(metin);
}

// ---------------------------------------------------------------------------
// Ayarlar
// ---------------------------------------------------------------------------

async function ayarlariYukle() {
  try {
    const a = await invoke("ayarlari_getir");
    el.ayriSayfa.checked = a.ayri_sayfa;
    el.uyari9mb.checked = a.uyari_9mb;
    el.ciktiKlasoru.value = a.cikti_klasoru;
    const tur = a.boyut.tur;
    document.querySelector(`input[name="boyut"][value="${tur}"]`).checked = true;
    if (tur === "cm") {
      el.boyutCm.value = a.boyut.deger;
      el.boyutCm.disabled = false;
    }
  } catch (e) {
    console.error("Ayarlar okunamadı:", e);
  }
}

let kaydetZamanlayici = null;
function ayarlariKaydet() {
  clearTimeout(kaydetZamanlayici);
  kaydetZamanlayici = setTimeout(async () => {
    try {
      await invoke("ayarlari_kaydet", {
        ayarlar: {
          ayri_sayfa: el.ayriSayfa.checked,
          boyut: boyutTercihi(),
          cikti_klasoru: el.ciktiKlasoru.value,
          uyari_9mb: el.uyari9mb.checked,
        },
      });
    } catch (e) {
      console.error("Ayarlar yazılamadı:", e);
    }
  }, 300);
}

// ---------------------------------------------------------------------------
// Resim listesi
// ---------------------------------------------------------------------------

function dugme(metin, sinif, isle) {
  const b = document.createElement("button");
  b.type = "button";
  b.className = sinif;
  b.textContent = metin;
  b.addEventListener("click", isle);
  return b;
}

async function listeyiCiz() {
  el.liste.innerHTML = "";
  el.sayac.textContent = resimler.length;
  el.listeBolumu.hidden = resimler.length === 0;
  el.toplu.hidden = resimler.length < 2;

  let olculer = [];
  try {
    olculer = await invoke("olculeri_hesapla", { boyut: boyutTercihi() });
  } catch (e) {
    console.error(e);
  }

  resimler.forEach((r, i) => {
    const o = olculer[i];
    const li = document.createElement("li");

    const img = document.createElement("img");
    img.src = r.onizleme;
    img.alt = "";
    li.appendChild(img);

    const bilgi = document.createElement("div");
    bilgi.className = "bilgi";

    const ad = document.createElement("div");
    ad.className = "ad";
    ad.textContent = r.ad;
    bilgi.appendChild(ad);

    const olcu = document.createElement("div");
    olcu.className = "olcu";
    const cm = o ? ` → ${o.genislik_cm.toFixed(1)} × ${o.yukseklik_cm.toFixed(1)} cm` : "";
    olcu.textContent = `${r.px_w} × ${r.px_h} px${cm} · ${r.bicim} · ${mb(r.bayt)}`;
    bilgi.appendChild(olcu);

    if (r.orijinal_korundu) {
      const isaret = document.createElement("div");
      isaret.className = "isaret";
      isaret.textContent = "Orijinal baytlar korunuyor — yeniden sıkıştırma yok";
      bilgi.appendChild(isaret);
    } else if (r.dondurul) {
      const isaret = document.createElement("div");
      isaret.className = "isaret";
      isaret.textContent = "EXIF yönü düzeltildi (kayıpsız PNG'ye çevrildi)";
      bilgi.appendChild(isaret);
    }

    if (o && o.sayfayi_asiyor) {
      const tas = document.createElement("div");
      tas.className = "tas";
      tas.textContent = "Bu boyut A4 sayfasına sığmıyor — taşabilir.";
      bilgi.appendChild(tas);
    }

    li.appendChild(bilgi);

    const eylemler = document.createElement("div");
    eylemler.className = "kart-eylem";
    eylemler.appendChild(dugme("Panoya kopyala", "birincil kucuk", () => kopyala([i])));
    eylemler.appendChild(dugme("UDF'de aç", "ikincil kucuk", () => ac([i])));
    eylemler.appendChild(
      dugme("Kaldır", "baglanti", async () => {
        resimler = await invoke("resmi_cikar", { indeks: i });
        listeyiCiz();
      })
    );
    li.appendChild(eylemler);

    el.liste.appendChild(li);
  });
}

async function yollariEkle(yollar) {
  if (!yollar || yollar.length === 0) return;
  mesgulYap(true, `${yollar.length} resim okunuyor…`);
  try {
    const sonuc = await invoke("resimleri_yukle", { yollar });
    resimler = sonuc.resimler;
    if (sonuc.hatalar.length > 0) {
      // Okunamayan dosya sessizce yutulmaz: adıyla söylenir.
      durum(`${resimler.length} resim hazır. Okunamayan: ${sonuc.hatalar.join(" · ")}`, "kotu");
    } else {
      durum(resimler.length === 1 ? "Resim hazır." : `${resimler.length} resim hazır.`);
    }
  } catch (e) {
    durum(String(e), "kotu");
  } finally {
    mesgulYap(false);
    listeyiCiz();
  }
}

// ---------------------------------------------------------------------------
// İki eylem: panoya kopyala / UDF'de aç
// ---------------------------------------------------------------------------

/// Panoya kopyala: UDE arka planda, hiç görünmeden açılıp kapanır.
async function kopyala(indeksler, kucult = false) {
  el.buyukUyari.hidden = true;
  el.kilavuz.hidden = true;
  sonIslem = { tur: "kopyala", indeksler };
  mesgulYap(true, "Panoya alınıyor… (UYAP editörü arka planda çalışıyor)");

  try {
    const k = await invoke("panoya_kopyala", {
      indeksler,
      istek: istekKur(kucult),
    });
    if (k.durum === "panoda") {
      durum(`${k.mesaj} (${mb(k.boyut_bayt)})`, "iyi");
      toast("Resim panoda", "Dilekçenizde Ctrl+V yapın.");
      if (k.buyuk && el.uyari9mb.checked && !kucult) buyukUyar(k.boyut_bayt);
    } else {
      durum("Panoya kopyalanamadı.", "kotu");
      kilavuzGoster("Kopyalama tamamlanamadı", k.mesaj);
    }
  } catch (e) {
    durum(String(e), "kotu");
  } finally {
    mesgulYap(false);
  }
}

/// UDF'de aç: belge kaydetme klasörüne yazılır ve editörde açılır.
async function ac(indeksler, kucult = false) {
  el.buyukUyari.hidden = true;
  el.kilavuz.hidden = true;
  sonIslem = { tur: "ac", indeksler };
  mesgulYap(true, "Belge hazırlanıyor ve açılıyor…");

  try {
    const s = await invoke("udfde_ac", { indeksler, istek: istekKur(kucult) });
    sonUretilenYol = s.yol;
    durum(`Belge hazır (${mb(s.boyut_bayt)}) ve açıldı.`, "iyi");
    kilavuzGoster("Belge kaydedildi", s.yol);
    if (s.buyuk && el.uyari9mb.checked && !kucult) buyukUyar(s.boyut_bayt);
  } catch (e) {
    durum(String(e), "kotu");
    kilavuzGoster("Açılamadı", String(e));
  } finally {
    mesgulYap(false);
  }
}

function buyukUyar(bayt) {
  el.buyukMetin.textContent =
    `Bu belge ${mb(bayt)}. UYAP 10 MB üstü UDF kabul etmiyor. ` +
    `300 DPI'lık (≈2200 px uzun kenar) küçük sürümle tekrar denemek ister misiniz?`;
  el.buyukUyari.hidden = false;
}

function kilavuzGoster(baslik, metin) {
  el.kilavuzBaslik.textContent = baslik;
  el.kilavuzMetin.textContent = metin;
  el.kilavuz.hidden = false;
}

async function toast(baslik, govde) {
  try {
    let izin = await bildirim.isPermissionGranted();
    if (!izin) izin = (await bildirim.requestPermission()) === "granted";
    if (izin) bildirim.sendNotification({ title: baslik, body: govde });
  } catch (e) {
    console.error("Bildirim gönderilemedi:", e);
  }
}

// ---------------------------------------------------------------------------
// Olaylar
// ---------------------------------------------------------------------------

el.dosyaSec.addEventListener("click", async () => {
  const secim = await dialog.open({
    multiple: true,
    filters: [{ name: "Resim", extensions: ["png", "jpg", "jpeg", "webp", "bmp", "tif", "tiff", "gif"] }],
  });
  if (!secim) return;
  yollariEkle(Array.isArray(secim) ? secim : [secim]);
});

el.panodanAl.addEventListener("click", async () => {
  mesgulYap(true, "Panodaki resim alınıyor…");
  try {
    resimler = await invoke("panodan_al");
    durum("Panodaki resim eklendi.");
  } catch (e) {
    durum(String(e), "kotu");
  } finally {
    mesgulYap(false);
    listeyiCiz();
  }
});

el.temizle.addEventListener("click", async () => {
  resimler = await invoke("listeyi_temizle");
  durum("");
  listeyiCiz();
});

el.klasorSec.addEventListener("click", async () => {
  const klasor = await dialog.open({ directory: true, defaultPath: el.ciktiKlasoru.value });
  if (klasor) {
    el.ciktiKlasoru.value = klasor;
    ayarlariKaydet();
  }
});

el.hepsiniKopyala.addEventListener("click", () => kopyala([]));
el.hepsiniAc.addEventListener("click", () => ac([]));

el.kucultUret.addEventListener("click", () => {
  if (!sonIslem) return;
  if (sonIslem.tur === "kopyala") kopyala(sonIslem.indeksler, true);
  else ac(sonIslem.indeksler, true);
});
el.buyukKapat.addEventListener("click", () => (el.buyukUyari.hidden = true));
el.kilavuzKapat.addEventListener("click", () => (el.kilavuz.hidden = true));
el.kilavuzKlasor.addEventListener("click", async () => {
  if (sonUretilenYol) {
    try {
      await opener.revealItemInDir(sonUretilenYol);
    } catch {
      await invoke("klasorde_goster", { yol: sonUretilenYol });
    }
  }
});

document.querySelectorAll('input[name="boyut"]').forEach((r) =>
  r.addEventListener("change", () => {
    // cm kutusu yalnız "Genişlik" seçiliyken açık.
    el.boyutCm.disabled =
      document.querySelector('input[name="boyut"]:checked').value !== "cm";
    ayarlariKaydet();
    listeyiCiz();
  })
);
el.boyutCm.addEventListener("input", () => {
  ayarlariKaydet();
  listeyiCiz();
});
[el.ayriSayfa, el.uyari9mb].forEach((c) => c.addEventListener("change", ayarlariKaydet));

// Sürükle-bırak (Tauri v2 penceresi; tarayıcı olayları yerine webview olayı)
webview.getCurrentWebview().onDragDropEvent((olay) => {
  const t = olay.payload.type;
  if (t === "over" || t === "enter") {
    el.birakma.classList.add("aktif");
  } else if (t === "drop") {
    el.birakma.classList.remove("aktif");
    yollariEkle(olay.payload.paths);
  } else {
    el.birakma.classList.remove("aktif");
  }
});

// Tarayıcının kendi sürükle-bırak davranışı (dosyayı açmak) engellensin.
window.addEventListener("dragover", (e) => e.preventDefault());
window.addEventListener("drop", (e) => e.preventDefault());

ayarlariYukle().then(listeyiCiz);

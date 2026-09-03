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
  otomasyon: $("otomasyon"),
  uyari9mb: $("uyari-9mb"),
  uret: $("uret"),
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

function seciliKip() {
  return document.querySelector('input[name="kip"]:checked').value;
}

function mesgulYap(evet, metin) {
  mesgul = evet;
  el.uret.disabled = evet || resimler.length === 0;
  el.dosyaSec.disabled = evet;
  el.panodanAl.disabled = evet;
  if (metin) durum(metin);
}

// ---------------------------------------------------------------------------
// Ayarlar
// ---------------------------------------------------------------------------

async function ayarlariYukle() {
  try {
    const a = await invoke("ayarlari_getir");
    document.querySelector(`input[name="kip"][value="${a.kip}"]`).checked = true;
    el.otomasyon.checked = a.otomasyon;
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
          kip: seciliKip(),
          otomasyon: el.otomasyon.checked,
          ayri_sayfa: el.ayriSayfa.checked,
          boyut: boyutTercihi(),
          cikti_klasoru: el.ciktiKlasoru.value,
          uyari_9mb: el.uyari9mb.checked,
          islem_sonrasi_kucult: false,
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

async function listeyiCiz() {
  el.liste.innerHTML = "";
  el.sayac.textContent = resimler.length;
  el.listeBolumu.hidden = resimler.length === 0;
  el.uret.disabled = mesgul || resimler.length === 0;

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

    const sil = document.createElement("button");
    sil.type = "button";
    sil.className = "baglanti";
    sil.textContent = "Kaldır";
    sil.addEventListener("click", async () => {
      resimler = await invoke("resmi_cikar", { indeks: i });
      listeyiCiz();
    });
    li.appendChild(sil);

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
      durum(`${resimler.length} resim hazır.`);
    }
  } catch (e) {
    durum(String(e), "kotu");
  } finally {
    mesgulYap(false);
    listeyiCiz();
  }
}

// ---------------------------------------------------------------------------
// Üretim
// ---------------------------------------------------------------------------

async function uret(kucult = false) {
  if (resimler.length === 0) return;
  el.buyukUyari.hidden = true;
  el.kilavuz.hidden = true;
  mesgulYap(true, kucult ? "Küçültülmüş belge hazırlanıyor…" : "UDF hazırlanıyor…");

  let sonuc;
  try {
    sonuc = await invoke("udf_uret", {
      istek: {
        ayri_sayfa: el.ayriSayfa.checked,
        boyut: boyutTercihi(),
        cikti_klasoru: el.ciktiKlasoru.value,
        kucult,
      },
    });
  } catch (e) {
    mesgulYap(false);
    durum(String(e), "kotu");
    return;
  }

  sonUretilenYol = sonuc.yol;

  if (sonuc.buyuk && el.uyari9mb.checked && !kucult) {
    el.buyukMetin.textContent =
      `Bu belge ${mb(sonuc.boyut_bayt)}. UYAP 10 MB üstü UDF kabul etmiyor. ` +
      `Küçültülmüş sürüm oluşturmak ister misiniz? (300 DPI, ≈2200 px uzun kenar)`;
    el.buyukUyari.hidden = false;
  }

  const kip = seciliKip();
  try {
    if (kip === "A") {
      durum("Belge UDE'de açılıyor, birkaç saniye…");
      const k = await invoke("kip_a_calistir", {
        yol: sonuc.yol,
        otomasyon: el.otomasyon.checked,
      });
      if (k.durum === "panoda") {
        durum(k.mesaj, "iyi");
        toast("Resim panoda", k.mesaj);
      } else {
        durum(`Belge hazır: ${mb(sonuc.boyut_bayt)}`);
        kilavuzGoster("Şimdi ne yapmalısınız", k.mesaj);
      }
    } else {
      const k = await invoke("kip_b_calistir", { yol: sonuc.yol });
      durum(`Belge hazır (${mb(sonuc.boyut_bayt)}) ve açıldı.`, "iyi");
      kilavuzGoster("Belge kaydedildi", k.mesaj);
      toast("UDF oluşturuldu", sonuc.yol);
    }
  } catch (e) {
    durum(String(e), "kotu");
    kilavuzGoster("Belge oluşturuldu ama açılamadı", `${e}\nDosya: ${sonuc.yol}`);
  } finally {
    mesgulYap(false);
  }
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

el.uret.addEventListener("click", () => uret(false));
el.kucultUret.addEventListener("click", () => uret(true));
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
[el.ayriSayfa, el.otomasyon, el.uyari9mb].forEach((c) =>
  c.addEventListener("change", ayarlariKaydet)
);
document.querySelectorAll('input[name="kip"]').forEach((r) =>
  r.addEventListener("change", ayarlariKaydet)
);

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

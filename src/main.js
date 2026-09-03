// UDF Resimcisi — arayüz mantığı.
// Ağır işlerin hepsi Rust tarafında; burada yalnız durum yönetimi ve geri bildirim var.

const { invoke } = window.__TAURI__.core;
const dialog = window.__TAURI__.dialog;
const opener = window.__TAURI__.opener;
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
  ayriSayfaSatiri: $("ayri-sayfa-satiri"),
  ayriSayfa: $("ayri-sayfa"),
  uret: $("uret"),
  boyut: $("boyut"),
  durum: $("durum"),
  // ayarlar
  ayarlarAc: $("ayarlar-ac"),
  ayarlarPerde: $("ayarlar-perde"),
  ayarlarKapat: $("ayarlar-kapat"),
  ciktiKlasoru: $("cikti-klasoru"),
  klasorSec: $("klasor-sec"),
  guncellemeDenetle: $("guncelleme-denetle"),
  guncellemeKur: $("guncelleme-kur"),
  guncellemeDurum: $("guncelleme-durum"),
  surum: $("surum"),
  gelistirici: $("gelistirici"),
};

let resimler = [];
let sonUretilenYol = null;
let indirmeAdresi = "";

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

function mesgulYap(evet, metin) {
  document.querySelectorAll("main button").forEach((b) => {
    b.disabled = evet;
  });
  if (!evet) el.uret.disabled = resimler.length === 0;
  if (metin) durum(metin);
}

// ---------------------------------------------------------------------------
// Ayarlar
// ---------------------------------------------------------------------------

function temaUygula(tema) {
  if (tema === "sistem") document.documentElement.removeAttribute("data-tema");
  else document.documentElement.setAttribute("data-tema", tema);
}

async function ayarlariYukle() {
  try {
    const a = await invoke("ayarlari_getir");
    el.ayriSayfa.checked = a.ayri_sayfa;
    el.ciktiKlasoru.value = a.cikti_klasoru;
    const t = document.querySelector(`input[name="tema"][value="${a.tema}"]`);
    if (t) t.checked = true;
    temaUygula(a.tema);
  } catch (e) {
    console.error("Ayarlar okunamadı:", e);
  }
}

function seciliTema() {
  return document.querySelector('input[name="tema"]:checked').value;
}

let kaydetZamanlayici = null;
function ayarlariKaydet() {
  clearTimeout(kaydetZamanlayici);
  kaydetZamanlayici = setTimeout(async () => {
    try {
      await invoke("ayarlari_kaydet", {
        ayarlar: {
          ayri_sayfa: el.ayriSayfa.checked,
          cikti_klasoru: el.ciktiKlasoru.value,
          tema: seciliTema(),
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

let boyutZamanlayici = null;
function boyutuTazele() {
  clearTimeout(boyutZamanlayici);
  if (resimler.length === 0) {
    el.boyut.textContent = "";
    return;
  }
  el.boyut.textContent = "Tahmini dosya boyutu: hesaplanıyor…";
  boyutZamanlayici = setTimeout(async () => {
    try {
      const b = await invoke("belge_boyutu", { ayriSayfa: el.ayriSayfa.checked });
      el.boyut.textContent = `Tahmini dosya boyutu: ${mb(b)}`;
    } catch (e) {
      el.boyut.textContent = "";
      console.error(e);
    }
  }, 150);
}

function dugmeMetni() {
  el.uret.textContent = resimler.length > 1 ? "Hepsini UDF'de aç" : "UDF'de aç";
}

function listeyiCiz() {
  el.liste.innerHTML = "";
  el.sayac.textContent = resimler.length;
  el.listeBolumu.hidden = resimler.length === 0;
  el.ayriSayfaSatiri.hidden = resimler.length < 2;
  el.uret.disabled = resimler.length === 0;
  dugmeMetni();

  resimler.forEach((r, i) => {
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
    olcu.textContent =
      `${r.px_w} × ${r.px_h} px → ${r.genislik_cm.toFixed(1)} × ${r.yukseklik_cm.toFixed(1)} cm` +
      ` · ${r.bicim} · ${mb(r.bayt)}`;
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

    li.appendChild(bilgi);

    const sil = document.createElement("button");
    sil.type = "button";
    sil.className = "baglanti";
    sil.textContent = "Kaldır";
    sil.addEventListener("click", async () => {
      resimler = await invoke("resmi_cikar", { indeks: i });
      listeyiCiz();
      boyutuTazele();
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
      durum(`${resimler.length} resim hazır. Okunamayan: ${sonuc.hatalar.join(" · ")}`, "kotu");
    } else {
      durum(resimler.length === 1 ? "Resim hazır." : `${resimler.length} resim hazır.`);
    }
  } catch (e) {
    durum(String(e), "kotu");
  } finally {
    mesgulYap(false);
    listeyiCiz();
    boyutuTazele();
  }
}

// ---------------------------------------------------------------------------
// Tek eylem: UDF'de aç
// ---------------------------------------------------------------------------

async function ac() {
  if (resimler.length === 0) return;
  mesgulYap(true, "Belge hazırlanıyor…");
  try {
    const s = await invoke("udfde_ac", {
      ayriSayfa: el.ayriSayfa.checked,
      ciktiKlasoru: el.ciktiKlasoru.value,
    });
    sonUretilenYol = s.yol;
    durumKlasorlu(
      s.ude_acildi
        ? `Belge hazır (${mb(s.boyut_bayt)}) ve açıldı.`
        : `Belge kaydedildi (${mb(s.boyut_bayt)}). UYAP Doküman Editörü bulunamadığı için açılamadı.`,
      s.ude_acildi ? "iyi" : "kotu"
    );
  } catch (e) {
    durum(String(e), "kotu");
  } finally {
    mesgulYap(false);
  }
}

/// Durum satırına "Klasörü aç" bağlantısını ekler (kaydedilen yer ayrı bir kutuda gösterilmez).
function durumKlasorlu(metin, sinif) {
  durum(metin, sinif);
  if (!sonUretilenYol) return;
  el.durum.append(" · ");
  const a = document.createElement("a");
  a.href = "#";
  a.textContent = "Klasörü aç";
  a.addEventListener("click", async (ev) => {
    ev.preventDefault();
    try {
      await opener.revealItemInDir(sonUretilenYol);
    } catch {
      await invoke("klasorde_goster", { yol: sonUretilenYol });
    }
  });
  el.durum.appendChild(a);
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
    boyutuTazele();
  }
});

el.temizle.addEventListener("click", async () => {
  resimler = await invoke("listeyi_temizle");
  sonUretilenYol = null;
  durum("");
  listeyiCiz();
  boyutuTazele();
});

el.uret.addEventListener("click", ac);

el.ayriSayfa.addEventListener("change", () => {
  ayarlariKaydet();
  boyutuTazele();
});

// --- ayarlar penceresi ---
el.ayarlarAc.addEventListener("click", () => (el.ayarlarPerde.hidden = false));
el.ayarlarKapat.addEventListener("click", () => (el.ayarlarPerde.hidden = true));
el.ayarlarPerde.addEventListener("click", (e) => {
  if (e.target === el.ayarlarPerde) el.ayarlarPerde.hidden = true;
});
document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") el.ayarlarPerde.hidden = true;
});

document.querySelectorAll('input[name="tema"]').forEach((r) =>
  r.addEventListener("change", () => {
    temaUygula(seciliTema());
    ayarlariKaydet();
  })
);

el.klasorSec.addEventListener("click", async () => {
  const klasor = await dialog.open({ directory: true, defaultPath: el.ciktiKlasoru.value });
  if (klasor) {
    el.ciktiKlasoru.value = klasor;
    ayarlariKaydet();
  }
});

el.gelistirici.addEventListener("click", async (e) => {
  e.preventDefault();
  try {
    await opener.openUrl("https://x.com/CgrShn");
  } catch (err) {
    console.error(err);
  }
});

el.guncellemeDenetle.addEventListener("click", async () => {
  el.guncellemeDenetle.disabled = true;
  el.guncellemeKur.hidden = true;
  el.guncellemeDurum.textContent = "Denetleniyor…";
  try {
    const g = await invoke("guncelleme_denetle");
    el.surum.textContent = g.bu_surum;
    el.guncellemeDurum.textContent = g.mesaj;
    if (g.durum === "yeni-surum-var" && g.indirme_adresi) {
      indirmeAdresi = g.indirme_adresi;
      el.guncellemeKur.hidden = false;
    }
  } catch (e) {
    el.guncellemeDurum.textContent = String(e);
  } finally {
    el.guncellemeDenetle.disabled = false;
  }
});

el.guncellemeKur.addEventListener("click", async () => {
  el.guncellemeKur.disabled = true;
  el.guncellemeDurum.textContent = "İndiriliyor…";
  try {
    await invoke("guncellemeyi_kur", { indirmeAdresi });
    el.guncellemeDurum.textContent = "Kurulum başlatıldı.";
  } catch (e) {
    el.guncellemeDurum.textContent = String(e);
  } finally {
    el.guncellemeKur.disabled = false;
  }
});

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

window.addEventListener("dragover", (e) => e.preventDefault());
window.addEventListener("drop", (e) => e.preventDefault());

(async () => {
  await ayarlariYukle();
  listeyiCiz();
  try {
    el.surum.textContent = await invoke("surum");
  } catch (e) {
    console.error(e);
  }
  try {
    const kurulu = await invoke("ude_kurulu_mu");
    if (!kurulu) {
      durum(
        "UYAP Doküman Editörü bulunamadı. Belgeler yine üretilip kaydedilir; açmak için editör gerekir."
      );
    }
  } catch (e) {
    console.error(e);
  }
})();

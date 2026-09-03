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
  listeBolumu: $("liste-bolumu"),
  liste: $("liste"),
  sayac: $("sayac"),
  temizle: $("listeyi-temizle"),
  kalite: $("kalite"),
  uret: $("uret"),
  boyut: $("boyut"),
  durum: $("durum"),
  // sağ tık menüsü
  baglamMenu: $("baglam-menu"),
  baglamYapistir: $("baglam-yapistir"),
  // ayarlar
  ayarlarAc: $("ayarlar-ac"),
  ayarlarPerde: $("ayarlar-perde"),
  ayarlarKapat: $("ayarlar-kapat"),
  ayriSayfa: $("ayri-sayfa"),
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

// "3 resim UDF belge içinde oluşturulmaya hazır."
function hazirMetni(n) {
  return `${n} resim UDF belge içinde oluşturulmaya hazır.`;
}

function mesgulYap(evet, metin) {
  document.querySelectorAll("main button, main select").forEach((b) => {
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
  el.boyut.textContent = "Üretilecek dosya boyutu: hesaplanıyor…";
  boyutZamanlayici = setTimeout(async () => {
    try {
      const b = await invoke("belge_boyutu", {
        ayriSayfa: el.ayriSayfa.checked,
        kalite: el.kalite.value,
      });
      el.boyut.textContent = `Üretilecek dosya boyutu: ${mb(b)}`;
    } catch (e) {
      el.boyut.textContent = "";
      console.error(e);
    }
  }, 150);
}

function listeyiCiz() {
  el.liste.innerHTML = "";
  el.sayac.textContent = resimler.length;
  el.listeBolumu.hidden = resimler.length === 0;
  el.uret.disabled = resimler.length === 0;

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
      durum(`${hazirMetni(resimler.length)} Okunamayan: ${sonuc.hatalar.join(" · ")}`, "kotu");
    } else {
      durum(hazirMetni(resimler.length));
    }
  } catch (e) {
    durum(String(e), "kotu");
  } finally {
    mesgulYap(false);
    listeyiCiz();
    boyutuTazele();
  }
}

// Panodaki resmi listeye ekler. Panoda resim yoksa Rust tarafı dürüstçe hata döndürür.
async function panodanEkle() {
  mesgulYap(true, "Panodaki resim alınıyor…");
  try {
    resimler = await invoke("panodan_al");
    durum(hazirMetni(resimler.length));
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
      kalite: el.kalite.value,
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

// Durum satırına "Klasörü aç" bağlantısını ekler (kaydedilen yer ayrı bir kutuda gösterilmez).
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
// Sağ tık menüsü — tek öğe: Yapıştır (yalnız resim)
// ---------------------------------------------------------------------------

function menuyuKapat() {
  el.baglamMenu.hidden = true;
}

async function menuyuAc(x, y) {
  // Menü ölçüsünü öğrenmek için önce görünür yapılır, sonra pencereye sığdırılır.
  el.baglamMenu.hidden = false;
  el.baglamMenu.style.left = "0px";
  el.baglamMenu.style.top = "0px";
  const k = el.baglamMenu.getBoundingClientRect();
  el.baglamMenu.style.left = `${Math.max(0, Math.min(x, window.innerWidth - k.width - 6))}px`;
  el.baglamMenu.style.top = `${Math.max(0, Math.min(y, window.innerHeight - k.height - 6))}px`;

  // Panoda resim yoksa öğe soluk kalsın; kullanıcı boşuna tıklamasın.
  el.baglamYapistir.disabled = true;
  try {
    el.baglamYapistir.disabled = !(await invoke("panoda_resim_var_mi"));
  } catch {
    el.baglamYapistir.disabled = false;
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

el.temizle.addEventListener("click", async () => {
  resimler = await invoke("listeyi_temizle");
  sonUretilenYol = null;
  durum("");
  listeyiCiz();
  boyutuTazele();
});

el.uret.addEventListener("click", ac);

el.kalite.addEventListener("change", boyutuTazele);

el.ayriSayfa.addEventListener("change", () => {
  ayarlariKaydet();
  boyutuTazele();
});

// --- yapıştırma: Ctrl+V ve sağ tık menüsü ---
document.addEventListener("keydown", (e) => {
  if ((e.ctrlKey || e.metaKey) && (e.key === "v" || e.key === "V")) {
    e.preventDefault();
    if (!el.ayarlarPerde.hidden) return; // ayarlar açıkken yapıştırma yok
    panodanEkle();
  }
});

document.addEventListener("contextmenu", (e) => {
  e.preventDefault();
  if (!el.ayarlarPerde.hidden) return; // ayarlar penceresinde menü çıkmasın
  menuyuAc(e.clientX, e.clientY);
});

el.baglamYapistir.addEventListener("click", () => {
  menuyuKapat();
  panodanEkle();
});

window.addEventListener("mousedown", (e) => {
  if (!el.baglamMenu.contains(e.target)) menuyuKapat();
});
window.addEventListener("blur", menuyuKapat);
window.addEventListener("resize", menuyuKapat);

// --- ayarlar penceresi ---
el.ayarlarAc.addEventListener("click", () => (el.ayarlarPerde.hidden = false));
el.ayarlarKapat.addEventListener("click", () => (el.ayarlarPerde.hidden = true));
el.ayarlarPerde.addEventListener("click", (e) => {
  if (e.target === el.ayarlarPerde) el.ayarlarPerde.hidden = true;
});
document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") {
    menuyuKapat();
    el.ayarlarPerde.hidden = true;
  }
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

// Sürükle-bırak (Tauri v2 penceresi; tarayıcı olayları yerine webview olayı).
// Pencerenin tamamı bırakma alanıdır; vurgu da tüm pencerede yapılır.
webview.getCurrentWebview().onDragDropEvent((olay) => {
  const t = olay.payload.type;
  if (t === "over" || t === "enter") {
    document.body.classList.add("birakma-aktif");
  } else if (t === "drop") {
    document.body.classList.remove("birakma-aktif");
    yollariEkle(olay.payload.paths);
  } else {
    document.body.classList.remove("birakma-aktif");
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

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
  klasoruAc: $("klasoru-ac"),
  boyut: $("boyut"),
  durum: $("durum"),
  // sağ tık menüsü
  baglamMenu: $("baglam-menu"),
  baglamYapistir: $("baglam-yapistir"),
  // güncelleme çubuğu (açılışta yeni sürüm bulunursa)
  cubuk: $("guncelleme-cubugu"),
  cubukMetin: $("guncelleme-cubugu-metin"),
  cubukKur: $("guncelleme-cubugu-kur"),
  cubukKapat: $("guncelleme-cubugu-kapat"),
  // ayarlar
  ayarlarAc: $("ayarlar-ac"),
  ayarlarPerde: $("ayarlar-perde"),
  ayarlarKapat: $("ayarlar-kapat"),
  ayriSayfa: $("ayri-sayfa"),
  ciktiKlasoru: $("cikti-klasoru"),
  klasorSec: $("klasor-sec"),
  otomatikGuncelleme: $("otomatik-guncelleme"),
  guncellemeDenetle: $("guncelleme-denetle"),
  guncellemeDurum: $("guncelleme-durum"),
  surum: $("surum"),
  gelistirici: $("gelistirici"),
};

let resimler = [];
let sonUretilenYol = null;
// Yeni eklenen resmin alacağı basamak: alttaki genel seçimin en son somut değeri.
let varsayilanKalite = "ideal";
// Her resmin satırındaki "Belgede: …" yazısı (liste sırasıyla); boyut hesabı bunları doldurur.
let payYazilari = [];

// Satırdaki kalite seçiminin seçenekleri (alttaki genel seçimle aynı basamaklar).
const KALITELER = [
  ["orijinal", "Orijinal"],
  ["ideal", "İdeal"],
  ["orta", "Orta"],
  ["kucuk", "Küçük"],
];
// UYAP Doküman Editörü bulunamadıysa üretme düğmesi hiç açılmaz: uygulama onsuz çalışmaz.
let udeVar = true;

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

function uretDugmesiniAyarla() {
  el.uret.disabled = resimler.length === 0 || !udeVar;
}

function mesgulYap(evet, metin) {
  document.querySelectorAll("main button, main select").forEach((b) => {
    b.disabled = evet;
  });
  if (!evet) uretDugmesiniAyarla();
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
    el.otomatikGuncelleme.checked = a.otomatik_guncelleme;
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
          otomatik_guncelleme: el.otomatikGuncelleme.checked,
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

// Alttaki genel seçim: bütün resimler aynı basamaktaysa onu, değilse "Özel" gösterir.
function genelSecimiGuncelle() {
  const basamaklar = new Set(resimler.map((r) => r.kalite));
  if (basamaklar.size === 1) varsayilanKalite = [...basamaklar][0];
  el.kalite.value = basamaklar.size > 1 ? "ozel" : varsayilanKalite;
}

let boyutZamanlayici = null;
// Hesap sürerken liste ya da seçim değişirse eski sonuç yenisinin üstüne yazılmasın.
let boyutSirasi = 0;
function boyutuTazele() {
  clearTimeout(boyutZamanlayici);
  const sira = ++boyutSirasi;
  if (resimler.length === 0) {
    el.boyut.textContent = "";
    return;
  }
  el.boyut.textContent = "Üretilecek dosya boyutu: hesaplanıyor…";
  payYazilari.forEach((p) => (p.textContent = "Belgede: …"));
  boyutZamanlayici = setTimeout(async () => {
    try {
      const b = await invoke("belge_boyutu", { ayriSayfa: el.ayriSayfa.checked });
      if (sira !== boyutSirasi) return;
      // İkinci sayı: UDE her yapıştırılan resmi PNG'ye çevirir; dilekçenin büyüyeceği miktar odur.
      el.boyut.textContent =
        `Üretilecek dosya boyutu: ${mb(b.dosya)} · ` +
        `Dilekçeye yapıştırıldığında yaklaşık ${mb(b.yapistirma)}`;
      b.resimler.forEach((bayt, i) => {
        if (payYazilari[i]) payYazilari[i].textContent = `Belgede: ${mb(bayt)}`;
      });
    } catch (e) {
      if (sira !== boyutSirasi) return;
      el.boyut.textContent = "";
      payYazilari.forEach((p) => (p.textContent = ""));
      console.error(e);
    }
  }, 150);
}

// Satırın yeşil işareti: orijinal baytlar korunuyor mu, EXIF yönü düzeltildi mi.
function isaretYaz(isaret, r) {
  if (r.orijinal_korundu && r.kalite === "orijinal") {
    isaret.textContent = "Orijinal baytlar korunuyor — yeniden sıkıştırma yok";
  } else if (r.dondurul) {
    isaret.textContent = "EXIF yönü düzeltildi (kayıpsız PNG'ye çevrildi)";
  } else {
    isaret.textContent = "";
  }
  isaret.hidden = isaret.textContent === "";
}

// Satırdaki kalite seçimi: yalnız o resmi değiştirir; alttaki seçim gerekirse "Özel" olur.
function kaliteSecimi(r, i, isaret) {
  const secim = document.createElement("select");
  secim.className = "kalite satir-kalite";
  secim.setAttribute("aria-label", `${r.ad} kalitesi`);
  for (const [deger, ad] of KALITELER) {
    secim.add(new Option(ad, deger, false, deger === r.kalite));
  }
  secim.addEventListener("change", async () => {
    const kalite = secim.value;
    try {
      await invoke("resim_kalitesi", { indeks: i, kalite });
    } catch (e) {
      secim.value = r.kalite;
      durum(String(e), "kotu");
      return;
    }
    r.kalite = kalite;
    isaretYaz(isaret, r);
    genelSecimiGuncelle();
    boyutuTazele();
  });
  return secim;
}

function listeyiCiz() {
  el.liste.innerHTML = "";
  el.sayac.textContent = resimler.length;
  el.listeBolumu.hidden = resimler.length === 0;
  uretDugmesiniAyarla();
  genelSecimiGuncelle();
  payYazilari = [];

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

    const isaret = document.createElement("div");
    isaret.className = "isaret";
    isaretYaz(isaret, r);
    bilgi.appendChild(isaret);

    li.appendChild(bilgi);

    const ayar = document.createElement("div");
    ayar.className = "ayar";
    ayar.appendChild(kaliteSecimi(r, i, isaret));
    const pay = document.createElement("div");
    pay.className = "pay";
    pay.title = "Bu resmin seçilen kalitede UDF belgesine gireceği boyut";
    ayar.appendChild(pay);
    payYazilari.push(pay);
    li.appendChild(ayar);

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
    const sonuc = await invoke("resimleri_yukle", { yollar, kalite: varsayilanKalite });
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
    resimler = await invoke("panodan_al", { kalite: varsayilanKalite });
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
      ciktiKlasoru: el.ciktiKlasoru.value,
    });
    sonUretilenYol = s.yol;
    durum(`Belge hazır (${mb(s.boyut_bayt)}) ve UYAP Doküman Editörü'nde açıldı.`, "iyi");
  } catch (e) {
    durum(String(e), "kotu");
  } finally {
    mesgulYap(false);
  }
}

// "Klasörü aç": son üretilen belge varsa onu seçili gösterir, yoksa kaydetme klasörünü açar.
async function klasoruAc() {
  if (sonUretilenYol) {
    try {
      await opener.revealItemInDir(sonUretilenYol);
      return;
    } catch {
      // Dosya silinmiş olabilir; klasörün kendisine düş.
      sonUretilenYol = null;
    }
  }
  try {
    await invoke("klasoru_ac", { klasor: el.ciktiKlasoru.value });
  } catch (e) {
    durum(String(e), "kotu");
  }
}

// ---------------------------------------------------------------------------
// Güncelleme
// ---------------------------------------------------------------------------

// Denetler; yeni sürüm varsa indirip kurulumu başlatır (uygulama kapanıp yeniden açılır).
// `bildir(metin)` ilerlemeyi yazdırır; `sessiz` açılıştaki arka plan denetimidir.
async function guncelle(bildir) {
  bildir("Denetleniyor…");
  const g = await invoke("guncelleme_denetle");
  el.surum.textContent = g.bu_surum;
  if (g.durum !== "yeni-surum-var" || !g.indirme_adresi) {
    bildir(g.mesaj);
    return false;
  }
  bildir(`Yeni sürüm ${g.yeni_surum} indiriliyor…`);
  await invoke("guncellemeyi_kur", { indirmeAdresi: g.indirme_adresi });
  bildir(`Sürüm ${g.yeni_surum} kuruluyor; uygulama birazdan yeniden açılacak.`);
  return true;
}

async function acilistaDenetle() {
  try {
    const g = await invoke("guncelleme_denetle");
    if (g.durum !== "yeni-surum-var" || !g.indirme_adresi) return;
    el.cubukMetin.textContent = `Yeni sürüm ${g.yeni_surum} hazır (kullandığınız: ${g.bu_surum}).`;
    el.cubuk.hidden = false;
  } catch (e) {
    console.error(e); // Açılışta sessiz: ağ yoksa kullanıcıyı rahatsız etme.
  }
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
el.klasoruAc.addEventListener("click", klasoruAc);

// Genel seçim: bütün resimler bu basamağa geçer, tek tek yapılan ayarlar silinir.
el.kalite.addEventListener("change", async () => {
  const kalite = el.kalite.value;
  if (kalite === "ozel") return; // "Özel" yalnız gösterilir, seçilemez
  try {
    await invoke("kaliteyi_hepsine_uygula", { kalite });
  } catch (e) {
    genelSecimiGuncelle();
    durum(String(e), "kotu");
    return;
  }
  varsayilanKalite = kalite;
  resimler.forEach((r) => (r.kalite = kalite));
  listeyiCiz();
  boyutuTazele();
});

el.ayriSayfa.addEventListener("change", () => {
  ayarlariKaydet();
  boyutuTazele();
});

el.otomatikGuncelleme.addEventListener("change", ayarlariKaydet);

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

// Ayarlardaki düğme: denetler, yeni sürüm varsa hemen kurar.
el.guncellemeDenetle.addEventListener("click", async () => {
  el.guncellemeDenetle.disabled = true;
  try {
    await guncelle((m) => (el.guncellemeDurum.textContent = m));
  } catch (e) {
    el.guncellemeDurum.textContent = String(e);
  } finally {
    el.guncellemeDenetle.disabled = false;
  }
});

// Açılış çubuğundaki "Güncelle": aynı yol, ilerleme çubuğun kendisine yazılır.
el.cubukKur.addEventListener("click", async () => {
  el.cubukKur.disabled = true;
  el.cubukKapat.hidden = true;
  try {
    await guncelle((m) => (el.cubukMetin.textContent = m));
  } catch (e) {
    el.cubukMetin.textContent = String(e);
    el.cubukKur.disabled = false;
    el.cubukKapat.hidden = false;
  }
});
el.cubukKapat.addEventListener("click", () => (el.cubuk.hidden = true));

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
    udeVar = await invoke("ude_kurulu_mu");
  } catch (e) {
    console.error(e);
  }
  if (!udeVar) {
    uretDugmesiniAyarla();
    durum(
      "UYAP Doküman Editörü bu bilgisayarda bulunamadı. Uygulama onsuz çalışmaz: önce UDE'yi kurun, sonra uygulamayı yeniden açın.",
      "kotu"
    );
  }
  // Yeni sürüm denetimi kullanıcıyı bekletmesin: pencere çizildikten sonra, arka planda.
  if (el.otomatikGuncelleme.checked) setTimeout(acilistaDenetle, 2500);
})();

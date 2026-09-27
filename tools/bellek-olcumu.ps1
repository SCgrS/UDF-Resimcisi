# Bellek olcumu (DOGRULAMA.md "1.7.1 surumu - bellek" tablolarinin kaynagi).
#
# Uygulamayi kullanicinin ekranina dokunmadan acar: gizli pencereyle derlenmis surum baslatilir,
# pencere once ekran disina tasinir, sonra odak verilmeden gosterilir. (Ekran disi x/y ile
# derlemek ise yaramiyor: tao hicbir ekrana dusmeyen konumu yok sayip pencereyi varsayilan yere
# koyuyor.) Resimler WebView2 hata ayiklama kapisindan sayfanin kendi islevleriyle eklenir.
# Olculen deger Gorev Yoneticisi'nin "Bellek" sutunu: uygulama sureci ve altindaki butun WebView2
# sureclerinin ozel calisma kumesi.
#
#   tools\bellek-olcumu.ps1 -Derle                        # gizli pencereli derleme + tam olcum
#   tools\bellek-olcumu.ps1 -Exe <exe> -Etiket eski       # hazir bir gizli pencereli derlemeyi olc
#   tools\bellek-olcumu.ps1 -Exe <exe> -Kisa -Argumanlar "<WebView2 ayarlari>"   # ayar denemesi
#
# Girdiler testdata\ altinda: test3000x2000.png, telefon-12mp.jpg, dilekce-tarama.jpg. Dikey
# fotograf, telefon-12mp.jpg'ye EXIF Orientation = 6 eklenerek uretilir. Ciktilar (CSV, derleme,
# WebView2 veri klasoru) %TEMP%\udf-resimcisi-bellek altina yazilir.
param(
  [string]$Exe = '',
  [string]$Etiket = 'olcum',
  [switch]$Derle,
  # Bos birakilirsa tauri.conf.json'daki additionalBrowserArgs kullanilir.
  [string]$Argumanlar = '',
  [int]$Port = 9333,
  [switch]$Kisa,
  # Verilirse 3 resimli pencere bu PNG'ye PrintWindow ile yakalanir.
  [string]$Yakala = ''
)
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.Encoding]::UTF8
$kok = Split-Path -Parent $PSScriptRoot

# Yerel komutu calistirip ciktisini metin olarak dondurur; basariyi cagiran $LASTEXITCODE ile
# denetler. PowerShell 5.1, cikti yonlendirildiginde yerel komutun hata akisina yazdigi her
# satiri (npx'in "Info ..." satirlari gibi) hata sayip durduruyor.
function Yerel([scriptblock]$komut) {
  $onceki = $ErrorActionPreference
  $ErrorActionPreference = 'Continue'
  try { & $komut 2>&1 | ForEach-Object { "$_" } } finally { $ErrorActionPreference = $onceki }
}
$td = Join-Path $kok 'testdata'
$cikti = Join-Path $env:TEMP 'udf-resimcisi-bellek'
New-Item -ItemType Directory -Force $cikti | Out-Null

$pencereAyari = (Get-Content (Join-Path $kok 'src-tauri\tauri.conf.json') -Raw -Encoding UTF8 | ConvertFrom-Json).app.windows[0]
if (-not $Argumanlar) {
  $Argumanlar = if ($pencereAyari.additionalBrowserArgs) { $pencereAyari.additionalBrowserArgs } else { '--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection' }
}

if ($Derle) {
  # Uygulamanin kendi pencere ayari, gizli ve odaksiz acilacak sekilde.
  $pencereAyari | Add-Member -Force -NotePropertyName visible -NotePropertyValue $false
  $pencereAyari | Add-Member -Force -NotePropertyName focus -NotePropertyValue $false
  $pencereAyari | Add-Member -Force -NotePropertyName skipTaskbar -NotePropertyValue $true
  $ayar = Join-Path $cikti 'gizli-pencere.json'
  [IO.File]::WriteAllText($ayar, (@{ app = @{ windows = @($pencereAyari) } } | ConvertTo-Json -Depth 5))
  $env:CARGO_TARGET_DIR = Join-Path $cikti 'hedef'
  Push-Location $kok
  try {
    $derleme = Yerel { npx tauri build --no-bundle --config $ayar }
    if ($LASTEXITCODE) { $derleme | Select-Object -Last 20 | Write-Host; throw 'derleme basarisiz' }
    $derleme | Select-Object -Last 2 | Write-Host
  }
  finally { Pop-Location; Remove-Item Env:CARGO_TARGET_DIR }
  $Exe = Join-Path $cikti 'hedef\release\udf-resimcisi.exe'
}
if (-not $Exe) { throw '-Exe ya da -Derle gerekli' }

# Dikey telefon fotografi: JPEG'in basina Orientation = 6 tasiyan bir EXIF (APP1) bolumu.
$dikey = Join-Path $cikti 'telefon-dikey.jpg'
if (-not (Test-Path $dikey)) {
  $jpeg = [IO.File]::ReadAllBytes((Join-Path $td 'telefon-12mp.jpg'))
  # TIFF: "II", 42, IFD 8'de; tek giris: 0x0112 (Orientation), SHORT, 1 adet, deger 6.
  $tiff = [byte[]](0x49,0x49, 42,0, 8,0,0,0, 1,0, 0x12,0x01, 3,0, 1,0,0,0, 6,0,0,0, 0,0,0,0)
  $app1 = [byte[]]([Text.Encoding]::ASCII.GetBytes("Exif`0`0") + $tiff)
  $bas = [byte[]](0xFF, 0xE1, 0, ($app1.Length + 2))
  $ms = New-Object IO.MemoryStream
  $ms.Write($jpeg, 0, 2); $ms.Write($bas, 0, $bas.Length); $ms.Write($app1, 0, $app1.Length)
  $ms.Write($jpeg, 2, $jpeg.Length - 2)
  [IO.File]::WriteAllBytes($dikey, $ms.ToArray())
}

Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class UdfPencere {
  delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder sb, int n);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint f);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint f);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  public static IntPtr Bul(uint hedef) {
    IntPtr bulunan = IntPtr.Zero;
    EnumWindows((h, l) => {
      uint pid; GetWindowThreadProcessId(h, out pid);
      if (pid == hedef) {
        var sb = new StringBuilder(256); GetWindowText(h, sb, 256);
        if (sb.ToString() == "UDF Resimcisi") { bulunan = h; return false; }
      }
      return true;
    }, IntPtr.Zero);
    return bulunan;
  }
}
"@ -ErrorAction SilentlyContinue

function Agac([int]$kokPid) {
  $surecler = @(Get-CimInstance Win32_Process -Property ProcessId, ParentProcessId, Name, CommandLine)
  $liste = [System.Collections.Generic.List[int]]::new(); $liste.Add($kokPid)
  for ($i = 0; $i -lt $liste.Count; $i++) {
    foreach ($c in $surecler) {
      if ($c.ParentProcessId -eq $liste[$i] -and -not $liste.Contains([int]$c.ProcessId)) { $liste.Add([int]$c.ProcessId) }
    }
  }
  foreach ($id in $liste) { $surecler | Where-Object ProcessId -eq $id }
}

function Tur($sp) {
  if ($sp.Name -notmatch 'msedgewebview2') { return 'uygulama' }
  if ($sp.CommandLine -match '--utility-sub-type=([\w\.]+)') { return 'yardimci:' + ($Matches[1] -replace '\.mojom\.', '/') }
  if ($sp.CommandLine -match '--type=([\w-]+)') { return $Matches[1] }
  return 'tarayici'
}

$sonuclar = [System.Collections.Generic.List[object]]::new()
function Olc([string]$ad) {
  $agac = @(Agac $script:uygPid)
  $perf = @(Get-CimInstance Win32_PerfRawData_PerfProc_Process -Property IDProcess, WorkingSetPrivate, PrivateBytes)
  $toplam = 0; $uyg = 0
  foreach ($sp in $agac) {
    $pf = $perf | Where-Object IDProcess -eq $sp.ProcessId | Select-Object -First 1
    if (-not $pf) { continue }
    $ck = [math]::Round($pf.WorkingSetPrivate / 1MB, 1)
    $toplam += $ck
    $tur = Tur $sp
    if ($tur -eq 'uygulama') { $uyg = $ck }
    $sonuclar.Add([pscustomobject]@{ Olcum = $ad; Tur = $tur; PID = $sp.ProcessId; OzelCalismaKumesiMB = $ck; OzelBaytMB = [math]::Round($pf.PrivateBytes / 1MB, 1) })
  }
  $p = Get-Process -Id $script:uygPid
  $sonuclar.Add([pscustomobject]@{ Olcum = $ad; Tur = 'TOPLAM'; PID = $agac.Count; OzelCalismaKumesiMB = [math]::Round($toplam, 1); OzelBaytMB = '' })
  $sonuclar.Add([pscustomobject]@{ Olcum = $ad; Tur = 'uygulama-tepe'; PID = 0; OzelCalismaKumesiMB = [math]::Round($p.PeakWorkingSet64 / 1MB, 1); OzelBaytMB = [math]::Round($p.PeakPagedMemorySize64 / 1MB, 1) })
  Write-Host ("{0,-30} toplam {1,6:N1} MB | uygulama {2,5:N1} MB | uygulamanin tepe noktasi (islenen) {3,6:N1} MB" -f $ad, $toplam, $uyg, ($p.PeakPagedMemorySize64 / 1MB))
}

function Js([string]$govde) {
  # Her ifade kendi async kapsaminda calisir; genel kapsamda tanim birakmaz.
  $ifade = @'
(async () => {
  const bekle = (ms) => new Promise((r) => setTimeout(r, ms));
  async function hesapBitsin(t0) {
    for (let i = 0; i < 100 && !el.boyut.textContent.includes('hesaplan'); i++) await bekle(20);
    for (;;) {
      if (!el.boyut.textContent.includes('hesaplan')) return { boyut: el.boyut.textContent, ms: Math.round(performance.now() - t0), n: resimler.length };
      await bekle(25);
    }
  }
'@ + $govde + "`n})()"
  $f = Join-Path $cikti 'ifade.js'
  [IO.File]::WriteAllText($f, $ifade)
  $sonuc = (Yerel { node (Join-Path $PSScriptRoot 'cdp.mjs') $Port $f }) -join ' '
  Write-Host "   $sonuc"
  if ($LASTEXITCODE -or $sonuc -match '"hata"') { throw 'sayfada hata' }
}

function Ekle([string[]]$yollar) {
  $j = ($yollar | ForEach-Object { '"' + ($_ -replace '\\', '\\') + '"' }) -join ','
  Js "const t0 = performance.now(); await yollariEkle([$j]); return await hesapBitsin(t0);"
}
function Kalite([string]$k) {
  Js "const t0 = performance.now(); el.kalite.value = '$k'; el.kalite.dispatchEvent(new Event('change')); return await hesapBitsin(t0);"
}

# --- calistir ---
$veri = Join-Path $cikti "wv2-$Etiket"
if (Test-Path $veri) { Remove-Item -Recurse -Force $veri }
$env:WEBVIEW2_USER_DATA_FOLDER = $veri
# Bu degisken uygulamanin kendi ayarinin yerine gecer; ayni ayarlar + hata ayiklama kapisi.
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$Port $Argumanlar"
# Surum denetimi ag'a cikmasin.
$env:UDF_RESIMCISI_SURUM_ADRESI = 'http://127.0.0.1:9/yok'
$proc = Start-Process -FilePath $Exe -PassThru
$script:uygPid = $proc.Id
Write-Host "PID $($proc.Id), WebView2: $Argumanlar"
try {
  $h = [IntPtr]::Zero
  for ($i = 0; $i -lt 100 -and $h -eq [IntPtr]::Zero; $i++) { Start-Sleep -Milliseconds 50; $h = [UdfPencere]::Bul([uint32]$proc.Id) }
  if ($h -eq [IntPtr]::Zero) { throw 'pencere bulunamadi' }
  if ([UdfPencere]::IsWindowVisible($h)) { throw 'pencere gorunur acildi: derleme gizli pencereli degil (-Derle ile derleyin)' }
  [void][UdfPencere]::SetWindowPos($h, [IntPtr]::Zero, -12000, -12000, 0, 0, 0x15)   # NOSIZE | NOZORDER | NOACTIVATE
  $r = New-Object UdfPencere+RECT; [void][UdfPencere]::GetWindowRect($h, [ref]$r)
  if ($r.L -gt -5000) { throw "pencere ekran disina tasinamadi ($($r.L),$($r.T))" }
  [void][UdfPencere]::ShowWindow($h, 4)   # SW_SHOWNOACTIVATE

  Js "for (let i = 0; i < 200 && typeof yollariEkle !== 'function'; i++) await bekle(50); return { hazir: typeof yollariEkle, gorunurluk: document.visibilityState };"
  Start-Sleep 10; Olc 'S0 bos (acilistan 10 sn)'
  Ekle @("$td\test3000x2000.png", "$td\telefon-12mp.jpg", "$td\dilekce-tarama.jpg")
  Start-Sleep 5; Olc 'S1 3 resim, Ideal'
  if ($Yakala) {
    Add-Type -AssemblyName System.Drawing
    [void][UdfPencere]::GetWindowRect($h, [ref]$r)
    $bmp = New-Object System.Drawing.Bitmap ($r.R - $r.L), ($r.B - $r.T)
    $g = [System.Drawing.Graphics]::FromImage($bmp); $hdc = $g.GetHdc()
    [void][UdfPencere]::PrintWindow($h, $hdc, 2)   # PW_RENDERFULLCONTENT
    $g.ReleaseHdc($hdc); $g.Dispose(); $bmp.Save($Yakala, [System.Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose()
    Write-Host "   yakalandi: $Yakala"
  }
  if (-not $Kisa) {
    Kalite 'orijinal'
    Start-Sleep 5; Olc 'S2 3 resim, Orijinal'
    Kalite 'ideal'
    Ekle @($dikey, $dikey, $dikey, "$td\telefon-12mp.jpg", "$td\telefon-12mp.jpg", "$td\telefon-12mp.jpg")
    Start-Sleep 5; Olc 'S3 9 resim (3 dikey), Ideal'
    Js "const t0 = performance.now(); el.temizle.click(); for (let i = 0; i < 200 && resimler.length; i++) await bekle(25); return { n: resimler.length, ms: Math.round(performance.now() - t0) };"
    Start-Sleep 5; Olc 'S4 liste temizlendi'
    [void][UdfPencere]::ShowWindow($h, 7)   # SW_SHOWMINNOACTIVE
    Start-Sleep 12; Olc 'S5 simge durumunda'
  }
}
finally {
  if (-not $proc.HasExited) { Stop-Process -Id $proc.Id -Force }
  $csv = Join-Path $cikti "olcum-$Etiket.csv"
  $sonuclar | Export-Csv -NoTypeInformation -Encoding UTF8 $csv
  Write-Host "Ayrinti: $csv"
}

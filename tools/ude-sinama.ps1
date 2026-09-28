# UDE ile acma sinamasi, kullanicinin ekranina dokunmadan (DOGRULAMA.md, "1.7.2 surumu").
#
# Her sey ayri, gorunmeyen bir masaustunde (CreateDesktop) olur: orada baslayan sureclerin ve
# onlarin actigi UDE'nin pencereleri kullanicinin ekranina dusmez, odak calmaz. Pencere
# basliklari EnumDesktopWindows ile, UDE'nin komut satiri WMI'dan okunur.
#
#   tools\ude-sinama.ps1 -Kip kayit                  # belge UDE'nin kaydiyla (ShellExecuteExW + SEE_MASK_CLASSKEY)
#   tools\ude-sinama.ps1 -Kip dogrudan               # UDE'nin exe'si jetonlarla dogrudan
#   tools\ude-sinama.ps1 -Kip uygulama -Exe <exe>    # uygulamanin kendisi, WebView2 hata ayiklama kapisindan
#
# UDE aciksa baslamaz: yeni belge kullanicinin acik UDE'sine giderdi. UDE'nin %USERPROFILE%\.uki
# ayar dosyalari yedeklenir, is bitince degistiyse geri konur. Uygulama kipinde uygulamanin ayar
# dosyasi da yedeklenip geri konur; surum denetimi ag'a cikmaz. Calisma klasoru sonunda silinir.
param(
  [ValidateSet('kayit', 'dogrudan', 'uygulama')][string]$Kip = 'uygulama',
  [string]$Exe = '',
  [int]$Port = 9334,
  [int]$Bekle = 40,
  # Ic kullanim: gorunmeyen masaustunde belgeyi kabuk uzerinden acan alt surec.
  [string]$Acici = '',
  [string]$ProgId = '',
  [string]$Gunluk = ''
)
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.Encoding]::UTF8

Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;

public static class GizliMasa {
  [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
  static extern IntPtr CreateDesktopW(string ad, IntPtr aygit, IntPtr mod, uint bayrak, uint erisim, IntPtr sa);
  [DllImport("user32.dll", SetLastError = true)]
  public static extern bool CloseDesktop(IntPtr masa);
  delegate bool PencereIslevi(IntPtr h, IntPtr p);
  [DllImport("user32.dll")] static extern bool EnumDesktopWindows(IntPtr masa, PencereIslevi f, IntPtr p);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetWindowTextW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetClassNameW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] static extern bool PostMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);

  [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
  struct STARTUPINFO {
    public int cb; public string lpReserved; public string lpDesktop; public string lpTitle;
    public int dwX, dwY, dwXSize, dwYSize, dwXCountChars, dwYCountChars, dwFillAttribute, dwFlags;
    public short wShowWindow, cbReserved2; public IntPtr lpReserved2, hStdInput, hStdOutput, hStdError;
  }
  [StructLayout(LayoutKind.Sequential)]
  struct PROCESS_INFORMATION { public IntPtr hProcess, hThread; public int dwProcessId, dwThreadId; }
  [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
  static extern bool CreateProcessW(string app, StringBuilder cmd, IntPtr pa, IntPtr ta, bool inherit,
    uint flags, IntPtr env, string dir, ref STARTUPINFO si, out PROCESS_INFORMATION pi);
  [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);

  public static IntPtr Olustur(string ad) {
    IntPtr h = CreateDesktopW(ad, IntPtr.Zero, IntPtr.Zero, 0, 0x10000000 /* GENERIC_ALL */, IntPtr.Zero);
    if (h == IntPtr.Zero) throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
    return h;
  }

  // Komutu verilen masaustunde baslatir (ortam degiskenleri bu surecten gecer).
  public static int Baslat(string masa, string komutSatiri, uint bayrak) {
    var si = new STARTUPINFO();
    si.cb = Marshal.SizeOf(typeof(STARTUPINFO));
    si.lpDesktop = "WinSta0\\" + masa;
    PROCESS_INFORMATION pi;
    var sb = new StringBuilder(komutSatiri, 32768);
    if (!CreateProcessW(null, sb, IntPtr.Zero, IntPtr.Zero, false, bayrak, IntPtr.Zero, null, ref si, out pi))
      throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());
    CloseHandle(pi.hThread);
    CloseHandle(pi.hProcess);
    return pi.dwProcessId;
  }

  public static List<string> Pencereler(IntPtr masa) {
    var l = new List<string>();
    EnumDesktopWindows(masa, (h, p) => {
      var t = new StringBuilder(1024); GetWindowTextW(h, t, 1024);
      var c = new StringBuilder(256); GetClassNameW(h, c, 256);
      uint pid; GetWindowThreadProcessId(h, out pid);
      l.Add(pid + "|" + (IsWindowVisible(h) ? "gorunur" : "gizli") + "|" + c + "|" + t);
      return true;
    }, IntPtr.Zero);
    return l;
  }

  // Masaustundeki gorunur pencerelere WM_CLOSE gonderir.
  public static int Kapat(IntPtr masa) {
    int n = 0;
    EnumDesktopWindows(masa, (h, p) => {
      if (IsWindowVisible(h)) { PostMessageW(h, 0x0010, IntPtr.Zero, IntPtr.Zero); n++; }
      return true;
    }, IntPtr.Zero);
    return n;
  }
}

public static class Kabuk {
  [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
  struct SEI {
    public int cbSize; public uint fMask; public IntPtr hwnd;
    public string lpVerb; public string lpFile; public string lpParameters; public string lpDirectory;
    public int nShow; public IntPtr hInstApp; public IntPtr lpIDList; public string lpClass;
    public IntPtr hkeyClass; public uint dwHotKey; public IntPtr hIcon; public IntPtr hProcess;
  }
  [DllImport("shell32.dll", CharSet = CharSet.Unicode, SetLastError = true)] static extern bool ShellExecuteExW(ref SEI i);
  [DllImport("advapi32.dll", CharSet = CharSet.Unicode)] static extern int RegOpenKeyExW(UIntPtr k, string alt, uint o, uint sam, out IntPtr sonuc);
  [DllImport("advapi32.dll")] static extern int RegCloseKey(IntPtr k);

  // Uygulamanin 1.7.2'deki yolu: belgeyi UDE'nin HKLM\Software\Classes kaydiyla ac.
  public static string KayitlaAc(string dosya, string progId) {
    IntPtr k;
    int r = RegOpenKeyExW(new UIntPtr(0x80000002u), "Software\\Classes\\" + progId, 0, 0x20019 /* KEY_READ */, out k);
    if (r != 0) return "RegOpenKeyExW hata " + r;
    var i = new SEI();
    i.cbSize = Marshal.SizeOf(typeof(SEI));
    i.fMask = 0x3 | 0x100 | 0x400;  // CLASSKEY | NOASYNC | FLAG_NO_UI
    i.lpVerb = "open"; i.lpFile = dosya; i.nShow = 1; i.hkeyClass = k;
    bool ok = ShellExecuteExW(ref i);
    int e = Marshal.GetLastWin32Error();
    RegCloseKey(k);
    return ok ? "tamam" : "ShellExecuteExW hata " + e;
  }
}
'@

# --- alt surec: gorunmeyen masaustunde belgeyi UDE'nin kaydiyla acar ---
if ($Acici) {
  try { $s = [Kabuk]::KayitlaAc($Acici, $ProgId) } catch { $s = "HATA: $_" }
  [IO.File]::WriteAllText($Gunluk, $s, [Text.Encoding]::UTF8)
  return
}

# Yerel komutu calistirip ciktisini metin olarak dondurur (PowerShell 5.1 hata akisina yazilan
# her satiri hata sayip durdurmasin diye).
function Yerel([scriptblock]$komut) {
  $onceki = $ErrorActionPreference
  $ErrorActionPreference = 'Continue'
  try { & $komut 2>&1 | ForEach-Object { "$_" } } finally { $ErrorActionPreference = $onceki }
}

function UdeSurecleri {
  @(Get-CimInstance Win32_Process | Where-Object {
      $_.Name -match '^uyap' -or ($_.Name -match '^javaw?\.exe$' -and "$($_.CommandLine)" -match 'uyap|havelsan')
    })
}

function Agac([int]$kokPid) {
  $surecler = @(Get-CimInstance Win32_Process -Property ProcessId, ParentProcessId)
  $liste = [System.Collections.Generic.List[int]]::new(); $liste.Add($kokPid)
  for ($i = 0; $i -lt $liste.Count; $i++) {
    foreach ($c in $surecler) {
      if ($c.ParentProcessId -eq $liste[$i] -and -not $liste.Contains([int]$c.ProcessId)) { $liste.Add([int]$c.ProcessId) }
    }
  }
  $liste
}

# UDE'nin kurucunun yazdigi kaydi: HKLM\Software\Classes\.udf -> ProgID -> acma komutu.
$udeProgId = (Get-ItemProperty -LiteralPath 'Registry::HKEY_LOCAL_MACHINE\SOFTWARE\Classes\.udf' -ErrorAction SilentlyContinue).'(default)'
$udeKomut = if ($udeProgId) { (Get-ItemProperty -LiteralPath "Registry::HKEY_LOCAL_MACHINE\SOFTWARE\Classes\$udeProgId\shell\open\command" -ErrorAction SilentlyContinue).'(default)' }
if ("$udeKomut" -notmatch '^\s*"([^"]+)"') { throw "UDE'nin HKLM kaydi bulunamadi (.udf = '$udeProgId')" }
$udeExe = $Matches[1]
if ($Kip -eq 'uygulama' -and -not (Test-Path -LiteralPath $Exe)) { throw '-Kip uygulama icin -Exe <udf-resimcisi.exe> gerekli' }
if ((UdeSurecleri).Count -gt 0) { throw 'UDE acik; kullanicinin oturumuna karismamak icin sinama yapilmadi.' }

$damga = Get-Date -Format 'HHmmssfff'
$calisma = Join-Path $env:TEMP "udf-resimcisi-ude-sinama-$damga"
New-Item -ItemType Directory -Force $calisma | Out-Null
# Turkce harfli ad: "Dilekce eki" / "Ekran goruntusu" ve ozel harfler (c-cedilla, g-breve ...).
$tr = [string]::new([char[]](0x00E7, 0x011F, 0x00FC, 0x015F, 0x0131, 0x00F6))

# Yedekler: UDE'nin ayarlari ve (uygulama kipinde) uygulamanin ayar dosyasi.
$uki = Join-Path $env:USERPROFILE '.uki'
$yedek = Join-Path $calisma 'yedek'
New-Item -ItemType Directory -Force $yedek | Out-Null
$ukiDosyalari = 'acilisDegerleri.xml', 'lastOpened.xml', 'tercihler.xml'
$ukiOz = @{}
foreach ($f in $ukiDosyalari) {
  $p = Join-Path $uki $f
  if (Test-Path -LiteralPath $p) { Copy-Item -LiteralPath $p -Destination $yedek; $ukiOz[$f] = (Get-FileHash -LiteralPath $p).Hash }
}
$ayarDosyasi = Join-Path $env:APPDATA 'UDF Resimcisi\ayarlar.json'
$ayarVardi = Test-Path -LiteralPath $ayarDosyasi
if ($ayarVardi) { Copy-Item -LiteralPath $ayarDosyasi -Destination (Join-Path $yedek 'ayarlar.json') }

$masaAdi = "udf-resimcisi-sinama-$damga"
$masa = [GizliMasa]::Olustur($masaAdi)
$baslangic = Get-Date
$uygPid = 0
$aranan = ''
try {
  switch ($Kip) {
    { $_ -in 'kayit', 'dogrudan' } {
      # Yalnizca metin iceren kucuk bir belge.
      $belge = Join-Path $calisma "Dilek$($tr[0])e eki $tr $damga.udf"
      $xml = '<?xml version="1.0" encoding="UTF-8" ?>' + "`n" + '<template format_id="1.8">' + "`n" +
        '<content><![CDATA[UDF Resimcisi UDE sinamasi' + "`n" + ']]></content><properties><pageFormat mediaSizeName="1" leftMargin="42.52" rightMargin="28.35" topMargin="14.17" bottomMargin="14.17" paperOrientation="1" headerFOffset="20.0" footerFOffset="20.0" /></properties>' + "`n" +
        '<elements resolver="hvl-default">' + "`n" + '<paragraph><content startOffset="0" length="27" /></paragraph>' + "`n" + '</elements>' + "`n" +
        '<styles><style name="default" family="Dialog" size="12" bold="false" italic="false" foreground="-13421773" />' +
        '<style name="hvl-default" family="Times New Roman" size="12" /></styles>' + "`n" + '</template>'
      Add-Type -AssemblyName System.IO.Compression
      $fs = [IO.File]::Open($belge, 'CreateNew')
      $zip = New-Object IO.Compression.ZipArchive($fs, [IO.Compression.ZipArchiveMode]::Create)
      $w = New-Object IO.StreamWriter($zip.CreateEntry('content.xml').Open(), (New-Object Text.UTF8Encoding $false))
      $w.Write($xml); $w.Dispose(); $zip.Dispose(); $fs.Dispose()
      $aranan = $damga
      if ($Kip -eq 'kayit') {
        $gunlukYolu = Join-Path $calisma 'acici.txt'
        $ps = "$env:WINDIR\System32\WindowsPowerShell\v1.0\powershell.exe"
        $komut = "`"$ps`" -NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`" -Acici `"$belge`" -ProgId `"$udeProgId`" -Gunluk `"$gunlukYolu`""
        $null = [GizliMasa]::Baslat($masaAdi, $komut, 0x08000000)   # CREATE_NO_WINDOW
      } else {
        $null = [GizliMasa]::Baslat($masaAdi, "`"$udeExe`" getNewWPInstance EDITOR_TYPE_DOCUMENT `"$belge`"", 0)
      }
    }
    'uygulama' {
      # Deneme resmi: Turkce harfli adli bir ekran goruntusu taklidi.
      Add-Type -AssemblyName System.Drawing
      $png = Join-Path $calisma "Ekran g$($tr[5])r$($tr[2])nt$($tr[2])s$($tr[2]) $damga.png"
      $bmp = New-Object System.Drawing.Bitmap 1600, 1000
      $g = [System.Drawing.Graphics]::FromImage($bmp)
      $g.Clear([System.Drawing.Color]::White)
      $g.DrawString("UDF Resimcisi UDE sinamasi $damga", (New-Object System.Drawing.Font 'Arial', 40), [System.Drawing.Brushes]::Black, 40, 40)
      $g.Dispose(); $bmp.Save($png, [System.Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose()
      $cikti = Join-Path $calisma 'cikti'
      $aranan = $damga

      $argumanlar = (Get-Content (Join-Path (Split-Path -Parent $PSScriptRoot) 'src-tauri\tauri.conf.json') -Raw -Encoding UTF8 | ConvertFrom-Json).app.windows[0].additionalBrowserArgs
      $env:WEBVIEW2_USER_DATA_FOLDER = Join-Path $calisma 'wv2'
      $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$Port $argumanlar"
      $env:UDF_RESIMCISI_SURUM_ADRESI = 'http://127.0.0.1:9/yok'
      $uygPid = [GizliMasa]::Baslat($masaAdi, "`"$Exe`"", 0)
      "uygulama: $Exe (PID $uygPid)"

      function Js([string]$govde) {
        $ifade = "(async () => {`nconst bekle = (ms) => new Promise((r) => setTimeout(r, ms));`nconst inv = window.__TAURI__.core.invoke;`n$govde`n})()"
        $f = Join-Path $calisma 'ifade.js'
        [IO.File]::WriteAllText($f, $ifade)
        (Yerel { node (Join-Path $PSScriptRoot 'cdp.mjs') $Port $f }) -join ' '
      }
      $j = @{ ude = (ConvertTo-Json $udeExe); png = (ConvertTo-Json $png); cikti = (ConvertTo-Json $cikti) }

      '1) Acilis (UDE kendiliginden bulundu mu?):'
      '   ' + (Js @'
for (let i = 0; i < 200 && typeof yollariEkle !== 'function'; i++) await bekle(50);
for (let i = 0; i < 200 && !el.surum.textContent; i++) await bekle(50);
await bekle(1500);
return { surum: el.surum.textContent, udeVar, dugmeGizli: el.udeSec ? el.udeSec.hidden : 'dugme yok', durum: el.durum.textContent };
'@)
      '2) UDE olmayan bir exe gosterilirse:'
      '   ' + (Js @'
try { await inv('ude_yerini_kaydet', { yol: 'C:\\Windows\\notepad.exe' }); return { sonuc: 'KABUL EDILDI' }; }
catch (e) { return { sonuc: 'reddedildi', mesaj: String(e) }; }
'@)
      '3) UDE gosterilir, sonra arayuz ayarlari kaydeder (yol korunmali):'
      '   ' + (Js @"
try {
  await inv('ude_yerini_kaydet', { yol: $($j.ude) });
  const a1 = await inv('ayarlari_getir');
  await inv('ayarlari_kaydet', { ayarlar: { ayri_sayfa: a1.ayri_sayfa, cikti_klasoru: a1.cikti_klasoru, tema: a1.tema, otomatik_guncelleme: a1.otomatik_guncelleme } });
  const a2 = await inv('ayarlari_getir');
  return { kaydedilen: a1.ude_yolu, ayarKaydindanSonra: a2.ude_yolu };
} catch (e) { return { sonuc: 'hata', mesaj: String(e) }; }
"@)
      "4) Resim eklenip 'UDF'de ac'a basilir:"
      '   ' + (Js @"
el.ciktiKlasoru.value = $($j.cikti);
await yollariEkle([$($j.png)]);
for (let i = 0; i < 400 && (el.boyut.textContent.includes('hesaplan') || el.uret.disabled); i++) await bekle(25);
await ac();
return { durum: el.durum.textContent, yol: sonUretilenYol };
"@)
    }
  }

  $bulundu = $null
  $sonListe = @()
  while (((Get-Date) - $baslangic).TotalSeconds -lt $Bekle) {
    $sonListe = [GizliMasa]::Pencereler($masa)
    $bulundu = $sonListe | Where-Object { $_ -match 'SunAwtFrame' -and $_ -match $aranan } | Select-Object -First 1
    if ($bulundu) { break }
    Start-Sleep -Milliseconds 500
  }
  $sure = [math]::Round(((Get-Date) - $baslangic).TotalSeconds, 1)
  if ($bulundu) { "SONUC: belge UDE'de acildi ($sure sn): $(($bulundu -split '\|', 4)[3])" }
  else { "SONUC: $Bekle sn icinde UDE'de belge penceresi gorulmedi" }
  if ($Kip -eq 'kayit' -and (Test-Path (Join-Path $calisma 'acici.txt'))) { 'kabuk: ' + (Get-Content (Join-Path $calisma 'acici.txt') -Raw -Encoding UTF8) }
  'UDE surecleri:'
  UdeSurecleri | ForEach-Object { "   [$($_.ProcessId)] $($_.CommandLine)" }
}
finally {
  if ($uygPid) {
    foreach ($id in @(Agac $uygPid)) { Stop-Process -Id $id -Force -ErrorAction SilentlyContinue }
  }
  # UDE: once nazikce, olmazsa zorla (yalnizca bu sinamada baslayan surecler).
  $null = [GizliMasa]::Kapat($masa)
  $son = (Get-Date).AddSeconds(10)
  while ((Get-Date) -lt $son -and (UdeSurecleri).Count -gt 0) { Start-Sleep -Milliseconds 500 }
  foreach ($s in (UdeSurecleri | Where-Object { $_.CreationDate -ge $baslangic.AddSeconds(-2) })) {
    Stop-Process -Id $s.ProcessId -Force -ErrorAction SilentlyContinue
  }
  Start-Sleep -Milliseconds 800
  $null = [GizliMasa]::CloseDesktop($masa)

  foreach ($f in $ukiDosyalari) {
    $p = Join-Path $uki $f
    if ($ukiOz.ContainsKey($f) -and (Test-Path -LiteralPath $p) -and (Get-FileHash -LiteralPath $p).Hash -ne $ukiOz[$f]) {
      Copy-Item -LiteralPath (Join-Path $yedek $f) -Destination $p -Force
      "UDE ayari geri kondu: $f"
    }
  }
  if ($Kip -eq 'uygulama') {
    if ($ayarVardi) { Copy-Item -LiteralPath (Join-Path $yedek 'ayarlar.json') -Destination $ayarDosyasi -Force }
    elseif (Test-Path -LiteralPath $ayarDosyasi) { Remove-Item -LiteralPath $ayarDosyasi -Force }
    'uygulamanin ayar dosyasi geri kondu'
  }
  "kalan UDE sureci: $((UdeSurecleri).Count)"
  for ($i = 0; $i -lt 10 -and (Test-Path $calisma); $i++) {
    Remove-Item -LiteralPath $calisma -Recurse -Force -ErrorAction SilentlyContinue
    if (Test-Path $calisma) { Start-Sleep -Milliseconds 500 }
  }
}

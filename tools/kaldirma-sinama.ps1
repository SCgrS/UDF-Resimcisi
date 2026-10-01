# Kaldirma kancasinin sinamasi (DOGRULAMA.md, "1.7.3 surumu").
#
# Kancayi (src-tauri/nsis/hooks.nsh) Tauri'nin kaldiricisindaki sirayla kucuk bir NSIS programina
# gomer ve uygulamanin gercek exe'siyle calistirir: PREUNINSTALL, program exe'sinin silinmesi,
# POSTUNINSTALL. Gercek ayar klasorune, Belgelerim'e ve kayit defterine dokunmaz: kancadaki bu
# yollar sinama klasorune cevrilir, exe'ye de APPDATA olarak sinama klasoru verilir.
#
#   tools\kaldirma-sinama.ps1 -Exe src-tauri\target\release\udf-resimcisi.exe
#   git show v1.7.2:src-tauri/nsis/hooks.nsh > $env:TEMP\eski.nsh
#   tools\kaldirma-sinama.ps1 -Exe <herhangi bir exe> -Kanca $env:TEMP\eski.nsh   # eski kanca
#
# makensis, Tauri'nin indirdigi NSIS'ten (%LOCALAPPDATA%\tauri\NSIS) alinir.
param(
  [Parameter(Mandatory)][string]$Exe,
  [string]$Kanca = (Join-Path (Split-Path -Parent $PSScriptRoot) 'src-tauri\nsis\hooks.nsh')
)
$ErrorActionPreference = 'Stop'

$makensis = Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) 'tauri\NSIS\makensis.exe'
$T = Join-Path $env:TEMP 'udf-resimcisi-kaldirma-sinama'
$Exe = (Resolve-Path -LiteralPath $Exe).Path
$Kanca = (Resolve-Path -LiteralPath $Kanca).Path

# Belgenin boyutu ve son degisme zamani, uygulamanin listesindeki bicimde (UNIX'ten nanosaniye).
function Izi([string]$yol) {
  $f = Get-Item -LiteralPath $yol
  $epok = [datetime]::new(1970, 1, 1, 0, 0, 0, [DateTimeKind]::Utc)
  [uint64]$tik = ($f.LastWriteTimeUtc - $epok).Ticks
  [ordered]@{ yol = $f.FullName; boyut = [uint64]$f.Length; degisme = $tik * [uint64]100 }
}

function Senaryo([string]$ad, [int]$kutu, [int]$guncelleme) {
  if (Test-Path -LiteralPath $T) { Remove-Item -LiteralPath $T -Recurse -Force }
  $kurulum = New-Item -ItemType Directory -Force (Join-Path $T 'kurulum')
  $ayar = New-Item -ItemType Directory -Force (Join-Path $T 'appdata\UDF Resimcisi')
  $kendi = New-Item -ItemType Directory -Force (Join-Path $T 'belgeler\UDF Resimcisi')
  $masa = New-Item -ItemType Directory -Force (Join-Path $T 'Masaustu')
  Copy-Item -LiteralPath $Exe (Join-Path $kurulum 'udf-resimcisi.exe')

  $dosyalar = [ordered]@{
    'uretilen, kendi klasorunde'      = Join-Path $kendi 'uretilen-1.udf'
    'uretilen, Masaustu'              = Join-Path $masa 'uretilen-2.udf'
    'kullanicinin dilekcesi (UDE)'    = Join-Path $masa 'dilekce.udf'
    'uretilen, uzerine kaydedilmis'   = Join-Path $masa 'uzerine-kaydedilen.udf'
    'Masaustu not.txt'                = Join-Path $masa 'not.txt'
  }
  foreach ($y in $dosyalar.Values) { [IO.File]::WriteAllText($y, "icerik $y") }
  $liste = @(
    Izi $dosyalar['uretilen, kendi klasorunde']
    Izi $dosyalar['uretilen, Masaustu']
    Izi $dosyalar['uretilen, uzerine kaydedilmis']
  )
  $utf8 = New-Object Text.UTF8Encoding $false
  [IO.File]::WriteAllText((Join-Path $ayar 'uretilenler.json'), (ConvertTo-Json -InputObject $liste -Depth 3), $utf8)
  [IO.File]::WriteAllText((Join-Path $ayar 'ayarlar.json'), '{}')
  Start-Sleep -Milliseconds 50
  [IO.File]::AppendAllText($dosyalar['uretilen, uzerine kaydedilmis'], ' + kullanicinin yazdigi')

  # 1.7.2'nin kancasi kaydetme klasorunu kayit defterinden okur: sinama anahtarina yazilir.
  $kayit = 'HKCU:\Software\UDF Resimcisi Sinama'
  New-Item -Force $kayit | Out-Null
  Set-ItemProperty $kayit -Name CiktiKlasoru -Value $masa.FullName

  $metin = [IO.File]::ReadAllText($Kanca)
  $metin = $metin.Replace('$APPDATA\UDF Resimcisi', "$T\appdata\UDF Resimcisi")
  $metin = $metin.Replace('$DOCUMENTS\UDF Resimcisi', "$T\belgeler\UDF Resimcisi")
  $metin = $metin.Replace('"Software\UDF Resimcisi"', '"Software\UDF Resimcisi Sinama"')
  $kod = ($metin -split "`n" | Where-Object { $_.Trim() -notlike ';*' }) -join "`n"
  if ($kod -match '\$APPDATA|\$LOCALAPPDATA|\$DOCUMENTS|\$PROFILE|"Software\\UDF Resimcisi"') {
    throw 'Kancada sinama klasorune cevrilmemis gercek bir yol kaldi; sinama durduruldu.'
  }
  $kancaKopya = Join-Path $T 'kanca.nsh'
  [IO.File]::WriteAllText($kancaKopya, $metin, $utf8)

  # Kaldiricinin bolumu (Tauri sablonundaki sira): PRE, exe'nin silinmesi, POST.
  $nsi = Join-Path $T 'sinama.nsi'
  $cikti = Join-Path $T 'sinama.exe'
  $satirlar = @(
    'Unicode true'
    '!include LogicLib.nsh'
    '!define MAINBINARYNAME "udf-resimcisi"'
    'Name "kaldirma sinamasi"'
    "OutFile `"$cikti`""
    'RequestExecutionLevel user'
    'SilentInstall silent'
    'Var DeleteAppDataCheckboxState'
    'Var UpdateMode'
    "!include `"$kancaKopya`""
    'Section'
    "  StrCpy `$INSTDIR `"$($kurulum.FullName)`""
    "  StrCpy `$DeleteAppDataCheckboxState $kutu"
    "  StrCpy `$UpdateMode $guncelleme"
    '  !ifmacrodef NSIS_HOOK_PREUNINSTALL'
    '    !insertmacro NSIS_HOOK_PREUNINSTALL'
    '  !endif'
    '  Delete "$INSTDIR\${MAINBINARYNAME}.exe"'
    '  !insertmacro NSIS_HOOK_POSTUNINSTALL'
    'SectionEnd'
  )
  # Yollarda Turkce harf olabilir: BOM'lu UTF-8 (NSIS boyle tanir).
  [IO.File]::WriteAllLines($nsi, $satirlar, (New-Object Text.UTF8Encoding $true))
  $derleme = & $makensis /V2 $nsi 2>&1
  if ($LASTEXITCODE -ne 0) { $derleme; throw 'makensis basarisiz' }

  $eski = $env:APPDATA
  $env:APPDATA = Join-Path $T 'appdata'
  try {
    $p = Start-Process -FilePath $cikti -PassThru
    $p.WaitForExit()
  } finally { $env:APPDATA = $eski }

  ''
  "--- $ad (kutu=$kutu, guncelleme=$guncelleme)"
  foreach ($k in $dosyalar.Keys) {
    '{0,-32} {1}' -f $k, $(if (Test-Path -LiteralPath $dosyalar[$k]) { 'duruyor' } else { 'SILINDI' })
  }
  '{0,-32} {1}' -f 'klasor: Belgeler\UDF Resimcisi', $(if (Test-Path -LiteralPath $kendi.FullName) { 'duruyor' } else { 'KALKTI' })
  '{0,-32} {1}' -f 'klasor: Masaustu', $(if (Test-Path -LiteralPath $masa.FullName) { 'duruyor' } else { 'KALKTI' })
  '{0,-32} {1}' -f 'ayar klasoru', $(if (Test-Path -LiteralPath $ayar.FullName) { 'duruyor' } else { 'KALKTI' })
  '{0,-32} {1}' -f 'kayit defteri anahtari', $(if (Test-Path $kayit) { 'duruyor' } else { 'KALKTI' })
  Remove-Item $kayit -Force -ErrorAction SilentlyContinue
}

try {
  Senaryo 'kutu isaretli' 1 0
  Senaryo 'kutu isaretsiz' 0 0
  Senaryo 'guncelleme (/UPDATE)' 1 1
} finally {
  if (Test-Path -LiteralPath $T) { Remove-Item -LiteralPath $T -Recurse -Force }
}

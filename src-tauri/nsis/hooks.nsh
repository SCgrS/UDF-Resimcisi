; UDF Resimcisi — kaldırma kancaları.
;
; Tauri'nin hazır kaldırıcısı "Uygulama verilerini sil" işaretlendiğinde yalnızca
; $APPDATA\<paket kimliği> ve $LOCALAPPDATA\<paket kimliği> klasörlerini siler. Bu uygulamanın
; ayarları "%APPDATA%\UDF Resimcisi" altında, ürettiği belgeler ise kullanıcının kaydetme
; klasöründe durur; ikisi de o klasörlere girmediği için kutu işaretlense bile yerinde
; kalıyordu. Burada onları da temizliyoruz.
;
; Belgeleri kaldırıcı kendisi seçmez. 1.7.2'ye kadar kaydetme klasöründeki bütün .udf
; dosyalarını siliyordu; klasör Masaüstü ya da bir dava klasörüyse kullanıcının UDE'de yazdığı
; belgeler de gidiyordu. Artık uygulamanın kendisi `--kaldirma-temizligi` ile çalıştırılır ve
; yalnızca kendi ürettiği, o günden beri değişmemiş belgeleri siler (bkz.
; src-tauri/src/uretilenler.rs). Program dosyaları silindikten sonra da çalışabilsin diye exe,
; kaldırma başlamadan kaldırıcının geçici klasörüne kopyalanır.
;
; $DeleteAppDataCheckboxState ve $UpdateMode Tauri'nin şablonundan gelir; ikisi de kaldırma
; bölümü başlamadan (onay sayfasında, komut satırından) belirlenir. Güncellemede eski sürümün
; kaldırıcısı /UPDATE ile çalışır; o zaman hiçbir şey silinmez.

!macro NSIS_HOOK_PREUNINSTALL
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    InitPluginsDir
    CopyFiles /SILENT "$INSTDIR\${MAINBINARYNAME}.exe" "$PLUGINSDIR\${MAINBINARYNAME}.exe"
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    SetShellVarContext current

    ; Uygulamanın ürettiği belgeler. Listeleri ayar klasöründe durduğu için o silinmeden önce.
    ; Exe kopyalanamadıysa belgelere dokunulmaz.
    ${If} ${FileExists} "$PLUGINSDIR\${MAINBINARYNAME}.exe"
      ExecWait '"$PLUGINSDIR\${MAINBINARYNAME}.exe" --kaldirma-temizligi'
    ${EndIf}

    ; Ayar klasörü baştan sona uygulamaya ait: olduğu gibi kaldırılır. Uygulama yeni
    ; kapanmışsa ayarlar.json'un tutamacı bir an daha açık kalabiliyor — o yüzden bir kez
    ; bekleyip yeniden deneniyor, olmazsa iş yeniden başlatmaya bırakılıyor.
    RMDir /r "$APPDATA\UDF Resimcisi"
    ${If} ${FileExists} "$APPDATA\UDF Resimcisi\*.*"
      Sleep 2000
      RMDir /r "$APPDATA\UDF Resimcisi"
    ${EndIf}
    ${If} ${FileExists} "$APPDATA\UDF Resimcisi\*.*"
      RMDir /r /REBOOTOK "$APPDATA\UDF Resimcisi"
    ${EndIf}

    ; 1.7.2'ye kadar uygulama kaydetme klasörünün yolunu kaldırıcı için buraya yazıyordu.
    DeleteRegKey HKCU "Software\UDF Resimcisi"
  ${EndIf}
!macroend

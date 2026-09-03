; UDF Resimcisi — kaldırma kancaları.
;
; Tauri'nin hazır kaldırıcısı "Uygulama verilerini sil" işaretlendiğinde yalnızca
; $APPDATA\<paket kimliği> ve $LOCALAPPDATA\<paket kimliği> klasörlerini siler. Bu uygulamanın
; ayarları "%APPDATA%\UDF Resimcisi" altında, ürettiği belgeler ise kullanıcının kaydetme
; klasöründe durur; ikisi de o klasörlere girmediği için kutu işaretlense bile yerinde
; kalıyordu. Burada onları da temizliyoruz.
;
; $DeleteAppDataCheckboxState ve $UpdateMode değişkenleri Tauri'nin şablonundan gelir; bu
; makro Section Uninstall'ın içine, o değişkenler okunduktan sonra yerleştirilir.

!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    SetShellVarContext current

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

    ; Kaydetme klasörü kullanıcının seçtiği herhangi bir yer olabilir — "Belgelerim"in
    ; kendisi bile. Bu yüzden asla özyinelemeli silinmez: yalnızca uygulamanın ürettiği
    ; .udf dosyaları silinir, klasör de ancak geriye bir şey kalmadıysa kaldırılır.
    ; Yolu uygulama HKCU'ya yazar (bkz. src-tauri/src/kayit.rs).
    ReadRegStr $0 HKCU "Software\UDF Resimcisi" "CiktiKlasoru"
    ${If} $0 != ""
      Delete "$0\*.udf"
      RMDir "$0"
    ${EndIf}

    ; Uygulama hiç çalıştırılmadıysa kayıt defterinde iz yoktur; varsayılan yeri de dene.
    Delete "$DOCUMENTS\UDF Resimcisi\*.udf"
    RMDir "$DOCUMENTS\UDF Resimcisi"

    DeleteRegKey HKCU "Software\UDF Resimcisi"
  ${EndIf}
!macroend

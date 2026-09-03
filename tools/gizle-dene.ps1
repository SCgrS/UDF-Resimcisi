# Deney: UDE penceresini TAŞIMADAN görünmez yapmak (WS_EX_LAYERED + alpha 0).
# Taşımak UDE'nin tercihler.xml dosyasındaki win_posx/win_posy değerini kalıcı bozuyor;
# saydamlaştırma pencere geometrisine hiç dokunmadığı için bu sorunu yaratmıyor.
param([Parameter(Mandatory=$true)][string]$Udf)

Add-Type @"
using System;using System.Text;using System.Runtime.InteropServices;
public class Gizle {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc f, IntPtr l);
  public delegate bool EnumWindowsProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr h, StringBuilder t, int c);
  [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetClassNameW(IntPtr h, StringBuilder t, int c);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern int GetWindowLong(IntPtr h, int i);
  [DllImport("user32.dll")] public static extern int SetWindowLong(IntPtr h, int i, int v);
  [DllImport("user32.dll")] public static extern bool SetLayeredWindowAttributes(IntPtr h, uint key, byte alpha, uint flags);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, IntPtr pid);
  [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint a, uint b, bool f);
  [DllImport("user32.dll")] public static extern IntPtr SetFocus(IntPtr h);
  [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
  [DllImport("user32.dll")] public static extern uint GetClipboardSequenceNumber();
  [DllImport("user32.dll")] public static extern uint SendInput(uint n, INPUT[] p, int cb);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);

  [StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk, wScan; public uint dwFlags, time; public IntPtr extra; }
  [StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx, dy; public uint d, f, t; public IntPtr e; }
  [StructLayout(LayoutKind.Explicit)] public struct U { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; }
  [StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public U u; }

  const int GWL_EXSTYLE = -20;
  const int WS_EX_LAYERED = 0x00080000;
  const uint LWA_ALPHA = 0x2;

  public static IntPtr Bul(string sinif, string desen) {
    IntPtr f = IntPtr.Zero;
    EnumWindows(delegate(IntPtr h, IntPtr p) {
      if (!IsWindowVisible(h)) return true;
      StringBuilder c = new StringBuilder(256); GetClassNameW(h, c, 256);
      if (sinif.Length > 0 && c.ToString() != sinif) return true;
      if (desen.Length > 0) {
        StringBuilder t = new StringBuilder(512); GetWindowTextW(h, t, 512);
        if (t.ToString().IndexOf(desen, StringComparison.OrdinalIgnoreCase) < 0) return true;
      }
      f = h; return false;
    }, IntPtr.Zero);
    return f;
  }

  /** Pencereyi yerinde görünmez yapar; konumu hiç değişmez. */
  public static void Saydamlastir(IntPtr h) {
    int ex = GetWindowLong(h, GWL_EXSTYLE);
    SetWindowLong(h, GWL_EXSTYLE, ex | WS_EX_LAYERED);
    SetLayeredWindowAttributes(h, 0, 0, LWA_ALPHA);
  }

  public static bool OneGetir(IntPtr h) {
    uint fg = GetWindowThreadProcessId(GetForegroundWindow(), IntPtr.Zero);
    uint hedef = GetWindowThreadProcessId(h, IntPtr.Zero);
    uint ben = GetCurrentThreadId();
    AttachThreadInput(ben, fg, true); AttachThreadInput(ben, hedef, true);
    bool ok = SetForegroundWindow(h); SetFocus(h);
    AttachThreadInput(ben, hedef, false); AttachThreadInput(ben, fg, false);
    return ok;
  }

  static INPUT T(ushort vk, bool up) {
    INPUT i = new INPUT(); i.type = 1; i.u.ki.wVk = vk; i.u.ki.dwFlags = up ? 2u : 0u; return i;
  }
  public static void Ctrl(ushort vk) {
    INPUT[] g = new INPUT[] { T(0x11,false), T(vk,false), T(vk,true), T(0x11,true) };
    SendInput(4, g, Marshal.SizeOf(typeof(INPUT)));
  }
  public static uint Seq(){ return GetClipboardSequenceNumber(); }
  public static void Kapat(IntPtr h){ PostMessage(h, 0x0010, IntPtr.Zero, IntPtr.Zero); }
}
"@

[void][Gizle]::SetProcessDPIAware()
$desen = [IO.Path]::GetFileNameWithoutExtension($Udf)
$onceki = [Gizle]::GetForegroundWindow()
$t = [Diagnostics.Stopwatch]::StartNew()
Start-Process -FilePath $Udf

$h = [IntPtr]::Zero
$splashGizlendi = $false
while ($t.Elapsed.TotalSeconds -lt 40) {
  $s = [Gizle]::Bul("JavaSplash", "")
  if ($s -ne [IntPtr]::Zero -and -not $splashGizlendi) { [Gizle]::Saydamlastir($s); $splashGizlendi = $true; Write-Output ("{0,6:N0} ms  acilis gorseli saydamlastirildi" -f $t.Elapsed.TotalMilliseconds) }
  $h = [Gizle]::Bul("SunAwtFrame", $desen)
  if ($h -ne [IntPtr]::Zero) { [Gizle]::Saydamlastir($h); Write-Output ("{0,6:N0} ms  belge penceresi saydamlastirildi" -f $t.Elapsed.TotalMilliseconds); break }
  Start-Sleep -Milliseconds 20
}
if ($h -eq [IntPtr]::Zero) { Write-Output "PENCERE-YOK"; exit 2 }

[void][Gizle]::OneGetir($h)
Start-Sleep -Milliseconds 450
$once = [Gizle]::Seq()
[Gizle]::Ctrl(0x41)   # Ctrl+A
Start-Sleep -Milliseconds 250
[Gizle]::Ctrl(0x43)   # Ctrl+C
Start-Sleep -Milliseconds 700
$sonra = [Gizle]::Seq()
[Gizle]::Kapat($h)
if ($onceki -ne [IntPtr]::Zero) { [void][Gizle]::OneGetir($onceki) }
Write-Output ("pano-once={0} pano-sonra={1} degisti={2} toplam={3:N0} ms" -f $once, $sonra, ($once -ne $sonra), $t.Elapsed.TotalMilliseconds)

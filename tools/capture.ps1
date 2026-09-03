# Ekran/pencere yakalama yardimcisi.
#   capture.ps1 -Out <png> [-TitleLike <desen>] [-WaitSec <sn>]
# Fiziksel piksel icin SetProcessDPIAware (DPI-unaware yakalama 150% ekrani kucultup
# pixelation'i gizler - ude-win-x64 CLAUDE.md notu).
param(
  [Parameter(Mandatory=$true)][string]$Out,
  [string]$TitleLike = "",
  [int]$WaitSec = 0,
  [switch]$Print,
  [string]$Pencere = ""
)

Add-Type -AssemblyName System.Windows.Forms, System.Drawing

Add-Type @"
using System;
using System.Text;
using System.Runtime.InteropServices;
public class Win32Cap {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);
  public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr hWnd, StringBuilder text, int count);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr hWnd, IntPtr hdcBlt, uint nFlags);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }

  public static IntPtr Find(string like) {
    IntPtr found = IntPtr.Zero;
    EnumWindows(delegate(IntPtr h, IntPtr p) {
      if (!IsWindowVisible(h)) return true;
      StringBuilder sb = new StringBuilder(512);
      GetWindowTextW(h, sb, 512);
      string t = sb.ToString();
      if (t.Length > 0 && t.IndexOf(like, StringComparison.OrdinalIgnoreCase) >= 0) { found = h; return false; }
      return true;
    }, IntPtr.Zero);
    return found;
  }

  public static string ListWindows() {
    StringBuilder all = new StringBuilder();
    EnumWindows(delegate(IntPtr h, IntPtr p) {
      if (!IsWindowVisible(h)) return true;
      StringBuilder sb = new StringBuilder(512);
      GetWindowTextW(h, sb, 512);
      if (sb.Length > 0) all.AppendLine(h.ToString() + "\t" + sb.ToString());
      return true;
    }, IntPtr.Zero);
    return all.ToString();
  }
}
"@

[void][Win32Cap]::SetProcessDPIAware()
if ($WaitSec -gt 0) { Start-Sleep -Seconds $WaitSec }

$rect = New-Object Win32Cap+RECT
$hwnd = [IntPtr]::Zero
if ($Pencere -ne "") { $hwnd = [IntPtr][int]$Pencere }
if ($TitleLike -ne "" -and $hwnd -eq [IntPtr]::Zero) {
  $hwnd = [Win32Cap]::Find($TitleLike)
  if ($hwnd -eq [IntPtr]::Zero) {
    Write-Output "PENCERE-YOK: '$TitleLike' bulunamadi. Gorunur pencereler:"
    Write-Output ([Win32Cap]::ListWindows())
    exit 2
  }
  if (-not $Print) {
    [void][Win32Cap]::ShowWindow($hwnd, 3)   # SW_MAXIMIZE
    [void][Win32Cap]::SetForegroundWindow($hwnd)
    Start-Sleep -Milliseconds 700
  }
}
if ($hwnd -ne [IntPtr]::Zero) {
  [void][Win32Cap]::GetWindowRect($hwnd, [ref]$rect)
  $x = $rect.Left; $y = $rect.Top
  $w = $rect.Right - $rect.Left; $h = $rect.Bottom - $rect.Top
} else {
  $b = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
  $x = $b.X; $y = $b.Y; $w = $b.Width; $h = $b.Height
}

if ($w -le 0 -or $h -le 0) { Write-Output "GECERSIZ-BOYUT: ${w}x${h}"; exit 3 }

$bmp = New-Object System.Drawing.Bitmap $w, $h
$g = [System.Drawing.Graphics]::FromImage($bmp)
if ($Print -and $hwnd -ne [IntPtr]::Zero) {
  $hdc = $g.GetHdc()
  $ok = [Win32Cap]::PrintWindow($hwnd, $hdc, 2)
  $g.ReleaseHdc($hdc)
  if (-not $ok) { Write-Output "PRINTWINDOW-BASARISIZ, ekrandan aliniyor"; $g.CopyFromScreen($x, $y, 0, 0, $bmp.Size) }
} else {
  $g.CopyFromScreen($x, $y, 0, 0, $bmp.Size)
}
$g.Dispose()
$bmp.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()
Write-Output "YAKALANDI: $Out (${w}x${h}) hwnd=$hwnd"

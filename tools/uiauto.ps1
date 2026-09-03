# UDE otomasyon prototipi: pencere bul -> one getir -> (istege bagli) tikla -> SendInput ile tus gonder.
# Rust ude.rs modulunun dogrulama prototipi.
#   uiauto.ps1 -TitleLike "turnusol" -ClickRel "0.5,0.6" -Keys "CTRL+A","CTRL+C"
param(
  [string]$TitleLike = "",
  [string]$Pencere = "",
  [string]$ClickRel = "",
  [string]$Text = "",
  [string]$ClickAbs = "",
  [string]$DragFrom = "",
  [string]$DragTo = "",
  [string]$Move = "",
  [switch]$Topmost,
  [switch]$TopmostOff,
  [string]$Keys = "",
  [int]$PreWaitSec = 1,
  [int]$PostWaitMs = 600
)

Add-Type @"
using System;
using System.Text;
using System.Runtime.InteropServices;
public class UdeAuto {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);
  public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr hWnd, StringBuilder text, int count);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, IntPtr pid);
  [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint idAttach, uint idAttachTo, bool fAttach);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr hWnd);
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr hWnd, IntPtr hWndInsertAfter, int X, int Y, int cx, int cy, uint uFlags);

  public static void Topmost(IntPtr h, bool on) {
    IntPtr TOP = new IntPtr(-1), NOTOP = new IntPtr(-2);
    SetWindowPos(h, on ? TOP : NOTOP, 0, 0, 0, 0, 0x0001 | 0x0002 | 0x0040);
  }
  [DllImport("user32.dll")] public static extern IntPtr SetFocus(IntPtr hWnd);
  [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
  [DllImport("user32.dll")] public static extern uint GetClipboardSequenceNumber();
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int X, int Y);
  [DllImport("user32.dll")] public static extern uint SendInput(uint nInputs, INPUT[] pInputs, int cbSize);

  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
  [StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx, dy; public uint mouseData, dwFlags, time; public IntPtr dwExtraInfo; }
  [StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort wVk, wScan; public uint dwFlags, time; public IntPtr dwExtraInfo; }
  [StructLayout(LayoutKind.Sequential)] public struct HARDWAREINPUT { public uint uMsg; public ushort wParamL, wParamH; }
  [StructLayout(LayoutKind.Explicit)] public struct INPUTUNION {
    [FieldOffset(0)] public MOUSEINPUT mi;
    [FieldOffset(0)] public KEYBDINPUT ki;
    [FieldOffset(0)] public HARDWAREINPUT hi;
  }
  [StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public INPUTUNION u; }

  const uint INPUT_MOUSE = 0, INPUT_KEYBOARD = 1;
  const uint KEYEVENTF_KEYUP = 0x0002;
  const uint MOUSEEVENTF_LEFTDOWN = 0x0002, MOUSEEVENTF_LEFTUP = 0x0004;

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

  public static bool ForceForeground(IntPtr hWnd) {
    uint fg = GetWindowThreadProcessId(GetForegroundWindow(), IntPtr.Zero);
    uint target = GetWindowThreadProcessId(hWnd, IntPtr.Zero);
    uint me = GetCurrentThreadId();
    AttachThreadInput(me, fg, true);
    AttachThreadInput(me, target, true);
    if (IsIconic(hWnd)) ShowWindow(hWnd, 9);
    bool ok = SetForegroundWindow(hWnd);
    SetFocus(hWnd);
    AttachThreadInput(me, target, false);
    AttachThreadInput(me, fg, false);
    return ok;
  }

  public static void Click(int x, int y) {
    SetCursorPos(x, y);
    System.Threading.Thread.Sleep(120);
    INPUT[] inp = new INPUT[2];
    inp[0].type = INPUT_MOUSE; inp[0].u.mi.dwFlags = MOUSEEVENTF_LEFTDOWN;
    inp[1].type = INPUT_MOUSE; inp[1].u.mi.dwFlags = MOUSEEVENTF_LEFTUP;
    SendInput(2, inp, Marshal.SizeOf(typeof(INPUT)));
  }

  public static void Key(ushort vk, bool ctrl) {
    int n = ctrl ? 4 : 2;
    INPUT[] inp = new INPUT[n];
    int i = 0;
    if (ctrl) { inp[i].type = INPUT_KEYBOARD; inp[i].u.ki.wVk = 0x11; i++; }        // VK_CONTROL down
    inp[i].type = INPUT_KEYBOARD; inp[i].u.ki.wVk = vk; i++;
    inp[i].type = INPUT_KEYBOARD; inp[i].u.ki.wVk = vk; inp[i].u.ki.dwFlags = KEYEVENTF_KEYUP; i++;
    if (ctrl) { inp[i].type = INPUT_KEYBOARD; inp[i].u.ki.wVk = 0x11; inp[i].u.ki.dwFlags = KEYEVENTF_KEYUP; i++; }
    SendInput((uint)n, inp, Marshal.SizeOf(typeof(INPUT)));
  }

  public static uint ClipSeq() { return GetClipboardSequenceNumber(); }
  /** Gercek OLE surukleme: basili tut, kucuk adimlarla ilerle, birak. */
  public static void Move(IntPtr h, int x, int y, int w, int ht) {
    SetWindowPos(h, IntPtr.Zero, x, y, w, ht, 0x0004 | 0x0040);
  }
  public static void Drag(int x1, int y1, int x2, int y2) {
    SetCursorPos(x1, y1);
    System.Threading.Thread.Sleep(250);
    INPUT[] down = new INPUT[1];
    down[0].type = INPUT_MOUSE; down[0].u.mi.dwFlags = MOUSEEVENTF_LEFTDOWN;
    SendInput(1, down, Marshal.SizeOf(typeof(INPUT)));
    System.Threading.Thread.Sleep(200);
    int adim = 40;
    for (int i = 1; i <= adim; i++) {
      int cx = x1 + (x2 - x1) * i / adim;
      int cy = y1 + (y2 - y1) * i / adim;
      SetCursorPos(cx, cy);
      System.Threading.Thread.Sleep(25);
    }
    System.Threading.Thread.Sleep(600);
    INPUT[] up = new INPUT[1];
    up[0].type = INPUT_MOUSE; up[0].u.mi.dwFlags = MOUSEEVENTF_LEFTUP;
    SendInput(1, up, Marshal.SizeOf(typeof(INPUT)));
  }
  /** Unicode karakter gonder (klavye duzeninden bagimsiz - KEYEVENTF_UNICODE). */
  public static void TypeText(string s) {
    foreach (char c in s) {
      INPUT[] inp = new INPUT[2];
      inp[0].type = INPUT_KEYBOARD; inp[0].u.ki.wVk = 0; inp[0].u.ki.wScan = c; inp[0].u.ki.dwFlags = 0x0004;
      inp[1].type = INPUT_KEYBOARD; inp[1].u.ki.wVk = 0; inp[1].u.ki.wScan = c; inp[1].u.ki.dwFlags = 0x0004 | KEYEVENTF_KEYUP;
      SendInput(2, inp, Marshal.SizeOf(typeof(INPUT)));
      System.Threading.Thread.Sleep(12);
    }
  }
}
"@

[void][UdeAuto]::SetProcessDPIAware()

$h = [IntPtr]::Zero
if ($Pencere -ne "") { $h = [IntPtr][int]$Pencere } else { $h = [UdeAuto]::Find($TitleLike) }
if ($h -eq [IntPtr]::Zero) { Write-Output "PENCERE-YOK: $TitleLike"; exit 2 }
if ($Topmost) { [void][UdeAuto]::Topmost($h, $true) }
if ($TopmostOff) { [void][UdeAuto]::Topmost($h, $false) }
[void][UdeAuto]::ForceForeground($h)
Start-Sleep -Seconds $PreWaitSec

if ($ClickRel -ne "") {
  $r = New-Object UdeAuto+RECT
  [void][UdeAuto]::GetWindowRect($h, [ref]$r)
  $parts = $ClickRel.Split(",")
  $cx = [int]($r.Left + ($r.Right - $r.Left) * [double]$parts[0])
  $cy = [int]($r.Top + ($r.Bottom - $r.Top) * [double]$parts[1])
  [UdeAuto]::Click($cx, $cy)
  Write-Output "TIKLANDI: $cx,$cy (pencere $($r.Left),$($r.Top)-$($r.Right),$($r.Bottom))"
  Start-Sleep -Milliseconds 400
}

if ($ClickAbs -ne "") {
  $pa = $ClickAbs.Split(",")
  [UdeAuto]::Click([int]$pa[0], [int]$pa[1])
  Write-Output "TIKLANDI-MUTLAK: $ClickAbs"
  Start-Sleep -Milliseconds 400
}
if ($Move -ne "") {
  $m = $Move.Split(",")
  [UdeAuto]::Move($h, [int]$m[0], [int]$m[1], [int]$m[2], [int]$m[3])
  Write-Output "TASINDI: $Move"
  Start-Sleep -Milliseconds 600
}
if ($DragFrom -ne "" -and $DragTo -ne "") {
  $a = $DragFrom.Split(","); $b = $DragTo.Split(",")
  [UdeAuto]::Drag([int]$a[0], [int]$a[1], [int]$b[0], [int]$b[1])
  Write-Output "SURUKLENDI: $DragFrom -> $DragTo"
  Start-Sleep -Milliseconds 1500
}
if ($Text -ne "") {
  [UdeAuto]::TypeText($Text)
  Write-Output "YAZILDI: $Text"
  Start-Sleep -Milliseconds 400
}
$vk = @{
  "A"=0x41; "C"=0x43; "S"=0x53; "V"=0x56; "N"=0x4E; "END"=0x23; "HOME"=0x24;
  "ENTER"=0x0D; "BACKSPACE"=0x08; "SPACE"=0x20; "TAB"=0x09; "ESC"=0x1B
}

$before = [UdeAuto]::ClipSeq()
foreach ($k in ($Keys.Split(",") | Where-Object { $_ -ne "" })) {
  $ctrl = $false; $name = $k
  if ($k.ToUpper().StartsWith("CTRL+")) { $ctrl = $true; $name = $k.Substring(5) }
  $code = $vk[$name.ToUpper()]
  if ($null -eq $code) { Write-Output "BILINMEYEN-TUS: $k"; exit 3 }
  [UdeAuto]::Key([uint16]$code, $ctrl)
  Start-Sleep -Milliseconds 350
}
Start-Sleep -Milliseconds $PostWaitMs
$after = [UdeAuto]::ClipSeq()
Write-Output ("TUSLAR-GONDERILDI pano-once={0} pano-sonra={1} degisti={2}" -f $before, $after, ($before -ne $after))

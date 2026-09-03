//! UDF belge modeli. `macospasterich/UdeDoc.java` referansının Rust portu.
//!
//! Saf veri: dosya sistemi, Tauri veya Windows bağımlılığı yok — `cargo test` ile
//! doğrudan sınanabilir.

/// Paragraf hizalaması (content.xml'deki `Alignment` sayısal değeri).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left = 0,
    Center = 1,
    Right = 2,
    Justify = 3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub font_family: String,
    pub font_size: f64,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    /// Java `Color.getRGB()` işaretli tamsayısı; siyah = -16777216 (varsayılan, yazılmaz).
    pub color: i32,
    /// -1 = beyaz/şeffaf (varsayılan, yazılmaz).
    pub background_color: i32,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: "Times New Roman".to_string(),
            font_size: 12.0,
            bold: false,
            italic: false,
            underline: false,
            color: -16_777_216,
            background_color: -1,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageRun {
    /// Resim baytlarının base64'ü — satır sonu ve data-URI öneki YOK.
    pub data_b64: String,
    /// Punto cinsinden görüntüleme genişliği (bitmap çözünürlüğünden bağımsız).
    pub width: f64,
    /// Punto cinsinden görüntüleme yüksekliği.
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextRun {
    pub text: String,
    pub style: TextStyle,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Run {
    Image(ImageRun),
    Text(TextRun),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph {
    pub alignment: Alignment,
    pub left_indent: f64,
    pub right_indent: f64,
    pub runs: Vec<Run>,
}

impl Default for Paragraph {
    fn default() -> Self {
        Self {
            alignment: Alignment::Left,
            left_indent: 0.0,
            right_indent: 0.0,
            runs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Paragraph(Paragraph),
    PageBreak,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageFormat {
    pub media_size_name: i32,
    pub left_margin: f64,
    pub right_margin: f64,
    pub top_margin: f64,
    pub bottom_margin: f64,
    pub paper_orientation: i32,
    pub header_foffset: f64,
    pub footer_foffset: f64,
}

impl Default for PageFormat {
    /// A4 dikey, UDE'nin varsayılan kenar boşlukları.
    fn default() -> Self {
        Self {
            media_size_name: 1,
            left_margin: 42.52,
            right_margin: 28.35,
            top_margin: 14.17,
            bottom_margin: 14.17,
            paper_orientation: 1,
            header_foffset: 20.0,
            footer_foffset: 20.0,
        }
    }
}

impl PageFormat {
    /// A4 sayfa genişliği (punto).
    pub const PAGE_WIDTH: f64 = 595.28;
    /// A4 sayfa yüksekliği (punto).
    pub const PAGE_HEIGHT: f64 = 841.89;

    /// Metnin/resmin sığabileceği genişlik (punto). Varsayılanlarda 524.41.
    pub fn usable_width(&self) -> f64 {
        Self::PAGE_WIDTH - self.left_margin - self.right_margin
    }

    /// Metnin/resmin sığabileceği yükseklik (punto). Varsayılanlarda 773.55.
    pub fn usable_height(&self) -> f64 {
        Self::PAGE_HEIGHT
            - self.top_margin
            - self.bottom_margin
            - self.header_foffset
            - self.footer_foffset
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Document {
    pub pages: PageFormat,
    pub body: Vec<Block>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kullanilabilir_alan_a4_varsayilan() {
        let pf = PageFormat::default();
        assert!((pf.usable_width() - 524.41).abs() < 0.005, "{}", pf.usable_width());
        assert!((pf.usable_height() - 773.55).abs() < 0.005, "{}", pf.usable_height());
    }
}

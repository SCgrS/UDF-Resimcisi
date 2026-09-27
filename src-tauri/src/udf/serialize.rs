//! Belge modeli → `content.xml`. `macospasterich/UdeXml.java` referansının Rust portu.
//!
//! Kritik kural: `<content>` içindeki CDATA belgenin düz metnidir; `<elements>` içindeki her
//! eleman bu metne `startOffset` + `length` ile bağlanır. Offset'ler **kod noktası** (code
//! point) cinsindendir, UTF-16 kod birimi değil.
//!
//! Ondalık ayırıcı her zaman noktadır (Rust `format!` yerel ayardan bağımsızdır; Java portunda
//! bunun için `Locale.US` gerekiyordu).
//!
//! Metin bir yazıcıya parça parça akar: 12 MP'lik bir fotoğrafın base64'ü tek başına ~5 MB,
//! bütün belgeyi önce `String` olarak kurmak her resmi bellekte birkaç kez çoğaltıyordu.

use std::io::{self, Write};

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;

use super::model::{Block, Document, ImageRun, Paragraph, Run, TextStyle};

/// CDATA akışında resmin durduğu yer: OBJECT REPLACEMENT CHARACTER.
const OBJECT_REPLACEMENT: char = '\u{FFFC}';
/// Boş paragrafın ve sayfa sonunun CDATA karşılığı: ZERO WIDTH SPACE.
const ZERO_WIDTH_SPACE: char = '\u{200B}';

/// CDATA akışındaki bir çalıştırmanın (run) offset kaydı.
struct Entry<'a> {
    run: &'a Run,
    start_offset: usize,
    length: usize,
    block_id: usize,
}

struct Builder<'a> {
    cdata: String,
    entries: Vec<Entry<'a>>,
    offset: usize,
    block_id: usize,
}

/// Belgeyi `content.xml` metnine çevirir.
pub fn serialize(doc: &Document) -> String {
    let mut xml = Vec::new();
    yaz(doc, &mut xml).expect("belleğe yazmak başarısız olmaz");
    String::from_utf8(xml).expect("content.xml UTF-8'dir")
}

/// Belgenin `content.xml` metnini `w`'ye yazar.
pub fn yaz<W: Write>(doc: &Document, w: &mut W) -> io::Result<()> {
    let mut b = Builder {
        cdata: String::new(),
        entries: Vec::new(),
        offset: 0,
        block_id: 0,
    };
    b.build_blocks(&doc.body);

    w.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n")?;
    w.write_all(b"<template format_id=\"1.8\">\n")?;
    w.write_all(b"<content><![CDATA[")?;
    w.write_all(b.cdata.as_bytes())?;
    w.write_all(b"]]></content>")?;
    w.write_all(page_format(doc).as_bytes())?;
    w.write_all(b"\n")?;
    w.write_all(b"<elements resolver=\"hvl-default\">\n")?;

    let mut block_id_counter = 0usize;
    let mut offset_cursor = 0usize;
    b.serialize_blocks(&doc.body, &mut block_id_counter, &mut offset_cursor, w)?;

    w.write_all(b"</elements>\n")?;
    w.write_all(b"<styles>")?;
    w.write_all(
        "<style name=\"default\" description=\"Geçerli\" family=\"Dialog\" size=\"12\" \
         bold=\"false\" italic=\"false\" foreground=\"-13421773\" \
         FONT_ATTRIBUTE_KEY=\"javax.swing.plaf.FontUIResource[family=Dialog,name=Dialog,style=plain,size=12]\" />"
            .as_bytes(),
    )?;
    w.write_all(
        "<style name=\"hvl-default\" family=\"Times New Roman\" size=\"12\" description=\"Gövde\" />"
            .as_bytes(),
    )?;
    w.write_all(b"</styles>\n")?;
    w.write_all(b"</template>")
}

impl<'a> Builder<'a> {
    // ---- 1. geçiş: CDATA metni + offset kayıtları ----

    fn build_blocks(&mut self, blocks: &'a [Block]) {
        for block in blocks {
            match block {
                Block::Paragraph(p) => self.build_paragraph(p),
                Block::PageBreak => {
                    self.cdata.push(ZERO_WIDTH_SPACE);
                    self.cdata.push('\n');
                    self.offset += 2;
                    self.block_id += 1;
                }
            }
        }
    }

    fn build_paragraph(&mut self, para: &'a Paragraph) {
        let current_block_id = self.block_id;
        self.block_id += 1;

        if para.runs.is_empty() {
            self.cdata.push(ZERO_WIDTH_SPACE);
            self.offset += 1;
        } else {
            for run in &para.runs {
                match run {
                    Run::Text(t) => {
                        let len = t.text.chars().count();
                        self.entries.push(Entry {
                            run,
                            start_offset: self.offset,
                            length: len,
                            block_id: current_block_id,
                        });
                        self.cdata.push_str(&t.text);
                        self.offset += len;
                    }
                    Run::Image(_) => {
                        self.entries.push(Entry {
                            run,
                            start_offset: self.offset,
                            length: 1,
                            block_id: current_block_id,
                        });
                        self.cdata.push(OBJECT_REPLACEMENT);
                        self.offset += 1;
                    }
                }
            }
        }
        self.cdata.push('\n');
        self.offset += 1;
    }

    // ---- 2. geçiş: eleman ağacı ----

    fn entries_for_block(&self, id: usize) -> Vec<&Entry<'a>> {
        self.entries.iter().filter(|e| e.block_id == id).collect()
    }

    fn serialize_blocks<W: Write>(
        &self,
        blocks: &[Block],
        block_id_counter: &mut usize,
        offset_cursor: &mut usize,
        w: &mut W,
    ) -> io::Result<()> {
        for block in blocks {
            match block {
                Block::Paragraph(p) => {
                    let id = *block_id_counter;
                    *block_id_counter += 1;
                    let be = self.entries_for_block(id);
                    serialize_paragraph(p, &be, *offset_cursor, w)?;
                    if be.is_empty() {
                        *offset_cursor += 2;
                    } else {
                        let last = be[be.len() - 1];
                        *offset_cursor = last.start_offset + last.length + 1;
                    }
                }
                Block::PageBreak => {
                    *block_id_counter += 1;
                    let empty = Paragraph::default();
                    let mut inner = Vec::new();
                    serialize_paragraph(&empty, &[], *offset_cursor, &mut inner)?;
                    if inner.last() == Some(&b'\n') {
                        inner.pop();
                    }
                    w.write_all(b"<page-break>")?;
                    w.write_all(&inner)?;
                    w.write_all(b"</page-break>\n")?;
                    *offset_cursor += 2;
                }
            }
        }
        Ok(())
    }
}

fn serialize_paragraph<W: Write>(
    para: &Paragraph,
    be: &[&Entry],
    empty_offset: usize,
    w: &mut W,
) -> io::Result<()> {
    write!(
        w,
        "<paragraph Alignment=\"{}\" LeftIndent=\"{}\" RightIndent=\"{}\">",
        para.alignment as i32,
        f1(para.left_indent),
        f1(para.right_indent)
    )?;

    if be.is_empty() {
        write!(
            w,
            "<content startOffset=\"{empty_offset}\" length=\"2\" family=\"Times New Roman\" size=\"10\" />"
        )?;
    } else {
        let last_idx = be.len() - 1;
        for (i, entry) in be.iter().enumerate() {
            match entry.run {
                Run::Text(t) => {
                    // Paragrafın son elemanı metinse `length` paragraf sonu `\n`'i de kapsar.
                    let len = entry.length + if i == last_idx { 1 } else { 0 };
                    write!(
                        w,
                        "<content startOffset=\"{}\" length=\"{}\"{} />",
                        entry.start_offset,
                        len,
                        font_attrs(&t.style)
                    )?;
                }
                Run::Image(img) => {
                    // Son eleman resimse `\n` KAPSANMAZ (referans uygulamanın davranışı).
                    image_tag(img, entry.start_offset, w)?;
                }
            }
        }
    }
    w.write_all(b"</paragraph>\n")
}

fn image_tag<W: Write>(img: &ImageRun, start_offset: usize, w: &mut W) -> io::Result<()> {
    w.write_all(b"<image imageData=\"")?;
    base64_yaz(&img.data, w)?;
    write!(
        w,
        "\" startOffset=\"{}\" length=\"1\" width=\"{}\" height=\"{}\" />",
        start_offset,
        f1(img.width),
        f1(img.height)
    )
}

/// Baytları base64 olarak parça parça yazar. Parçalar 3 baytın katı olduğu için aralarda dolgu
/// (`=`) oluşmaz; çıktı `B64.encode` ile tek seferde kodlanmışıyla aynıdır.
fn base64_yaz<W: Write>(veri: &[u8], w: &mut W) -> io::Result<()> {
    const PARCA: usize = 3 * 16 * 1024;
    let mut tampon = vec![0u8; PARCA / 3 * 4];
    for parca in veri.chunks(PARCA) {
        let n = B64.encode_slice(parca, &mut tampon).map_err(io::Error::other)?;
        w.write_all(&tampon[..n])?;
    }
    Ok(())
}

fn page_format(doc: &Document) -> String {
    let pf = &doc.pages;
    format!(
        "<properties><pageFormat mediaSizeName=\"{}\" leftMargin=\"{}\" rightMargin=\"{}\" \
         topMargin=\"{}\" bottomMargin=\"{}\" paperOrientation=\"{}\" headerFOffset=\"{}\" \
         footerFOffset=\"{}\" /></properties>",
        pf.media_size_name,
        f2(pf.left_margin),
        f2(pf.right_margin),
        f2(pf.top_margin),
        f2(pf.bottom_margin),
        pf.paper_orientation,
        f1(pf.header_foffset),
        f1(pf.footer_foffset)
    )
}

fn font_attrs(s: &TextStyle) -> String {
    let mut b = String::new();
    b.push_str(&format!(" family=\"{}\"", esc(&s.font_family)));
    b.push_str(&format!(" size=\"{}\"", num_str(s.font_size)));
    if s.bold {
        b.push_str(" bold=\"true\"");
    }
    if s.italic {
        b.push_str(" italic=\"true\"");
    }
    if s.underline {
        b.push_str(" underline=\"true\"");
    }
    if s.color != -16_777_216 {
        b.push_str(&format!(" foreground=\"{}\"", s.color));
    }
    if s.background_color != -1 {
        b.push_str(&format!(" background=\"{}\"", s.background_color));
    }
    b
}

/// XML öznitelik kaçışı. CDATA içindeki metne uygulanmaz.
fn esc(v: &str) -> String {
    v.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn f1(d: f64) -> String {
    format!("{d:.1}")
}

fn f2(d: f64) -> String {
    format!("{d:.2}")
}

/// Tam sayıysa ondalıksız (Java portundaki `numStr` ile aynı): 12 → "12", 8.25 → "8.25".
fn num_str(d: f64) -> String {
    if d.fract() == 0.0 && d.is_finite() {
        format!("{}", d as i64)
    } else {
        format!("{d}")
    }
}

/// Metin çalıştırması içindeki CDATA-güvensiz diziyi kırar. `]]>` CDATA'yı erken kapatır.
pub fn cdata_guvenli(text: &str) -> String {
    text.replace("]]>", "]]&gt;")
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::udf::model::{Alignment, PageFormat, TextRun};

    fn img(veri: &[u8], w: f64, h: f64) -> Run {
        Run::Image(ImageRun {
            data: veri.into(),
            width: w,
            height: h,
        })
    }

    fn para_with(runs: Vec<Run>) -> Block {
        Block::Paragraph(Paragraph {
            alignment: Alignment::Center,
            runs,
            ..Default::default()
        })
    }

    fn cdata_of(xml: &str) -> String {
        let s = xml.find("<![CDATA[").unwrap() + 9;
        let e = xml.find("]]></content>").unwrap();
        xml[s..e].to_string()
    }

    #[test]
    fn tek_resim_cdata_ve_offset() {
        let doc = Document {
            pages: PageFormat::default(),
            body: vec![para_with(vec![img(b"AAA", 524.4, 349.6)])],
        };
        let xml = serialize(&doc);

        assert_eq!(cdata_of(&xml), "\u{FFFC}\n");
        assert!(xml.contains("startOffset=\"0\" length=\"1\""), "{xml}");
        assert!(xml.contains("width=\"524.4\" height=\"349.6\""), "{xml}");
        assert!(xml.contains("<paragraph Alignment=\"1\" LeftIndent=\"0.0\" RightIndent=\"0.0\">"));
    }

    #[test]
    fn uc_resim_alt_alta_offsetler_ikiser_artar() {
        let doc = Document {
            pages: PageFormat::default(),
            body: vec![
                para_with(vec![img(b"A", 100.0, 100.0)]),
                para_with(vec![img(b"B", 100.0, 100.0)]),
                para_with(vec![img(b"C", 100.0, 100.0)]),
            ],
        };
        let xml = serialize(&doc);

        assert_eq!(cdata_of(&xml), "\u{FFFC}\n\u{FFFC}\n\u{FFFC}\n");
        let offsets: Vec<&str> = xml.match_indices("startOffset=\"").map(|(i, _)| {
            let s = i + 13;
            let e = xml[s..].find('"').unwrap() + s;
            &xml[s..e]
        }).collect();
        assert_eq!(offsets, vec!["0", "2", "4"]);
    }

    #[test]
    fn uc_resim_ayri_sayfa_page_break_offsetleri() {
        // Ayrı sayfa: her <page-break> CDATA'ya U+200B + \n (2 kod noktası) ekler,
        // dolayısıyla resim offset'leri 0 / 4 / 8 olur.
        let doc = Document {
            pages: PageFormat::default(),
            body: vec![
                para_with(vec![img(b"A", 100.0, 100.0)]),
                Block::PageBreak,
                para_with(vec![img(b"B", 100.0, 100.0)]),
                Block::PageBreak,
                para_with(vec![img(b"C", 100.0, 100.0)]),
            ],
        };
        let xml = serialize(&doc);

        assert_eq!(
            cdata_of(&xml),
            "\u{FFFC}\n\u{200B}\n\u{FFFC}\n\u{200B}\n\u{FFFC}\n"
        );
        assert_eq!(xml.matches("<page-break>").count(), 2);

        let img_offsets: Vec<&str> = xml
            .match_indices("<image ")
            .map(|(i, _)| {
                let s = xml[i..].find("startOffset=\"").unwrap() + i + 13;
                let e = xml[s..].find('"').unwrap() + s;
                &xml[s..e]
            })
            .collect();
        assert_eq!(img_offsets, vec!["0", "4", "8"]);

        // page-break içindeki boş paragraf, aradaki iki kod noktasını kapsar.
        assert!(xml.contains("<page-break><paragraph Alignment=\"0\" LeftIndent=\"0.0\" RightIndent=\"0.0\"><content startOffset=\"2\" length=\"2\""), "{xml}");
        assert!(xml.contains("<content startOffset=\"6\" length=\"2\""), "{xml}");
    }

    #[test]
    fn metin_ve_resim_karisik_son_metin_newline_kapsar() {
        let doc = Document {
            pages: PageFormat::default(),
            body: vec![Block::Paragraph(Paragraph {
                alignment: Alignment::Left,
                runs: vec![
                    img(b"A", 10.0, 10.0),
                    Run::Text(TextRun {
                        text: "Merhaba".to_string(),
                        style: TextStyle::default(),
                    }),
                ],
                ..Default::default()
            })],
        };
        let xml = serialize(&doc);

        assert_eq!(cdata_of(&xml), "\u{FFFC}Merhaba\n");
        // resim: offset 0, uzunluk 1 | metin: offset 1, uzunluk 7+1 (paragraf sonu dahil)
        assert!(xml.contains("startOffset=\"0\" length=\"1\""), "{xml}");
        assert!(xml.contains("<content startOffset=\"1\" length=\"8\""), "{xml}");
    }

    #[test]
    fn ondalik_ayirici_her_zaman_nokta_ve_tek_hane() {
        assert_eq!(f1(524.41), "524.4");
        assert_eq!(f1(0.0), "0.0");
        assert_eq!(f2(42.52), "42.52");
        assert_eq!(f1(1234.0), "1234.0");
        assert_eq!(num_str(12.0), "12");
        assert_eq!(num_str(8.25), "8.25");
    }

    #[test]
    fn xml_kacisi() {
        assert_eq!(esc("a&b<c>d\"e"), "a&amp;b&lt;c&gt;d&quot;e");
        let doc = Document {
            pages: PageFormat::default(),
            body: vec![Block::Paragraph(Paragraph {
                runs: vec![Run::Text(TextRun {
                    text: "x".to_string(),
                    style: TextStyle {
                        font_family: "Ari<al> & \"Co\"".to_string(),
                        ..Default::default()
                    },
                })],
                ..Default::default()
            })],
        };
        let xml = serialize(&doc);
        assert!(xml.contains("family=\"Ari&lt;al&gt; &amp; &quot;Co&quot;\""), "{xml}");
    }

    #[test]
    fn parca_parca_base64_tek_seferlikle_ayni() {
        // Parça sınırının iki yanı ve 3'e bölünmeyen uzunluklar (dolgu yalnız sonda olmalı).
        for n in [0, 1, 2, 3, 49_151, 49_152, 49_153, 150_001] {
            let veri: Vec<u8> = (0..n).map(|i| (i * 7 + i / 251) as u8).collect();
            let mut yazilan = Vec::new();
            base64_yaz(&veri, &mut yazilan).unwrap();
            assert_eq!(String::from_utf8(yazilan).unwrap(), B64.encode(&veri), "n = {n}");
        }
    }

    #[test]
    fn cdata_kapanisi_kirilir() {
        assert_eq!(cdata_guvenli("a]]>b"), "a]]&gt;b");
    }

    #[test]
    fn bos_paragraf_iki_kod_noktasi() {
        let doc = Document {
            pages: PageFormat::default(),
            body: vec![Block::Paragraph(Paragraph::default())],
        };
        let xml = serialize(&doc);
        assert_eq!(cdata_of(&xml), "\u{200B}\n");
        assert!(xml.contains("<content startOffset=\"0\" length=\"2\" family=\"Times New Roman\" size=\"10\" />"), "{xml}");
    }

    #[test]
    fn iskelet_sabit_kaliyor() {
        // Snapshot: bilinen girdi için content.xml çıktısı sabit.
        let doc = Document {
            pages: PageFormat::default(),
            body: vec![para_with(vec![img(b"ABC", 524.4, 349.6)])],
        };
        let beklenen = concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n",
            "<template format_id=\"1.8\">\n",
            "<content><![CDATA[\u{FFFC}\n]]></content>",
            "<properties><pageFormat mediaSizeName=\"1\" leftMargin=\"42.52\" rightMargin=\"28.35\" ",
            "topMargin=\"14.17\" bottomMargin=\"14.17\" paperOrientation=\"1\" headerFOffset=\"20.0\" ",
            "footerFOffset=\"20.0\" /></properties>\n",
            "<elements resolver=\"hvl-default\">\n",
            "<paragraph Alignment=\"1\" LeftIndent=\"0.0\" RightIndent=\"0.0\">",
            "<image imageData=\"QUJD\" startOffset=\"0\" length=\"1\" width=\"524.4\" height=\"349.6\" />",
            "</paragraph>\n",
            "</elements>\n",
            "<styles><style name=\"default\" description=\"Geçerli\" family=\"Dialog\" size=\"12\" ",
            "bold=\"false\" italic=\"false\" foreground=\"-13421773\" ",
            "FONT_ATTRIBUTE_KEY=\"javax.swing.plaf.FontUIResource[family=Dialog,name=Dialog,style=plain,size=12]\" />",
            "<style name=\"hvl-default\" family=\"Times New Roman\" size=\"12\" description=\"Gövde\" />",
            "</styles>\n",
            "</template>"
        );
        assert_eq!(serialize(&doc), beklenen);
    }
}

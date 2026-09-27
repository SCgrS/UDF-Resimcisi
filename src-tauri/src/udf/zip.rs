//! `content.xml` → `.udf` (tek girişli ZIP arşivi).

use std::io::{self, Seek, Write};

use anyhow::Result;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

/// UDF arşivindeki tek girişin adı. Başka isim UDE tarafından okunmaz.
pub const CONTENT_ENTRY: &str = "content.xml";

/// `hedef`e tek girişli bir ZIP (yani `.udf` baytları) yazar; `content.xml` metnini `icerik`
/// sıkıştırıcıya parça parça verir, arşiv bellekte bütün olarak kurulmaz. Metin UTF-8 olarak,
/// **BOM'suz** yazılır.
pub fn paketle<W, F>(hedef: W, icerik: F) -> Result<W>
where
    W: Write + Seek,
    F: FnOnce(&mut ZipWriter<W>) -> io::Result<()>,
{
    let mut zw = ZipWriter::new(hedef);
    let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    zw.start_file(CONTENT_ENTRY, opts)?;
    icerik(&mut zw)?;
    Ok(zw.finish()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Read};

    fn paketle_metin(xml: &str) -> Vec<u8> {
        paketle(Cursor::new(Vec::new()), |w| w.write_all(xml.as_bytes()))
            .unwrap()
            .into_inner()
    }

    #[test]
    fn tek_giris_ve_adi_content_xml() {
        let bytes = paketle_metin("<merhaba/>");
        let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        assert_eq!(zip.len(), 1, "arşivde tam olarak bir giriş olmalı");

        let mut f = zip.by_index(0).unwrap();
        assert_eq!(f.name(), "content.xml");
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        assert_eq!(s, "<merhaba/>");
    }

    #[test]
    fn utf8_bom_yok() {
        let bytes = paketle_metin("<a>ğüşİÖÇ</a>");
        let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        let mut f = zip.by_index(0).unwrap();
        let mut raw = Vec::new();
        f.read_to_end(&mut raw).unwrap();
        assert_ne!(&raw[0..3], &[0xEF, 0xBB, 0xBF], "BOM yazılmamalı");
        assert_eq!(String::from_utf8(raw).unwrap(), "<a>ğüşİÖÇ</a>");
    }
}

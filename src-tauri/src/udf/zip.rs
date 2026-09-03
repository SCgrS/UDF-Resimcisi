//! `content.xml` → `.udf` (tek girişli ZIP arşivi).

use std::io::{Cursor, Write};

use anyhow::Result;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

/// UDF arşivindeki tek girişin adı. Başka isim UDE tarafından okunmaz.
pub const CONTENT_ENTRY: &str = "content.xml";

/// `content.xml` metnini tek girişli bir ZIP'e (yani `.udf` baytlarına) sarar.
/// Metin UTF-8 olarak, **BOM'suz** yazılır.
pub fn paketle(content_xml: &str) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    {
        let mut zw = ZipWriter::new(Cursor::new(&mut buf));
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        zw.start_file(CONTENT_ENTRY, opts)?;
        zw.write_all(content_xml.as_bytes())?;
        zw.finish()?;
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn tek_giris_ve_adi_content_xml() {
        let bytes = paketle("<merhaba/>").unwrap();
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
        let bytes = paketle("<a>ğüşİÖÇ</a>").unwrap();
        let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        let mut f = zip.by_index(0).unwrap();
        let mut raw = Vec::new();
        f.read_to_end(&mut raw).unwrap();
        assert_ne!(&raw[0..3], &[0xEF, 0xBB, 0xBF], "BOM yazılmamalı");
        assert_eq!(String::from_utf8(raw).unwrap(), "<a>ğüşİÖÇ</a>");
    }
}

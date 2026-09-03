package macospasterich;

import java.awt.BasicStroke;
import java.awt.Color;
import java.awt.Font;
import java.awt.Graphics2D;
import java.awt.RenderingHints;
import java.awt.image.BufferedImage;
import java.io.ByteArrayOutputStream;
import java.io.File;
import java.io.FileOutputStream;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Base64;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;

import javax.imageio.ImageIO;

/**
 * Turnusol testi: referans serializer (UdeXml) ile tek resimli .udf uretir.
 * Kullanim:
 *   ProtoMain gen  png-yolu w h        -- test gorseli uret
 *   ProtoMain udf  png-yolu udf-yolu   -- .udf uret
 *   ProtoMain read udf-yolu            -- icindeki imageData'yi cozup piksel olc
 */
public final class ProtoMain {

    static final double PAGE_W = 595.28, PAGE_H = 841.89;
    static final double L = 42.52, R = 28.35, T = 14.17, B = 14.17, HF = 20.0, FF = 20.0;
    static final double USABLE_W = PAGE_W - L - R;              // 524.41
    static final double USABLE_H = PAGE_H - T - B - HF - FF;    // 773.55

    public static void main(String[] args) throws Exception {
        if (args[0].equals("gen")) {
            genTestImage(args[1], Integer.parseInt(args[2]), Integer.parseInt(args[3]));
        } else if (args[0].equals("udf")) {
            makeUdf(args[1], args[2]);
        } else if (args[0].equals("tojpg")) {
            BufferedImage src = ImageIO.read(new File(args[1]));
            BufferedImage rgb = new BufferedImage(src.getWidth(), src.getHeight(), BufferedImage.TYPE_INT_RGB);
            rgb.getGraphics().drawImage(src, 0, 0, null);
            ImageIO.write(rgb, "JPEG", new File(args[2]));
            System.out.println("tojpg: " + args[2] + " " + new File(args[2]).length() + " bayt");
        } else if (args[0].equals("read")) {
            readUdf(args[1]);
        }
    }

    /** Yeniden ornekleme yapilirsa GORUNUR sekilde bozulan desenler iceren test gorseli. */
    static void genTestImage(String out, int w, int h) throws Exception {
        BufferedImage img = new BufferedImage(w, h, BufferedImage.TYPE_INT_RGB);
        Graphics2D g = img.createGraphics();
        g.setColor(Color.WHITE);
        g.fillRect(0, 0, w, h);
        g.setRenderingHint(RenderingHints.KEY_TEXT_ANTIALIASING, RenderingHints.VALUE_TEXT_ANTIALIAS_ON);
        g.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);

        // 1) 1 piksel dama tahtasi: her turlu yeniden ornekleme bunu gri bulamaca cevirir
        for (int y = 100; y < 500; y++) {
            for (int x = 100; x < 900; x++) {
                if (((x + y) & 1) == 0) img.setRGB(x, y, 0x000000);
            }
        }
        // 2) 1px cizgiler, 2..6 px araliklarla
        int yy = 560;
        for (int gap = 2; gap <= 6; gap++) {
            for (int k = 0; k < 30; k++) {
                int ly = yy + k * gap;
                if (ly >= h) break;
                for (int x = 100; x < 900; x++) img.setRGB(x, ly, 0x000000);
            }
            yy += 30 * gap + 40;
        }
        // 3) Kucuk punto metin (taranmis dilekce benzeri)
        g.setColor(Color.BLACK);
        int ty = 140;
        int[] sizes = {8, 10, 12, 14, 18, 24, 36, 48};
        for (int s : sizes) {
            g.setFont(new Font("Times New Roman", Font.PLAIN, s));
            g.drawString(s + " pt - Turkce: agirsiz sicak ogun, ILGI, ustunluk. 0123456789 iIsSgGuUoOcC", 1000, ty);
            ty += s + 26;
        }
        // 4) Buyuk etiket (ekran goruntusunden okunabilsin)
        g.setFont(new Font("Arial", Font.BOLD, 96));
        g.drawString(w + " x " + h + " px", 1000, ty + 120);
        g.setFont(new Font("Arial", Font.PLAIN, 40));
        g.drawString("UDF Resimcisi - tam cozunurluk testi", 1000, ty + 190);
        // 5) Cerceve + koseler
        g.setStroke(new BasicStroke(4));
        g.drawRect(2, 2, w - 5, h - 5);
        g.setFont(new Font("Arial", Font.BOLD, 60));
        g.drawString("SOL-UST", 30, 80);
        g.drawString("SAG-ALT", w - 340, h - 30);
        g.dispose();
        ImageIO.write(img, "PNG", new File(out));
        System.out.println("gen: " + out + " " + w + "x" + h + " " + new File(out).length() + " bayt");
    }

    static void makeUdf(String pngPath, String udfPath) throws Exception {
        byte[] png = Files.readAllBytes(Paths.get(pngPath));
        BufferedImage bi = ImageIO.read(new File(pngPath));
        int pw = bi.getWidth(), ph = bi.getHeight();

        double k = Math.min(1.0, Math.min(USABLE_W / pw, USABLE_H / ph));

        UdeDoc.Document doc = new UdeDoc.Document();
        UdeDoc.Paragraph p = new UdeDoc.Paragraph();
        p.alignment = UdeDoc.ALIGN_CENTER;
        UdeDoc.ImageRun ir = new UdeDoc.ImageRun();
        ir.data = Base64.getEncoder().encodeToString(png);
        ir.width = pw * k;
        ir.height = ph * k;
        p.runs.add(ir);
        doc.body.add(p);

        String xml = UdeXml.serialize(doc);
        writeUdf(udfPath, xml);
        System.out.println("udf: " + udfPath + " " + new File(udfPath).length() + " bayt"
                + " | bitmap " + pw + "x" + ph
                + " | goruntuleme " + String.format(java.util.Locale.US, "%.1fx%.1f pt", ir.width, ir.height));
    }

    static void writeUdf(String path, String contentXml) throws Exception {
        OutputStream fos = new FileOutputStream(path);
        ZipOutputStream zos = new ZipOutputStream(fos);
        try {
            zos.putNextEntry(new ZipEntry("content.xml"));
            zos.write(contentXml.getBytes(StandardCharsets.UTF_8));
            zos.closeEntry();
        } finally {
            zos.close();
        }
    }

    static void readUdf(String udfPath) throws Exception {
        java.util.zip.ZipFile zf = new java.util.zip.ZipFile(udfPath);
        try {
            java.util.Enumeration<? extends ZipEntry> en = zf.entries();
            while (en.hasMoreElements()) {
                ZipEntry e = en.nextElement();
                System.out.println("giris: " + e.getName() + " (" + e.getSize() + " bayt)");
                if (!e.getName().equals("content.xml")) continue;
                ByteArrayOutputStream bos = new ByteArrayOutputStream();
                byte[] buf = new byte[65536];
                int n;
                java.io.InputStream is = zf.getInputStream(e);
                try {
                    while ((n = is.read(buf)) > 0) bos.write(buf, 0, n);
                } finally {
                    is.close();
                }
                String xml = new String(bos.toByteArray(), StandardCharsets.UTF_8);
                Path dump = Paths.get(udfPath + ".content.xml");
                Files.write(dump, bos.toByteArray());
                System.out.println("content.xml -> " + dump + " (" + xml.length() + " karakter)");
                int idx = 0, img = 0;
                while ((idx = xml.indexOf("imageData=\"", idx)) >= 0) {
                    int s = idx + 11;
                    int t = xml.indexOf('"', s);
                    String b64 = xml.substring(s, t);
                    byte[] raw = Base64.getMimeDecoder().decode(b64);
                    BufferedImage bi = ImageIO.read(new java.io.ByteArrayInputStream(raw));
                    String fmt = sniff(raw);
                    int tagStart = xml.lastIndexOf('<', idx);
                    int tagEnd = xml.indexOf("/>", t);
                    String tag = xml.substring(tagStart, Math.min(tagEnd + 2, xml.length()));
                    String wAttr = attr(tag, "width"), hAttr = attr(tag, "height"), off = attr(tag, "startOffset");
                    System.out.println("  resim#" + img + ": " + (bi == null ? "COZULEMEDI" : bi.getWidth() + "x" + bi.getHeight() + " px")
                            + " | bicim=" + fmt + " | gomulu bayt=" + raw.length
                            + " | width=" + wAttr + " height=" + hAttr + " startOffset=" + off);
                    Files.write(Paths.get(udfPath + ".img" + img + "." + fmt.toLowerCase()), raw);
                    img++;
                    idx = t;
                }
                System.out.println("  toplam resim: " + img);
            }
        } finally {
            zf.close();
        }
    }

    static String attr(String tag, String name) {
        int i = tag.indexOf(name + "=\"");
        if (i < 0) return "?";
        int s = i + name.length() + 2;
        return tag.substring(s, tag.indexOf('"', s));
    }

    static String sniff(byte[] b) {
        if (b.length > 8 && (b[0] & 0xFF) == 0x89 && b[1] == 'P' && b[2] == 'N' && b[3] == 'G') return "PNG";
        if (b.length > 3 && (b[0] & 0xFF) == 0xFF && (b[1] & 0xFF) == 0xD8) return "JPG";
        return "BILINMEYEN";
    }

    private ProtoMain() { }
}

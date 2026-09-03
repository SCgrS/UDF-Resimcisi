package macospasterich;

import java.awt.Color;
import java.awt.Font;
import java.awt.Graphics2D;
import java.awt.RenderingHints;
import java.awt.image.BufferedImage;
import java.io.File;
import java.util.Random;

import javax.imageio.IIOImage;
import javax.imageio.ImageIO;
import javax.imageio.ImageWriteParam;
import javax.imageio.ImageWriter;
import javax.imageio.stream.ImageOutputStream;

/**
 * Olcum icin gercekci buyuk gorseller uretir.
 *   BuyukGorsel foto <cikti.jpg> <w> <h> <kalite>  -- 12 MP telefon fotografi benzeri
 *   BuyukGorsel gurultu <cikti.png> <w> <h>        -- sikismayan PNG (9 MB uyarisi testi)
 */
public final class BuyukGorsel {

    public static void main(String[] args) throws Exception {
        int w = Integer.parseInt(args[2]);
        int h = Integer.parseInt(args[3]);
        BufferedImage img = new BufferedImage(w, h, BufferedImage.TYPE_INT_RGB);
        Random r = new Random(42);

        if (args[0].equals("foto")) {
            // Yumusak gradyan + hafif gurultu: fotograf gibi sikisir.
            for (int y = 0; y < h; y++) {
                for (int x = 0; x < w; x++) {
                    int base = (int) (128 + 100 * Math.sin(x * 0.0016) * Math.cos(y * 0.0021));
                    int n = r.nextInt(24) - 12;
                    int rr = clamp(base + n + 30), gg = clamp(base + n), bb = clamp(base + n - 20);
                    img.setRGB(x, y, (rr << 16) | (gg << 8) | bb);
                }
            }
            Graphics2D g = img.createGraphics();
            g.setRenderingHint(RenderingHints.KEY_TEXT_ANTIALIASING, RenderingHints.VALUE_TEXT_ANTIALIAS_ON);
            g.setColor(Color.BLACK);
            g.setFont(new Font("Times New Roman", Font.PLAIN, 22));
            for (int i = 0; i < 40; i++) {
                g.drawString("22 pt ince metin satiri " + i + " - taranmis dilekce benzeri okunabilirlik olcumu", 200, 300 + i * 46);
            }
            g.setFont(new Font("Arial", Font.BOLD, 140));
            g.drawString(w + " x " + h, 200, h - 300);
            g.dispose();

            float kalite = Float.parseFloat(args[4]);
            ImageWriter wr = ImageIO.getImageWritersByFormatName("jpeg").next();
            ImageWriteParam p = wr.getDefaultWriteParam();
            p.setCompressionMode(ImageWriteParam.MODE_EXPLICIT);
            p.setCompressionQuality(kalite);
            try (ImageOutputStream os = ImageIO.createImageOutputStream(new File(args[1]))) {
                wr.setOutput(os);
                wr.write(null, new IIOImage(img, null, null), p);
            }
            wr.dispose();
        } else {
            // Saf gurultu: PNG neredeyse hic sikismaz.
            for (int y = 0; y < h; y++) {
                for (int x = 0; x < w; x++) {
                    img.setRGB(x, y, r.nextInt(0xFFFFFF));
                }
            }
            ImageIO.write(img, "PNG", new File(args[1]));
        }

        File f = new File(args[1]);
        System.out.println(args[0] + ": " + f.getName() + " " + w + "x" + h + " "
                + f.length() + " bayt (" + String.format(java.util.Locale.US, "%.1f", f.length() / 1048576.0) + " MB)");
    }

    static int clamp(int v) {
        return v < 0 ? 0 : (v > 255 ? 255 : v);
    }

    private BuyukGorsel() { }
}

package macospasterich;

import java.awt.BasicStroke;
import java.awt.Color;
import java.awt.Font;
import java.awt.GradientPaint;
import java.awt.Graphics2D;
import java.awt.RenderingHints;
import java.awt.geom.Ellipse2D;
import java.awt.geom.GeneralPath;
import java.awt.geom.RoundRectangle2D;
import java.awt.image.BufferedImage;
import java.io.File;

import javax.imageio.ImageIO;

/** Uygulama simgesi: belge sayfasi + uzerinde fotograf. 32 px'te de okunur kalmali. */
public final class IconGen {

    public static void main(String[] args) throws Exception {
        int S = 1024;
        BufferedImage img = new BufferedImage(S, S, BufferedImage.TYPE_INT_ARGB);
        Graphics2D g = img.createGraphics();
        g.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);
        g.setRenderingHint(RenderingHints.KEY_STROKE_CONTROL, RenderingHints.VALUE_STROKE_PURE);

        // Zemin: koyu lacivert yuvarlak kare
        g.setPaint(new GradientPaint(0, 0, new Color(0x1B3A5C), 0, S, new Color(0x0E2338)));
        g.fill(new RoundRectangle2D.Float(0, 0, S, S, 220, 220));

        // Belge sayfasi
        int px = 200, py = 150, pw = 624, ph = 724;
        g.setColor(Color.WHITE);
        g.fill(new RoundRectangle2D.Float(px, py, pw, ph, 28, 28));

        // Kivrik kose
        GeneralPath kose = new GeneralPath();
        kose.moveTo(px + pw - 150, py);
        kose.lineTo(px + pw, py + 150);
        kose.lineTo(px + pw - 150, py + 150);
        kose.closePath();
        g.setColor(new Color(0xD7DEE7));
        g.fill(kose);

        // Fotograf cercevesi
        int fx = px + 70, fy = py + 230, fw = pw - 140, fh = 262;
        g.setColor(new Color(0x2E7BC4));
        g.fillRect(fx, fy, fw, fh);

        // Gunes
        g.setColor(new Color(0xFFD166));
        g.fill(new Ellipse2D.Float(fx + fw - 130, fy + 50, 78, 78));

        // Daglar
        GeneralPath dag = new GeneralPath();
        dag.moveTo(fx, fy + fh);
        dag.lineTo(fx + 150, fy + 120);
        dag.lineTo(fx + 265, fy + fh);
        dag.closePath();
        g.setColor(new Color(0x1F5E36));
        g.fill(dag);
        GeneralPath dag2 = new GeneralPath();
        dag2.moveTo(fx + 170, fy + fh);
        dag2.lineTo(fx + 320, fy + 60);
        dag2.lineTo(fx + fw, fy + fh);
        dag2.closePath();
        g.setColor(new Color(0x2A7C47));
        g.fill(dag2);

        // Metin satirlari (belge oldugu belli olsun)
        g.setColor(new Color(0x8FA3B8));
        for (int i = 0; i < 3; i++) {
            g.fillRoundRect(px + 70, py + 90 + i * 44, (i == 2 ? 300 : fw), 22, 11, 11);
        }

        // "UDF" rozeti
        g.setColor(new Color(0xC0392B));
        g.fillRoundRect(px + 70, py + ph - 175, 330, 118, 24, 24);
        g.setColor(Color.WHITE);
        g.setFont(new Font("Arial", Font.BOLD, 92));
        g.drawString("UDF", px + 100, py + ph - 90);

        // Ince cerceve
        g.setColor(new Color(0x0A1A2A));
        g.setStroke(new BasicStroke(6));
        g.draw(new RoundRectangle2D.Float(3, 3, S - 6, S - 6, 218, 218));

        g.dispose();
        File out = new File(args[0]);
        ImageIO.write(img, "PNG", out);
        System.out.println("ikon: " + out.getAbsolutePath() + " " + out.length() + " bayt");
    }

    private IconGen() { }
}

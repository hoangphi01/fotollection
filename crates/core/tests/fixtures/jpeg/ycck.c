// SPDX-License-Identifier: GPL-3.0-only
// Build: cc -I$(brew --prefix jpeg-turbo)/include ycck.c -L$(brew --prefix jpeg-turbo)/lib -ljpeg -o ycck
// Run:   ./ycck source.ppm ycck.jpg   (source.png converted to PPM)
/* Writes an Adobe YCCK JPEG (APP14 transform 2) from a binary PPM, as Photoshop stores
   CMYK: inverted values (255 = no ink). A fixture tool, not part of the app. */
#include <stdio.h>
#include <stdlib.h>
#include <jpeglib.h>
int main(int argc, char **argv) {
  if (argc != 3) return 2;
  FILE *in = fopen(argv[1], "rb"), *out = fopen(argv[2], "wb");
  int w, h, max;
  if (!in || !out || fscanf(in, "P6 %d %d %d", &w, &h, &max) != 3) return 1;
  fgetc(in);
  unsigned char *rgb = malloc((size_t)w * h * 3), *row = malloc((size_t)w * 4);
  if (fread(rgb, 3, (size_t)w * h, in) != (size_t)w * h) return 1;
  struct jpeg_compress_struct c; struct jpeg_error_mgr e;
  c.err = jpeg_std_error(&e); jpeg_create_compress(&c); jpeg_stdio_dest(&c, out);
  c.image_width = w; c.image_height = h; c.input_components = 4; c.in_color_space = JCS_CMYK;
  jpeg_set_defaults(&c); jpeg_set_colorspace(&c, JCS_YCCK); jpeg_set_quality(&c, 90, TRUE);
  jpeg_start_compress(&c, TRUE);
  while (c.next_scanline < (unsigned)h) {
    unsigned char *p = rgb + (size_t)c.next_scanline * w * 3;
    for (int x = 0; x < w; x++) {
      /* Ink c = 255 - r etc. with no black; stored inverted: r, g, b, 255. */
      row[4*x] = p[3*x]; row[4*x+1] = p[3*x+1]; row[4*x+2] = p[3*x+2]; row[4*x+3] = 255;
    }
    JSAMPROW r = row; jpeg_write_scanlines(&c, &r, 1);
  }
  jpeg_finish_compress(&c); fclose(out); return 0;
}

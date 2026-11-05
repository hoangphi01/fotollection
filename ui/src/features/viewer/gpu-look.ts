// SPDX-License-Identifier: GPL-3.0-only

/** Grid points a side of Rust's look table (`looks::GRID`). */
const GRID = 33

/** A GPU step slower than this loses to Rust's path, about 10 ms a step (RESULT §22). */
const PROBE_LIMIT_MS = 8

/** Whether the GPU is worth it here, once known. */
let verdict: boolean | null = null

const VERTEX = `#version 300 es
in vec2 p;
out vec2 uv;
void main() {
  uv = vec2(p.x * 0.5 + 0.5, 0.5 - p.y * 0.5);
  gl_Position = vec4(p, 0.0, 1.0);
}`

// Tetrahedral interpolation of Rust's table, by texelFetch only: no hardware filtering, the
// same tetrahedra as looks.rs. The mask keeps the stamp, the frame and the border untouched.
const FRAGMENT = `#version 300 es
precision highp float;
precision highp sampler3D;
uniform sampler2D photo;
uniform sampler2D mask;
uniform sampler3D lut;
in vec2 uv;
out vec4 colour;
vec3 at(ivec3 q) { return texelFetch(lut, q, 0).rgb; }
void main() {
  vec3 c = texelFetch(photo, ivec2(uv * vec2(textureSize(photo, 0))), 0).rgb;
  float m = texelFetch(mask, ivec2(uv * vec2(textureSize(mask, 0))), 0).r;
  vec3 g = c * 32.0;
  ivec3 b = ivec3(min(floor(g), vec3(31.0)));
  vec3 f = g - vec3(b);
  ivec3 s1; ivec3 s2; vec3 w;
  if (f.r >= f.g) {
    if (f.g >= f.b) { s1 = ivec3(1, 0, 0); s2 = ivec3(1, 1, 0); w = f.rgb; }
    else if (f.r >= f.b) { s1 = ivec3(1, 0, 0); s2 = ivec3(1, 0, 1); w = f.rbg; }
    else { s1 = ivec3(0, 0, 1); s2 = ivec3(1, 0, 1); w = f.brg; }
  } else if (f.b > f.g) { s1 = ivec3(0, 0, 1); s2 = ivec3(0, 1, 1); w = f.bgr; }
  else if (f.b > f.r) { s1 = ivec3(0, 1, 0); s2 = ivec3(0, 1, 1); w = f.gbr; }
  else { s1 = ivec3(0, 1, 0); s2 = ivec3(1, 1, 0); w = f.grb; }
  vec3 c0 = at(b), c1 = at(b + s1), c2 = at(b + s2), c3 = at(b + ivec3(1));
  vec3 looked = c0 + w.x * (c1 - c0) + w.y * (c2 - c1) + w.z * (c3 - c2);
  colour = vec4(mix(c, looked, m), 1.0);
}`

/**
 * The colour look drawn by the webview's GPU while a slider is dragged (ADR-0039): the
 * plain preview and its mask are uploaded once, then each step only uploads Rust's 33³
 * table and draws, about 1–2 ms instead of a JPEG from Rust (RESULT §22). Rust's render
 * replaces it when the drag ends, so what settles is what prints.
 */
export class GpuLook {
  private gl: WebGL2RenderingContext
  private lut: WebGLTexture

  private constructor(gl: WebGL2RenderingContext, lut: WebGLTexture) {
    this.gl = gl
    this.lut = lut
  }

  /** `null` when the webview has no WebGL2, or a shader does not build: the CPU path stays. */
  static create(canvas: HTMLCanvasElement): GpuLook | null {
    const gl = canvas.getContext('webgl2', { premultipliedAlpha: false, preserveDrawingBuffer: true })
    if (!gl) return null
    const shader = (kind: number, source: string) => {
      const s = gl.createShader(kind)
      if (!s) return null
      gl.shaderSource(s, source)
      gl.compileShader(s)
      return gl.getShaderParameter(s, gl.COMPILE_STATUS) ? s : null
    }
    const vs = shader(gl.VERTEX_SHADER, VERTEX)
    const fs = shader(gl.FRAGMENT_SHADER, FRAGMENT)
    const program = gl.createProgram()
    if (!vs || !fs || !program) return null
    gl.attachShader(program, vs)
    gl.attachShader(program, fs)
    gl.linkProgram(program)
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) return null
    gl.useProgram(program)
    const quad = gl.createBuffer()
    gl.bindBuffer(gl.ARRAY_BUFFER, quad)
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]), gl.STATIC_DRAW)
    const p = gl.getAttribLocation(program, 'p')
    gl.enableVertexAttribArray(p)
    gl.vertexAttribPointer(p, 2, gl.FLOAT, false, 0, 0)
    gl.uniform1i(gl.getUniformLocation(program, 'photo'), 0)
    gl.uniform1i(gl.getUniformLocation(program, 'mask'), 1)
    gl.uniform1i(gl.getUniformLocation(program, 'lut'), 2)
    const lut = gl.createTexture()
    if (!lut) return null
    return new GpuLook(gl, lut)
  }

  /**
   * Whether this machine's GPU is worth it, decided once, without asking the user
   * (ADR-0039, owner 2026-11-05: no switch). Not when the renderer is a software one
   * (SwiftShader, llvmpipe, WARP), and not when a test step at the preview's size is slower
   * than Rust's path (about 10 ms): a weak GPU keeps the CPU.
   */
  static worthIt(): boolean {
    if (verdict !== null) return verdict
    verdict = false
    try {
      const canvas = document.createElement('canvas')
      const look = GpuLook.create(canvas)
      const gl = canvas.getContext('webgl2')
      if (!look || !gl) return verdict
      const info = gl.getExtension('WEBGL_debug_renderer_info')
      const renderer = String(gl.getParameter(info ? info.UNMASKED_RENDERER_WEBGL : gl.RENDERER))
      if (/swiftshader|llvmpipe|softpipe|software|warp|basic render/i.test(renderer)) return verdict
      const size = { width: 1600, height: 1067 }
      const blank = new ImageData(size.width, size.height)
      look.setPhotoData(blank, blank)
      const table = new Uint8Array(GRID * GRID * GRID * 4)
      const pixel = new Uint8Array(4)
      const times: number[] = []
      for (let i = 0; i < 4; i++) {
        const start = performance.now()
        look.draw(table)
        gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixel)
        times.push(performance.now() - start)
      }
      // The first step builds the pipeline; the others are what a drag costs.
      const step = Math.max(...times.slice(1))
      verdict = step < PROBE_LIMIT_MS
      gl.getExtension('WEBGL_lose_context')?.loseContext()
    } catch {
      verdict = false
    }
    return verdict
  }

  private texture(unit: number, target: number): WebGLTexture | null {
    const gl = this.gl
    const t = gl.createTexture()
    gl.activeTexture(gl.TEXTURE0 + unit)
    gl.bindTexture(target, t)
    gl.texParameteri(target, gl.TEXTURE_MIN_FILTER, gl.NEAREST)
    gl.texParameteri(target, gl.TEXTURE_MAG_FILTER, gl.NEAREST)
    gl.texParameteri(target, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE)
    gl.texParameteri(target, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE)
    return t
  }

  /** The plain preview (no look) and where it shows the photo; once per photo and drag. */
  setPhoto(photo: ImageBitmap, mask: ImageBitmap) {
    const gl = this.gl
    gl.canvas.width = photo.width
    gl.canvas.height = photo.height
    gl.viewport(0, 0, photo.width, photo.height)
    gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, false)
    this.texture(0, gl.TEXTURE_2D)
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, gl.RGBA, gl.UNSIGNED_BYTE, photo)
    this.texture(1, gl.TEXTURE_2D)
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, gl.RGBA, gl.UNSIGNED_BYTE, mask)
    gl.activeTexture(gl.TEXTURE2)
    gl.bindTexture(gl.TEXTURE_3D, this.lut)
    gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_MIN_FILTER, gl.NEAREST)
    gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_MAG_FILTER, gl.NEAREST)
  }

  /** As `setPhoto`, from pixels already in memory (the probe's blank image). */
  setPhotoData(photo: ImageData, mask: ImageData) {
    const gl = this.gl
    gl.canvas.width = photo.width
    gl.canvas.height = photo.height
    gl.viewport(0, 0, photo.width, photo.height)
    this.texture(0, gl.TEXTURE_2D)
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, gl.RGBA, gl.UNSIGNED_BYTE, photo)
    this.texture(1, gl.TEXTURE_2D)
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, gl.RGBA, gl.UNSIGNED_BYTE, mask)
  }

  /** Draws the photo through `table`, Rust's RGBA8 table; false when it is not one. */
  draw(table: Uint8Array): boolean {
    const gl = this.gl
    if (table.length !== GRID * GRID * GRID * 4 || gl.isContextLost()) return false
    gl.activeTexture(gl.TEXTURE2)
    gl.bindTexture(gl.TEXTURE_3D, this.lut)
    gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA8, GRID, GRID, GRID, 0, gl.RGBA, gl.UNSIGNED_BYTE, table)
    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4)
    return gl.getError() === gl.NO_ERROR
  }
}

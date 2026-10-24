// SPDX-License-Identifier: GPL-3.0-only
// Renders a bundled brand logo SVG to a transparent PNG (ADR-0020). macOS only, dev time.
//   swift scripts/brands/render.swift <in.svg> <out.png> <height px>
import AppKit

let args = CommandLine.arguments
guard args.count == 4, let height = Double(args[3]), let image = NSImage(contentsOfFile: args[1]),
      image.size.height > 0 else {
    FileHandle.standardError.write("usage: render.swift <in.svg> <out.png> <height>\n".data(using: .utf8)!)
    exit(1)
}
let width = (image.size.width * height / image.size.height).rounded()
guard let rep = NSBitmapImageRep(
    bitmapDataPlanes: nil, pixelsWide: Int(width), pixelsHigh: Int(height), bitsPerSample: 8,
    samplesPerPixel: 4, hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB,
    bytesPerRow: 0, bitsPerPixel: 0) else { exit(1) }
rep.size = NSSize(width: width, height: height)
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
NSColor.clear.set()
NSRect(x: 0, y: 0, width: width, height: height).fill()
image.draw(in: NSRect(x: 0, y: 0, width: width, height: height))
NSGraphicsContext.restoreGraphicsState()
guard let png = rep.representation(using: .png, properties: [:]) else { exit(1) }
try png.write(to: URL(fileURLWithPath: args[2]))

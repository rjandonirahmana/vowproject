// scripts/gerak/img2apng.swift — rangkaian PNG transparan → APNG berulang
// (gambar bergerak ber-alfa; didukung semua browser modern termasuk Safari).
//   swift scripts/gerak/img2apng.swift <folder-frame-png> <keluaran.png> <fps>
import Foundation
import ImageIO
import UniformTypeIdentifiers

let a = CommandLine.arguments
guard a.count >= 4, let fps = Double(a[3]) else { print("pakai: img2apng <folder> <out.png> <fps>"); exit(1) }
let dir = URL(fileURLWithPath: a[1])
let files = try FileManager.default.contentsOfDirectory(atPath: dir.path).filter { $0.hasSuffix(".png") }.sorted()
guard let dst = CGImageDestinationCreateWithURL(URL(fileURLWithPath: a[2]) as CFURL, UTType.png.identifier as CFString, files.count, nil) else { exit(1) }
CGImageDestinationSetProperties(dst, [kCGImagePropertyPNGDictionary: [kCGImagePropertyAPNGLoopCount: 0]] as CFDictionary)
for f in files {
    guard let src = CGImageSourceCreateWithURL(dir.appendingPathComponent(f) as CFURL, nil),
          let img = CGImageSourceCreateImageAtIndex(src, 0, nil) else { continue }
    CGImageDestinationAddImage(dst, img, [kCGImagePropertyPNGDictionary: [kCGImagePropertyAPNGDelayTime: 1.0 / fps]] as CFDictionary)
}
print(CGImageDestinationFinalize(dst) ? "ok \(files.count) frame → \(a[2])" : "gagal")

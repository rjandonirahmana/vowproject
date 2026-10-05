// scripts/gerak/img2mp4.swift — rangkaian JPEG → MP4 H.264 (diputar di semua
// browser termasuk iPhone). Dipakai untuk video pembuka & video latar tema.
//
//   swift scripts/gerak/img2mp4.swift <folder-frame> <keluaran.mp4> <fps> [bitrate]
//
// Frame diurutkan menurut nama (0000.jpg, 0001.jpg, …); ukuran video = ukuran
// frame pertama.
import AVFoundation
import AppKit

let args = CommandLine.arguments
guard args.count >= 4, let fps = Int32(args[3]) else {
    print("pakai: img2mp4 <folder> <out.mp4> <fps> [bitrate]"); exit(1)
}
let dir = URL(fileURLWithPath: args[1]), out = URL(fileURLWithPath: args[2])
let bitrate = args.count > 4 ? Int(args[4]) ?? 2_500_000 : 2_500_000
let files = try FileManager.default.contentsOfDirectory(atPath: dir.path).filter { $0.hasSuffix(".jpg") }.sorted()
guard let first = NSImage(contentsOf: dir.appendingPathComponent(files[0])),
      let cg0 = first.cgImage(forProposedRect: nil, context: nil, hints: nil) else { print("frame pertama tak terbaca"); exit(1) }
let w = cg0.width, h = cg0.height
try? FileManager.default.removeItem(at: out)
let writer = try AVAssetWriter(outputURL: out, fileType: .mp4)
let input = AVAssetWriterInput(mediaType: .video, outputSettings: [
    AVVideoCodecKey: AVVideoCodecType.h264, AVVideoWidthKey: w, AVVideoHeightKey: h,
    AVVideoCompressionPropertiesKey: [AVVideoAverageBitRateKey: bitrate, AVVideoProfileLevelKey: AVVideoProfileLevelH264HighAutoLevel,
                                      AVVideoMaxKeyFrameIntervalKey: Int(fps)],
])
input.expectsMediaDataInRealTime = false
let adaptor = AVAssetWriterInputPixelBufferAdaptor(assetWriterInput: input, sourcePixelBufferAttributes: [
    kCVPixelBufferPixelFormatTypeKey as String: kCVPixelFormatType_32ARGB, kCVPixelBufferWidthKey as String: w, kCVPixelBufferHeightKey as String: h,
])
writer.add(input)
writer.shouldOptimizeForNetworkUse = true   // moov di depan → bisa diputar sambil diunduh
writer.startWriting(); writer.startSession(atSourceTime: .zero)
for (i, f) in files.enumerated() {
    guard let img = NSImage(contentsOf: dir.appendingPathComponent(f)),
          let cg = img.cgImage(forProposedRect: nil, context: nil, hints: nil) else { continue }
    while !input.isReadyForMoreMediaData { usleep(2000) }
    var pb: CVPixelBuffer?
    CVPixelBufferPoolCreatePixelBuffer(nil, adaptor.pixelBufferPool!, &pb)
    guard let buf = pb else { continue }
    CVPixelBufferLockBaseAddress(buf, [])
    let ctx = CGContext(data: CVPixelBufferGetBaseAddress(buf), width: w, height: h, bitsPerComponent: 8,
                        bytesPerRow: CVPixelBufferGetBytesPerRow(buf), space: CGColorSpaceCreateDeviceRGB(),
                        bitmapInfo: CGImageAlphaInfo.noneSkipFirst.rawValue)!
    ctx.draw(cg, in: CGRect(x: 0, y: 0, width: w, height: h))
    CVPixelBufferUnlockBaseAddress(buf, [])
    adaptor.append(buf, withPresentationTime: CMTime(value: CMTimeValue(i), timescale: fps))
}
input.markAsFinished()
let sem = DispatchSemaphore(value: 0)
writer.finishWriting { sem.signal() }
sem.wait()
print(writer.status == .completed ? "ok \(files.count) frame \(w)x\(h) → \(out.path)" : "gagal: \(String(describing: writer.error))")

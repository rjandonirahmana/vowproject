# Storyboard beranda 3D (gulir 0→100% dari panggung setinggi 420svh)
Tujuan satu-satunya: pengunjung paham produk, lalu membuka demo tema di katalog.

| Gulir | Yang terlihat | Aksi pengunjung |
|---|---|---|
| 0–12% | Tema 1 (Pendopo & Candi Bentar) utuh, judul + "Lihat tema" | "Lewati cerita ↓" selalu ada |
| 12–22% | Kamera maju: lapisan depan membesar & memudar, bangunan utama membesar (paralaks 3 kecepatan) | – |
| 22–38% | Cross-fade ke tema 2 (Gerbang Bulan) saat kamera "menembus" gerbang | – |
| 38–52% | Tema 2 tampil, tombol "Lihat tema" | klik → `/?q=tionghoa#katalog` |
| 52–68% | Maju lagi → cross-fade ke tema 3 (Masjid Putih) | – |
| 68–84% | Tema 3 | klik → `/?q=masjid#katalog` |
| 84–100% | Tema 3 bertahan, panggung berakhir → katalog sungguhan di bawahnya | gulir ke katalog |
Interaksi tambahan (mouse saja): miring 3D mengikuti pointer, lapisan beda kedalaman.
Tanpa scroll-timeline / gerak dikurangi: satu layar diam + daftar tautan tema (tanpa JS).

## Anggaran (diukur → harus lulus di perangkat nyata)
JS tambahan ≤ 1 KB (sekarang ~0,6 KB) · aset adegan ≤ 100 KB (prototipe: 75 KB total HTML) · animasi hanya transform/opacity · LCP ≤ 2,5 dtk · CLS ≤ 0,1 · INP ≤ 200 ms (Android menengah, 4G lambat).

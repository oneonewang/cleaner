// 用 Node.js 生成简单的占位图标 (32x32, 128x128, 128x128@2x=256x256, icon.ico)
const fs = require('fs');
const path = require('path');
const zlib = require('zlib');

// __dirname = D:\cargo\oneonecleaner\scripts
// 目标 = D:\cargo\oneonecleaner\src-tauri\icons
const PROJECT_ROOT = path.resolve(__dirname, '..');
const OUT_DIR = path.join(PROJECT_ROOT, 'src-tauri', 'icons');
fs.mkdirSync(OUT_DIR, { recursive: true });

/** 计算 PNG 的 CRC32 */
const CRC_TABLE = (() => {
  const t = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = (c & 1) ? (0xEDB88320 ^ (c >>> 1)) : (c >>> 1);
    t[n] = c;
  }
  return t;
})();

function crc32(buf) {
  let c = 0xFFFFFFFF;
  for (let i = 0; i < buf.length; i++) c = CRC_TABLE[(c ^ buf[i]) & 0xFF] ^ (c >>> 8);
  return (c ^ 0xFFFFFFFF) >>> 0;
}

function makeChunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const typeBuf = Buffer.from(type, 'ascii');
  const crcBuf = Buffer.alloc(4);
  const crcInput = Buffer.concat([typeBuf, data]);
  crcBuf.writeUInt32BE(crc32(crcInput), 0);
  return Buffer.concat([len, typeBuf, data, crcBuf]);
}

function makePng(width, height, color = [43, 127, 255, 255]) {
  // 渐变填充 + 中心画一个圆形 logo
  const pixels = Buffer.alloc(width * height * 4);
  const cx = width / 2, cy = height / 2;
  const r = Math.min(width, height) * 0.4;
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const i = (y * width + x) * 4;
      const dx = x - cx, dy = y - cy;
      const dist = Math.sqrt(dx*dx + dy*dy);
      if (dist < r) {
        // 圆形:白色
        pixels[i] = 255; pixels[i+1] = 255; pixels[i+2] = 255; pixels[i+3] = 255;
      } else {
        // 背景:品牌蓝
        pixels[i] = color[0]; pixels[i+1] = color[1]; pixels[i+2] = color[2]; pixels[i+3] = color[3];
      }
    }
  }
  // 中心画一个刷子形状(简化)
  if (width >= 32) {
    const innerR = r * 0.4;
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const i = (y * width + x) * 4;
        const dx = x - cx, dy = y - cy;
        const dist = Math.sqrt(dx*dx + dy*dy);
        if (dist < innerR) {
          pixels[i] = 43; pixels[i+1] = 127; pixels[i+2] = 255; pixels[i+3] = 255;
        }
      }
    }
  }

  // PNG filter byte 0 放在每行前
  const rawData = Buffer.alloc(height * (1 + width * 4));
  for (let y = 0; y < height; y++) {
    rawData[y * (1 + width * 4)] = 0; // filter: None
    pixels.copy(rawData, y * (1 + width * 4) + 1, y * width * 4, (y + 1) * width * 4);
  }
  const compressed = zlib.deflateSync(rawData);

  const signature = Buffer.from([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8;  // bit depth
  ihdr[9] = 6;  // color type: RGBA
  ihdr[10] = 0; // compression
  ihdr[11] = 0; // filter
  ihdr[12] = 0; // interlace

  return Buffer.concat([
    signature,
    makeChunk('IHDR', ihdr),
    makeChunk('IDAT', compressed),
    makeChunk('IEND', Buffer.alloc(0)),
  ]);
}

function makeIco(pngBuffer, size) {
  // ICONDIR(6 bytes) + ICONDIRENTRY(16 bytes) + image data
  const dir = Buffer.alloc(6);
  dir.writeUInt16LE(0, 0); // reserved
  dir.writeUInt16LE(1, 2); // type: 1=icon
  dir.writeUInt16LE(1, 4); // count

  const entry = Buffer.alloc(16);
  entry[0] = size >= 256 ? 0 : size; // width
  entry[1] = size >= 256 ? 0 : size; // height
  entry[2] = 0; // color count
  entry[3] = 0; // reserved
  entry.writeUInt16LE(1, 4);  // planes
  entry.writeUInt16LE(32, 6); // bit count
  entry.writeUInt32LE(pngBuffer.length, 8); // size
  entry.writeUInt32LE(6 + 16, 12); // offset

  return Buffer.concat([dir, entry, pngBuffer]);
}

const sizes = [
  { name: '32x32.png', size: 32 },
  { name: '128x128.png', size: 128 },
  { name: '128x128@2x.png', size: 256 },
  { name: 'icon.png', size: 512 },
];

for (const { name, size } of sizes) {
  const png = makePng(size, size);
  fs.writeFileSync(path.join(OUT_DIR, name), png);
  console.log(`Wrote ${name} (${size}x${size}, ${png.length} bytes)`);
}

// .ico(用 256x256 PNG 嵌入)
const icoSrc = makePng(256, 256);
const ico = makeIco(icoSrc, 256);
fs.writeFileSync(path.join(OUT_DIR, 'icon.ico'), ico);
console.log(`Wrote icon.ico (${ico.length} bytes)`);

// 给 Windows 用的 Square* Logo 资源(tauri-cli 在某些场景会引用)
fs.writeFileSync(path.join(OUT_DIR, 'Square30x30Logo.png'), makePng(30, 30));
fs.writeFileSync(path.join(OUT_DIR, 'Square44x44Logo.png'), makePng(44, 44));
fs.writeFileSync(path.join(OUT_DIR, 'Square71x71Logo.png'), makePng(71, 71));
fs.writeFileSync(path.join(OUT_DIR, 'Square89x89Logo.png'), makePng(89, 89));
fs.writeFileSync(path.join(OUT_DIR, 'Square107x107Logo.png'), makePng(107, 107));
fs.writeFileSync(path.join(OUT_DIR, 'Square142x142Logo.png'), makePng(142, 142));
fs.writeFileSync(path.join(OUT_DIR, 'Square150x150Logo.png'), makePng(150, 150));
fs.writeFileSync(path.join(OUT_DIR, 'Square284x284Logo.png'), makePng(284, 284));
fs.writeFileSync(path.join(OUT_DIR, 'Square310x310Logo.png'), makePng(310, 310));
fs.writeFileSync(path.join(OUT_DIR, 'StoreLogo.png'), makePng(50, 50));
console.log('Wrote Windows store logos');

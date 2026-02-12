import { mkdir, readdir, writeFile } from "node:fs/promises";
import path from "node:path";
import sharp from "sharp";

const root = process.cwd();
const svgDir = path.join(root, "assets", "menu-icons", "svg");
const outDir = path.join(root, "src-tauri", "icons", "menu");

await mkdir(outDir, { recursive: true });

const entries = await readdir(svgDir);
const svgFiles = entries.filter((file) => file.endsWith(".svg"));

const size = 16;

await Promise.all(
  svgFiles.map(async (file) => {
    const input = path.join(svgDir, file);
    const output = path.join(outDir, file.replace(".svg", ".png"));
    const png = await sharp(input, { density: 96 })
      .resize(size, size, { fit: "contain" })
      .png()
      .toBuffer();
    await writeFile(output, png);
  })
);

console.log(`Rendered ${svgFiles.length} menu icons to ${outDir}`);

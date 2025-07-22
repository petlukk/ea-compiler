Projektmål
🧩 En terminalapplikation där användaren laddar en bild (t.ex. PNG eller PGM), kör olika filter (blur, sharpen, edge, grayscale), och sparar resultatet – allt med SIMD-acceleration i Eä.

🧱 Stegvis utvecklingsplan
🔹 Steg 1: Minimal MVP (1–3 dagar)
🎯 Mål:
Läs en enkel bildfil (t.ex. PGM)

Kör ett enkelt filter (grayscale eller brightness adjust)

Skriv tillbaka bilden

✅ Uppgifter:
Lägg till read_pgm(path: string) -> Vec<u8>

Lägg till write_pgm(path: string, data: Vec<u8>)

Implementera adjust_brightness(vec: u8x16, amount: u8x16) -> u8x16

📁 Filformat:
Använd PGM (portable graymap) – extremt enkelt textformat
Exempel:

nginx
Kopiera
Redigera
P2
4 4
255
0 50 100 150
200 220 240 255
🔹 Steg 2: SIMD-förbättrade filter (2–5 dagar)
🎯 Mål:
Implementera minst 3 SIMD-accelererade filter:

Gaussian blur (3x3 kernel)

Sobel edge detection

Sharpen

✅ Uppgifter:
Skapa apply_kernel(image: Vec<u8>, kernel: f32[3][3])

Implementera linje-baserad SIMD-bearbetning (rullande rader)

Bygg fallback-sökväg för äldre CPU:er (utan AVX2)

🧠 SIMD-strategi:
Ladda 16 pixlar i bredd

Kör 3 SIMD-multiplikationer + summering för convolutions

Justera ramar med padding eller clipping

🔹 Steg 3: CLI-gränssnitt (1–2 dagar)
🎯 Mål:
Användaren ska kunna köra:

bash
Kopiera
Redigera
ea-imagefilter --input lena.pgm --output blur.pgm --filter blur
✅ Uppgifter:
CLI-parser i Eä (args.get("--filter"))

Visuell statistik: tid, SIMD-användning

Kodstruktur:

css
Kopiera
Redigera
src/
filters.ea
image_io.ea
main.ea
🔹 Steg 4: Benchmark + profiler (2 dagar)
🎯 Mål:
Visa tydlig skillnad mellan SIMD och fallback

Profilera tid per filter

Skriv ut antal instruktioner eller tid

✅ Uppgifter:
Lägg till --benchmark-flagga

Mät tid i mikrosekunder före/efter filter

Visa “SIMD used: AVX2” eller fallback

🔹 Steg 5: Showcase-optimering (frivilligt)
🧨 Bonus:
Lägg till stöd för PPM (RGB)

Visa diff mellan original och filtrerat

Generera histogram

🧪 Exempelfunktioner i Eä (pseudokod)
ea
Kopiera
Redigera
func adjust_brightness(pixels: u8x16, offset: u8x16) -> u8x16 {
return pixels .+ offset;
}

func apply_sobel(pixels: [u8]) -> [u8] {
// SIMD-baserad kantdetektering här
}
🚀 Leveransmål
Leveransdel Statusmål
Fungerande ea-imagefilter CLI ✅
3 SIMD-filter ✅
Benchmark-logik ✅
README med exempelbilder och tider ✅
GitHub repo med #madeWithEa ✅

💡 Varför detta är rätt demo
Eä-styrka Syns tydligt i projektet
SIMD Ja – filter kör parallellt
JIT / CLI‑exekvering Ja – snabbt testbart
Native binärer Ja – 16 KB output
File I/O Ja – bildfiler
Profilering Ja – med --benchmark
Kodförståelse för publiken Ja – filtren är intuitiva

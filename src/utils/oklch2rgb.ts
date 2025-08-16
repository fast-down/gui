export function oklchToRgb(l: number, c: number, h: number, alpha = 1) {
  const a = c * Math.cos((h * Math.PI) / 180)
  const b = c * Math.sin((h * Math.PI) / 180)
  const l_ = (l + 0.3963377774 * a + 0.2158037573 * b) ** 3
  const m_ = (l - 0.1055613458 * a - 0.0638541728 * b) ** 3
  const s_ = (l - 0.0894841775 * a - 1.291485548 * b) ** 3
  const x = 1.2270138511 * l_ - 0.5577999807 * m_ + 0.281256149 * s_
  const y = -0.0405801784 * l_ + 1.1122568696 * m_ - 0.0716766787 * s_
  const z = -0.0763812845 * l_ - 0.4214819784 * m_ + 1.5861632204 * s_
  let r = 3.2404542 * x - 1.5371385 * y - 0.4985314 * z
  let g = -0.969266 * x + 1.8760108 * y + 0.041556 * z
  let b_ = 0.0556434 * x - 0.2040259 * y + 1.0572252 * z
  r = r > 0.0031308 ? 1.055 * r ** (1 / 2.4) - 0.055 : 12.92 * r
  g = g > 0.0031308 ? 1.055 * g ** (1 / 2.4) - 0.055 : 12.92 * g
  b_ = b_ > 0.0031308 ? 1.055 * b_ ** (1 / 2.4) - 0.055 : 12.92 * b_
  r = Math.min(Math.max(0, r), 1)
  g = Math.min(Math.max(0, g), 1)
  b_ = Math.min(Math.max(0, b_), 1)
  const to255 = (v: number) => Math.round(v * 255)
  if (alpha === 1) {
    return `rgb(${to255(r)}, ${to255(g)}, ${to255(b_)})`
  } else {
    return `rgba(${to255(r)}, ${to255(g)}, ${to255(b_)}, ${alpha})`
  }
}

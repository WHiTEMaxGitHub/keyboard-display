<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { useTheme } from "../composables/useTheme";

const { themeId } = useTheme();
const canvasRef = ref<HTMLCanvasElement | null>(null);

let rafId = 0;
let width = 0;
let height = 0;
let dpr = 1;

function cssVar(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

function rgbaVar(name: string): [number, number, number, number] {
  const raw = cssVar(name);
  const m = raw.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([\d.]+))?\)/);
  if (m) return [+m[1], +m[2], +m[3], parseFloat(m[4] || "1")];
  return [128, 128, 128, 0.3];
}

function rgba(color: [number, number, number, number], alphaScale = 1) {
  const [r, g, b, a] = color;
  return `rgba(${r}, ${g}, ${b}, ${Math.max(0, Math.min(1, a * alphaScale))})`;
}

function ribbonOffset(x: number, phase: number, freq: number, amp: number) {
  return Math.sin(x * freq + phase) * amp
    + Math.sin(x * freq * 0.48 - phase * 1.45) * amp * 0.62
    + Math.sin(x * freq * 1.82 + phase * 0.72) * amp * 0.28;
}

function drawFlow(time: number) {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, width, height);

  const colors = [
    rgbaVar("--theme-flow-1"),
    rgbaVar("--theme-flow-2"),
    rgbaVar("--theme-flow-3"),
    rgbaVar("--theme-flow-4"),
  ];
  const themeStrength: Record<string, number> = {
    vibrant: 1.28,
    dark: 0.78,
    midnight: 1.06,
    light: 0.9,
    custom: 0.82,
  };
  const strength = themeStrength[themeId.value] ?? 1;

  ctx.globalCompositeOperation = themeId.value === "light" ? "multiply" : "screen";

  const bands = [
    { y: 0.18, amp: 0.18, width: 0.42, speed: 0.00072, freq: 0.0028, color: colors[0] },
    { y: 0.40, amp: 0.22, width: 0.50, speed: 0.00058, freq: 0.0023, color: colors[1] },
    { y: 0.62, amp: 0.18, width: 0.45, speed: 0.00082, freq: 0.0032, color: colors[2] },
    { y: 0.80, amp: 0.14, width: 0.38, speed: 0.00064, freq: 0.0026, color: colors[3] },
  ];

  for (let i = 0; i < bands.length; i++) {
    const band = bands[i];
    const phase = time * band.speed + i * 2.1;
    const top: Array<[number, number]> = [];
    const bottom: Array<[number, number]> = [];

    for (let x = -80; x <= width + 80; x += 8) {
      const drift = ribbonOffset(x, phase, band.freq, height * band.amp);
      const center = height * band.y + drift;
      const bandWidth = height * band.width
        * (0.66 + Math.sin(phase * 0.82 + x * band.freq * 0.34) * 0.22);
      top.push([x, center - bandWidth * 0.5]);
      bottom.push([x, center + bandWidth * 0.5]);
    }

    const gradient = ctx.createLinearGradient(0, 0, width, height);
    gradient.addColorStop(0, "transparent");
    gradient.addColorStop(0.25, rgba(band.color, 0.35 * strength));
    gradient.addColorStop(0.5, rgba(band.color, 0.95 * strength));
    gradient.addColorStop(0.75, rgba(band.color, 0.35 * strength));
    gradient.addColorStop(1, "transparent");

    ctx.save();
    ctx.fillStyle = gradient;
    ctx.beginPath();
    ctx.moveTo(top[0][0], top[0][1]);
    for (const [x, y] of top) {
      ctx.lineTo(x, y);
    }
    for (let j = bottom.length - 1; j >= 0; j--) {
      ctx.lineTo(bottom[j][0], bottom[j][1]);
    }
    ctx.closePath();
    ctx.fill();
    ctx.restore();

    ctx.save();
    ctx.globalCompositeOperation = themeId.value === "light" ? "multiply" : "screen";
    ctx.lineWidth = themeId.value === "light" ? 1.4 : 1.8;
    ctx.strokeStyle = rgba(band.color, (themeId.value === "dark" ? 0.22 : 0.38) * strength);
    ctx.beginPath();
    for (let x = -80; x <= width + 80; x += 8) {
      const y = height * band.y
        + ribbonOffset(x, phase + Math.PI * 0.18, band.freq, height * band.amp * 0.68);
      if (x === -80) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
    }
    ctx.stroke();
    ctx.restore();
  }

  ctx.globalCompositeOperation = "source-over";
}

function draw(time: number) {
  drawFlow(time);
  rafId = requestAnimationFrame(draw);
}

function resize() {
  width = window.innerWidth;
  height = window.innerHeight;
  dpr = Math.min(window.devicePixelRatio || 1, 2);
  const canvas = canvasRef.value;
  if (canvas) {
    canvas.width = Math.ceil(width * dpr);
    canvas.height = Math.ceil(height * dpr);
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;
  }
}

onMounted(() => {
  resize();
  window.addEventListener("resize", resize);
  rafId = requestAnimationFrame(draw);
});

onUnmounted(() => {
  window.removeEventListener("resize", resize);
  cancelAnimationFrame(rafId);
});
</script>

<template>
  <canvas ref="canvasRef" class="ambient-canvas" aria-hidden="true" />
</template>

<style scoped>
.ambient-canvas {
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
  filter: blur(14px) saturate(125%);
  opacity: 0.95;
}
</style>

export type RgbColor = {
  r: number;
  g: number;
  b: number;
  a?: number;
};

export function normalizeHexColor(value: string, fallback = "#000000") {
  const trimmed = value.trim().replace(/^#/, "");

  if (/^[0-9a-fA-F]{3}$/.test(trimmed)) {
    return `#${trimmed
      .split("")
      .map((character) => character + character)
      .join("")
      .toLowerCase()}`;
  }

  if (/^[0-9a-fA-F]{4}$/.test(trimmed)) {
    return `#${trimmed
      .split("")
      .map((character) => character + character)
      .join("")
      .toLowerCase()}`;
  }

  if (/^[0-9a-fA-F]{6}$/.test(trimmed)) {
    return `#${trimmed.toLowerCase()}`;
  }

  if (/^[0-9a-fA-F]{8}$/.test(trimmed)) {
    return `#${trimmed.toLowerCase()}`;
  }

  return fallback;
}

export function hexToRgb(value: string): RgbColor {
  const hex = normalizeHexColor(value).slice(1);

  return {
    r: Number.parseInt(hex.slice(0, 2), 16),
    g: Number.parseInt(hex.slice(2, 4), 16),
    b: Number.parseInt(hex.slice(4, 6), 16),
    ...(hex.length === 8 ? { a: Number.parseInt(hex.slice(6, 8), 16) } : {}),
  };
}

export function rgbToHex(rgb: RgbColor) {
  const alpha = rgb.a === undefined ? "" : toHexChannel(rgb.a);
  return `#${toHexChannel(rgb.r)}${toHexChannel(rgb.g)}${toHexChannel(rgb.b)}${alpha}`;
}

function toHexChannel(value: number) {
  return clampChannel(value).toString(16).padStart(2, "0");
}

function clampChannel(value: number) {
  return Math.min(255, Math.max(0, Math.round(value)));
}

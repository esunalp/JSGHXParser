import * as THREE from './vendor/three.module.js';

const entries = new Map();

function keyify(value) {
  return value ? String(value).toLowerCase() : null;
}

function register(keys, config) {
  const normalized = Array.isArray(keys) ? keys : [keys];
  for (const key of normalized) {
    const normalizedKey = keyify(key);
    if (!normalizedKey) continue;
    entries.set(normalizedKey, config);
  }
}

function resolveEntry(node) {
  if (!node) return null;
  const guidKey = keyify(node.guid);
  if (guidKey && entries.has(guidKey)) {
    return entries.get(guidKey);
  }
  const nameKey = keyify(node.name);
  if (nameKey && entries.has(nameKey)) {
    return entries.get(nameKey);
  }
  return null;
}

function baseSliderState(node) {
  const defaults = {
    value: node?.meta?.value ?? node?.inputs?.value ?? 1,
    min: node?.meta?.min ?? node?.inputs?.min ?? 0,
    max: node?.meta?.max ?? node?.inputs?.max ?? 10,
    step: node?.meta?.step ?? node?.inputs?.step ?? 0.01,
    label: node?.meta?.label ?? node?.name ?? 'Slider'
  };
  if (defaults.step <= 0) {
    defaults.step = (defaults.max - defaults.min) / 100 || 0.1;
  }
  return defaults;
}

register(['{5e0b22ab-f3aa-4cc2-8329-7e548bb9a58b}', 'number slider', 'slider'], {
  type: 'slider',
  createState: baseSliderState,
  eval: ({ state }) => ({ value: state.value }),
  describe: (state) => state?.label ?? 'Slider'
});

register(['{56f1d440-0b71-44de-93d5-3c96bf53b78f}', 'box'], {
  type: 'geometry',
  inputs: ['W', 'H', 'D'],
  outputs: ['geom'],
  eval: ({ inputs }) => {
    const width = Number(inputs.W ?? inputs.width ?? 1) || 1;
    const height = Number(inputs.H ?? inputs.height ?? 1) || 1;
    const depth = Number(inputs.D ?? inputs.depth ?? 1) || 1;
    const geometry = new THREE.BoxGeometry(width, height, depth);
    return { geom: geometry };
  }
});

export const defaultRegistry = {
  lookup(node) {
    return resolveEntry(node);
  },
  register(keys, config) {
    register(keys, config);
  },
  entries,
};

export function describeNode(node) {
  const entry = resolveEntry(node);
  if (!entry) return node?.name ?? node?.guid ?? 'Onbekende node';
  if (entry.type === 'slider') {
    const state = entry.createState(node);
    return entry.describe?.(state) ?? state.label;
  }
  return node?.name ?? node?.guid ?? 'Node';
}

import * as THREE from 'three';
import { withVersion } from './version.js';

const versionedImport = (path) => import(withVersion(path));

const [
  {
    registerMathDomainComponents,
    registerMathMatrixComponents,
    registerMathTrigComponents,
    registerMathBooleanComponents,
    registerMathPolynomialComponents,
    registerMathOperatorComponents,
    registerMathScriptComponents,
  },
  {
    registerCurveAnalysisComponents,
    registerCurveDivisionComponents,
    registerCurvePrimitiveComponents,
    registerCurveSplineComponents,
    registerCurveUtilComponents,
  },
  {
    registerSurfacePrimitiveComponents,
    registerSurfaceFreeformComponents,
    registerSurfaceAnalysisComponents,
    registerSurfaceSubDComponents,
    registerSurfaceUtilComponents,
  },
  {
    registerTransformEuclideanComponents,
    registerTransformAffineComponents,
    registerTransformMorphComponents,
    registerTransformUtilComponents
  },
  {
    registerVectorComponents,
  },
] = await Promise.all([
  versionedImport('./registry-components-maths.js'),
  versionedImport('./registry-components-curve.js'),
  versionedImport('./registry-components-surface.js'),
  versionedImport('./registry-components-transform.js'),
  versionedImport('./registry-components-vector.js'),
]);

const entries = new Map();

function keyify(value) {
  return value ? String(value).toLowerCase() : null;
}

function normalizePinName(pin) {
  if (!pin && pin !== 0) return null;
  return String(pin).trim().toLowerCase();
}

function normalizePinEntries(pinConfig) {
  if (!pinConfig) return [];
  const entries = [];
  for (const [ghName, internalName] of Object.entries(pinConfig)) {
    if (!ghName || internalName === undefined || internalName === null) continue;
    const gh = String(ghName);
    entries.push({ gh, normalized: normalizePinName(gh), internal: String(internalName) });
  }
  return entries;
}

function mapInputPins(inputs = {}, pinEntries = []) {
  if (!pinEntries.length) {
    return { ...inputs };
  }
  const mapped = { ...inputs };
  const lookup = new Map();
  for (const entry of pinEntries) {
    lookup.set(entry.normalized, entry);
  }
  for (const [key, value] of Object.entries(inputs)) {
    const normalizedKey = normalizePinName(key);
    if (!normalizedKey) continue;
    const match = lookup.get(normalizedKey);
    if (match && mapped[match.internal] === undefined) {
      mapped[match.internal] = value;
    }
  }
  return mapped;
}

function mapOutputPins(outputs = {}, pinEntries = []) {
  if (!pinEntries.length) {
    return outputs || {};
  }
  const normalized = { ...(outputs || {}) };
  const lookup = new Map();
  for (const entry of pinEntries) {
    lookup.set(entry.normalized, entry);
  }

  for (const entry of pinEntries) {
    const value = normalized[entry.internal];
    if (value !== undefined && normalized[entry.gh] === undefined) {
      normalized[entry.gh] = value;
    }
  }

  for (const [key, value] of Object.entries(outputs || {})) {
    const normalizedKey = normalizePinName(key);
    if (!normalizedKey) continue;
    const match = lookup.get(normalizedKey);
    if (match && normalized[match.internal] === undefined) {
      normalized[match.internal] = value;
    }
  }

  return normalized;
}

function prepareConfig(config) {
  if (!config) return config;
  if (!config.pinMap) {
    return config;
  }
  const pinMap = {
    inputs: normalizePinEntries(config.pinMap.inputs ?? {}),
    outputs: normalizePinEntries(config.pinMap.outputs ?? {}),
  };
  const prepared = { ...config, pinMap };
  if (typeof config.eval === 'function') {
    prepared.eval = (payload) => {
      const incomingInputs = payload?.inputs ?? {};
      const mappedInputs = mapInputPins(incomingInputs, pinMap.inputs);
      const nextPayload = payload ? { ...payload, inputs: mappedInputs } : { inputs: mappedInputs };
      const result = config.eval(nextPayload) ?? {};
      return mapOutputPins(result, pinMap.outputs);
    };
  }
  return prepared;
}

function register(keys, config) {
  const normalized = Array.isArray(keys) ? keys : [keys];
  const preparedConfig = prepareConfig(config);
  for (const key of normalized) {
    const normalizedKey = keyify(key);
    if (!normalizedKey) continue;
    entries.set(normalizedKey, preparedConfig);
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

function toNumber(value, fallback = 0) {
  if (value === null || value === undefined) return fallback;
  if (Array.isArray(value)) {
    if (!value.length) return fallback;
    return toNumber(value[0], fallback);
  }
  const numeric = Number(value);
  return Number.isFinite(numeric) ? numeric : fallback;
}

function toVector3(value, fallback = new THREE.Vector3()) {
  if (value?.isVector3) {
    return value.clone();
  }
  if (Array.isArray(value)) {
    const [x, y, z] = value;
    return new THREE.Vector3(toNumber(x, 0), toNumber(y, 0), toNumber(z, 0));
  }
  if (typeof value === 'number') {
    return new THREE.Vector3(0, 0, toNumber(value, 0));
  }
  if (value && typeof value === 'object') {
    const x = toNumber(value.x, 0);
    const y = toNumber(value.y, 0);
    const z = toNumber(value.z, 0);
    if (Number.isFinite(x) || Number.isFinite(y) || Number.isFinite(z)) {
      return new THREE.Vector3(x, y, z);
    }
  }
  return fallback.clone ? fallback.clone() : fallback;
}

function collectNumericValues(input) {
  const numbers = [];
  const stack = [input];
  while (stack.length) {
    const current = stack.pop();
    if (current === undefined || current === null) continue;
    if (Array.isArray(current)) {
      for (const item of current) {
        stack.push(item);
      }
      continue;
    }
    if (current?.isVector3) {
      numbers.push(current.length());
      continue;
    }
    if (typeof current === 'object') {
      if (typeof current.value !== 'undefined') {
        stack.push(current.value);
        continue;
      }
    }
    const numeric = Number(current);
    if (Number.isFinite(numeric)) {
      numbers.push(numeric);
    }
  }
  return numbers;
}

registerMathDomainComponents({ register, toNumber });
registerMathMatrixComponents({ register, toNumber });
registerMathTrigComponents({ register, toNumber, toVector3 });
registerMathBooleanComponents({ register });
registerMathPolynomialComponents({ register, toNumber });
registerMathScriptComponents({ register, toNumber, toVector3 });
registerMathOperatorComponents({ register, toNumber, toVector3 });
registerCurvePrimitiveComponents({ register, toNumber, toVector3 });
registerCurveDivisionComponents({ register, toNumber, toVector3 });
registerCurveSplineComponents({ register, toNumber, toVector3 });
registerCurveAnalysisComponents({ register, toNumber, toVector3 });
registerCurveUtilComponents({ register, toNumber, toVector3 });
registerSurfacePrimitiveComponents({ register, toNumber, toVector3 });
registerSurfaceSubDComponents({ register, toNumber, toVector3 });
registerSurfaceAnalysisComponents({ register, toNumber, toVector3 });
registerSurfaceFreeformComponents({ register, toNumber, toVector3 });
registerSurfaceUtilComponents({ register, toNumber, toVector3 });
registerTransformEuclideanComponents({ register, toNumber, toVector3 });
registerTransformAffineComponents({ register, toNumber, toVector3 });
registerTransformMorphComponents({ register, toNumber, toVector3 });
registerTransformUtilComponents({ register, toNumber, toVector3 });
registerVectorComponents({ register, toNumber, toVector3 });

register(['{5e0b22ab-f3aa-4cc2-8329-7e548bb9a58b}', 'number slider', 'slider'], {
  type: 'slider',
  pinMap: {
    outputs: { value: 'value' },
  },
  createState: baseSliderState,
  eval: ({ state }) => ({ value: state.value }),
  describe: (state) => state?.label ?? 'Slider'
});

register(['{56f1d440-0b71-44de-93d5-3c96bf53b78f}', 'box'], {
  type: 'geometry',
  pinMap: {
    inputs: { W: 'width', H: 'height', D: 'depth' },
    outputs: { geom: 'geometry', geometry: 'geometry' },
  },
  eval: ({ inputs }) => {
    const width = toNumber(inputs.width, 1) || 1;
    const height = toNumber(inputs.height, 1) || 1;
    const depth = toNumber(inputs.depth, 1) || 1;
    const geometry = new THREE.BoxGeometry(width, height, depth);
    return { geometry };
  }
});

register([
  '{0d1e2027-f153-460d-84c0-f9af431b08cb}',
  'maximum',
  'max',
], {
  type: 'math',
  pinMap: {
    inputs: { A: 'a', B: 'b' },
    outputs: { R: 'result', result: 'result' },
  },
  eval: ({ inputs }) => {
    const a = toNumber(inputs.a, Number.NEGATIVE_INFINITY);
    const b = toNumber(inputs.b, Number.NEGATIVE_INFINITY);
    return { result: Math.max(a, b) };
  }
});

register([
  '{57308b30-772d-4919-ac67-e86c18f3a996}',
  'minimum',
  'min',
], {
  type: 'math',
  pinMap: {
    inputs: { A: 'a', B: 'b' },
    outputs: { R: 'result', result: 'result' },
  },
  eval: ({ inputs }) => {
    const a = toNumber(inputs.a, Number.POSITIVE_INFINITY);
    const b = toNumber(inputs.b, Number.POSITIVE_INFINITY);
    return { result: Math.min(a, b) };
  }
});

register([
  '{a50c4a3b-0177-4c91-8556-db95de6c56c8}',
  'round',
], {
  type: 'math',
  pinMap: {
    inputs: { x: 'value', Number: 'value', X: 'value' },
    outputs: { N: 'nearest', nearest: 'nearest', F: 'floor', floor: 'floor', C: 'ceiling', ceiling: 'ceiling' },
  },
  eval: ({ inputs }) => {
    const value = toNumber(inputs.value, 0);
    return {
      nearest: Math.round(value),
      floor: Math.floor(value),
      ceiling: Math.ceil(value),
    };
  }
});

register([
  '{0d2ccfb3-9d41-4759-9452-da6a522c3eaa}',
  'pi',
], {
  type: 'math',
  pinMap: {
    inputs: { N: 'factor', Factor: 'factor' },
    outputs: { y: 'result', Y: 'result', Output: 'result' },
  },
  eval: ({ inputs }) => {
    const factor = toNumber(inputs.factor, 1);
    return { result: Math.PI * factor };
  }
});

register([
  '{7986486c-621a-48fb-8f27-a28a22c91cc9}',
  'average',
  'avr',
], {
  type: 'math',
  pinMap: {
    inputs: { I: 'values', Input: 'values' },
    outputs: { AM: 'mean', 'Arithmetic mean': 'mean' },
  },
  eval: ({ inputs }) => {
    const numbers = collectNumericValues(inputs.values);
    if (!numbers.length) {
      return { mean: 0 };
    }
    const total = numbers.reduce((sum, value) => sum + value, 0);
    return { mean: total / numbers.length };
  }
});

function createCircleShape(radius = 1, segments = 64) {
  const shape = new THREE.Shape();
  shape.absarc(0, 0, Math.max(radius, 0.0001), 0, Math.PI * 2, false);
  return { shape, segments };
}

register([
  '{807b86e3-be8d-4970-92b5-f8cdcb45b06b}',
  'circle',
], {
  type: 'curve',
  pinMap: {
    inputs: { R: 'radius', Radius: 'radius', radius: 'radius', P: 'plane', Plane: 'plane' },
    outputs: { C: 'curve', curve: 'curve' },
  },
  eval: ({ inputs }) => {
    const radius = Math.max(toNumber(inputs.radius, 1), 0.0001);
    const { shape, segments } = createCircleShape(radius);
    return { curve: { type: 'circle', radius, shape, segments } };
  }
});

function resolveExtrudeDirection(input) {
  if (input === undefined || input === null) {
    return new THREE.Vector3(0, 0, 1);
  }
  if (Array.isArray(input) && input.length === 1) {
    return resolveExtrudeDirection(input[0]);
  }
  if (typeof input === 'number') {
    return new THREE.Vector3(0, 0, toNumber(input, 1));
  }
  if (input?.isVector3) {
    return input.clone();
  }
  if (typeof input === 'object') {
    const vector = toVector3(input, new THREE.Vector3(0, 0, 1));
    return vector.lengthSq() === 0 ? new THREE.Vector3(0, 0, 1) : vector;
  }
  return new THREE.Vector3(0, 0, 1);
}

function extrudeShape(curve, directionInput, options = {}) {
  if (!curve) return null;
  const shape = curve.shape ?? curve;
  if (!shape) return null;
  const direction = resolveExtrudeDirection(directionInput);
  const depth = direction.length() || options.defaultDepth || 1;
  const extrudeSettings = {
    depth,
    steps: options.steps ?? 16,
    bevelEnabled: false,
  };
  const geometry = new THREE.ExtrudeGeometry(shape, extrudeSettings);
  if (direction.x !== 0 || direction.y !== 0) {
    const axis = new THREE.Vector3(0, 0, 1).cross(direction).normalize();
    const angle = new THREE.Vector3(0, 0, 1).angleTo(direction);
    if (axis.lengthSq() > 0 && angle) {
      geometry.applyMatrix4(new THREE.Matrix4().makeRotationAxis(axis, angle));
    }
  }
  const translateZ = direction.clone().normalize().multiplyScalar(depth / 2);
  geometry.translate(translateZ.x, translateZ.y, translateZ.z);
  return geometry;
}

register([
  '{962034e9-cc27-4394-afc4-5c16e3447cf9}',
  'extrude',
], {
  type: 'geometry',
  pinMap: {
    inputs: { B: 'base', Base: 'base', C: 'curve', curve: 'curve', D: 'direction', Direction: 'direction', H: 'height', height: 'height' },
    outputs: { E: 'geom', geom: 'geom', geometry: 'geom' },
  },
  eval: ({ inputs }) => {
    const curveInput = inputs.base ?? inputs.curve;
    const directionInput = inputs.direction ?? inputs.height;
    const geometry = extrudeShape(curveInput, directionInput, { defaultDepth: toNumber(directionInput, 1) });
    if (!geometry) {
      return {};
    }
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

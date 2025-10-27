import * as THREE from 'three/webgpu';
import {
  cloneSurfaceMaterial,
  convertMaterialToNode,
  createStandardSurfaceMaterial,
  createTlsMaterial,
  ensureGeometryHasVertexNormals,
} from './material-utils.js';
import { createWaterSurfaceMaterial, isWaterPreviewColor } from './water-material.js';
import { withVersion } from './version.js';

const { surfaceToGeometry, isSurfaceDefinition } = await import(withVersion('./surface-mesher.js'));

const GUID_KEYS = (guids = []) => {
  const keys = new Set();
  for (const guid of guids) {
    if (!guid && guid !== 0) continue;
    const text = String(guid).trim();
    if (!text) continue;
    const bare = text.replace(/^{+/, '').replace(/}+$/, '');
    if (!bare) continue;
    keys.add(bare);
    keys.add(`{${bare}}`);
  }
  return Array.from(keys);
};

const IDENTITY_QUATERNION = new THREE.Quaternion();
const TEMP_MATRIX = new THREE.Matrix4();
const TEMP_SCALE = new THREE.Vector3();

function ensureRegisterFunction(register) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register display preview components.');
  }
}

function ensureToNumberFunction(toNumber) {
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register display preview components.');
  }
}

function ensureToVector3Function(toVector3) {
  if (typeof toVector3 !== 'function') {
    throw new Error('toVector3 function is required to register display preview components.');
  }
}

function ensureNumber(toNumber, value, fallback = 0) {
  const numeric = toNumber(value, Number.NaN);
  if (Number.isFinite(numeric)) {
    return numeric;
  }
  return fallback;
}

function ensureBoolean(value, fallback = false) {
  if (value === undefined || value === null) {
    return fallback;
  }
  if (Array.isArray(value)) {
    if (!value.length) {
      return fallback;
    }
    return ensureBoolean(value[0], fallback);
  }
  if (typeof value === 'string') {
    const normalized = value.trim().toLowerCase();
    if (!normalized) {
      return fallback;
    }
    if (['true', 'yes', '1', 'on'].includes(normalized)) {
      return true;
    }
    if (['false', 'no', '0', 'off'].includes(normalized)) {
      return false;
    }
    return fallback;
  }
  return Boolean(value);
}

function normalizeList(value) {
  if (value === undefined || value === null) {
    return [];
  }
  if (Array.isArray(value)) {
    return value.slice();
  }
  return [value];
}

function getListValue(list, index, fallback) {
  if (!list.length) {
    return fallback;
  }
  if (index < list.length) {
    return list[index];
  }
  return list[list.length - 1];
}

function parseDelimitedColorText(text) {
  if (typeof text !== 'string') {
    return null;
  }

  const segments = text
    .split(/[;,]/)
    .map((segment) => segment.trim())
    .filter((segment) => segment.length);

  if (segments.length < 3) {
    return null;
  }

  const values = segments.slice(0, 3).map((segment) => Number(segment));
  if (!values.every((value) => Number.isFinite(value))) {
    return null;
  }

  const requiresScaling = values.some((value) => Math.abs(value) > 1);
  const [r, g, b] = requiresScaling
    ? values.map((value) => value / 255)
    : values;

  const clamp01 = (value) => {
    if (!Number.isFinite(value)) {
      return 0;
    }
    if (value <= 0) {
      return 0;
    }
    if (value >= 1) {
      return 1;
    }
    return value;
  };

  return new THREE.Color(clamp01(r), clamp01(g), clamp01(b));
}

function parseColor(input, fallback = null) {
  if (input === undefined || input === null) {
    return fallback ? fallback.clone() : null;
  }
  if (Array.isArray(input)) {
    if (!input.length) {
      return fallback ? fallback.clone() : null;
    }
    return parseColor(input[0], fallback);
  }
  if (input?.isColor) {
    return input.clone();
  }
  if (typeof input === 'number') {
    const color = new THREE.Color();
    color.set(Number(input));
    return color;
  }
  if (typeof input === 'string') {
    const text = input.trim();
    if (!text) {
      return fallback ? fallback.clone() : null;
    }

    const delimitedColor = parseDelimitedColorText(text);
    if (delimitedColor) {
      return delimitedColor;
    }

    try {
      const color = new THREE.Color(text);
      return color;
    } catch (_error) {
      return fallback ? fallback.clone() : null;
    }
  }
  if (typeof input === 'object') {
    if (Object.prototype.hasOwnProperty.call(input, 'color')) {
      return parseColor(input.color, fallback);
    }
    const r = Number(input.r ?? input.red ?? Number.NaN);
    const g = Number(input.g ?? input.green ?? Number.NaN);
    const b = Number(input.b ?? input.blue ?? Number.NaN);
    if (Number.isFinite(r) && Number.isFinite(g) && Number.isFinite(b)) {
      const color = new THREE.Color();
      if (Math.abs(r) > 1 || Math.abs(g) > 1 || Math.abs(b) > 1) {
        color.setRGB(r / 255, g / 255, b / 255);
      } else {
        color.setRGB(r, g, b);
      }
      return color;
    }
  }
  return fallback ? fallback.clone() : null;
}

function ensureColor(input, fallback = new THREE.Color(0xffffff)) {
  const base = fallback ?? new THREE.Color(0xffffff);
  const parsed = parseColor(input, base);
  return parsed ?? base.clone();
}

function collectPoints(toVector3, input) {
  const stack = [input];
  const points = [];
  const visited = new Set();
  while (stack.length) {
    const current = stack.pop();
    if (current === undefined || current === null) {
      continue;
    }
    if (typeof current === 'object') {
      if (visited.has(current)) {
        continue;
      }
      visited.add(current);
    }
    if (Array.isArray(current)) {
      for (const entry of current) {
        stack.push(entry);
      }
      continue;
    }
    if (current?.isVector3) {
      points.push(current.clone());
      continue;
    }
    if (typeof current === 'object') {
      if (Object.prototype.hasOwnProperty.call(current, 'point')) {
        stack.push(current.point);
      }
      if (Object.prototype.hasOwnProperty.call(current, 'points')) {
        stack.push(current.points);
      }
      if (Object.prototype.hasOwnProperty.call(current, 'location')) {
        stack.push(current.location);
      }
      const point = toVector3(current, null);
      if (point) {
        points.push(point);
      }
      continue;
    }
    const point = toVector3(current, null);
    if (point) {
      points.push(point);
    }
  }
  return points;
}

function ensureMaterial(value) {
  if (value === undefined || value === null) {
    return null;
  }
  if (Array.isArray(value)) {
    if (!value.length) {
      return null;
    }
    return ensureMaterial(value[0]);
  }
  if (value?.isMaterial) {
    const cloned = cloneSurfaceMaterial(value);
    return convertMaterialToNode(cloned, { side: THREE.DoubleSide });
  }
  if (value && typeof value === 'object') {
    if (Object.prototype.hasOwnProperty.call(value, 'material')) {
      return ensureMaterial(value.material);
    }
  }
  return null;
}

function toFiniteNumber(value) {
  if (typeof value === 'number') {
    return Number.isFinite(value) ? value : null;
  }
  if (typeof value === 'string') {
    const numeric = Number(value);
    return Number.isFinite(numeric) ? numeric : null;
  }
  return null;
}

function toVector3Like(value) {
  if (!value && value !== 0) {
    return null;
  }
  if (value.isVector3) {
    return value.clone();
  }
  if (Array.isArray(value)) {
    if (value.length < 3) {
      return null;
    }
    const [x, y, z] = value;
    const numeric = [x, y, z].map((component) => toFiniteNumber(component));
    if (numeric.every((component) => component !== null)) {
      return new THREE.Vector3(numeric[0], numeric[1], numeric[2]);
    }
    return null;
  }
  if (typeof value === 'object') {
    const lower = ['x', 'y', 'z'].map((key) => toFiniteNumber(value[key]));
    if (lower.every((component) => component !== null)) {
      return new THREE.Vector3(lower[0], lower[1], lower[2]);
    }
    const upper = ['X', 'Y', 'Z'].map((key) => toFiniteNumber(value[key]));
    if (upper.every((component) => component !== null)) {
      return new THREE.Vector3(upper[0], upper[1], upper[2]);
    }
    if (value.point) {
      const nested = toVector3Like(value.point);
      if (nested) {
        return nested;
      }
    }
    if (Array.isArray(value.vertices)) {
      return null;
    }
  }
  return null;
}

function extractFaceIndices(face, vertexCount) {
  if (!face) {
    return [];
  }

  const normalized = [];

  const addIndex = (entry) => {
    const numeric = toFiniteNumber(entry);
    if (numeric !== null && numeric >= 0 && numeric < vertexCount) {
      normalized.push(numeric);
    }
  };

  if (Array.isArray(face)) {
    face.forEach(addIndex);
  } else if (typeof face === 'object') {
    if (Array.isArray(face.vertices)) {
      face.vertices.forEach(addIndex);
    } else {
      const keys = ['a', 'b', 'c', 'd', 'A', 'B', 'C', 'D', 'i', 'j', 'k', 'l'];
      keys.forEach((key) => addIndex(face[key]));
    }
  }

  return normalized;
}

function createGeometryFromMeshLike(data) {
  if (!data || typeof data !== 'object') {
    return null;
  }

  if (data.isBufferGeometry || data.isGeometry) {
    return data.clone?.() ?? data;
  }

  if (data.geometry) {
    const direct = createGeometryFromMeshLike(data.geometry);
    if (direct) {
      return direct;
    }
  }

  const vertices = Array.isArray(data.vertices) ? data.vertices.map((vertex) => toVector3Like(vertex)).filter(Boolean) : [];
  const faces = Array.isArray(data.faces) ? data.faces : [];

  if (!vertices.length || !faces.length) {
    return null;
  }

  const positionArray = new Float32Array(vertices.length * 3);
  vertices.forEach((vertex, index) => {
    positionArray[index * 3] = vertex.x;
    positionArray[index * 3 + 1] = vertex.y;
    positionArray[index * 3 + 2] = vertex.z;
  });

  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.BufferAttribute(positionArray, 3));

  const indices = [];
  faces.forEach((face) => {
    const faceIndices = extractFaceIndices(face, vertices.length);
    if (faceIndices.length >= 3) {
      for (let i = 1; i < faceIndices.length - 1; i += 1) {
        indices.push(faceIndices[0], faceIndices[i], faceIndices[i + 1]);
      }
    }
  });

  if (!indices.length) {
    return null;
  }

  const IndexArray = indices.length > 65535 ? Uint32Array : Uint16Array;
  geometry.setIndex(new THREE.BufferAttribute(new IndexArray(indices), 1));
  geometry.computeVertexNormals();

  if (data.metadata && typeof data.metadata === 'object') {
    geometry.userData = { ...(geometry.userData ?? {}), ...data.metadata };
  }

  return geometry;
}

function collectGeometryEntries(input, results = [], visited = new Set()) {
  if (input === undefined || input === null) {
    return results;
  }

  const type = typeof input;
  if (type !== 'object' && type !== 'function') {
    return results;
  }

  if (visited.has(input)) {
    return results;
  }
  visited.add(input);

  if (Array.isArray(input)) {
    for (const entry of input) {
      collectGeometryEntries(entry, results, visited);
    }
    return results;
  }

  if (isSurfaceDefinition(input)) {
    const geometry = surfaceToGeometry(input);
    if (geometry) {
      results.push(geometry);
    }
    return results;
  }

  if (input.isObject3D || input.isBufferGeometry || input.isGeometry) {
    results.push(input);
    return results;
  }

  const meshGeometry = createGeometryFromMeshLike(input);
  if (meshGeometry) {
    results.push(meshGeometry);
    return results;
  }

  if (Object.prototype.hasOwnProperty.call(input, 'mesh')) {
    collectGeometryEntries(input.mesh, results, visited);
  }
  if (Object.prototype.hasOwnProperty.call(input, 'geometry')) {
    collectGeometryEntries(input.geometry, results, visited);
  }
  if (Object.prototype.hasOwnProperty.call(input, 'surface')) {
    collectGeometryEntries(input.surface, results, visited);
  }
  if (Object.prototype.hasOwnProperty.call(input, 'surfaces')) {
    collectGeometryEntries(input.surfaces, results, visited);
  }
  if (Object.prototype.hasOwnProperty.call(input, 'value')) {
    collectGeometryEntries(input.value, results, visited);
  }
  if (Object.prototype.hasOwnProperty.call(input, 'values')) {
    collectGeometryEntries(input.values, results, visited);
  }
  return results;
}

function parseSymbolDisplayConfig(toNumber, value) {
  if (!value || typeof value !== 'object') {
    return null;
  }

  const styleText = String(value.style ?? value.Style ?? value.X ?? 'circle').trim().toLowerCase();
  const sizePrimary = ensureNumber(toNumber, value.sizePrimary ?? value.size ?? value.S ?? 1, 1);
  const sizeSecondary = ensureNumber(toNumber, value.sizeSecondary ?? value.S2 ?? value.secondary ?? sizePrimary, sizePrimary);
  const rotationDegrees = ensureNumber(toNumber, value.rotation ?? value.R ?? 0, 0);
  const rotation = THREE.MathUtils.degToRad(rotationDegrees);
  const colourInput = value.colour ?? value.color ?? value.C ?? value.fillColour ?? value.fillColor ?? value.Cf;
  const fillColor = ensureColor(colourInput, new THREE.Color(0x2c9cf5));
  const edgeInput = value.edgeColour ?? value.edgeColor ?? value.edge ?? value.Ce;
  const edgeColor = ensureColor(edgeInput, fillColor);
  const edgeWidth = ensureNumber(toNumber, value.edgeWidth ?? value.width ?? value.W ?? sizePrimary * 0.05, sizePrimary * 0.05);
  const adjust = ensureBoolean(value.adjust ?? value.A ?? value.Adjust ?? false, false);

  return {
    type: 'symbol-display',
    style: styleText || 'circle',
    sizePrimary: Math.max(Math.abs(sizePrimary), 0.001),
    sizeSecondary: Math.max(Math.abs(sizeSecondary), 0.001),
    rotation,
    fillColor,
    edgeColor,
    edgeWidth: Math.max(Math.abs(edgeWidth), 0),
    adjust,
  };
}

function createSymbolMesh(config) {
  if (!config) {
    return null;
  }
  const style = config.style ?? 'circle';
  const primary = config.sizePrimary ?? 1;
  const secondary = config.sizeSecondary ?? primary;
  const fillColor = config.fillColor ?? new THREE.Color(0x2c9cf5);

  let geometry;
  switch (style) {
    case 'square':
    case 'box':
      geometry = new THREE.BoxGeometry(primary, primary, primary * 0.1);
      break;
    case 'diamond':
      geometry = new THREE.BoxGeometry(primary, primary, primary * 0.1);
      geometry.rotateZ(Math.PI / 4);
      break;
    case 'triangle':
      geometry = new THREE.ConeGeometry(primary, primary * 0.1, 3);
      geometry.rotateX(Math.PI / 2);
      break;
    case 'ring':
      geometry = new THREE.TorusGeometry(primary / 2, Math.max(secondary * 0.15, primary * 0.05), 12, 32);
      break;
    default:
      geometry = new THREE.SphereGeometry(primary / 2, 18, 14);
      break;
  }

  ensureGeometryHasVertexNormals(geometry);
  const material = createStandardSurfaceMaterial(
    {
      color: fillColor?.clone?.() ?? fillColor,
      metalness: 0.05,
      roughness: 0.55,
    },
    { side: THREE.DoubleSide },
  );

  const mesh = new THREE.Mesh(geometry, material);
  mesh.castShadow = false;
  mesh.receiveShadow = false;
  if (Number.isFinite(config.rotation)) {
    mesh.rotation.z = config.rotation;
  }
  mesh.userData.symbolConfig = config;
  return mesh;
}

function applyMaterialToObject(object, material) {
  if (!object) {
    return null;
  }
  if (object.isMesh) {
    if (object.geometry?.clone) {
      object.geometry = object.geometry.clone();
    }
    const candidateMaterial = cloneSurfaceMaterial(material);
    if (candidateMaterial) {
      object.material = convertMaterialToNode(candidateMaterial, { side: THREE.DoubleSide });
    } else {
      object.material = convertMaterialToNode(object.material, { side: THREE.DoubleSide });
    }
    ensureGeometryHasVertexNormals(object.geometry);
    object.castShadow = true;
    object.receiveShadow = true;
  }
  if (object.children && object.children.length) {
    object.children.forEach((child) => applyMaterialToObject(child, material));
  }
  return object;
}

function createMeshFromGeometry(entry, material) {
  if (!entry) {
    return null;
  }
  if (entry.isMesh) {
    return applyMaterialToObject(entry.clone(true), material);
  }
  if (entry.isObject3D) {
    const cloned = entry.clone(true);
    return applyMaterialToObject(cloned, material);
  }
  if (entry.isBufferGeometry || entry.isGeometry) {
    const geometry = entry.clone ? entry.clone() : entry;
    ensureGeometryHasVertexNormals(geometry);
    const baseMaterial = cloneSurfaceMaterial(material);
    const meshMaterial = convertMaterialToNode(baseMaterial, { side: THREE.DoubleSide })
      ?? createStandardSurfaceMaterial(
        {
          color: 0x2c9cf5,
          metalness: 0.1,
          roughness: 0.65,
        },
        { side: THREE.DoubleSide },
      );
    const mesh = new THREE.Mesh(geometry, meshMaterial);
    mesh.castShadow = true;
    mesh.receiveShadow = true;
    return mesh;
  }
  return null;
}

export function registerDisplayPreviewComponents({ register, toNumber, toVector3 }) {
  ensureRegisterFunction(register);
  ensureToNumberFunction(toNumber);
  ensureToVector3Function(toVector3);

  const fallbackColor = new THREE.Color(0x2c9cf5);

  register([
    ...GUID_KEYS(['059b72b0-9bb3-4542-a805-2dcd27493164']),
    'Cloud Display',
    'cloud display',
    'Cloud',
    'cloud',
  ], {
    type: 'display:preview',
    pinMap: {
      inputs: {
        P: 'points',
        Points: 'points',
        points: 'points',
        C: 'colours',
        Colours: 'colours',
        colours: 'colours',
        colors: 'colours',
        S: 'sizes',
        Size: 'sizes',
        size: 'sizes',
      },
    },
    eval: ({ inputs }) => {
      const points = collectPoints(toVector3, inputs.points);
      if (!points.length) {
        return {};
      }
      const colourList = normalizeList(inputs.colours);
      const sizeList = normalizeList(inputs.sizes);
      const defaultSize = Math.max(Math.abs(ensureNumber(toNumber, sizeList[0] ?? 1, 1)), 0.01);

      const baseRadius = 0.5;
      const geometry = new THREE.SphereGeometry(baseRadius, 14, 10);
      ensureGeometryHasVertexNormals(geometry);
      const material = createStandardSurfaceMaterial(
        {
          color: 0xffffff,
          vertexColors: true,
          metalness: 0.05,
          roughness: 0.75,
          transparent: true,
          opacity: 0.85,
        },
        { side: THREE.DoubleSide },
      );
      const instanced = new THREE.InstancedMesh(geometry, material, points.length);
      const colours = new Float32Array(points.length * 3);

      points.forEach((point, index) => {
        const sizeValue = Math.max(Math.abs(ensureNumber(toNumber, getListValue(sizeList, index, defaultSize), defaultSize)), 0.01);
        const scale = sizeValue / baseRadius;
        TEMP_SCALE.set(scale, scale, scale);
        TEMP_MATRIX.compose(point, IDENTITY_QUATERNION, TEMP_SCALE);
        instanced.setMatrixAt(index, TEMP_MATRIX);
        const colour = ensureColor(getListValue(colourList, index, colourList[0] ?? fallbackColor), fallbackColor);
        const offset = index * 3;
        colours[offset + 0] = colour.r;
        colours[offset + 1] = colour.g;
        colours[offset + 2] = colour.b;
      });

      instanced.instanceMatrix.needsUpdate = true;
      instanced.instanceColor = new THREE.InstancedBufferAttribute(colours, 3);
      instanced.instanceColor.needsUpdate = true;
      instanced.castShadow = false;
      instanced.receiveShadow = false;
      instanced.userData.dispose = () => {
        instanced.instanceColor = null;
      };

      return { mesh: instanced };
    },
  });

  register([
    ...GUID_KEYS(['537b0419-bbc2-4ff4-bf08-afe526367b2c']),
    'Custom Preview',
    'custom preview',
    'Preview',
    'preview',
  ], {
    type: 'display:preview',
    pinMap: {
      inputs: {
        G: 'geometry',
        Geometry: 'geometry',
        geometry: 'geometry',
        M: 'material',
        Material: 'material',
        material: 'material',
      },
    },
    eval: ({ inputs }) => {
      const geometries = collectGeometryEntries(inputs.geometry);
      if (!geometries.length) {
        return {};
      }
      let material = ensureMaterial(inputs.material);

      if (!material) {
        const colourCandidate = parseColor(inputs.material, null);
        if (colourCandidate && isWaterPreviewColor(colourCandidate)) {
          material = createWaterSurfaceMaterial({ side: THREE.DoubleSide });
          material.userData.source = 'procedural-water';
        }
      }

      if (!material) {
        material = createStandardSurfaceMaterial(
          {
            color: fallbackColor,
            metalness: 0.1,
            roughness: 0.65,
          },
          { side: THREE.DoubleSide },
        );
      }

      const meshes = geometries
        .map((entry) => createMeshFromGeometry(entry, material))
        .filter((entry) => entry);

      if (!meshes.length) {
        return {};
      }

      if (meshes.length === 1) {
        return { mesh: meshes[0] };
      }
      return { mesh: meshes };
    },
  });

  register([
    ...GUID_KEYS(['62d5ead4-53c4-4d0b-b5ce-6bd6e0850ab8']),
    'Symbol Display',
    'symbol display',
    'Symbol',
    'symbol',
  ], {
    type: 'display:preview',
    pinMap: {
      inputs: {
        P: 'location',
        Point: 'location',
        Location: 'location',
        D: 'display',
        Display: 'display',
        display: 'display',
      },
    },
    eval: ({ inputs }) => {
      const locations = collectPoints(toVector3, inputs.location);
      const displayConfigs = normalizeList(inputs.display)
        .map((entry) => parseSymbolDisplayConfig(toNumber, entry))
        .filter((entry) => entry);

      if (!locations.length || !displayConfigs.length) {
        return {};
      }

      const meshes = [];
      for (const location of locations) {
        for (const config of displayConfigs) {
          const mesh = createSymbolMesh(config);
          if (!mesh) {
            continue;
          }
          mesh.position.copy(location);
          meshes.push(mesh);
        }
      }

      if (!meshes.length) {
        return {};
      }

      if (meshes.length === 1) {
        return { mesh: meshes[0] };
      }
      return { mesh: meshes };
    },
  });

  register([
    ...GUID_KEYS(['6b1bd8b2-47a4-4aa6-a471-3fd91c62a486']),
    'Dot Display',
    'dot display',
    'Dots',
    'dots',
  ], {
    type: 'display:preview',
    pinMap: {
      inputs: {
        P: 'point',
        Point: 'point',
        point: 'point',
        C: 'colour',
        Colour: 'colour',
        colour: 'colour',
        color: 'colour',
        Color: 'colour',
        S: 'size',
        Size: 'size',
        size: 'size',
      },
    },
    eval: ({ inputs }) => {
      const point = collectPoints(toVector3, inputs.point)[0];
      if (!point) {
        return {};
      }
      const colour = ensureColor(inputs.colour, fallbackColor);
      const size = Math.max(Math.abs(ensureNumber(toNumber, inputs.size ?? 1, 1)), 0.01);
      const radius = size / 2;
      const geometry = new THREE.SphereGeometry(radius, 18, 14);
      ensureGeometryHasVertexNormals(geometry);
      const material = createStandardSurfaceMaterial(
        {
          color: colour,
          metalness: 0.05,
          roughness: 0.5,
        },
        { side: THREE.DoubleSide },
      );
      const mesh = new THREE.Mesh(geometry, material);
      mesh.position.copy(point);
      mesh.castShadow = false;
      mesh.receiveShadow = false;
      return { mesh };
    },
  });

  register([
    ...GUID_KEYS(['76975309-75a6-446a-afed-f8653720a9f2']),
    'Create Material',
    'create material',
    'Material',
    'material',
  ], {
    type: 'display:preview',
    pinMap: {
      inputs: {
        Kd: 'diffuse',
        Diffuse: 'diffuse',
        diffuse: 'diffuse',
        Ks: 'specular',
        Specular: 'specular',
        specular: 'specular',
        Ke: 'emission',
        Emission: 'emission',
        emission: 'emission',
        T: 'transparency',
        Transparency: 'transparency',
        transparency: 'transparency',
        S: 'shine',
        Shine: 'shine',
        shine: 'shine',
      },
      outputs: {
        M: 'material',
        Material: 'material',
        material: 'material',
      },
    },
    eval: ({ inputs }) => {
      const diffuse = ensureColor(inputs.diffuse, new THREE.Color(0x9aa5b1));
      const specular = ensureColor(inputs.specular, new THREE.Color(0xcccccc));
      const emissive = ensureColor(inputs.emission, new THREE.Color(0x000000));
      const transparency = THREE.MathUtils.clamp(ensureNumber(toNumber, inputs.transparency ?? 0, 0), 0, 1);
      const shineInput = ensureNumber(toNumber, inputs.shine ?? 30, 30);
      const shininess = THREE.MathUtils.clamp(shineInput * 1.28, 0, 256);

      let material;
      if (isWaterPreviewColor(diffuse)) {
        material = createWaterSurfaceMaterial({ side: THREE.DoubleSide });
        material.userData.source = 'procedural-water';
      } else {
        material = createTlsMaterial(
          {
            diffuse,
            specular,
            emissive,
            transparency,
            shininess,
          },
          { side: THREE.DoubleSide },
        );

        material.userData.source = 'create-material';
      }

      return { material };
    },
  });

  register([
    ...GUID_KEYS(['79747717-1874-4c34-b790-faef53b50569']),
    'Symbol (Simple)',
    'symbol (simple)',
    'SymSim',
    'symsim',
  ], {
    type: 'display:preview',
    pinMap: {
      inputs: {
        X: 'style',
        Style: 'style',
        style: 'style',
        S: 'size',
        Size: 'size',
        size: 'size',
        R: 'rotation',
        Rotation: 'rotation',
        rotation: 'rotation',
        C: 'colour',
        Colour: 'colour',
        colour: 'colour',
        color: 'colour',
      },
      outputs: {
        D: 'display',
        Display: 'display',
        display: 'display',
      },
    },
    eval: ({ inputs }) => {
      const config = parseSymbolDisplayConfig(toNumber, {
        style: inputs.style,
        size: inputs.size,
        rotation: inputs.rotation,
        colour: inputs.colour,
      });
      if (!config) {
        return {};
      }
      return { display: config };
    },
  });

  register([
    ...GUID_KEYS(['e5c82975-8011-412c-b56d-bb7fc9e7f28d']),
    'Symbol (Advanced)',
    'symbol (advanced)',
    'SymAdv',
    'symadv',
  ], {
    type: 'display:preview',
    pinMap: {
      inputs: {
        X: 'style',
        Style: 'style',
        style: 'style',
        S1: 'sizePrimary',
        'Size Primary': 'sizePrimary',
        sizePrimary: 'sizePrimary',
        S2: 'sizeSecondary',
        'Size Secondary': 'sizeSecondary',
        sizeSecondary: 'sizeSecondary',
        R: 'rotation',
        Rotation: 'rotation',
        rotation: 'rotation',
        Cf: 'fill',
        Fill: 'fill',
        fill: 'fill',
        Ce: 'edge',
        Edge: 'edge',
        edge: 'edge',
        W: 'width',
        Width: 'width',
        width: 'width',
        A: 'adjust',
        Adjust: 'adjust',
        adjust: 'adjust',
      },
      outputs: {
        D: 'display',
        Display: 'display',
        display: 'display',
      },
    },
    eval: ({ inputs }) => {
      const config = parseSymbolDisplayConfig(toNumber, {
        style: inputs.style,
        sizePrimary: inputs.sizePrimary,
        sizeSecondary: inputs.sizeSecondary,
        rotation: inputs.rotation,
        fillColour: inputs.fill,
        edgeColour: inputs.edge,
        edgeWidth: inputs.width,
        adjust: inputs.adjust,
      });
      if (!config) {
        return {};
      }
      return { display: config };
    },
  });
}

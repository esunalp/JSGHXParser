import * as THREE from 'three';

const EPSILON = 1e-6;
const DEFAULT_SEGMENTS = {
  radial: 48,
  height: 1,
  planeU: 1,
  planeV: 1,
  sphereWidth: 48,
  sphereHeight: 24,
  sampleU: 48,
  sampleV: 24,
};

function toFiniteNumber(value, fallback = null) {
  if (value === undefined || value === null) {
    return fallback;
  }
  if (typeof value === 'number') {
    return Number.isFinite(value) ? value : fallback;
  }
  const numeric = Number(value);
  return Number.isFinite(numeric) ? numeric : fallback;
}

function toVector3(value, fallback = null) {
  if (!value && value !== 0) {
    return fallback ? fallback.clone() : null;
  }
  if (value.isVector3) {
    return value.clone();
  }
  if (Array.isArray(value) && value.length >= 3) {
    const x = toFiniteNumber(value[0], 0);
    const y = toFiniteNumber(value[1], 0);
    const z = toFiniteNumber(value[2], 0);
    return new THREE.Vector3(x, y, z);
  }
  if (typeof value === 'object') {
    const x = toFiniteNumber(value.x ?? value[0], null);
    const y = toFiniteNumber(value.y ?? value[1], null);
    const z = toFiniteNumber(value.z ?? value[2], null);
    if (x === null || y === null || z === null) {
      return fallback ? fallback.clone() : null;
    }
    return new THREE.Vector3(x, y, z);
  }
  if (typeof value === 'number') {
    return Number.isFinite(value) ? new THREE.Vector3(value, 0, 0) : fallback ? fallback.clone() : null;
  }
  return fallback ? fallback.clone() : null;
}

function toUnitVector(value, fallback) {
  const candidate = toVector3(value, fallback);
  if (!candidate) {
    return fallback ? fallback.clone().normalize() : null;
  }
  if (candidate.lengthSq() <= EPSILON) {
    if (fallback) {
      const normalizedFallback = fallback.clone();
      if (normalizedFallback.lengthSq() <= EPSILON) {
        normalizedFallback.set(1, 0, 0);
      }
      return normalizedFallback.normalize();
    }
    return null;
  }
  return candidate.normalize();
}

function firstDefined(...values) {
  for (const value of values) {
    if (value !== undefined && value !== null) {
      return value;
    }
  }
  return undefined;
}

function isSurfaceCandidate(value) {
  if (!value || typeof value !== 'object') {
    return false;
  }
  if (value.isBufferGeometry || value.isGeometry || value.isMesh) {
    return false;
  }
  if (typeof value.evaluate === 'function' || typeof value.getPoint === 'function') {
    return true;
  }
  if (Array.isArray(value.points)) {
    return true;
  }
  if (value.surface && value.surface !== value) {
    return isSurfaceCandidate(value.surface);
  }
  return false;
}

function collectSurfaceChain(value) {
  const chain = [];
  const visited = new Set();
  let current = value;
  while (current && typeof current === 'object' && !visited.has(current)) {
    chain.push(current);
    visited.add(current);
    if (!current.surface || current.surface === current) {
      break;
    }
    current = current.surface;
  }
  return chain;
}

function pickFromChain(chain, selector) {
  for (const entry of chain) {
    const result = selector(entry);
    if (result !== undefined && result !== null) {
      return result;
    }
  }
  return undefined;
}

function getDomain(input, fallbackStart = 0, fallbackEnd = 1) {
  if (input === undefined || input === null) {
    const start = fallbackStart;
    const end = fallbackEnd;
    return {
      start,
      end,
      span: end - start,
      length: Math.abs(end - start),
      min: Math.min(start, end),
      max: Math.max(start, end),
    };
  }

  if (Array.isArray(input) && input.length >= 2) {
    const start = toFiniteNumber(input[0], fallbackStart);
    const end = toFiniteNumber(input[1], fallbackEnd);
    return {
      start,
      end,
      span: end - start,
      length: Math.abs(end - start),
      min: Math.min(start, end),
      max: Math.max(start, end),
    };
  }

  if (typeof input === 'object') {
    const start = toFiniteNumber(
      input.start ?? input.min ?? input.a ?? input.from ?? input.lower ?? input.t0,
      undefined,
    );
    const end = toFiniteNumber(
      input.end ?? input.max ?? input.b ?? input.to ?? input.upper ?? input.t1,
      undefined,
    );
    if (start !== undefined && end !== undefined && start !== null && end !== null) {
      return {
        start,
        end,
        span: end - start,
        length: Math.abs(end - start),
        min: Math.min(start, end),
        max: Math.max(start, end),
      };
    }
    if (Array.isArray(input.values) && input.values.length >= 2) {
      return getDomain(input.values, fallbackStart, fallbackEnd);
    }
  }

  const numeric = toFiniteNumber(input, null);
  if (numeric !== null) {
    return {
      start: numeric,
      end: numeric,
      span: 0,
      length: 0,
      min: numeric,
      max: numeric,
    };
  }

  const start = fallbackStart;
  const end = fallbackEnd;
  return {
    start,
    end,
    span: end - start,
    length: Math.abs(end - start),
    min: Math.min(start, end),
    max: Math.max(start, end),
  };
}

function createSurfaceInfo(value) {
  if (!isSurfaceCandidate(value)) {
    return null;
  }
  const chain = collectSurfaceChain(value);
  if (!chain.length) {
    return null;
  }

  const metadata = pickFromChain(chain, (entry) => entry.metadata) ?? {};
  const plane = pickFromChain(chain, (entry) => entry.plane) ?? null;
  const domainU = pickFromChain(chain, (entry) => entry.domainU) ?? null;
  const domainV = pickFromChain(chain, (entry) => entry.domainV) ?? null;
  const evaluate = pickFromChain(chain, (entry) => (typeof entry.evaluate === 'function' ? entry.evaluate : undefined));
  const getPoint = pickFromChain(chain, (entry) => (typeof entry.getPoint === 'function' ? entry.getPoint : undefined));
  const points = pickFromChain(chain, (entry) => (Array.isArray(entry.points) ? entry.points : undefined));

  const extras = {
    radius: pickFromChain(chain, (entry) => entry.radius),
    height: pickFromChain(chain, (entry) => entry.height),
    center: pickFromChain(chain, (entry) => entry.center),
  };

  return {
    chain,
    metadata,
    plane,
    domainU,
    domainV,
    evaluate,
    getPoint,
    points,
    extras,
    base: chain[chain.length - 1],
    root: chain[0],
  };
}

function evaluateSurface(info, u, v) {
  if (info.evaluate) {
    const result = info.evaluate.call(info.root ?? info.base, u, v);
    return toVector3(result, null);
  }
  if (info.getPoint) {
    const owner = info.root ?? info.base;
    const target = new THREE.Vector3();
    try {
      info.getPoint.call(owner, u, v, target);
      return target.clone();
    } catch (error) {
      const expectsSingleParameter = info.domainV === undefined || info.domainV === null;
      const isOptionalTargetError =
        error instanceof TypeError && typeof error.message === 'string' && error.message.includes('point.set');

      if (expectsSingleParameter && isOptionalTargetError) {
        const fallbackTarget = new THREE.Vector3();
        const result = info.getPoint.call(owner, u, fallbackTarget);
        return toVector3(result, fallbackTarget);
      }

      throw error;
    }
  }
  if (Array.isArray(info.points) && info.points.length) {
    const domainU = getDomain(info.domainU ?? { start: 0, end: 1 });
    const domainV = getDomain(info.domainV ?? { start: 0, end: 1 });
    const rows = info.points.length;
    const cols = Array.isArray(info.points[0]) ? info.points[0].length : 0;
    if (!rows || !cols) {
      return null;
    }
    const uNormalized = domainU.length <= EPSILON ? 0 : (u - domainU.start) / (domainU.end - domainU.start);
    const vNormalized = domainV.length <= EPSILON ? 0 : (v - domainV.start) / (domainV.end - domainV.start);
    const uClamped = THREE.MathUtils.clamp(uNormalized, 0, 1);
    const vClamped = THREE.MathUtils.clamp(vNormalized, 0, 1);
    const uScaled = uClamped * (cols - 1);
    const vScaled = vClamped * (rows - 1);
    const i0 = Math.floor(uScaled);
    const i1 = Math.min(i0 + 1, cols - 1);
    const j0 = Math.floor(vScaled);
    const j1 = Math.min(j0 + 1, rows - 1);
    const fu = uScaled - i0;
    const fv = vScaled - j0;
    const p00 = toVector3(info.points[j0][i0], new THREE.Vector3());
    const p01 = toVector3(info.points[j0][i1], p00);
    const p10 = toVector3(info.points[j1][i0], p00);
    const p11 = toVector3(info.points[j1][i1], p00);
    const a = p00.clone().lerp(p01, fu);
    const b = p10.clone().lerp(p11, fu);
    return a.lerp(b, fv);
  }
  return null;
}

function applyPlaneFrame(geometry, plane) {
  if (!geometry) {
    return null;
  }
  if (!plane) {
    return geometry;
  }
  const origin = toVector3(plane.origin, new THREE.Vector3());
  const xAxis = toUnitVector(plane.xAxis, new THREE.Vector3(1, 0, 0));
  const yAxis = toUnitVector(plane.yAxis, new THREE.Vector3(0, 1, 0));
  const zAxis = toUnitVector(plane.zAxis, new THREE.Vector3(0, 0, 1));

  if (xAxis && yAxis && zAxis) {
    const basis = new THREE.Matrix4().makeBasis(xAxis, yAxis, zAxis);
    geometry.applyMatrix4(basis);
  }
  geometry.translate(origin.x, origin.y, origin.z);
  return geometry;
}

function cylinderSurfaceToGeometry(info, options = {}) {
  const radiusCandidate = firstDefined(info.extras.radius, info.metadata?.radius);
  const heightCandidate = firstDefined(info.extras.height, info.metadata?.height);
  const domainV = getDomain(info.domainV ?? { start: 0, end: 1 });

  const radius = Math.max(toFiniteNumber(radiusCandidate, Math.abs(domainV.length) || 1), EPSILON);
  const heightValue = toFiniteNumber(heightCandidate, domainV.span);
  if (!Number.isFinite(heightValue) || Math.abs(heightValue) <= EPSILON) {
    return null;
  }

  const absHeight = Math.max(Math.abs(heightValue), EPSILON);
  const radialSegments = Math.max(3, options.radialSegments ?? DEFAULT_SEGMENTS.radial);
  const heightSegments = Math.max(1, options.heightSegments ?? DEFAULT_SEGMENTS.height);

  const geometry = new THREE.CylinderGeometry(radius, radius, absHeight, radialSegments, heightSegments, true);
  geometry.translate(0, heightValue >= 0 ? absHeight / 2 : -absHeight / 2, 0);
  return applyPlaneFrame(geometry, info.plane);
}

function coneSurfaceToGeometry(info, options = {}) {
  const radiusCandidate = firstDefined(info.extras.radius, info.metadata?.radius);
  const heightCandidate = firstDefined(info.extras.height, info.metadata?.height);
  const domainV = getDomain(info.domainV ?? { start: 0, end: 1 });

  const radius = Math.max(toFiniteNumber(radiusCandidate, Math.abs(domainV.length) || 1), EPSILON);
  const heightValue = toFiniteNumber(heightCandidate, domainV.span);
  if (!Number.isFinite(heightValue) || Math.abs(heightValue) <= EPSILON) {
    return null;
  }

  const absHeight = Math.max(Math.abs(heightValue), EPSILON);
  const radialSegments = Math.max(3, options.radialSegments ?? DEFAULT_SEGMENTS.radial);
  const heightSegments = Math.max(1, options.heightSegments ?? DEFAULT_SEGMENTS.height);

  const geometry = new THREE.CylinderGeometry(0, radius, absHeight, radialSegments, heightSegments, true);
  geometry.translate(0, heightValue >= 0 ? absHeight / 2 : -absHeight / 2, 0);
  return applyPlaneFrame(geometry, info.plane);
}

function sphereSurfaceToGeometry(info, options = {}) {
  const radiusCandidate = firstDefined(info.extras.radius, info.metadata?.radius);
  const radius = Math.max(toFiniteNumber(radiusCandidate, 1), EPSILON);
  const center = toVector3(firstDefined(info.extras.center, info.plane?.origin), new THREE.Vector3());
  const widthSegments = Math.max(6, options.widthSegments ?? DEFAULT_SEGMENTS.sphereWidth);
  const heightSegments = Math.max(4, options.heightSegments ?? DEFAULT_SEGMENTS.sphereHeight);
  const geometry = new THREE.SphereGeometry(radius, widthSegments, heightSegments);
  geometry.translate(center.x, center.y, center.z);
  return geometry;
}

function planeSurfaceToGeometry(info, options = {}) {
  const domainU = getDomain(info.domainU ?? { start: -0.5, end: 0.5 });
  const domainV = getDomain(info.domainV ?? { start: -0.5, end: 0.5 });
  const width = domainU.length > EPSILON ? domainU.length : 1;
  const height = domainV.length > EPSILON ? domainV.length : 1;
  const offsetX = domainU.length > EPSILON ? (domainU.start + domainU.end) / 2 : 0;
  const offsetY = domainV.length > EPSILON ? (domainV.start + domainV.end) / 2 : 0;
  const segmentsU = Math.max(1, options.widthSegments ?? DEFAULT_SEGMENTS.planeU);
  const segmentsV = Math.max(1, options.heightSegments ?? DEFAULT_SEGMENTS.planeV);
  const geometry = new THREE.PlaneGeometry(width, height, segmentsU, segmentsV);
  geometry.translate(offsetX, offsetY, 0);
  return applyPlaneFrame(geometry, info.plane);
}

function gridMetadataToGeometry(info) {
  const grid = Array.isArray(info.metadata?.grid) ? info.metadata.grid : null;
  if (!grid || grid.length < 2) {
    return null;
  }
  const rows = grid.length;
  const columns = Array.isArray(grid[0]) ? grid[0].length : 0;
  if (columns < 2) {
    return null;
  }

  const closedU = Boolean(info.metadata?.closedU);
  const closedV = Boolean(info.metadata?.closedV);
  const expandedColumns = columns + (closedU ? 1 : 0);
  const expandedRows = rows + (closedV ? 1 : 0);

  if (expandedColumns < 2 || expandedRows < 2) {
    return null;
  }

  const positions = new Float32Array(expandedColumns * expandedRows * 3);
  let ptr = 0;
  for (let row = 0; row < expandedRows; row += 1) {
    const sourceRow = grid[row % rows];
    for (let col = 0; col < expandedColumns; col += 1) {
      const source = toVector3(sourceRow[col % columns], null);
      if (!source) {
        return null;
      }
      positions[ptr] = source.x;
      positions[ptr + 1] = source.y;
      positions[ptr + 2] = source.z;
      ptr += 3;
    }
  }

  const faceCount = (expandedColumns - 1) * (expandedRows - 1) * 2;
  if (faceCount <= 0) {
    return null;
  }
  const indexArray = expandedColumns * expandedRows > 65535
    ? new Uint32Array(faceCount * 3)
    : new Uint16Array(faceCount * 3);

  let indexPtr = 0;
  for (let row = 0; row < expandedRows - 1; row += 1) {
    for (let col = 0; col < expandedColumns - 1; col += 1) {
      const a = row * expandedColumns + col;
      const b = a + 1;
      const c = a + expandedColumns;
      const d = c + 1;

      indexArray[indexPtr] = a;
      indexArray[indexPtr + 1] = c;
      indexArray[indexPtr + 2] = b;
      indexArray[indexPtr + 3] = b;
      indexArray[indexPtr + 4] = c;
      indexArray[indexPtr + 5] = d;
      indexPtr += 6;
    }
  }

  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
  geometry.setIndex(new THREE.BufferAttribute(indexArray, 1));
  geometry.computeVertexNormals();
  return geometry;
}

function sampledSurfaceToGeometry(info, options = {}) {
  const evaluator = (u, v) => evaluateSurface(info, u, v);
  const domainU = getDomain(info.domainU ?? { start: 0, end: 1 });
  const domainV = getDomain(info.domainV ?? { start: 0, end: 1 });

  if (domainU.length <= EPSILON || domainV.length <= EPSILON) {
    return null;
  }

  const segmentsU = Math.max(1, options.sampleSegmentsU ?? DEFAULT_SEGMENTS.sampleU);
  const segmentsV = Math.max(1, options.sampleSegmentsV ?? DEFAULT_SEGMENTS.sampleV);
  const columns = segmentsU + 1;
  const rows = segmentsV + 1;
  const positions = new Float32Array(columns * rows * 3);

  let index = 0;
  for (let iv = 0; iv <= segmentsV; iv += 1) {
    const fv = segmentsV ? iv / segmentsV : 0;
    const v = domainV.start + domainV.span * fv;
    for (let iu = 0; iu <= segmentsU; iu += 1) {
      const fu = segmentsU ? iu / segmentsU : 0;
      const u = domainU.start + domainU.span * fu;
      const point = evaluator(u, v);
      if (!point) {
        return null;
      }
      positions[index] = point.x;
      positions[index + 1] = point.y;
      positions[index + 2] = point.z;
      index += 3;
    }
  }

  const faceCount = segmentsU * segmentsV * 2;
  const indexArray = columns * rows > 65535
    ? new Uint32Array(faceCount * 3)
    : new Uint16Array(faceCount * 3);

  let ptr = 0;
  for (let iv = 0; iv < segmentsV; iv += 1) {
    for (let iu = 0; iu < segmentsU; iu += 1) {
      const a = iv * columns + iu;
      const b = a + 1;
      const c = a + columns;
      const d = c + 1;

      indexArray[ptr] = a;
      indexArray[ptr + 1] = c;
      indexArray[ptr + 2] = b;
      indexArray[ptr + 3] = b;
      indexArray[ptr + 4] = c;
      indexArray[ptr + 5] = d;
      ptr += 6;
    }
  }

  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
  geometry.setIndex(new THREE.BufferAttribute(indexArray, 1));
  geometry.computeVertexNormals();
  return geometry;
}

export function isSurfaceDefinition(value) {
  return isSurfaceCandidate(value);
}

export function surfaceToGeometry(input, options = {}) {
  const info = createSurfaceInfo(input);
  if (!info) {
    return null;
  }

  const type = typeof info.metadata?.type === 'string' ? info.metadata.type.toLowerCase() : null;
  let geometry = null;

  if (type === 'cylinder') {
    geometry = cylinderSurfaceToGeometry(info, options);
  } else if (type === 'cone') {
    geometry = coneSurfaceToGeometry(info, options);
  } else if (type === 'sphere') {
    geometry = sphereSurfaceToGeometry(info, options);
  } else if (type === 'plane') {
    geometry = planeSurfaceToGeometry(info, options);
  }

  if (!geometry) {
    geometry = sampledSurfaceToGeometry(info, options);
  }

  if (!geometry) {
    geometry = gridMetadataToGeometry(info, options);
  }

  if (geometry) {
    geometry.computeBoundingSphere();
  }

  return geometry;
}

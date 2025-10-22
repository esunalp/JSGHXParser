import * as THREE from 'three';

const REGISTER_SURFACE_FREEFORM_ONLY = Symbol('register-surface-freeform-only');
const REGISTER_SURFACE_ANALYSIS_ONLY = Symbol('register-surface-analysis-only');
const REGISTER_SURFACE_SUBD_ONLY = Symbol('register-surface-subd-only');

export function registerSurfacePrimitiveComponents({
  register,
  toNumber,
  toVector3,
  mode = null,
  includeFreeform = false,
}) {
  const freeformOnly = mode === REGISTER_SURFACE_FREEFORM_ONLY;
  const analysisOnly = mode === REGISTER_SURFACE_ANALYSIS_ONLY;
  const subdOnly = mode === REGISTER_SURFACE_SUBD_ONLY;
  const shouldRegisterFreeform = includeFreeform && !analysisOnly && !subdOnly;
  if (typeof register !== 'function') {
    throw new Error('register function is required to register surface primitive components.');
  }

  if (freeformOnly) {
    registerFreeformComponents();
    return;
  }

  if (analysisOnly) {
    registerAnalysisComponents();
    return;
  }

  if (subdOnly) {
    registerSubDComponents();
    return;
  }

  if (shouldRegisterFreeform) {
    registerFreeformComponents();
  }
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register surface primitive components.');
  }
  if (typeof toVector3 !== 'function') {
    throw new Error('toVector3 function is required to register surface primitive components.');
  }

  const EPSILON = 1e-9;

  function clamp(value, min, max) {
    return Math.min(Math.max(value, min), max);
  }

  function clamp01(value) {
    return clamp(value, 0, 1);
  }

  function createDomain(startValue = 0, endValue = 1) {
    const start = Number(startValue);
    const end = Number(endValue);
    const min = Math.min(start, end);
    const max = Math.max(start, end);
    const span = end - start;
    const length = max - min;
    const center = (start + end) / 2;
    return { start, end, min, max, span, length, center, dimension: 1 };
  }

  function ensurePoint(value, fallback = new THREE.Vector3()) {
    return toVector3(value, fallback.clone());
  }

  function ensureNumeric(value, fallback = 0) {
    const numeric = toNumber(value, Number.NaN);
    return Number.isFinite(numeric) ? numeric : fallback;
  }

  function ensureBoolean(value, fallback = false) {
    if (value === undefined || value === null) {
      return fallback;
    }
    if (typeof value === 'boolean') {
      return value;
    }
    if (typeof value === 'number') {
      return value !== 0;
    }
    if (typeof value === 'string') {
      const normalized = value.trim().toLowerCase();
      if (!normalized) return fallback;
      if (['true', 'yes', 'y', '1', 'on'].includes(normalized)) return true;
      if (['false', 'no', 'n', '0', 'off'].includes(normalized)) return false;
      const numeric = Number(normalized);
      if (Number.isFinite(numeric)) {
        return numeric !== 0;
      }
      return fallback;
    }
    if (Array.isArray(value)) {
      return ensureBoolean(value[value.length - 1], fallback);
    }
    if (typeof value === 'object' && 'value' in value) {
      return ensureBoolean(value.value, fallback);
    }
    return Boolean(value);
  }

  function normalizeVector(vector, fallback = new THREE.Vector3(1, 0, 0)) {
    if (!vector) {
      return fallback.clone();
    }
    const candidate = vector.clone();
    if (candidate.lengthSq() <= EPSILON) {
      return fallback.clone();
    }
    return candidate.normalize();
  }

  function orthogonalVector(vector) {
    const absX = Math.abs(vector.x);
    const absY = Math.abs(vector.y);
    const absZ = Math.abs(vector.z);
    if (absX <= absY && absX <= absZ) {
      return new THREE.Vector3(0, -vector.z, vector.y).normalize();
    }
    if (absY <= absX && absY <= absZ) {
      return new THREE.Vector3(-vector.z, 0, vector.x).normalize();
    }
    return new THREE.Vector3(-vector.y, vector.x, 0).normalize();
  }

  function normalizePlaneAxes(origin, xAxis, yAxis, zAxisHint) {
    const zAxis = normalizeVector(zAxisHint ?? xAxis.clone().cross(yAxis), new THREE.Vector3(0, 0, 1));
    const xProjected = xAxis.clone().sub(zAxis.clone().multiplyScalar(xAxis.dot(zAxis))).normalize();
    let yProjected = yAxis.clone().sub(zAxis.clone().multiplyScalar(yAxis.dot(zAxis)));
    if (yProjected.lengthSq() <= EPSILON) {
      yProjected = zAxis.clone().cross(xProjected).normalize();
    } else {
      yProjected.normalize();
    }
    const orthogonalZ = xProjected.clone().cross(yProjected);
    if (orthogonalZ.lengthSq() <= EPSILON) {
      yProjected = zAxis.clone().cross(xProjected).normalize();
    }
    return {
      origin: origin.clone(),
      xAxis: xProjected.normalize(),
      yAxis: yProjected.normalize(),
      zAxis: zAxis.normalize(),
    };
  }

  function defaultPlane() {
    return {
      origin: new THREE.Vector3(0, 0, 0),
      xAxis: new THREE.Vector3(1, 0, 0),
      yAxis: new THREE.Vector3(0, 1, 0),
      zAxis: new THREE.Vector3(0, 0, 1),
    };
  }

  function planeFromPoints(a, b, c) {
    const origin = ensurePoint(a, new THREE.Vector3());
    const ab = ensurePoint(b, origin.clone()).sub(origin.clone());
    const ac = ensurePoint(c, origin.clone()).sub(origin.clone());
    const normal = ab.clone().cross(ac);
    if (normal.lengthSq() <= EPSILON) {
      return defaultPlane();
    }
    const xAxis = ab.lengthSq() <= EPSILON ? orthogonalVector(normal) : ab.clone().normalize();
    const yAxis = normal.clone().cross(xAxis).normalize();
    return normalizePlaneAxes(origin, xAxis, yAxis, normal);
  }

  function ensurePlane(input) {
    if (!input) {
      return defaultPlane();
    }
    if (input.origin && input.xAxis && input.yAxis && input.zAxis) {
      return normalizePlaneAxes(
        ensurePoint(input.origin, new THREE.Vector3()),
        ensurePoint(input.xAxis, new THREE.Vector3(1, 0, 0)),
        ensurePoint(input.yAxis, new THREE.Vector3(0, 1, 0)),
        ensurePoint(input.zAxis, new THREE.Vector3(0, 0, 1)),
      );
    }
    if (input.origin && input.normal) {
      const origin = ensurePoint(input.origin, new THREE.Vector3());
      const normal = normalizeVector(ensurePoint(input.normal, new THREE.Vector3(0, 0, 1)), new THREE.Vector3(0, 0, 1));
      const xAxis = orthogonalVector(normal);
      const yAxis = normal.clone().cross(xAxis).normalize();
      return normalizePlaneAxes(origin, xAxis, yAxis, normal);
    }
    if (input.point && input.normal) {
      const origin = ensurePoint(input.point, new THREE.Vector3());
      const normal = normalizeVector(ensurePoint(input.normal, new THREE.Vector3(0, 0, 1)), new THREE.Vector3(0, 0, 1));
      const xAxis = orthogonalVector(normal);
      const yAxis = normal.clone().cross(xAxis).normalize();
      return normalizePlaneAxes(origin, xAxis, yAxis, normal);
    }
    if (input?.isPlane) {
      const normal = normalizeVector(input.normal ?? new THREE.Vector3(0, 0, 1), new THREE.Vector3(0, 0, 1));
      const origin = normal.clone().multiplyScalar(-ensureNumeric(input.constant, 0));
      const xAxis = orthogonalVector(normal);
      const yAxis = normal.clone().cross(xAxis).normalize();
      return normalizePlaneAxes(origin, xAxis, yAxis, normal);
    }
    if (Array.isArray(input)) {
      if (input.length >= 3) {
        return planeFromPoints(input[0], input[1], input[2]);
      }
      if (input.length === 2) {
        const origin = ensurePoint(input[0], new THREE.Vector3());
        const xAxis = ensurePoint(input[1], origin.clone()).sub(origin.clone());
        if (xAxis.lengthSq() <= EPSILON) {
          return defaultPlane();
        }
        const normal = orthogonalVector(xAxis);
        const yAxis = normal.clone().cross(xAxis).normalize();
        return normalizePlaneAxes(origin, xAxis.normalize(), yAxis, normal);
      }
      if (input.length === 1) {
        return ensurePlane(input[0]);
      }
    }
    if (typeof input === 'object' && input !== null) {
      if (input.plane) {
        return ensurePlane(input.plane);
      }
      if (input.points && Array.isArray(input.points) && input.points.length >= 3) {
        return planeFromPoints(input.points[0], input.points[1], input.points[2]);
      }
    }
    return defaultPlane();
  }

  function applyPlane(plane, u, v, w = 0) {
    const result = plane.origin.clone();
    result.add(plane.xAxis.clone().multiplyScalar(u));
    result.add(plane.yAxis.clone().multiplyScalar(v));
    result.add(plane.zAxis.clone().multiplyScalar(w));
    return result;
  }

  function planeCoordinates(pointInput, plane) {
    const point = ensurePoint(pointInput, plane.origin.clone());
    const relative = point.clone().sub(plane.origin);
    return {
      x: relative.dot(plane.xAxis),
      y: relative.dot(plane.yAxis),
      z: relative.dot(plane.zAxis),
    };
  }

  function createParametricSurface({ evaluate, domainU, domainV, plane, metadata = {} }) {
    return {
      type: 'surface',
      evaluate,
      domainU,
      domainV,
      plane,
      metadata,
    };
  }

  function evaluateSurfacePoint(surface, u, v) {
    if (!surface) {
      return null;
    }
    if (surface.evaluate) {
      const point = surface.evaluate(u, v);
      if (point?.isVector3) {
        return point.clone();
      }
      return ensurePoint(point, new THREE.Vector3());
    }
    if (typeof surface.getPoint === 'function') {
      const target = new THREE.Vector3();
      surface.getPoint(u, v, target);
      return target;
    }
    if (surface.points && Array.isArray(surface.points) && surface.points.length) {
      const rows = surface.points.length;
      const cols = Array.isArray(surface.points[0]) ? surface.points[0].length : 0;
      if (rows && cols) {
        const uClamped = clamp01(u);
        const vClamped = clamp01(v);
        const uScaled = uClamped * (cols - 1);
        const vScaled = vClamped * (rows - 1);
        const i0 = Math.floor(uScaled);
        const i1 = Math.min(i0 + 1, cols - 1);
        const j0 = Math.floor(vScaled);
        const j1 = Math.min(j0 + 1, rows - 1);
        const fu = uScaled - i0;
        const fv = vScaled - j0;
        const p00 = ensurePoint(surface.points[j0][i0], new THREE.Vector3());
        const p01 = ensurePoint(surface.points[j0][i1], p00.clone());
        const p10 = ensurePoint(surface.points[j1][i0], p00.clone());
        const p11 = ensurePoint(surface.points[j1][i1], p00.clone());
        const a = p00.clone().lerp(p01, fu);
        const b = p10.clone().lerp(p11, fu);
        return a.lerp(b, fv);
      }
    }
    return null;
  }

  function sampleSurfacePoints(surface, segmentsU = 8, segmentsV = 8) {
    if (!surface) {
      return [];
    }
    const points = [];
    const domainU = surface.domainU ?? createDomain(0, 1);
    const domainV = surface.domainV ?? createDomain(0, 1);
    for (let iu = 0; iu <= segmentsU; iu += 1) {
      const fu = iu / Math.max(segmentsU, 1);
      const u = domainU.start + (domainU.end - domainU.start) * fu;
      for (let iv = 0; iv <= segmentsV; iv += 1) {
        const fv = iv / Math.max(segmentsV, 1);
        const v = domainV.start + (domainV.end - domainV.start) * fv;
        const point = evaluateSurfacePoint(surface, u, v);
        if (point) {
          points.push(point);
        }
      }
    }
    return points;
  }

  function ensureArray(value) {
    if (value === undefined || value === null) {
      return [];
    }
    if (Array.isArray(value)) {
      return value;
    }
    return [value];
  }

  function collectPoints(input, visited = new Set()) {
    if (visited.has(input)) {
      return [];
    }
    if (input && typeof input === 'object') {
      visited.add(input);
    }
    const points = [];

    function visit(value) {
      if (value === undefined || value === null) {
        return;
      }
      if (visited.has(value)) {
        return;
      }
      if (value?.isVector3) {
        points.push(value.clone());
        return;
      }
      if (value?.isBox3) {
        const min = value.min ?? new THREE.Vector3();
        const max = value.max ?? new THREE.Vector3();
        points.push(
          new THREE.Vector3(min.x, min.y, min.z),
          new THREE.Vector3(max.x, min.y, min.z),
          new THREE.Vector3(max.x, max.y, min.z),
          new THREE.Vector3(min.x, max.y, min.z),
          new THREE.Vector3(min.x, min.y, max.z),
          new THREE.Vector3(max.x, min.y, max.z),
          new THREE.Vector3(max.x, max.y, max.z),
          new THREE.Vector3(min.x, max.y, max.z),
        );
        return;
      }
      if (value?.isBufferGeometry) {
        const position = value.getAttribute?.('position');
        if (position) {
          const vector = new THREE.Vector3();
          for (let i = 0; i < position.count; i += 1) {
            vector.fromBufferAttribute(position, i);
            points.push(vector.clone());
          }
        }
        return;
      }
      if (value?.isGeometry && value.vertices) {
        for (const vertex of value.vertices) {
          visit(vertex);
        }
        return;
      }
      if (value?.isMesh) {
        visit(value.geometry);
        return;
      }
      if (Array.isArray(value)) {
        for (const entry of value) {
          visit(entry);
        }
        return;
      }
      if (typeof value === 'object') {
        if (value.point !== undefined) {
          visit(value.point);
          return;
        }
        if (value.points) {
          visit(value.points);
          return;
        }
        if (value.position) {
          visit(value.position);
          return;
        }
        if (value.vertices) {
          visit(value.vertices);
          return;
        }
        if (value.corners) {
          visit(value.corners);
          return;
        }
        if (value.geometry) {
          visit(value.geometry);
          return;
        }
        if (value.box3) {
          visit(value.box3);
          return;
        }
        if (value.surface) {
          visit(value.surface);
          return;
        }
        if (value.curve && typeof value.curve.getPoints === 'function') {
          const samples = value.curve.getPoints?.(32);
          if (samples && samples.length) {
            visit(samples);
            return;
          }
        }
        if (value.evaluate || value.getPoint || value.points) {
          const samples = sampleSurfacePoints(value, 6, 6);
          visit(samples);
          return;
        }
        if ('x' in value || 'y' in value || 'z' in value) {
          const point = toVector3(value, null);
          if (point) {
            points.push(point);
          }
          return;
        }
        if ('center' in value && 'radius' in value) {
          const center = ensurePoint(value.center, new THREE.Vector3());
          const radius = Math.abs(ensureNumeric(value.radius, 0));
          if (radius > EPSILON) {
            points.push(center.clone().add(new THREE.Vector3(radius, 0, 0)));
            points.push(center.clone().add(new THREE.Vector3(-radius, 0, 0)));
            points.push(center.clone().add(new THREE.Vector3(0, radius, 0)));
            points.push(center.clone().add(new THREE.Vector3(0, -radius, 0)));
            points.push(center.clone().add(new THREE.Vector3(0, 0, radius)));
            points.push(center.clone().add(new THREE.Vector3(0, 0, -radius)));
            return;
          }
        }
      }
      const numeric = ensureNumeric(value, Number.NaN);
      if (Number.isFinite(numeric)) {
        points.push(new THREE.Vector3(numeric, 0, 0));
      }
    }

    visit(input);
    return points;
  }
  function computeBoundingBoxFromPoints(points) {
    if (!points || !points.length) {
      return null;
    }
    const box = new THREE.Box3();
    box.setFromPoints(points);
    if (Number.isNaN(box.min.x) || Number.isNaN(box.max.x)) {
      return null;
    }
    return box;
  }

  function createBoxDataFromPlaneExtents({ plane, min, max }) {
    const normalizedPlane = normalizePlaneAxes(plane.origin.clone(), plane.xAxis.clone(), plane.yAxis.clone(), plane.zAxis.clone());
    const localMin = new THREE.Vector3(
      Math.min(min.x, max.x),
      Math.min(min.y, max.y),
      Math.min(min.z, max.z),
    );
    const localMax = new THREE.Vector3(
      Math.max(min.x, max.x),
      Math.max(min.y, max.y),
      Math.max(min.z, max.z),
    );
    const corners = [];
    for (const x of [localMin.x, localMax.x]) {
      for (const y of [localMin.y, localMax.y]) {
        for (const z of [localMin.z, localMax.z]) {
          corners.push(applyPlane(normalizedPlane, x, y, z));
        }
      }
    }
    const box3 = new THREE.Box3();
    box3.setFromPoints(corners);
    const size = new THREE.Vector3(
      localMax.x - localMin.x,
      localMax.y - localMin.y,
      localMax.z - localMin.z,
    );
    const centerLocal = new THREE.Vector3(
      (localMin.x + localMax.x) / 2,
      (localMin.y + localMax.y) / 2,
      (localMin.z + localMax.z) / 2,
    );
    const center = applyPlane(normalizedPlane, centerLocal.x, centerLocal.y, centerLocal.z);
    let geometry = null;
    if (size.x > EPSILON && size.y > EPSILON && size.z > EPSILON) {
      geometry = new THREE.BoxGeometry(size.x, size.y, size.z);
      const rotation = new THREE.Matrix4().makeBasis(
        normalizedPlane.xAxis.clone(),
        normalizedPlane.yAxis.clone(),
        normalizedPlane.zAxis.clone(),
      );
      geometry.applyMatrix4(rotation);
      geometry.translate(center.x, center.y, center.z);
    }
    return {
      type: 'box',
      plane: normalizedPlane,
      localMin,
      localMax,
      size,
      center,
      corners,
      box3,
      geometry,
    };
  }

  function createAxisAlignedBoxFromPoints(points) {
    const box = computeBoundingBoxFromPoints(points);
    if (!box) {
      return null;
    }
    return createBoxDataFromPlaneExtents({
      plane: defaultPlane(),
      min: box.min,
      max: box.max,
    });
  }

  function unionBoxes(boxes) {
    const valid = boxes.filter(Boolean);
    if (!valid.length) {
      return null;
    }
    const globalBox = new THREE.Box3();
    for (const entry of valid) {
      const entryBox = entry.box3 ?? computeBoundingBoxFromPoints(entry.corners);
      if (entryBox) {
        globalBox.union(entryBox);
      }
    }
    if (!globalBox.isBox3) {
      return null;
    }
    return createBoxDataFromPlaneExtents({ plane: defaultPlane(), min: globalBox.min, max: globalBox.max });
  }

  function ensureDomainInput(input, fallbackStart = 0, fallbackEnd = 1) {
    if (input === undefined || input === null) {
      return createDomain(fallbackStart, fallbackEnd);
    }
    if (input.dimension === 1 && typeof input.start !== 'undefined' && typeof input.end !== 'undefined') {
      return createDomain(input.start, input.end);
    }
    if (typeof input === 'object') {
      const start = ensureNumeric(input.start ?? input.min ?? input.a ?? input.from ?? input.t0, Number.NaN);
      const end = ensureNumeric(input.end ?? input.max ?? input.b ?? input.to ?? input.t1, Number.NaN);
      if (Number.isFinite(start) && Number.isFinite(end)) {
        return createDomain(start, end);
      }
      if (Array.isArray(input) && input.length >= 2) {
        return createDomain(input[0], input[1]);
      }
    }
    if (Array.isArray(input)) {
      if (input.length >= 2) {
        return createDomain(input[0], input[1]);
      }
      if (input.length === 1) {
        return ensureDomainInput(input[0], fallbackStart, fallbackEnd);
      }
    }
    const numeric = ensureNumeric(input, Number.NaN);
    if (Number.isFinite(numeric)) {
      return createDomain(numeric, numeric);
    }
    return createDomain(fallbackStart, fallbackEnd);
  }

  function createCylinderSurface(plane, radius, height) {
    const normalizedPlane = normalizePlaneAxes(plane.origin.clone(), plane.xAxis.clone(), plane.yAxis.clone(), plane.zAxis.clone());
    const domainU = createDomain(0, Math.PI * 2);
    const domainV = createDomain(0, height);
    const r = Math.max(Math.abs(radius), EPSILON);
    const evaluate = (u, v) => {
      const angle = clamp(u, domainU.min, domainU.max);
      const start = Math.min(domainV.start, domainV.end);
      const end = Math.max(domainV.start, domainV.end);
      const heightValue = clamp(v, start, end);
      const cos = Math.cos(angle);
      const sin = Math.sin(angle);
      return normalizedPlane.origin.clone()
        .add(normalizedPlane.xAxis.clone().multiplyScalar(r * cos))
        .add(normalizedPlane.yAxis.clone().multiplyScalar(r * sin))
        .add(normalizedPlane.zAxis.clone().multiplyScalar(heightValue));
    };
    return createParametricSurface({
      evaluate,
      domainU,
      domainV,
      plane: normalizedPlane,
      metadata: { type: 'cylinder', radius: r, height },
    });
  }

  function createConeSurface(plane, radius, height) {
    const normalizedPlane = normalizePlaneAxes(plane.origin.clone(), plane.xAxis.clone(), plane.yAxis.clone(), plane.zAxis.clone());
    const domainU = createDomain(0, Math.PI * 2);
    const domainV = createDomain(0, height);
    const r = Math.max(Math.abs(radius), EPSILON);
    const evaluate = (u, v) => {
      const angle = clamp(u, domainU.min, domainU.max);
      const start = domainV.start;
      const end = domainV.end;
      const min = Math.min(start, end);
      const max = Math.max(start, end);
      const clamped = clamp(v, min, max);
      const denominator = end - start;
      const ratio = Math.abs(denominator) <= EPSILON ? 0 : (clamped - start) / denominator;
      const t = clamp(ratio, 0, 1);
      const currentRadius = r * (1 - t);
      const cos = Math.cos(angle);
      const sin = Math.sin(angle);
      return normalizedPlane.origin.clone()
        .add(normalizedPlane.xAxis.clone().multiplyScalar(currentRadius * cos))
        .add(normalizedPlane.yAxis.clone().multiplyScalar(currentRadius * sin))
        .add(normalizedPlane.zAxis.clone().multiplyScalar(clamped));
    };
    return createParametricSurface({
      evaluate,
      domainU,
      domainV,
      plane: normalizedPlane,
      metadata: { type: 'cone', radius: r, height },
    });
  }
  function projectPointsToPlaneBounds(points, plane) {
    if (!points.length) {
      return null;
    }
    let minX = Number.POSITIVE_INFINITY;
    let minY = Number.POSITIVE_INFINITY;
    let minZ = Number.POSITIVE_INFINITY;
    let maxX = Number.NEGATIVE_INFINITY;
    let maxY = Number.NEGATIVE_INFINITY;
    let maxZ = Number.NEGATIVE_INFINITY;
    for (const point of points) {
      const coords = planeCoordinates(point, plane);
      if (coords.x < minX) minX = coords.x;
      if (coords.y < minY) minY = coords.y;
      if (coords.z < minZ) minZ = coords.z;
      if (coords.x > maxX) maxX = coords.x;
      if (coords.y > maxY) maxY = coords.y;
      if (coords.z > maxZ) maxZ = coords.z;
    }
    if (!Number.isFinite(minX) || !Number.isFinite(maxX)) {
      return null;
    }
    return {
      min: new THREE.Vector3(minX, minY, minZ),
      max: new THREE.Vector3(maxX, maxY, maxZ),
    };
  }

  function expandPlaneBounds(bounds, inflateX = 0, inflateY = inflateX) {
    if (!bounds) {
      return null;
    }
    const expandedMin = bounds.min.clone();
    const expandedMax = bounds.max.clone();
    expandedMin.x -= inflateX;
    expandedMin.y -= inflateY;
    expandedMax.x += inflateX;
    expandedMax.y += inflateY;
    return { min: expandedMin, max: expandedMax };
  }

  function combinePlaneBounds(boundsList) {
    const valid = boundsList.filter(Boolean);
    if (!valid.length) {
      return null;
    }
    const min = new THREE.Vector3(Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY, Number.POSITIVE_INFINITY);
    const max = new THREE.Vector3(Number.NEGATIVE_INFINITY, Number.NEGATIVE_INFINITY, Number.NEGATIVE_INFINITY);
    for (const bounds of valid) {
      if (bounds.min.x < min.x) min.x = bounds.min.x;
      if (bounds.min.y < min.y) min.y = bounds.min.y;
      if (bounds.min.z < min.z) min.z = bounds.min.z;
      if (bounds.max.x > max.x) max.x = bounds.max.x;
      if (bounds.max.y > max.y) max.y = bounds.max.y;
      if (bounds.max.z > max.z) max.z = bounds.max.z;
    }
    if (!Number.isFinite(min.x) || !Number.isFinite(max.x)) {
      return null;
    }
    return { min, max };
  }

  function createPlanarSurfaceFromBounds(plane, minX, maxX, minY, maxY) {
    const normalizedPlane = normalizePlaneAxes(plane.origin.clone(), plane.xAxis.clone(), plane.yAxis.clone(), plane.zAxis.clone());
    const domainU = createDomain(minX, maxX);
    const domainV = createDomain(minY, maxY);
    const evaluate = (u, v) => {
      const clampedU = clamp(u, domainU.min, domainU.max);
      const clampedV = clamp(v, domainV.min, domainV.max);
      return applyPlane(normalizedPlane, clampedU, clampedV, 0);
    };
    return createParametricSurface({
      evaluate,
      domainU,
      domainV,
      plane: normalizedPlane,
      metadata: { type: 'plane' },
    });
  }

  function createPlanarSurfaceFromSize(plane, sizeX, sizeY) {
    const halfX = Math.abs(sizeX) / 2;
    const halfY = Math.abs(sizeY) / 2;
    return createPlanarSurfaceFromBounds(plane, -halfX, halfX, -halfY, halfY);
  }

  function createSphereSurface(center, plane, radius) {
    const normalizedPlane = normalizePlaneAxes(center.clone(), plane.xAxis.clone(), plane.yAxis.clone(), plane.zAxis.clone());
    const r = Math.max(Math.abs(radius), EPSILON);
    const domainU = createDomain(0, Math.PI * 2);
    const domainV = createDomain(0, Math.PI);
    const evaluate = (u, v) => {
      const angleU = clamp(u, domainU.min, domainU.max);
      const angleV = clamp(v, domainV.min, domainV.max);
      const sinV = Math.sin(angleV);
      const cosV = Math.cos(angleV);
      const cosU = Math.cos(angleU);
      const sinU = Math.sin(angleU);
      const offset = normalizedPlane.xAxis.clone().multiplyScalar(r * sinV * cosU)
        .add(normalizedPlane.yAxis.clone().multiplyScalar(r * sinV * sinU))
        .add(normalizedPlane.zAxis.clone().multiplyScalar(r * cosV));
      return normalizedPlane.origin.clone().add(offset);
    };
    return createParametricSurface({
      evaluate,
      domainU,
      domainV,
      plane: normalizedPlane,
      metadata: { type: 'sphere', radius: r },
    });
  }

  function solveLinearSystem(matrix, vector) {
    const size = vector.length;
    const augmented = matrix.map((row, i) => [...row, vector[i]]);
    for (let i = 0; i < size; i += 1) {
      let pivot = augmented[i][i];
      let pivotRow = i;
      for (let j = i + 1; j < size; j += 1) {
        if (Math.abs(augmented[j][i]) > Math.abs(pivot)) {
          pivot = augmented[j][i];
          pivotRow = j;
        }
      }
      if (Math.abs(pivot) <= EPSILON) {
        return null;
      }
      if (pivotRow !== i) {
        const temp = augmented[i];
        augmented[i] = augmented[pivotRow];
        augmented[pivotRow] = temp;
      }
      const pivotValue = augmented[i][i];
      for (let j = i; j <= size; j += 1) {
        augmented[i][j] /= pivotValue;
      }
      for (let row = 0; row < size; row += 1) {
        if (row === i) continue;
        const factor = augmented[row][i];
        for (let col = i; col <= size; col += 1) {
          augmented[row][col] -= factor * augmented[i][col];
        }
      }
    }
    return augmented.map((row) => row[size]);
  }

  function fitSphereToPoints(points) {
    if (!points || points.length < 4) {
      return null;
    }
    const A = [];
    const b = [];
    for (const pt of points) {
      const x = pt.x;
      const y = pt.y;
      const z = pt.z;
      A.push([x, y, z, 1]);
      b.push(-(x * x + y * y + z * z));
    }
    const AT = A[0].map((_, colIndex) => A.map((row) => row[colIndex]));
    const ATA = AT.map((row, i) => row.map((_, j) => row.reduce((sum, value, idx) => sum + value * AT[j][idx], 0)));
    const ATb = AT.map((row) => row.reduce((sum, value, idx) => sum + value * b[idx], 0));
    const solution = solveLinearSystem(ATA, ATb);
    if (!solution) {
      return null;
    }
    const [D, E, F, G] = solution;
    const center = new THREE.Vector3(-D / 2, -E / 2, -F / 2);
    const radiusSquared = center.lengthSq() - G;
    if (radiusSquared <= 0) {
      return null;
    }
    const radius = Math.sqrt(radiusSquared);
    return { center, radius };
  }

  function sphereFromFourPoints(points) {
    const unique = points.filter((_, index, arr) => arr.findIndex((pt) => pt.distanceToSquared(points[index]) <= EPSILON) === index);
    const base = unique.length >= 4 ? unique.slice(0, 4) : points.slice(0, 4);
    return fitSphereToPoints(base);
  }

  function computeSphereFromPoints(points) {
    if (!points || !points.length) {
      return null;
    }
    let result = null;
    if (points.length >= 4) {
      result = fitSphereToPoints(points);
      if (!result) {
        result = sphereFromFourPoints(points);
      }
    } else if (points.length === 3) {
      result = fitSphereToPoints(points);
    }
    if (!result) {
      const sphere = new THREE.Sphere();
      sphere.setFromPoints(points);
      if (Number.isFinite(sphere.radius) && sphere.radius > EPSILON) {
        result = { center: sphere.center.clone(), radius: sphere.radius };
      }
    }
    return result;
  }

  function createSphereResult(center, radius, planeHint) {
    const basePlane = planeHint ? ensurePlane({ origin: center, normal: planeHint.zAxis ?? planeHint.normal ?? planeHint }) : normalizePlaneAxes(center.clone(), new THREE.Vector3(1, 0, 0), new THREE.Vector3(0, 1, 0));
    const surface = createSphereSurface(center, basePlane, radius);
    return { center, radius, surface };
  }

  function computeBoxesForContent(contentInput, { plane = null, union = false } = {}) {
    const items = ensureArray(contentInput);
    const worldBoxes = [];
    const planeBoxes = [];
    const orientationPlane = plane ? ensurePlane(plane) : null;
    const allPoints = [];
    const planeBoundsList = [];
    for (const item of items) {
      const points = collectPoints(item);
      if (!points.length) {
        continue;
      }
      allPoints.push(...points);
      const worldBox = createAxisAlignedBoxFromPoints(points);
      if (worldBox) {
        worldBoxes.push(worldBox);
      }
      if (orientationPlane) {
        const bounds = projectPointsToPlaneBounds(points, orientationPlane);
        if (bounds) {
          planeBoundsList.push(bounds);
          const box = createBoxDataFromPlaneExtents({ plane: orientationPlane, min: bounds.min, max: bounds.max });
          if (box) {
            planeBoxes.push(box);
          }
        }
      }
    }
    const result = {
      worldBoxes: union ? [] : worldBoxes.filter(Boolean),
      planeBoxes: union ? [] : planeBoxes.filter(Boolean),
    };
    if (union && allPoints.length) {
      const box = createAxisAlignedBoxFromPoints(allPoints);
      if (box) {
        result.worldBoxes.push(box);
      }
    }
    if (union && orientationPlane && planeBoundsList.length) {
      const combinedBounds = combinePlaneBounds(planeBoundsList);
      if (combinedBounds) {
        const box = createBoxDataFromPlaneExtents({ plane: orientationPlane, min: combinedBounds.min, max: combinedBounds.max });
        if (box) {
          result.planeBoxes.push(box);
        }
      }
    }
    return result;
  }

  function createBoxFromPlaneDimensions(plane, sizeX, sizeY, sizeZ) {
    const halfX = Math.abs(sizeX) / 2;
    const halfY = Math.abs(sizeY) / 2;
    const halfZ = Math.abs(sizeZ) / 2;
    const minZ = sizeZ >= 0 ? -halfZ : halfZ;
    const maxZ = sizeZ >= 0 ? halfZ : -halfZ;
    return createBoxDataFromPlaneExtents({
      plane,
      min: new THREE.Vector3(-halfX, -halfY, minZ),
      max: new THREE.Vector3(halfX, halfY, maxZ),
    });
  }

  function createBoxFromDomains(plane, domainX, domainY, domainZ) {
    return createBoxDataFromPlaneExtents({
      plane,
      min: new THREE.Vector3(domainX.min, domainY.min, domainZ.min),
      max: new THREE.Vector3(domainX.max, domainY.max, domainZ.max),
    });
  }

  function extractRectangleData(rectangleInput) {
    if (!rectangleInput) {
      return null;
    }
    if (rectangleInput.type === 'rectangle' && rectangleInput.plane) {
      return {
        plane: ensurePlane(rectangleInput.plane),
        width: ensureNumeric(rectangleInput.width, 0),
        height: ensureNumeric(rectangleInput.height, 0),
        corners: rectangleInput.corners ? rectangleInput.corners.map((corner) => ensurePoint(corner, new THREE.Vector3())) : null,
      };
    }
    if (rectangleInput.corners && Array.isArray(rectangleInput.corners) && rectangleInput.corners.length >= 3) {
      const plane = planeFromPoints(rectangleInput.corners[0], rectangleInput.corners[1], rectangleInput.corners[2]);
      const coords = rectangleInput.corners.map((corner) => planeCoordinates(corner, plane));
      const xs = coords.map((c) => c.x);
      const ys = coords.map((c) => c.y);
      return {
        plane,
        width: Math.abs(Math.max(...xs) - Math.min(...xs)),
        height: Math.abs(Math.max(...ys) - Math.min(...ys)),
        corners: rectangleInput.corners.map((corner) => ensurePoint(corner, new THREE.Vector3())),
      };
    }
    return null;
  }

  function createBoxFromRectangle(rectangleData, height) {
    if (!rectangleData) {
      return null;
    }
    const plane = rectangleData.plane ?? defaultPlane();
    const coords = rectangleData.corners ? rectangleData.corners.map((corner) => planeCoordinates(corner, plane)) : [
      new THREE.Vector3(-rectangleData.width / 2, -rectangleData.height / 2, 0),
      new THREE.Vector3(rectangleData.width / 2, rectangleData.height / 2, 0),
    ];
    let minX = Number.POSITIVE_INFINITY;
    let maxX = Number.NEGATIVE_INFINITY;
    let minY = Number.POSITIVE_INFINITY;
    let maxY = Number.NEGATIVE_INFINITY;
    for (const coord of coords) {
      if (coord.x < minX) minX = coord.x;
      if (coord.x > maxX) maxX = coord.x;
      if (coord.y < minY) minY = coord.y;
      if (coord.y > maxY) maxY = coord.y;
    }
    const minZ = Math.min(0, height);
    const maxZ = Math.max(0, height);
    const min = new THREE.Vector3(minX, minY, minZ);
    const max = new THREE.Vector3(maxX, maxY, maxZ);
    return createBoxDataFromPlaneExtents({ plane, min, max });
  }

  function wrapSurface(surface, extras = {}) {
    if (!surface) {
      return null;
    }
    return {
      surface,
      evaluate: surface.evaluate,
      domainU: surface.domainU,
      domainV: surface.domainV,
      plane: surface.plane,
      metadata: surface.metadata,
      ...extras,
    };
  }

  const DEFAULT_CURVE_SEGMENTS = 32;

  function computePolylineLength(points, closed = false) {
    if (!points || points.length < 2) {
      return 0;
    }
    let length = 0;
    for (let i = 1; i < points.length; i += 1) {
      length += points[i].distanceTo(points[i - 1]);
    }
    if (closed) {
      length += points[0].distanceTo(points[points.length - 1]);
    }
    return length;
  }

  function resamplePolyline(pointsInput, count, { closed = false } = {}) {
    if (!pointsInput || !pointsInput.length) {
      return [];
    }
    const points = pointsInput.map((pt) => ensurePoint(pt, new THREE.Vector3()));
    if (count <= 1) {
      return [points[0].clone()];
    }
    const base = points.map((pt) => pt.clone());
    if (closed) {
      base.push(points[0].clone());
    }
    const distances = [0];
    for (let i = 1; i < base.length; i += 1) {
      distances[i] = distances[i - 1] + base[i].distanceTo(base[i - 1]);
    }
    const total = distances[distances.length - 1];
    if (total <= EPSILON) {
      return Array.from({ length: count }, () => base[0].clone());
    }
    const samples = [];
    const denom = closed ? count : Math.max(count - 1, 1);
    for (let i = 0; i < count; i += 1) {
      const target = (denom === 0 ? 0 : (i / denom)) * total;
      let index = 1;
      while (index < distances.length && distances[index] < target) {
        index += 1;
      }
      const prevIndex = Math.max(0, index - 1);
      const nextIndex = Math.min(base.length - 1, index);
      const prevDist = distances[prevIndex];
      const nextDist = distances[nextIndex];
      const segment = nextDist - prevDist;
      const factor = segment <= EPSILON ? 0 : (target - prevDist) / segment;
      const point = base[prevIndex].clone().lerp(base[nextIndex], factor);
      samples.push(point);
    }
    return samples;
  }

  function sampleCurvePoints(curveInput, segments = DEFAULT_CURVE_SEGMENTS, visited = new Set()) {
    if (curveInput === undefined || curveInput === null) {
      return { points: [], closed: false };
    }
    if (visited.has(curveInput)) {
      return { points: [], closed: false };
    }
    if (typeof curveInput === 'object') {
      visited.add(curveInput);
    }

    function normalizeSamples(points, closedFlag = false) {
      if (!points || !points.length) {
        return { points: [], closed: Boolean(closedFlag) };
      }
      return {
        points: points.map((pt) => ensurePoint(pt, new THREE.Vector3())),
        closed: Boolean(closedFlag),
      };
    }

    if (curveInput.curve && curveInput.curve !== curveInput) {
      const result = sampleCurvePoints(curveInput.curve, segments, visited);
      if (result.points.length) {
        if (curveInput.closed !== undefined) {
          result.closed = Boolean(curveInput.closed);
        }
        return result;
      }
    }

    if (curveInput.polyline && curveInput.polyline !== curveInput) {
      const result = sampleCurvePoints(curveInput.polyline, segments, visited);
      if (result.points.length) {
        return result;
      }
    }

    if (curveInput.path && typeof curveInput.path.getPoints === 'function') {
      return normalizeSamples(curveInput.path.getPoints(Math.max(segments, 8)), curveInput.closed ?? false);
    }

    if (typeof curveInput.getPoints === 'function') {
      return normalizeSamples(curveInput.getPoints(Math.max(segments, 8)), curveInput.closed ?? curveInput.isClosed);
    }

    if (typeof curveInput.getSpacedPoints === 'function') {
      return normalizeSamples(curveInput.getSpacedPoints(Math.max(segments, 8)), curveInput.closed ?? curveInput.isClosed);
    }

    if (typeof curveInput.getPoint === 'function') {
      const count = Math.max(segments, 8);
      const pts = [];
      for (let i = 0; i <= count; i += 1) {
        const t = i / count;
        const point = curveInput.getPoint(t);
        if (point) {
          pts.push(ensurePoint(point, new THREE.Vector3()));
        }
      }
      return normalizeSamples(pts, curveInput.closed ?? false);
    }

    if (Array.isArray(curveInput.points) && curveInput.points.length) {
      return normalizeSamples(curveInput.points, curveInput.closed ?? curveInput.isClosed ?? false);
    }

    if (Array.isArray(curveInput.vertices) && curveInput.vertices.length) {
      return normalizeSamples(curveInput.vertices, curveInput.closed ?? curveInput.isClosed ?? false);
    }

    if (Array.isArray(curveInput) && curveInput.length) {
      return normalizeSamples(curveInput, curveInput.closed ?? false);
    }

    if (curveInput.start !== undefined && curveInput.end !== undefined) {
      const start = ensurePoint(curveInput.start, new THREE.Vector3());
      const end = ensurePoint(curveInput.end, start.clone().add(new THREE.Vector3(1, 0, 0)));
      return normalizeSamples([start, end], false);
    }

    if (curveInput.center !== undefined && curveInput.radius !== undefined) {
      const center = ensurePoint(curveInput.center, new THREE.Vector3());
      const radius = Math.abs(ensureNumeric(curveInput.radius, 1));
      if (radius > EPSILON) {
        const plane = curveInput.plane ? ensurePlane(curveInput.plane) : defaultPlane();
        const xAxis = normalizeVector(plane.xAxis.clone(), new THREE.Vector3(1, 0, 0));
        const yAxis = normalizeVector(plane.yAxis.clone(), new THREE.Vector3(0, 1, 0));
        const pts = [];
        const count = Math.max(segments, 32);
        for (let i = 0; i < count; i += 1) {
          const angle = (i / count) * Math.PI * 2;
          const point = center.clone()
            .add(xAxis.clone().multiplyScalar(radius * Math.cos(angle)))
            .add(yAxis.clone().multiplyScalar(radius * Math.sin(angle)));
          pts.push(point);
        }
        return normalizeSamples(pts, true);
      }
    }

    if (curveInput.points && typeof curveInput.points === 'function') {
      try {
        const pts = curveInput.points(Math.max(segments, 8));
        if (Array.isArray(pts) && pts.length) {
          return normalizeSamples(pts, curveInput.closed ?? false);
        }
      } catch (error) {
        // ignore failures
      }
    }

    return { points: [], closed: false };
  }

  function createGridSurface(rowsInput, { metadata = {}, closedU = false, closedV = false } = {}) {
    if (!rowsInput || !rowsInput.length) {
      return null;
    }
    const rows = rowsInput.map((row) => row.map((pt) => ensurePoint(pt, new THREE.Vector3())));
    const columnCount = rows.reduce((max, row) => Math.max(max, row.length), 0);
    if (rows.length < 2 || columnCount < 2) {
      return null;
    }
    const normalizedRows = rows.map((row) => {
      if (row.length === columnCount) {
        return row.map((pt) => pt.clone());
      }
      return resamplePolyline(row, columnCount, { closed: closedU });
    });
    const domainU = createDomain(0, 1);
    const domainV = createDomain(0, 1);
    const maxUIndex = columnCount - 1;
    const maxVIndex = normalizedRows.length - 1;
    const evaluate = (uInput, vInput) => {
      const u = clamp(uInput, domainU.min, domainU.max);
      const v = clamp(vInput, domainV.min, domainV.max);
      const scaledU = (u - domainU.min) / (domainU.max - domainU.min || 1);
      const scaledV = (v - domainV.min) / (domainV.max - domainV.min || 1);
      const targetU = scaledU * (closedU ? columnCount : maxUIndex);
      const targetV = scaledV * (closedV ? normalizedRows.length : maxVIndex);
      const i0 = closedU ? Math.floor(targetU) % columnCount : Math.min(Math.floor(targetU), maxUIndex);
      const j0 = closedV ? Math.floor(targetV) % normalizedRows.length : Math.min(Math.floor(targetV), maxVIndex);
      const du = targetU - Math.floor(targetU);
      const dv = targetV - Math.floor(targetV);
      const i1 = closedU ? (i0 + 1) % columnCount : Math.min(i0 + 1, maxUIndex);
      const j1 = closedV ? (j0 + 1) % normalizedRows.length : Math.min(j0 + 1, maxVIndex);
      const p00 = normalizedRows[j0][i0] ?? normalizedRows[j0][0];
      const p10 = normalizedRows[j0][i1] ?? normalizedRows[j0][i0];
      const p01 = normalizedRows[j1][i0] ?? normalizedRows[j0][i0];
      const p11 = normalizedRows[j1][i1] ?? normalizedRows[j1][i0];
      const a = p00.clone().lerp(p10, du);
      const b = p01.clone().lerp(p11, du);
      return a.lerp(b, dv);
    };
    return createParametricSurface({
      evaluate,
      domainU,
      domainV,
      plane: null,
      metadata: {
        ...metadata,
        grid: normalizedRows,
        closedU,
        closedV,
      },
    });
  }

  function createLoftSurfaceFromSections(sections, { metadata = {}, closed = false } = {}) {
    if (!sections || !sections.length) {
      return null;
    }
    const filtered = sections.filter((section) => section?.points?.length >= 2);
    if (!filtered.length) {
      return null;
    }
    const segmentCount = filtered.reduce((max, section) => Math.max(max, section.points.length), 0);
    const count = Math.max(segmentCount, 8);
    const rows = filtered.map((section) => resamplePolyline(section.points, count, { closed: closed || section.closed }));
    return createGridSurface(rows, {
      metadata: {
        ...metadata,
        sections: filtered.map((section) => ({
          closed: section.closed ?? false,
          count: section.points.length,
        })),
      },
      closedU: closed || filtered.some((section) => section.closed),
      closedV: false,
    });
  }

  function createSurfaceFromPointGrid(points, countU, { metadata = {} } = {}) {
    const total = points.length;
    const uCount = Math.max(2, Math.round(countU || 0));
    if (!total || !uCount) {
      return null;
    }
    const vCount = Math.max(2, Math.floor(total / uCount));
    if (vCount < 2) {
      return null;
    }
    const rows = [];
    for (let v = 0; v < vCount; v += 1) {
      const row = [];
      for (let u = 0; u < uCount; u += 1) {
        const index = v * uCount + u;
        row.push(points[index] ? ensurePoint(points[index], new THREE.Vector3()) : new THREE.Vector3());
      }
      rows.push(row);
    }
    return createGridSurface(rows, { metadata });
  }

  function computeCentroid(points) {
    if (!points || !points.length) {
      return new THREE.Vector3();
    }
    const sum = new THREE.Vector3();
    for (const point of points) {
      sum.add(point);
    }
    return sum.multiplyScalar(1 / points.length);
  }

  function extractProfileData(profileInput, { segments = DEFAULT_CURVE_SEGMENTS } = {}) {
    const sample = sampleCurvePoints(profileInput, segments);
    let points = sample.points;
    let closed = sample.closed;
    if (!points.length) {
      points = collectPoints(profileInput);
      closed = false;
    }
    if (!points.length) {
      return {
        plane: defaultPlane(),
        coords: [],
        points: [],
        centroid: new THREE.Vector3(),
        closed: false,
      };
    }
    const centroid = computeCentroid(points);
    let plane;
    if (points.length >= 3) {
      plane = planeFromPoints(points[0], points[1], points[2]);
    } else if (points.length === 2) {
      const fallback = points[0].clone().add(orthogonalVector(points[1].clone().sub(points[0])));
      plane = planeFromPoints(points[0], points[1], fallback);
    } else {
      plane = defaultPlane();
    }
    const normalizedPlane = normalizePlaneAxes(
      centroid.clone(),
      plane.xAxis.clone(),
      plane.yAxis.clone(),
      plane.zAxis.clone(),
    );
    const coords = points.map((point) => {
      const relative = point.clone().sub(normalizedPlane.origin);
      return new THREE.Vector2(
        relative.dot(normalizedPlane.xAxis),
        relative.dot(normalizedPlane.yAxis),
      );
    });
    return {
      plane: normalizedPlane,
      coords,
      points: points.map((pt) => pt.clone()),
      centroid,
      closed,
    };
  }

  function createFramesAlongPath(pathPointsInput, basePlane, { closed = false } = {}) {
    const pathPoints = pathPointsInput.map((pt) => ensurePoint(pt, new THREE.Vector3()));
    if (!pathPoints.length) {
      return [];
    }
    const frames = [];
    let previousXAxis = basePlane?.xAxis?.clone() ?? new THREE.Vector3(1, 0, 0);
    let previousYAxis = basePlane?.yAxis?.clone() ?? new THREE.Vector3(0, 1, 0);
    for (let i = 0; i < pathPoints.length; i += 1) {
      const current = pathPoints[i];
      const prev = pathPoints[i - 1] ?? (closed ? pathPoints[pathPoints.length - 1] : pathPoints[i]);
      const next = pathPoints[i + 1] ?? (closed ? pathPoints[(i + 1) % pathPoints.length] : pathPoints[i]);
      const tangent = next.clone().sub(prev).normalize();
      const zAxis = normalizeVector(tangent, basePlane?.zAxis ?? new THREE.Vector3(0, 0, 1));
      let xAxis = previousXAxis.clone();
      xAxis.sub(zAxis.clone().multiplyScalar(xAxis.dot(zAxis)));
      if (xAxis.lengthSq() <= EPSILON) {
        xAxis = basePlane?.xAxis ? basePlane.xAxis.clone() : orthogonalVector(zAxis);
      }
      xAxis.normalize();
      let yAxis = zAxis.clone().cross(xAxis);
      if (yAxis.lengthSq() <= EPSILON) {
        yAxis = previousYAxis.clone();
        yAxis.sub(zAxis.clone().multiplyScalar(yAxis.dot(zAxis)));
        if (yAxis.lengthSq() <= EPSILON) {
          yAxis = zAxis.clone().cross(xAxis).normalize();
        } else {
          yAxis.normalize();
        }
      } else {
        yAxis.normalize();
      }
      if (frames.length) {
        const prevFrame = frames[frames.length - 1];
        if (prevFrame.xAxis.dot(xAxis) < 0) {
          xAxis.negate();
        }
        if (prevFrame.yAxis.dot(yAxis) < 0) {
          yAxis.negate();
        }
      }
      frames.push({ origin: current.clone(), xAxis, yAxis, zAxis });
      previousXAxis = xAxis.clone();
      previousYAxis = yAxis.clone();
    }
    return frames;
  }

  function createExtrusionSurface(profileData, pathFrames, { metadata = {}, closedPath = false } = {}) {
    if (!profileData || !profileData.coords.length || !pathFrames.length) {
      return null;
    }
    const rows = pathFrames.map((frame) => profileData.coords.map((coord) => {
      const point = frame.origin.clone()
        .add(frame.xAxis.clone().multiplyScalar(coord.x))
        .add(frame.yAxis.clone().multiplyScalar(coord.y));
      return point;
    }));
    return createGridSurface(rows, {
      metadata: {
        ...metadata,
        frames: pathFrames.map((frame) => ({
          origin: frame.origin.clone(),
          xAxis: frame.xAxis.clone(),
          yAxis: frame.yAxis.clone(),
          zAxis: frame.zAxis.clone(),
        })),
      },
      closedU: profileData.closed,
      closedV: closedPath,
    });
  }

  function createPipeSurface(pathFrames, radii, { metadata = {}, radialSegments = 24, closed = false } = {}) {
    if (!pathFrames || !pathFrames.length) {
      return null;
    }
    const rows = pathFrames.map((frame, index) => {
      const radius = Math.max(Math.abs(radii[index] ?? radii[radii.length - 1] ?? 0), EPSILON);
      const row = [];
      for (let i = 0; i < radialSegments; i += 1) {
        const angle = (i / radialSegments) * Math.PI * 2;
        const x = Math.cos(angle) * radius;
        const y = Math.sin(angle) * radius;
        const point = frame.origin.clone()
          .add(frame.xAxis.clone().multiplyScalar(x))
          .add(frame.yAxis.clone().multiplyScalar(y));
        row.push(point);
      }
      return row;
    });
    return createGridSurface(rows, {
      metadata,
      closedU: true,
      closedV: closed,
    });
  }

  function parseAxisInput(axisInput, fallbackOrigin = new THREE.Vector3(), fallbackDirection = new THREE.Vector3(0, 0, 1)) {
    if (!axisInput) {
      const direction = normalizeVector(fallbackDirection.clone(), new THREE.Vector3(0, 0, 1));
      return { origin: fallbackOrigin.clone(), direction, vector: direction.clone(), length: 1 };
    }
    if (axisInput.mode === 'vector' && axisInput.origin && axisInput.direction) {
      const origin = ensurePoint(axisInput.origin, fallbackOrigin.clone());
      const direction = normalizeVector(ensurePoint(axisInput.direction, fallbackDirection.clone()), fallbackDirection.clone());
      return { origin, direction, vector: direction.clone(), length: 1 };
    }
    if (axisInput.line) {
      return parseAxisInput(axisInput.line, fallbackOrigin, fallbackDirection);
    }
    if (axisInput.origin && axisInput.direction) {
      const origin = ensurePoint(axisInput.origin, fallbackOrigin.clone());
      const vector = ensurePoint(axisInput.direction, fallbackDirection.clone());
      const length = vector.length();
      const direction = length > EPSILON ? vector.clone().normalize() : fallbackDirection.clone().normalize();
      return { origin, direction, vector, length: length > EPSILON ? length : 1 };
    }
    if (axisInput.start && axisInput.end) {
      const start = ensurePoint(axisInput.start, fallbackOrigin.clone());
      const end = ensurePoint(axisInput.end, start.clone().add(fallbackDirection.clone()));
      const vector = end.clone().sub(start);
      const length = vector.length();
      const direction = length > EPSILON ? vector.clone().normalize() : fallbackDirection.clone().normalize();
      return { origin: start, direction, vector, length: length > EPSILON ? length : 1 };
    }
    if (Array.isArray(axisInput) && axisInput.length >= 2) {
      return parseAxisInput({ start: axisInput[0], end: axisInput[1] }, fallbackOrigin, fallbackDirection);
    }
    if (axisInput.point && axisInput.vector) {
      return parseAxisInput({ origin: axisInput.point, direction: axisInput.vector }, fallbackOrigin, fallbackDirection);
    }
    if (axisInput.direction || axisInput.vector) {
      const origin = ensurePoint(axisInput.origin ?? axisInput.point ?? fallbackOrigin, fallbackOrigin.clone());
      const vector = ensurePoint(axisInput.direction ?? axisInput.vector, fallbackDirection.clone());
      const length = vector.length();
      const direction = length > EPSILON ? vector.clone().normalize() : fallbackDirection.clone().normalize();
      return { origin, direction, vector, length: length > EPSILON ? length : 1 };
    }
    if (axisInput.length !== undefined) {
      const direction = normalizeVector(fallbackDirection.clone(), new THREE.Vector3(0, 0, 1));
      const vector = direction.clone().multiplyScalar(Math.abs(ensureNumeric(axisInput.length, 1)) || 1);
      return { origin: fallbackOrigin.clone(), direction, vector, length: vector.length() };
    }
    if (axisInput.x !== undefined || axisInput.y !== undefined || axisInput.z !== undefined) {
      const origin = fallbackOrigin.clone();
      const vector = ensurePoint(axisInput, fallbackDirection.clone());
      const length = vector.length();
      const direction = length > EPSILON ? vector.clone().normalize() : fallbackDirection.clone().normalize();
      return { origin, direction, vector, length: length > EPSILON ? length : 1 };
    }
    return {
      origin: fallbackOrigin.clone(),
      direction: normalizeVector(fallbackDirection.clone(), new THREE.Vector3(0, 0, 1)),
      vector: normalizeVector(fallbackDirection.clone(), new THREE.Vector3(0, 0, 1)),
      length: 1,
    };
  }

  function createLinearPath(origin, vector, segments = 8) {
    const points = [];
    for (let i = 0; i <= segments; i += 1) {
      const t = i / segments;
      points.push(origin.clone().add(vector.clone().multiplyScalar(t)));
    }
    return points;
  }

  function createRevolutionSurface(profilePoints, axisData, { domain, metadata = {}, segments = 48 } = {}) {
    if (!profilePoints || !profilePoints.length) {
      return null;
    }
    const startAngle = Number.isFinite(domain?.start) ? domain.start : 0;
    const endAngle = Number.isFinite(domain?.end) ? domain.end : Math.PI * 2;
    const angleSpan = endAngle - startAngle;
    const rowCount = Math.max(3, Math.round(Math.abs(angleSpan) / (Math.PI / 16)));
    const rows = [];
    const axisDirection = axisData.direction.clone().normalize();
    const axisOrigin = axisData.origin.clone();
    for (let i = 0; i <= rowCount; i += 1) {
      const t = i / rowCount;
      const angle = startAngle + angleSpan * t;
      const rotation = new THREE.Quaternion().setFromAxisAngle(axisDirection, angle);
      const row = profilePoints.map((point) => {
        const relative = point.clone().sub(axisOrigin);
        const rotated = relative.applyQuaternion(rotation).add(axisOrigin);
        return rotated;
      });
      rows.push(row);
    }
    return createGridSurface(rows, {
      metadata: {
        ...metadata,
        axis: {
          origin: axisOrigin,
          direction: axisDirection,
          startAngle,
          endAngle,
        },
      },
      closedU: Math.abs(Math.abs(angleSpan) - Math.PI * 2) <= 1e-6,
      closedV: false,
    });
  }

  function createBoundarySurfaceFromCurve(curveInput, options = {}) {
    const sample = sampleCurvePoints(curveInput, options.segments ?? DEFAULT_CURVE_SEGMENTS);
    if (!sample.points.length) {
      return null;
    }
    const basePlane = sample.points.length >= 3
      ? planeFromPoints(sample.points[0], sample.points[1], sample.points[2])
      : defaultPlane();
    const coords = sample.points.map((point) => planeCoordinates(point, basePlane));
    const xs = coords.map((coord) => coord.x);
    const ys = coords.map((coord) => coord.y);
    const minX = Math.min(...xs);
    const maxX = Math.max(...xs);
    const minY = Math.min(...ys);
    const maxY = Math.max(...ys);
    return createPlanarSurfaceFromBounds(basePlane, minX, maxX, minY, maxY);
  }

  function ensureBoxData(boxInput) {
    if (!boxInput) {
      return null;
    }
    if (boxInput.type === 'box' && boxInput.localMin && boxInput.localMax && boxInput.plane) {
      const plane = normalizePlaneAxes(
        ensurePoint(boxInput.plane.origin ?? new THREE.Vector3(), new THREE.Vector3()),
        ensurePoint(boxInput.plane.xAxis ?? new THREE.Vector3(1, 0, 0), new THREE.Vector3(1, 0, 0)),
        ensurePoint(boxInput.plane.yAxis ?? new THREE.Vector3(0, 1, 0), new THREE.Vector3(0, 1, 0)),
        ensurePoint(boxInput.plane.zAxis ?? new THREE.Vector3(0, 0, 1), new THREE.Vector3(0, 0, 1)),
      );
      const localMin = ensurePoint(boxInput.localMin, new THREE.Vector3());
      const localMax = ensurePoint(boxInput.localMax, new THREE.Vector3(1, 1, 1));
      const size = boxInput.size ?? new THREE.Vector3(
        Math.abs(localMax.x - localMin.x),
        Math.abs(localMax.y - localMin.y),
        Math.abs(localMax.z - localMin.z),
      );
      const center = boxInput.center ?? applyPlane(
        plane,
        (localMin.x + localMax.x) / 2,
        (localMin.y + localMax.y) / 2,
        (localMin.z + localMax.z) / 2,
      );
      return { ...boxInput, plane, localMin, localMax, size, center };
    }
    if (boxInput.box) {
      return ensureBoxData(boxInput.box);
    }
    if (boxInput.box3) {
      const plane = boxInput.plane ? ensurePlane(boxInput.plane) : defaultPlane();
      const min = ensurePoint(boxInput.box3.min ?? boxInput.min ?? new THREE.Vector3(), new THREE.Vector3());
      const max = ensurePoint(boxInput.box3.max ?? boxInput.max ?? new THREE.Vector3(), new THREE.Vector3(1, 1, 1));
      return createBoxDataFromPlaneExtents({ plane, min, max });
    }
    if (boxInput.isBox3 || (boxInput.min && boxInput.max)) {
      const min = ensurePoint(boxInput.min ?? new THREE.Vector3(), new THREE.Vector3());
      const max = ensurePoint(boxInput.max ?? new THREE.Vector3(), new THREE.Vector3(1, 1, 1));
      return createBoxDataFromPlaneExtents({ plane: defaultPlane(), min, max });
    }
    if (boxInput.center && boxInput.size) {
      const plane = boxInput.plane ? ensurePlane(boxInput.plane) : defaultPlane();
      const size = ensurePoint(boxInput.size, new THREE.Vector3(1, 1, 1));
      const half = size.clone().multiplyScalar(0.5);
      const localMin = new THREE.Vector3(-half.x, -half.y, -half.z);
      const localMax = new THREE.Vector3(half.x, half.y, half.z);
      const oriented = normalizePlaneAxes(
        ensurePoint(boxInput.center, plane.origin.clone()),
        plane.xAxis.clone(),
        plane.yAxis.clone(),
        plane.zAxis.clone(),
      );
      return createBoxDataFromPlaneExtents({ plane: oriented, min: localMin, max: localMax });
    }
    const points = collectPoints(boxInput);
    if (points.length) {
      return createAxisAlignedBoxFromPoints(points);
    }
    return null;
  }

  function computeBoxMetrics(box) {
    if (!box) {
      return null;
    }
    const plane = box.plane ? normalizePlaneAxes(
      ensurePoint(box.plane.origin ?? new THREE.Vector3(), new THREE.Vector3()),
      ensurePoint(box.plane.xAxis ?? new THREE.Vector3(1, 0, 0), new THREE.Vector3(1, 0, 0)),
      ensurePoint(box.plane.yAxis ?? new THREE.Vector3(0, 1, 0), new THREE.Vector3(0, 1, 0)),
      ensurePoint(box.plane.zAxis ?? new THREE.Vector3(0, 0, 1), new THREE.Vector3(0, 0, 1)),
    ) : defaultPlane();
    const localMin = box.localMin ?? new THREE.Vector3(
      box.min?.x ?? 0,
      box.min?.y ?? 0,
      box.min?.z ?? 0,
    );
    const localMax = box.localMax ?? new THREE.Vector3(
      box.max?.x ?? 0,
      box.max?.y ?? 0,
      box.max?.z ?? 0,
    );
    const size = box.size ?? new THREE.Vector3(
      Math.abs(localMax.x - localMin.x),
      Math.abs(localMax.y - localMin.y),
      Math.abs(localMax.z - localMin.z),
    );
    const volume = Math.abs(size.x * size.y * size.z);
    const area = 2 * (size.x * size.y + size.y * size.z + size.x * size.z);
    const center = box.center ?? applyPlane(
      plane,
      (localMin.x + localMax.x) / 2,
      (localMin.y + localMax.y) / 2,
      (localMin.z + localMax.z) / 2,
    );
    return { plane, localMin, localMax, size, volume, area, center };
  }

  function computeBoxMoments(box) {
    const metrics = computeBoxMetrics(box);
    if (!metrics) {
      return {
        volume: 0,
        area: 0,
        centroid: new THREE.Vector3(),
        inertia: new THREE.Vector3(),
        secondary: new THREE.Vector3(),
        inertiaError: new THREE.Vector3(),
        secondaryError: new THREE.Vector3(),
        gyration: new THREE.Vector3(),
      };
    }
    const { size, volume, area, center } = metrics;
    const mass = volume;
    const inertia = new THREE.Vector3(
      (mass / 12) * (size.y * size.y + size.z * size.z),
      (mass / 12) * (size.x * size.x + size.z * size.z),
      (mass / 12) * (size.x * size.x + size.y * size.y),
    );
    const gyration = new THREE.Vector3(
      volume > EPSILON ? Math.sqrt(Math.abs(inertia.x / volume)) : 0,
      volume > EPSILON ? Math.sqrt(Math.abs(inertia.y / volume)) : 0,
      volume > EPSILON ? Math.sqrt(Math.abs(inertia.z / volume)) : 0,
    );
    return {
      volume,
      area,
      centroid: center.clone(),
      inertia,
      secondary: new THREE.Vector3(0, 0, 0),
      inertiaError: new THREE.Vector3(),
      secondaryError: new THREE.Vector3(),
      gyration,
    };
  }

  function ensureSurfaceEntries(input) {
    const entries = [];
    const visited = new Set();
    function visit(value) {
      if (value === undefined || value === null || visited.has(value)) {
        return;
      }
      if (typeof value === 'object') {
        visited.add(value);
      }
      if (Array.isArray(value)) {
        for (const item of value) {
          visit(item);
        }
        return;
      }
      if (value.surface && typeof value.surface === 'object') {
        entries.push({ surface: value.surface, wrapper: value });
        return;
      }
      if (value.surfaces) {
        visit(value.surfaces);
        return;
      }
      if (value.brep) {
        visit(value.brep);
        return;
      }
      if (value.breps) {
        visit(value.breps);
        return;
      }
      if (value.surfaceData) {
        visit(value.surfaceData);
        return;
      }
      if (value.evaluate || value.getPoint || value.points) {
        entries.push({ surface: value, wrapper: wrapSurface(value) });
      }
    }
    visit(input);
    return entries;
  }

  function sampleSurfaceGrid(surface, segmentsU = 24, segmentsV = 24) {
    if (!surface) {
      return [];
    }
    const domainU = surface.domainU ?? createDomain(0, 1);
    const domainV = surface.domainV ?? createDomain(0, 1);
    const rows = [];
    for (let iv = 0; iv <= segmentsV; iv += 1) {
      const fv = segmentsV === 0 ? 0 : iv / segmentsV;
      const v = domainV.start + (domainV.end - domainV.start) * fv;
      const row = [];
      for (let iu = 0; iu <= segmentsU; iu += 1) {
        const fu = segmentsU === 0 ? 0 : iu / segmentsU;
        const u = domainU.start + (domainU.end - domainU.start) * fu;
        const point = evaluateSurfacePoint(surface, u, v);
        row.push(point ? point.clone() : new THREE.Vector3());
      }
      rows.push(row);
    }
    return rows;
  }

  function triangulateSurfaceGrid(grid) {
    const rows = grid.length;
    const cols = rows ? grid[0].length : 0;
    const triangles = [];
    if (rows < 2 || cols < 2) {
      return triangles;
    }
    for (let j = 0; j < rows - 1; j += 1) {
      for (let i = 0; i < cols - 1; i += 1) {
        const p00 = grid[j][i];
        const p10 = grid[j][i + 1];
        const p01 = grid[j + 1][i];
        const p11 = grid[j + 1][i + 1];
        triangles.push([p00, p10, p11], [p00, p11, p01]);
      }
    }
    return triangles;
  }

  function computeTriangleArea(a, b, c) {
    const ab = b.clone().sub(a);
    const ac = c.clone().sub(a);
    return ab.cross(ac).length() * 0.5;
  }

  function computeTriangleCentroid(a, b, c) {
    return a.clone().add(b).add(c).multiplyScalar(1 / 3);
  }

  function estimateSurfaceArea(surface, segmentsU = 24, segmentsV = 24) {
    if (!surface) {
      return { area: 0, centroid: new THREE.Vector3(), grid: [] };
    }
    const grid = sampleSurfaceGrid(surface, segmentsU, segmentsV);
    const triangles = triangulateSurfaceGrid(grid);
    if (!triangles.length) {
      return { area: 0, centroid: new THREE.Vector3(), grid };
    }
    let area = 0;
    const centroid = new THREE.Vector3();
    for (const [p0, p1, p2] of triangles) {
      const triArea = computeTriangleArea(p0, p1, p2);
      if (!Number.isFinite(triArea) || triArea <= 0) {
        continue;
      }
      area += triArea;
      centroid.add(computeTriangleCentroid(p0, p1, p2).multiplyScalar(triArea));
    }
    if (area > EPSILON) {
      centroid.multiplyScalar(1 / area);
    }
    return { area, centroid, grid };
  }

  function clampToDomain(value, domain) {
    if (!domain) {
      return value;
    }
    const min = Number.isFinite(domain.min) ? domain.min : Number.isFinite(domain.start) ? domain.start : 0;
    const max = Number.isFinite(domain.max) ? domain.max : Number.isFinite(domain.end) ? domain.end : 1;
    if (min === max) {
      return min;
    }
    return clamp(value, Math.min(min, max), Math.max(min, max));
  }

  function ensureUVPoint(input, surface) {
    const domainU = surface?.domainU ?? createDomain(0, 1);
    const domainV = surface?.domainV ?? createDomain(0, 1);
    const fallback = {
      u: (domainU.min + domainU.max) / 2,
      v: (domainV.min + domainV.max) / 2,
    };
    if (input === undefined || input === null) {
      return fallback;
    }
    if (Array.isArray(input)) {
      if (input.length >= 2) {
        return {
          u: ensureNumeric(input[0], fallback.u),
          v: ensureNumeric(input[1], fallback.v),
        };
      }
      if (input.length === 1) {
        return ensureUVPoint(input[0], surface);
      }
    }
    if (typeof input === 'object') {
      if (input.u !== undefined || input.v !== undefined) {
        return {
          u: ensureNumeric(input.u ?? input.x ?? fallback.u, fallback.u),
          v: ensureNumeric(input.v ?? input.y ?? fallback.v, fallback.v),
        };
      }
      if (input.x !== undefined || input.y !== undefined) {
        return {
          u: ensureNumeric(input.x, fallback.u),
          v: ensureNumeric(input.y, fallback.v),
        };
      }
      if (input.uv !== undefined) {
        return ensureUVPoint(input.uv, surface);
      }
      if (input.point !== undefined) {
        return ensureUVPoint(input.point, surface);
      }
    }
    return fallback;
  }

  function evaluateSurfaceDerivatives(surface, u, v, deltaScale = 1e-4) {
    if (!surface) {
      return null;
    }
    const domainU = surface.domainU ?? createDomain(0, 1);
    const domainV = surface.domainV ?? createDomain(0, 1);
    const spanU = Math.max(Math.abs(domainU.max - domainU.min), EPSILON);
    const spanV = Math.max(Math.abs(domainV.max - domainV.min), EPSILON);
    const stepU = spanU * deltaScale;
    const stepV = spanV * deltaScale;
    const uForward = clampToDomain(u + stepU, domainU);
    const uBackward = clampToDomain(u - stepU, domainU);
    const vForward = clampToDomain(v + stepV, domainV);
    const vBackward = clampToDomain(v - stepV, domainV);
    const point = evaluateSurfacePoint(surface, clampToDomain(u, domainU), clampToDomain(v, domainV)) ?? new THREE.Vector3();
    const pointForwardU = evaluateSurfacePoint(surface, uForward, v) ?? point.clone();
    const pointBackwardU = evaluateSurfacePoint(surface, uBackward, v) ?? point.clone();
    const pointForwardV = evaluateSurfacePoint(surface, u, vForward) ?? point.clone();
    const pointBackwardV = evaluateSurfacePoint(surface, u, vBackward) ?? point.clone();
    const tangentU = pointForwardU.clone().sub(pointBackwardU).multiplyScalar(1 / Math.max(Math.abs(uForward - uBackward), EPSILON));
    const tangentV = pointForwardV.clone().sub(pointBackwardV).multiplyScalar(1 / Math.max(Math.abs(vForward - vBackward), EPSILON));
    const Suu = pointForwardU.clone().add(pointBackwardU).sub(point.clone().multiplyScalar(2)).multiplyScalar(1 / Math.pow(Math.max(stepU, EPSILON), 2));
    const Svv = pointForwardV.clone().add(pointBackwardV).sub(point.clone().multiplyScalar(2)).multiplyScalar(1 / Math.pow(Math.max(stepV, EPSILON), 2));
    const pointForwardUV = evaluateSurfacePoint(surface, uForward, vForward) ?? point.clone();
    const pointForwardUBackwardV = evaluateSurfacePoint(surface, uForward, vBackward) ?? point.clone();
    const pointBackwardUForwardV = evaluateSurfacePoint(surface, uBackward, vForward) ?? point.clone();
    const pointBackwardUV = evaluateSurfacePoint(surface, uBackward, vBackward) ?? point.clone();
    const Suv = pointForwardUV.clone()
      .sub(pointForwardUBackwardV)
      .sub(pointBackwardUForwardV)
      .add(pointBackwardUV)
      .multiplyScalar(1 / (4 * Math.max(stepU, EPSILON) * Math.max(stepV, EPSILON)));
    const normal = tangentU.clone().cross(tangentV);
    if (normal.lengthSq() <= EPSILON) {
      const fallbackNormal = surface.plane?.zAxis ?? new THREE.Vector3(0, 0, 1);
      normal.copy(fallbackNormal).normalize();
    } else {
      normal.normalize();
    }
    const E = tangentU.dot(tangentU);
    const F = tangentU.dot(tangentV);
    const G = tangentV.dot(tangentV);
    const e = normal.dot(Suu);
    const f = normal.dot(Suv);
    const g = normal.dot(Svv);
    return {
      point,
      tangentU,
      tangentV,
      normal,
      Suu,
      Svv,
      Suv,
      firstForm: { E, F, G },
      secondForm: { e, f, g },
    };
  }

  function computeSurfaceCurvatureData(surface, u, v) {
    const derivatives = evaluateSurfaceDerivatives(surface, u, v);
    if (!derivatives) {
      return null;
    }
    const { point, tangentU, tangentV, normal, firstForm, secondForm } = derivatives;
    const { E, F, G } = firstForm;
    const { e, f, g } = secondForm;
    const denom = E * G - F * F;
    let gaussian = 0;
    let mean = 0;
    if (Math.abs(denom) > EPSILON) {
      gaussian = (e * g - f * f) / denom;
      mean = (E * g - 2 * F * f + G * e) / (2 * denom);
    }
    const discriminant = Math.max(mean * mean - gaussian, 0);
    const sqrtDisc = Math.sqrt(discriminant);
    const k1 = mean + sqrtDisc;
    const k2 = mean - sqrtDisc;
    let dir1 = tangentU.clone().normalize();
    let dir2 = tangentV.clone().normalize();
    if (Math.abs(denom) > EPSILON) {
      const invDenom = 1 / denom;
      const S11 = (G * e - F * f) * invDenom;
      const S12 = (G * f - F * g) * invDenom;
      const S21 = (-F * e + E * f) * invDenom;
      const S22 = (-F * f + E * g) * invDenom;
      const eigenDirection = (k) => {
        let a = S11 - k;
        let b = S12;
        let c = S21;
        let d = S22 - k;
        if (Math.abs(a) < EPSILON && Math.abs(b) < EPSILON && Math.abs(c) < EPSILON && Math.abs(d) < EPSILON) {
          return tangentU.clone().normalize();
        }
        let vecU = 1;
        let vecV = 0;
        if (Math.abs(b) > Math.abs(c)) {
          if (Math.abs(b) > EPSILON) {
            vecU = d;
            vecV = -b;
          } else {
            vecU = 1;
            vecV = 0;
          }
        } else if (Math.abs(c) > EPSILON) {
          vecU = -c;
          vecV = a;
        }
        const direction = tangentU.clone().multiplyScalar(vecU).add(tangentV.clone().multiplyScalar(vecV));
        if (direction.lengthSq() <= EPSILON) {
          return tangentU.clone().normalize();
        }
        return direction.normalize();
      };
      dir1 = eigenDirection(k1);
      dir2 = eigenDirection(k2);
    }
    return {
      point,
      normal,
      tangentU,
      tangentV,
      gaussian,
      mean,
      principalCurvatures: [
        { value: k1, direction: dir1 },
        { value: k2, direction: dir2 },
      ],
    };
  }

  function approximateSurfaceClosestPoint(surface, targetPoint, { segmentsU = 20, segmentsV = 20 } = {}) {
    if (!surface || !targetPoint) {
      return null;
    }
    const domainU = surface.domainU ?? createDomain(0, 1);
    const domainV = surface.domainV ?? createDomain(0, 1);
    let bestU = domainU.min;
    let bestV = domainV.min;
    let bestPoint = evaluateSurfacePoint(surface, bestU, bestV) ?? new THREE.Vector3();
    let bestDistanceSq = bestPoint.distanceToSquared(targetPoint);
    for (let iv = 0; iv <= segmentsV; iv += 1) {
      const fv = segmentsV === 0 ? 0 : iv / segmentsV;
      const v = domainV.start + (domainV.end - domainV.start) * fv;
      for (let iu = 0; iu <= segmentsU; iu += 1) {
        const fu = segmentsU === 0 ? 0 : iu / segmentsU;
        const u = domainU.start + (domainU.end - domainU.start) * fu;
        const sample = evaluateSurfacePoint(surface, u, v);
        if (!sample) {
          continue;
        }
        const distSq = sample.distanceToSquared(targetPoint);
        if (distSq < bestDistanceSq) {
          bestDistanceSq = distSq;
          bestPoint = sample.clone();
          bestU = u;
          bestV = v;
        }
      }
    }
    let currentU = bestU;
    let currentV = bestV;
    let currentPoint = bestPoint.clone();
    for (let iteration = 0; iteration < 8; iteration += 1) {
      const derivatives = evaluateSurfaceDerivatives(surface, currentU, currentV);
      if (!derivatives) {
        break;
      }
      const diff = currentPoint.clone().sub(targetPoint);
      const a11 = derivatives.tangentU.dot(derivatives.tangentU) + EPSILON;
      const a12 = derivatives.tangentU.dot(derivatives.tangentV);
      const a22 = derivatives.tangentV.dot(derivatives.tangentV) + EPSILON;
      const b1 = derivatives.tangentU.dot(diff);
      const b2 = derivatives.tangentV.dot(diff);
      const denom = a11 * a22 - a12 * a12;
      if (Math.abs(denom) < EPSILON) {
        break;
      }
      const du = (a12 * b2 - a22 * b1) / denom;
      const dv = (a12 * b1 - a11 * b2) / denom;
      currentU = clampToDomain(currentU - du, domainU);
      currentV = clampToDomain(currentV - dv, domainV);
      const nextPoint = evaluateSurfacePoint(surface, currentU, currentV);
      if (!nextPoint) {
        break;
      }
      currentPoint.copy(nextPoint);
      bestDistanceSq = currentPoint.distanceToSquared(targetPoint);
      if (Math.abs(du) <= 1e-6 && Math.abs(dv) <= 1e-6) {
        break;
      }
    }
    const derivatives = evaluateSurfaceDerivatives(surface, currentU, currentV);
    const normal = derivatives?.normal?.clone() ?? surface.plane?.zAxis?.clone() ?? new THREE.Vector3(0, 0, 1);
    return {
      point: currentPoint,
      u: currentU,
      v: currentV,
      distance: Math.sqrt(bestDistanceSq),
      normal: normal.normalize(),
    };
  }

  function createPolylineSignature(points, tolerance = 1e-4) {
    if (!points || !points.length) {
      return '';
    }
    const scale = 1 / tolerance;
    const format = (pt) => [
      Math.round(pt.x * scale),
      Math.round(pt.y * scale),
      Math.round(pt.z * scale),
    ].join(',');
    const forward = points.map(format).join('|');
    const reversed = points.slice().reverse().map(format).join('|');
    return forward < reversed ? forward : reversed;
  }

  function computeSurfaceEdges(surfaceEntries, { segments = 32 } = {}) {
    const edgesBySignature = new Map();
    surfaceEntries.forEach((entry, faceIndex) => {
      const surface = entry.surface;
      if (!surface) {
        return;
      }
      const domainU = surface.domainU ?? createDomain(0, 1);
      const domainV = surface.domainV ?? createDomain(0, 1);
      const boundaries = [
        { key: 'u-min', constant: domainU.min, varying: 'v' },
        { key: 'u-max', constant: domainU.max, varying: 'v' },
        { key: 'v-min', constant: domainV.min, varying: 'u' },
        { key: 'v-max', constant: domainV.max, varying: 'u' },
      ];
      for (const boundary of boundaries) {
        const points = [];
        for (let i = 0; i <= segments; i += 1) {
          const t = segments === 0 ? 0 : i / segments;
          const uValue = boundary.varying === 'u'
            ? domainU.start + (domainU.end - domainU.start) * t
            : boundary.constant;
          const vValue = boundary.varying === 'v'
            ? domainV.start + (domainV.end - domainV.start) * t
            : boundary.constant;
          const point = evaluateSurfacePoint(surface, uValue, vValue);
          if (point) {
            points.push(point.clone());
          }
        }
        if (points.length >= 2) {
          const signature = createPolylineSignature(points);
          const record = {
            faceIndex,
            points,
            boundary: boundary.key,
          };
          if (!edgesBySignature.has(signature)) {
            edgesBySignature.set(signature, []);
          }
          edgesBySignature.get(signature).push(record);
        }
      }
    });
    const edges = [];
    const naked = [];
    const interior = [];
    const nonManifold = [];
    edgesBySignature.forEach((records, signature) => {
      if (!records.length) {
        return;
      }
      const sample = records[0];
      const faces = Array.from(new Set(records.map((record) => record.faceIndex)));
      const edge = {
        signature,
        points: sample.points.map((pt) => pt.clone()),
        records,
        faces,
      };
      edges.push(edge);
      if (records.length <= 1) {
        naked.push(edge);
      } else if (records.length === 2) {
        interior.push(edge);
      } else {
        nonManifold.push(edge);
      }
    });
    return { edges, naked, interior, nonManifold };
  }

  function edgeToCurve(edge) {
    if (!edge) {
      return null;
    }
    return {
      type: 'polyline',
      points: edge.points.map((pt) => pt.clone()),
      closed: false,
      length: computePolylineLength(edge.points, false),
      faces: edge.faces.slice(),
    };
  }

  function dedupePoints(points, tolerance = 1e-6) {
    if (!points || !points.length) {
      return [];
    }
    const result = [];
    const tolSq = tolerance * tolerance;
    for (const point of points) {
      const exists = result.some((existing) => existing.distanceToSquared(point) <= tolSq);
      if (!exists) {
        result.push(point.clone());
      }
    }
    return result;
  }

  function computeBoundingBoxForContent(content) {
    const points = collectPoints(content);
    if (!points.length) {
      return null;
    }
    return createAxisAlignedBoxFromPoints(points);
  }

  function estimateGeometryAreaAndCentroid(geometryInput) {
    if (!geometryInput) {
      return { area: 0, centroid: new THREE.Vector3(), source: null };
    }
    const surfaces = ensureSurfaceEntries(geometryInput);
    if (surfaces.length) {
      let totalArea = 0;
      const centroid = new THREE.Vector3();
      for (const entry of surfaces) {
        const { area, centroid: surfaceCentroid } = estimateSurfaceArea(entry.surface);
        if (area > EPSILON) {
          totalArea += area;
          centroid.add(surfaceCentroid.clone().multiplyScalar(area));
        }
      }
      if (totalArea > EPSILON) {
        centroid.multiplyScalar(1 / totalArea);
      }
      return { area: totalArea, centroid, source: surfaces };
    }
    const box = ensureBoxData(geometryInput) ?? computeBoundingBoxForContent(geometryInput);
    if (box) {
      const metrics = computeBoxMetrics(box);
      return { area: metrics.area, centroid: metrics.center.clone(), source: box };
    }
    return { area: 0, centroid: new THREE.Vector3(), source: null };
  }

  function estimateGeometryVolumeAndCentroid(geometryInput) {
    if (!geometryInput) {
      return { volume: 0, centroid: new THREE.Vector3(), box: null };
    }
    const box = ensureBoxData(geometryInput) ?? computeBoundingBoxForContent(geometryInput);
    if (!box) {
      return { volume: 0, centroid: new THREE.Vector3(), box: null };
    }
    const metrics = computeBoxMetrics(box);
    return { volume: metrics.volume, centroid: metrics.center.clone(), box };
  }

  function computeBoxInclusion(box, point, { strict = false, tolerance = 1e-6 } = {}) {
    if (!box || !point) {
      return false;
    }
    const plane = box.plane ? normalizePlaneAxes(
      box.plane.origin.clone(),
      box.plane.xAxis.clone(),
      box.plane.yAxis.clone(),
      box.plane.zAxis.clone(),
    ) : defaultPlane();
    const coords = planeCoordinates(point, plane);
    const min = box.localMin ?? new THREE.Vector3(box.min?.x ?? 0, box.min?.y ?? 0, box.min?.z ?? 0);
    const max = box.localMax ?? new THREE.Vector3(box.max?.x ?? 0, box.max?.y ?? 0, box.max?.z ?? 0);
    const compare = (value, minValue, maxValue) => {
      if (strict) {
        return value > minValue + tolerance && value < maxValue - tolerance;
      }
      return value >= minValue - tolerance && value <= maxValue + tolerance;
    };
    return compare(coords.x, min.x, max.x)
      && compare(coords.y, min.y, max.y)
      && compare(coords.z, min.z, max.z);
  }

  function computeSurfaceDimensions(surface, segments = 12) {
    if (!surface) {
      return { u: 0, v: 0 };
    }
    const domainU = surface.domainU ?? createDomain(0, 1);
    const domainV = surface.domainV ?? createDomain(0, 1);
    const samples = Math.max(segments, 2);
    let uTotal = 0;
    let uCount = 0;
    let vTotal = 0;
    let vCount = 0;
    for (let iv = 0; iv <= samples; iv += 1) {
      const fv = samples === 0 ? 0 : iv / samples;
      const v = domainV.start + (domainV.end - domainV.start) * fv;
      const start = evaluateSurfacePoint(surface, domainU.min, v);
      const end = evaluateSurfacePoint(surface, domainU.max, v);
      if (start && end) {
        uTotal += start.distanceTo(end);
        uCount += 1;
      }
    }
    for (let iu = 0; iu <= samples; iu += 1) {
      const fu = samples === 0 ? 0 : iu / samples;
      const u = domainU.start + (domainU.end - domainU.start) * fu;
      const start = evaluateSurfacePoint(surface, u, domainV.min);
      const end = evaluateSurfacePoint(surface, u, domainV.max);
      if (start && end) {
        vTotal += start.distanceTo(end);
        vCount += 1;
      }
    }
    const uDimension = uCount ? uTotal / uCount : 0;
    const vDimension = vCount ? vTotal / vCount : 0;
    return { u: uDimension, v: vDimension };
  }

  function computeSurfaceInflectionCurves(surface, { segmentsU = 24, segmentsV = 24, tolerance = 1e-6 } = {}) {
    if (!surface) {
      return [];
    }
    const domainU = surface.domainU ?? createDomain(0, 1);
    const domainV = surface.domainV ?? createDomain(0, 1);
    const gaussianGrid = [];
    const uvGrid = [];
    for (let iv = 0; iv <= segmentsV; iv += 1) {
      const fv = segmentsV === 0 ? 0 : iv / segmentsV;
      const v = domainV.start + (domainV.end - domainV.start) * fv;
      const gaussianRow = [];
      const uvRow = [];
      for (let iu = 0; iu <= segmentsU; iu += 1) {
        const fu = segmentsU === 0 ? 0 : iu / segmentsU;
        const u = domainU.start + (domainU.end - domainU.start) * fu;
        const curvature = computeSurfaceCurvatureData(surface, u, v);
        gaussianRow.push(curvature ? curvature.gaussian : 0);
        uvRow.push({ u, v });
      }
      gaussianGrid.push(gaussianRow);
      uvGrid.push(uvRow);
    }
    const curves = [];
    const addCurve = (points) => {
      const valid = points.filter(Boolean);
      const unique = dedupePoints(valid, tolerance);
      if (unique.length >= 2) {
        curves.push({
          type: 'polyline',
          points: unique.map((pt) => pt.clone()),
          closed: false,
          length: computePolylineLength(unique, false),
        });
      }
    };
    for (let iv = 0; iv <= segmentsV; iv += 1) {
      const rowPoints = [];
      for (let iu = 0; iu < segmentsU; iu += 1) {
        const g0 = gaussianGrid[iv][iu];
        const g1 = gaussianGrid[iv][iu + 1];
        const uv0 = uvGrid[iv][iu];
        const uv1 = uvGrid[iv][iu + 1];
        if (!Number.isFinite(g0) || !Number.isFinite(g1)) {
          continue;
        }
        if (Math.abs(g0) <= tolerance) {
          const point = evaluateSurfacePoint(surface, uv0.u, uv0.v);
          if (point) {
            rowPoints.push(point.clone());
          }
        }
        if (g0 * g1 < 0) {
          const ratio = g0 / (g0 - g1);
          const u = uv0.u + (uv1.u - uv0.u) * ratio;
          const v = uv0.v + (uv1.v - uv0.v) * ratio;
          const point = evaluateSurfacePoint(surface, u, v);
          if (point) {
            rowPoints.push(point.clone());
          }
        }
      }
      if (rowPoints.length >= 2) {
        addCurve(rowPoints);
      }
    }
    for (let iu = 0; iu <= segmentsU; iu += 1) {
      const columnPoints = [];
      for (let iv = 0; iv < segmentsV; iv += 1) {
        const g0 = gaussianGrid[iv][iu];
        const g1 = gaussianGrid[iv + 1][iu];
        const uv0 = uvGrid[iv][iu];
        const uv1 = uvGrid[iv + 1][iu];
        if (!Number.isFinite(g0) || !Number.isFinite(g1)) {
          continue;
        }
        if (Math.abs(g0) <= tolerance) {
          const point = evaluateSurfacePoint(surface, uv0.u, uv0.v);
          if (point) {
            columnPoints.push(point.clone());
          }
        }
        if (g0 * g1 < 0) {
          const ratio = g0 / (g0 - g1);
          const u = uv0.u + (uv1.u - uv0.u) * ratio;
          const v = uv0.v + (uv1.v - uv0.v) * ratio;
          const point = evaluateSurfacePoint(surface, u, v);
          if (point) {
            columnPoints.push(point.clone());
          }
        }
      }
      if (columnPoints.length >= 2) {
        addCurve(columnPoints);
      }
    }
    const uniqueCurves = [];
    const signatures = new Set();
    for (const curve of curves) {
      const signature = createPolylineSignature(curve.points);
      if (!signatures.has(signature)) {
        signatures.add(signature);
        uniqueCurves.push(curve);
      }
    }
    return uniqueCurves;
  }

  function evaluateSurfacePlanarity(surface, { tolerance = 1e-4 } = {}) {
    if (!surface) {
      return { planar: false, plane: defaultPlane(), deviation: Number.POSITIVE_INFINITY };
    }
    const samples = sampleSurfacePoints(surface, 12, 12);
    const points = dedupePoints(samples, tolerance);
    if (points.length < 3) {
      return { planar: true, plane: surface.plane ?? defaultPlane(), deviation: 0 };
    }
    let basePlane = null;
    for (let i = 0; i < points.length - 2 && !basePlane; i += 1) {
      for (let j = i + 1; j < points.length - 1 && !basePlane; j += 1) {
        for (let k = j + 1; k < points.length && !basePlane; k += 1) {
          const normal = points[j].clone().sub(points[i]).cross(points[k].clone().sub(points[i]));
          if (normal.lengthSq() > EPSILON) {
            basePlane = planeFromPoints(points[i], points[j], points[k]);
          }
        }
      }
    }
    if (!basePlane) {
      return { planar: true, plane: surface.plane ?? defaultPlane(), deviation: 0 };
    }
    const referencePlane = new THREE.Plane().setFromNormalAndCoplanarPoint(
      basePlane.zAxis.clone(),
      basePlane.origin.clone(),
    );
    let maxDeviation = 0;
    for (const point of points) {
      const distance = Math.abs(referencePlane.distanceToPoint(point));
      if (distance > maxDeviation) {
        maxDeviation = distance;
      }
    }
    return {
      planar: maxDeviation <= tolerance,
      plane: basePlane,
      deviation: maxDeviation,
    };
  }

  function registerAnalysisComponents() {
    function primarySurfaceEntry(surfaceInput) {
      const surfaces = ensureSurfaceEntries(surfaceInput);
      return surfaces.length ? surfaces[0] : null;
    }

    function evaluateSurfaceSample(surfaceInput, uvInput) {
      const entry = primarySurfaceEntry(surfaceInput);
      const surface = entry?.surface ?? null;
      const uv = ensureUVPoint(uvInput, surface);
      const fallbackPlane = surface?.plane ?? defaultPlane();
      if (!surface) {
        const point = fallbackPlane.origin.clone();
        const normal = fallbackPlane.zAxis.clone();
        const tangentU = fallbackPlane.xAxis.clone();
        const tangentV = fallbackPlane.yAxis.clone();
        const frame = normalizePlaneAxes(point.clone(), tangentU.clone(), tangentV.clone(), normal.clone());
        return {
          entry: null,
          surface: null,
          uv,
          point,
          normal,
          tangentU,
          tangentV,
          frame,
          derivatives: null,
        };
      }
      const derivatives = evaluateSurfaceDerivatives(surface, uv.u, uv.v);
      const point = derivatives?.point?.clone()
        ?? evaluateSurfacePoint(surface, uv.u, uv.v)
        ?? fallbackPlane.origin.clone();
      let normal = derivatives?.normal?.clone() ?? fallbackPlane.zAxis.clone();
      if (normal.lengthSq() <= EPSILON) {
        normal = fallbackPlane.zAxis.clone();
      }
      normal.normalize();
      let tangentU = derivatives?.tangentU?.clone();
      if (!tangentU || tangentU.lengthSq() <= EPSILON) {
        tangentU = fallbackPlane.xAxis.clone();
        if (tangentU.lengthSq() <= EPSILON) {
          tangentU = orthogonalVector(normal);
        }
      }
      tangentU.normalize();
      let tangentV = derivatives?.tangentV?.clone();
      if (!tangentV || tangentV.lengthSq() <= EPSILON) {
        tangentV = fallbackPlane.yAxis.clone();
        if (tangentV.lengthSq() <= EPSILON) {
          tangentV = normal.clone().cross(tangentU);
          if (tangentV.lengthSq() <= EPSILON) {
            tangentV = orthogonalVector(tangentU);
          }
        }
      }
      tangentV.normalize();
      const frame = normalizePlaneAxes(point.clone(), tangentU.clone(), tangentV.clone(), normal.clone());
      return {
        entry,
        surface,
        uv,
        point,
        normal,
        tangentU,
        tangentV,
        frame,
        derivatives,
      };
    }

    function closestPointOnSurfaceEntries(surfaceEntries, point) {
      if (!surfaceEntries.length || !point) {
        return null;
      }
      let best = null;
      for (const entry of surfaceEntries) {
        const result = approximateSurfaceClosestPoint(entry.surface, point, { segmentsU: 32, segmentsV: 32 });
        if (!result) {
          continue;
        }
        if (!best || result.distance < best.distance) {
          best = { ...result, surfaceEntry: entry };
        }
      }
      return best;
    }

    function createOsculatingCircle(point, normal, direction, curvature) {
      if (!point || !normal || !direction || !Number.isFinite(curvature) || Math.abs(curvature) <= EPSILON) {
        return null;
      }
      const signedRadius = 1 / curvature;
      if (!Number.isFinite(signedRadius) || Math.abs(signedRadius) <= EPSILON) {
        return null;
      }
      const unitNormal = normal.clone().normalize();
      const unitDirection = direction.clone().normalize();
      let yAxis = unitNormal.clone().cross(unitDirection);
      if (yAxis.lengthSq() <= EPSILON) {
        yAxis = orthogonalVector(unitDirection);
      }
      yAxis.normalize();
      const center = point.clone().add(unitNormal.clone().multiplyScalar(signedRadius));
      const plane = normalizePlaneAxes(center.clone(), unitDirection.clone(), yAxis.clone(), unitNormal.clone());
      const shape = new THREE.Shape();
      shape.absarc(0, 0, Math.abs(signedRadius), 0, Math.PI * 2, false);
      return {
        type: 'circle',
        plane,
        center,
        radius: Math.abs(signedRadius),
        shape,
        segments: 128,
      };
    }

    function computeMomentsForGeometry(geometryInput, { massOverride = null } = {}) {
      const box = ensureBoxData(geometryInput) ?? computeBoundingBoxForContent(geometryInput);
      const base = computeBoxMoments(box);
      const centroid = base.centroid.clone();
      let inertia = base.inertia.clone();
      let secondary = base.secondary.clone();
      let inertiaError = base.inertiaError.clone();
      let secondaryError = base.secondaryError.clone();
      let gyration = base.gyration.clone();
      let volume = base.volume;
      if (massOverride !== null && base.volume > EPSILON) {
        const mass = Math.max(massOverride, 0);
        const scale = mass / base.volume;
        inertia = inertia.multiplyScalar(scale);
        secondary = secondary.multiplyScalar(scale);
        inertiaError = inertiaError.multiplyScalar(scale);
        secondaryError = secondaryError.multiplyScalar(scale);
        gyration = new THREE.Vector3(
          mass > EPSILON ? Math.sqrt(Math.abs(inertia.x / mass)) : 0,
          mass > EPSILON ? Math.sqrt(Math.abs(inertia.y / mass)) : 0,
          mass > EPSILON ? Math.sqrt(Math.abs(inertia.z / mass)) : 0,
        );
        volume = mass;
      }
      return { volume, area: base.area, centroid, inertia, secondary, inertiaError, secondaryError, gyration };
    }

    function generateSurfaceIsoCurves(surface, density = 3, segments = 32) {
      if (!surface) {
        return [];
      }
      const domainU = surface.domainU ?? createDomain(0, 1);
      const domainV = surface.domainV ?? createDomain(0, 1);
      const stepCount = Math.max(1, Math.round(Math.abs(density)));
      const segmentCount = Math.max(segments, 8);
      const curves = [];
      const signatures = new Set();
      const addCurve = (points) => {
        if (!points || points.length < 2) {
          return;
        }
        const signature = createPolylineSignature(points);
        if (signatures.has(signature)) {
          return;
        }
        signatures.add(signature);
        curves.push({
          type: 'polyline',
          points: points.map((pt) => pt.clone()),
          closed: false,
          length: computePolylineLength(points, false),
        });
      };
      for (let i = 0; i <= stepCount; i += 1) {
        const factor = stepCount === 0 ? 0 : i / stepCount;
        const u = domainU.start + (domainU.end - domainU.start) * factor;
        const points = [];
        for (let j = 0; j <= segmentCount; j += 1) {
          const vf = segmentCount === 0 ? 0 : j / segmentCount;
          const v = domainV.start + (domainV.end - domainV.start) * vf;
          const point = evaluateSurfacePoint(surface, u, v);
          if (point) {
            points.push(point);
          }
        }
        addCurve(points);
      }
      for (let j = 0; j <= stepCount; j += 1) {
        const factor = stepCount === 0 ? 0 : j / stepCount;
        const v = domainV.start + (domainV.end - domainV.start) * factor;
        const points = [];
        for (let i = 0; i <= segmentCount; i += 1) {
          const uf = segmentCount === 0 ? 0 : i / segmentCount;
          const u = domainU.start + (domainU.end - domainU.start) * uf;
          const point = evaluateSurfacePoint(surface, u, v);
          if (point) {
            points.push(point);
          }
        }
        addCurve(points);
      }
      return curves;
    }

    function cloneFrame(frame) {
      if (!frame) {
        const fallback = defaultPlane();
        return {
          origin: fallback.origin.clone(),
          xAxis: fallback.xAxis.clone(),
          yAxis: fallback.yAxis.clone(),
          zAxis: fallback.zAxis.clone(),
        };
      }
      return {
        origin: frame.origin.clone(),
        xAxis: frame.xAxis.clone(),
        yAxis: frame.yAxis.clone(),
        zAxis: frame.zAxis.clone(),
      };
    }

    register('{0148a65d-6f42-414a-9db7-9a9b2eb78437}', {
      type: 'surface',
      pinMap: {
        inputs: { B: 'brep', Brep: 'brep', brep: 'brep' },
        outputs: {
          En: 'naked',
          Naked: 'naked',
          Ei: 'interior',
          Interior: 'interior',
          Em: 'nonManifold',
          'Non-Manifold': 'nonManifold',
        },
      },
      eval: ({ inputs }) => {
        const surfaces = ensureSurfaceEntries(inputs.brep);
        if (!surfaces.length) {
          return { naked: [], interior: [], nonManifold: [] };
        }
        const edgesInfo = computeSurfaceEdges(surfaces);
        return {
          naked: edgesInfo.naked.map(edgeToCurve).filter(Boolean),
          interior: edgesInfo.interior.map(edgeToCurve).filter(Boolean),
          nonManifold: edgesInfo.nonManifold.map(edgeToCurve).filter(Boolean),
        };
      },
    });

    register('{0efd7f0c-f63d-446d-970e-9fb0e636ea41}', {
      type: 'surface',
      pinMap: {
        inputs: { S: 'surface', Surface: 'surface', surface: 'surface' },
        outputs: { C: 'curves', Curves: 'curves', curves: 'curves' },
      },
      eval: ({ inputs }) => {
        const surfaces = ensureSurfaceEntries(inputs.surface);
        if (!surfaces.length) {
          return { curves: [] };
        }
        const curves = computeSurfaceInflectionCurves(surfaces[0].surface);
        return { curves };
      },
    });

    register('{13b40e9c-3aed-4669-b2e8-60bd02091421}', {
      type: 'geometry',
      pinMap: {
        inputs: {
          B: 'box', Box: 'box', box: 'box',
          U: 'u', 'U parameter': 'u', u: 'u',
          V: 'v', 'V parameter': 'v', v: 'v',
          W: 'w', 'W parameter': 'w', w: 'w',
        },
        outputs: {
          Pl: 'plane', Plane: 'plane', plane: 'plane',
          Pt: 'point', Point: 'point', point: 'point',
          I: 'include', Include: 'include', include: 'include',
        },
      },
      eval: ({ inputs }) => {
        const box = ensureBoxData(inputs.box) ?? computeBoundingBoxForContent(inputs.box);
        if (!box) {
          return { plane: null, point: null, include: false };
        }
        const planeData = box.plane ?? defaultPlane();
        const u = clamp01(ensureNumeric(inputs.u, 0));
        const v = clamp01(ensureNumeric(inputs.v, 0));
        const w = clamp01(ensureNumeric(inputs.w, 0));
        const localX = box.localMin.x + (box.localMax.x - box.localMin.x) * u;
        const localY = box.localMin.y + (box.localMax.y - box.localMin.y) * v;
        const localZ = box.localMin.z + (box.localMax.z - box.localMin.z) * w;
        const point = applyPlane(planeData, localX, localY, localZ);
        const plane = {
          origin: point.clone(),
          xAxis: planeData.xAxis.clone(),
          yAxis: planeData.yAxis.clone(),
          zAxis: planeData.zAxis.clone(),
        };
        const include = computeBoxInclusion(box, point, { strict: false });
        return { plane, point, include };
      },
    });

    register('{15128198-399d-4d6c-9586-1f65db3ce7bf}', {
      type: 'surface',
      pinMap: {
        inputs: { S: 'surface', Surface: 'surface', surface: 'surface' },
        outputs: {
          P: 'points', Points: 'points', points: 'points',
          W: 'weights', Weights: 'weights', weights: 'weights',
          G: 'greville', Greville: 'greville', greville: 'greville',
          U: 'uCount', 'U Count': 'uCount',
          V: 'vCount', 'V Count': 'vCount',
        },
      },
      eval: ({ inputs }) => {
        const surfaces = ensureSurfaceEntries(inputs.surface);
        if (!surfaces.length) {
          return { points: [], weights: [], greville: [], uCount: 0, vCount: 0 };
        }
        const entry = surfaces[0];
        const surface = entry.surface;
        const gridData = entry.wrapper?.metadata?.grid ?? entry.surface?.metadata?.grid ?? null;
        const grid = Array.isArray(gridData) && gridData.length
          ? gridData.map((row) => row.map((pt) => ensurePoint(pt, new THREE.Vector3())))
          : sampleSurfaceGrid(surface, 6, 6);
        const vCount = grid.length;
        const uCount = vCount ? grid[0].length : 0;
        const points = [];
        const weights = [];
        const greville = [];
        if (vCount && uCount) {
          for (let j = 0; j < vCount; j += 1) {
            for (let i = 0; i < uCount; i += 1) {
              const point = grid[j][i]?.clone?.() ? grid[j][i].clone() : ensurePoint(grid[j][i], new THREE.Vector3());
              points.push(point);
              weights.push(1);
              const uParam = uCount > 1 ? i / (uCount - 1) : 0;
              const vParam = vCount > 1 ? j / (vCount - 1) : 0;
              greville.push({ u: uParam, v: vParam });
            }
          }
        }
        return { points, weights, greville, uCount, vCount };
      },
    });

    register('{224f7648-5956-4b26-80d9-8d771f3dfd5d}', {
      type: 'geometry',
      pinMap: {
        inputs: { G: 'geometry', Geometry: 'geometry', geometry: 'geometry' },
        outputs: { V: 'volume', Volume: 'volume', C: 'centroid', Centroid: 'centroid' },
      },
      eval: ({ inputs }) => {
        const { volume, centroid } = estimateGeometryVolumeAndCentroid(inputs.geometry);
        return { volume, centroid };
      },
    });

    register('{2ba64356-be21-4c12-bbd4-ced54f04c8ef}', {
      type: 'surface',
      pinMap: {
        inputs: {
          B: 'brep', Brep: 'brep', brep: 'brep',
          S: 'shape', Shape: 'shape', shape: 'shape',
        },
        outputs: { R: 'relation', Relation: 'relation', relation: 'relation' },
      },
      eval: ({ inputs }) => {
        const brepBox = ensureBoxData(inputs.brep) ?? computeBoundingBoxForContent(inputs.brep);
        const shapeBox = ensureBoxData(inputs.shape) ?? computeBoundingBoxForContent(inputs.shape);
        if (!brepBox || !shapeBox) {
          return { relation: 2 };
        }
        const corners = shapeBox.corners ?? [];
        const inside = corners.length
          ? corners.every((corner) => computeBoxInclusion(brepBox, corner, { strict: false }))
          : computeBoxInclusion(brepBox, applyPlane(shapeBox.plane ?? defaultPlane(), 0, 0, 0), { strict: false });
        if (inside) {
          return { relation: 0 };
        }
        const brepBounds = brepBox.box3 ?? computeBoundingBoxFromPoints(brepBox.corners ?? []);
        const shapeBounds = shapeBox.box3 ?? computeBoundingBoxFromPoints(shapeBox.corners ?? []);
        if (brepBounds && shapeBounds && brepBounds.intersectsBox(shapeBounds)) {
          return { relation: 1 };
        }
        return { relation: 2 };
      },
    });

    register('{2e205f24-9279-47b2-b414-d06dcd0b21a7}', {
      type: 'geometry',
      pinMap: {
        inputs: { G: 'geometry', Geometry: 'geometry', geometry: 'geometry' },
        outputs: { A: 'area', Area: 'area', C: 'centroid', Centroid: 'centroid' },
      },
      eval: ({ inputs }) => {
        const { area, centroid } = estimateGeometryAreaAndCentroid(inputs.geometry);
        return { area, centroid };
      },
    });

    register('{353b206e-bde5-4f02-a913-b3b8a977d4b9}', {
      type: 'surface',
      pinMap: {
        inputs: {
          S: 'surface', Surface: 'surface', surface: 'surface',
          uv: 'uv', UV: 'uv', Point: 'uv', point: 'uv',
        },
        outputs: {
          P: 'point', Point: 'point', point: 'point',
          N: 'normal', Normal: 'normal', normal: 'normal',
          U: 'uDirection', 'U direction': 'uDirection', u: 'uDirection',
          V: 'vDirection', 'V direction': 'vDirection', v: 'vDirection',
          F: 'frame', Frame: 'frame', frame: 'frame',
        },
      },
      eval: ({ inputs }) => {
        const data = evaluateSurfaceSample(inputs.surface, inputs.uv);
        const point = data.point ? data.point.clone() : null;
        const normal = data.normal ? data.normal.clone() : null;
        const uDirection = data.tangentU ? data.tangentU.clone() : null;
        const vDirection = data.tangentV ? data.tangentV.clone() : null;
        const frame = cloneFrame(data.frame);
        return { point, normal, uDirection, vDirection, frame };
      },
    });

    register('{404f75ac-5594-4c48-ad8a-7d0f472bbf8a}', {
      type: 'surface',
      pinMap: {
        inputs: {
          S: 'surface', Surface: 'surface', surface: 'surface',
          uv: 'uv', UV: 'uv', Point: 'uv', point: 'uv',
        },
        outputs: {
          F: 'frame', Frame: 'frame', frame: 'frame',
          Maximum: 'maxCurvature', 'C¹': 'maxCurvature', C1: 'maxCurvature',
          Minimum: 'minCurvature', 'C²': 'minCurvature', C2: 'minCurvature',
          'Max direction': 'direction1', 'K¹': 'direction1', K1: 'direction1',
          'Min direction': 'direction2', 'K²': 'direction2', K2: 'direction2',
        },
      },
      eval: ({ inputs }) => {
        const data = evaluateSurfaceSample(inputs.surface, inputs.uv);
        const curvature = data.surface
          ? computeSurfaceCurvatureData(data.surface, data.uv.u, data.uv.v)
          : null;
        const principal = curvature?.principalCurvatures ?? [];
        const first = principal[0] ?? null;
        const second = principal[1] ?? null;
        const maxCurvature = Number.isFinite(first?.value) ? first.value : 0;
        const minCurvature = Number.isFinite(second?.value) ? second.value : 0;
        const direction1 = (first?.direction ? first.direction.clone() : data.tangentU.clone()).normalize();
        const direction2 = (second?.direction ? second.direction.clone() : data.tangentV.clone()).normalize();
        return {
          frame: cloneFrame(data.frame),
          maxCurvature,
          minCurvature,
          direction1,
          direction2,
        };
      },
    });

    register('{4139f3a3-cf93-4fc0-b5e0-18a3acd0b003}', {
      type: 'surface',
      pinMap: {
        inputs: {
          S: 'surface', Surface: 'surface', surface: 'surface',
          uv: 'uv', UV: 'uv', Point: 'uv', point: 'uv',
        },
        outputs: {
          F: 'frame', Frame: 'frame', frame: 'frame',
          G: 'gaussian', Gaussian: 'gaussian', gaussian: 'gaussian',
          M: 'mean', Mean: 'mean', mean: 'mean',
        },
      },
      eval: ({ inputs }) => {
        const data = evaluateSurfaceSample(inputs.surface, inputs.uv);
        const curvature = data.surface
          ? computeSurfaceCurvatureData(data.surface, data.uv.u, data.uv.v)
          : null;
        return {
          frame: cloneFrame(data.frame),
          gaussian: curvature?.gaussian ?? 0,
          mean: curvature?.mean ?? 0,
        };
      },
    });

    register('{4a9e9a8e-0943-4438-b360-129c30f2bb0f}', {
      type: 'surface',
      pinMap: {
        inputs: {
          P: 'point', Point: 'point', point: 'point',
          S: 'surface', Surface: 'surface', surface: 'surface',
        },
        outputs: {
          P: 'closestPoint', Point: 'closestPoint', point: 'closestPoint',
          uvP: 'uvPoint', 'UV Point': 'uvPoint', uv: 'uvPoint',
          D: 'distance', Distance: 'distance', distance: 'distance',
        },
      },
      eval: ({ inputs }) => {
        const target = inputs.point ? ensurePoint(inputs.point, null) : null;
        const entry = primarySurfaceEntry(inputs.surface);
        if (!entry || !target) {
          return {
            closestPoint: entry?.surface ? evaluateSurfacePoint(entry.surface, 0.5, 0.5) ?? null : null,
            uvPoint: { u: 0, v: 0 },
            distance: 0,
          };
        }
        const result = approximateSurfaceClosestPoint(entry.surface, target, { segmentsU: 32, segmentsV: 32 });
        if (!result) {
          const sample = evaluateSurfaceSample(entry.surface, { u: 0.5, v: 0.5 });
          const point = sample.point ? sample.point.clone() : null;
          return {
            closestPoint: point,
            uvPoint: sample.uv,
            distance: point ? point.distanceTo(target) : 0,
          };
        }
        return {
          closestPoint: result.point.clone(),
          uvPoint: { u: result.u, v: result.v },
          distance: result.distance,
        };
      },
    });

    register('{4b5f79e1-c2b3-4b9c-b97d-470145a3ca74}', {
      type: 'geometry',
      pinMap: {
        inputs: { G: 'geometry', Geometry: 'geometry', geometry: 'geometry' },
        outputs: {
          V: 'volume', Volume: 'volume',
          C: 'centroid', Centroid: 'centroid',
          I: 'inertia', Inertia: 'inertia',
          'I±': 'inertiaError', 'Inertia (error)': 'inertiaError',
          S: 'secondary', Secondary: 'secondary',
          'S±': 'secondaryError', 'Secondary (error)': 'secondaryError',
          G: 'gyration', Gyration: 'gyration',
        },
      },
      eval: ({ inputs }) => {
        const moments = computeMomentsForGeometry(inputs.geometry);
        return {
          volume: moments.volume,
          centroid: moments.centroid.clone(),
          inertia: moments.inertia.clone(),
          inertiaError: moments.inertiaError.clone(),
          secondary: moments.secondary.clone(),
          secondaryError: moments.secondaryError.clone(),
          gyration: moments.gyration.clone(),
        };
      },
    });

    register('{4beead95-8aa2-4613-8bb9-24758a0f5c4c}', {
      type: 'surface',
      pinMap: {
        inputs: {
          P: 'point', Point: 'point', point: 'point',
          B: 'brep', Brep: 'brep', brep: 'brep',
        },
        outputs: {
          P: 'closestPoint', Point: 'closestPoint', point: 'closestPoint',
          N: 'normal', Normal: 'normal', normal: 'normal',
          D: 'distance', Distance: 'distance', distance: 'distance',
        },
      },
      eval: ({ inputs }) => {
        const target = inputs.point ? ensurePoint(inputs.point, null) : null;
        const surfaces = ensureSurfaceEntries(inputs.brep);
        let result = target ? closestPointOnSurfaceEntries(surfaces, target) : null;
        if (!result && target) {
          const fallbackBox = ensureBoxData(inputs.brep) ?? computeBoundingBoxForContent(inputs.brep);
          if (fallbackBox) {
            const center = fallbackBox.center?.clone() ?? applyPlane(fallbackBox.plane ?? defaultPlane(), 0, 0, 0);
            const normal = fallbackBox.plane?.zAxis?.clone() ?? new THREE.Vector3(0, 0, 1);
            result = {
              point: center.clone(),
              normal: normal.normalize(),
              distance: center.distanceTo(target),
            };
          }
        }
        if (!result) {
          return { closestPoint: null, normal: null, distance: 0 };
        }
        return {
          closestPoint: result.point.clone(),
          normal: result.normal?.clone() ?? (surfaces[0]?.surface?.plane?.zAxis?.clone()?.normalize() ?? new THREE.Vector3(0, 0, 1)),
          distance: result.distance ?? 0,
        };
      },
    });

    register('{5d2fb801-2905-4a55-9d48-bbb22c73ad13}', {
      type: 'geometry',
      pinMap: {
        inputs: { B: 'brep', Brep: 'brep', brep: 'brep' },
        outputs: {
          A: 'area', Area: 'area',
          C: 'centroid', Centroid: 'centroid',
          I: 'inertia', Inertia: 'inertia',
          'I±': 'inertiaError', 'Inertia (error)': 'inertiaError',
          S: 'secondary', Secondary: 'secondary',
          'S±': 'secondaryError', 'Secondary (error)': 'secondaryError',
          G: 'gyration', Gyration: 'gyration',
        },
      },
      eval: ({ inputs }) => {
        const { area, centroid } = estimateGeometryAreaAndCentroid(inputs.brep);
        const moments = computeMomentsForGeometry(inputs.brep, { massOverride: area });
        return {
          area,
          centroid,
          inertia: moments.inertia.clone(),
          inertiaError: moments.inertiaError.clone(),
          secondary: moments.secondary.clone(),
          secondaryError: moments.secondaryError.clone(),
          gyration: moments.gyration.clone(),
        };
      },
    });

    register('{859daa86-3ab7-49cb-9eda-f2811c984070}', {
      type: 'surface',
      pinMap: {
        inputs: {
          B: 'breps', Brep: 'breps', brep: 'breps', Breps: 'breps',
          P: 'point', Point: 'point', point: 'point',
          S: 'strict', Strict: 'strict', strict: 'strict',
        },
        outputs: { I: 'inside', Inside: 'inside', inside: 'inside', i: 'index', Index: 'index' },
      },
      eval: ({ inputs }) => {
        const point = inputs.point ? ensurePoint(inputs.point, null) : null;
        const breps = ensureArray(inputs.breps);
        const strict = ensureBoolean(inputs.strict, false);
        let inside = false;
        let index = -1;
        if (point) {
          for (let i = 0; i < breps.length; i += 1) {
            const box = ensureBoxData(breps[i]) ?? computeBoundingBoxForContent(breps[i]);
            if (box && computeBoxInclusion(box, point, { strict })) {
              inside = true;
              index = i;
              break;
            }
          }
        }
        return { inside, index };
      },
    });

    register('{866ee39d-9ebf-4e1d-b209-324c56825605}', {
      type: 'surface',
      pinMap: {
        inputs: { B: 'brep', Brep: 'brep', brep: 'brep' },
        outputs: {
          FF: 'faceFace', 'Face|Face Adjacency': 'faceFace',
          FE: 'faceEdge', 'Face|Edge Adjacency': 'faceEdge',
          EF: 'edgeFace', 'Edge|Face Adjacency': 'edgeFace',
        },
      },
      eval: ({ inputs }) => {
        const surfaces = ensureSurfaceEntries(inputs.brep);
        if (!surfaces.length) {
          return { faceFace: [], faceEdge: [], edgeFace: [] };
        }
        const edgesInfo = computeSurfaceEdges(surfaces);
        const edges = edgesInfo.edges;
        const faceFace = surfaces.map(() => new Set());
        const faceEdge = surfaces.map(() => []);
        edges.forEach((edge, edgeIndex) => {
          const faces = edge.faces ?? [];
          faces.forEach((faceIndex) => {
            if (faceEdge[faceIndex]) {
              faceEdge[faceIndex].push(edgeIndex);
            }
            faces.forEach((other) => {
              if (other !== faceIndex) {
                faceFace[faceIndex]?.add(other);
              }
            });
          });
        });
        return {
          faceFace: faceFace.map((set) => Array.from(set ?? [])),
          faceEdge: faceEdge.map((list) => Array.from(list ?? [])),
          edgeFace: edges.map((edge) => edge.faces ? edge.faces.slice() : []),
        };
      },
    });

    register('{8d372bdc-9800-45e9-8a26-6e33c5253e21}', {
      type: 'surface',
      pinMap: {
        inputs: { B: 'brep', Brep: 'brep', brep: 'brep' },
        outputs: {
          F: 'faces', Faces: 'faces', faces: 'faces',
          E: 'edges', Edges: 'edges', edges: 'edges',
          V: 'vertices', Vertices: 'vertices', vertices: 'vertices',
        },
      },
      eval: ({ inputs }) => {
        const surfaces = ensureSurfaceEntries(inputs.brep);
        const faces = surfaces.map((entry) => entry.wrapper ?? wrapSurface(entry.surface));
        const edgesInfo = computeSurfaceEdges(surfaces);
        const edges = edgesInfo.edges.map(edgeToCurve).filter(Boolean);
        const vertexPoints = [];
        edges.forEach((edge) => {
          edge.points.forEach((pt) => vertexPoints.push(pt));
        });
        const vertices = dedupePoints(vertexPoints, 1e-6);
        return { faces, edges, vertices };
      },
    });

    register('{a10e8cdf-7c7a-4aac-aa70-ddb7010ab231}', {
      type: 'geometry',
      pinMap: {
        inputs: { B: 'box', Box: 'box', box: 'box' },
        outputs: {
          A: 'cornerA', 'Corner A': 'cornerA',
          B: 'cornerB', 'Corner B': 'cornerB',
          C: 'cornerC', 'Corner C': 'cornerC',
          D: 'cornerD', 'Corner D': 'cornerD',
          E: 'cornerE', 'Corner E': 'cornerE',
          F: 'cornerF', 'Corner F': 'cornerF',
          G: 'cornerG', 'Corner G': 'cornerG',
          H: 'cornerH', 'Corner H': 'cornerH',
        },
      },
      eval: ({ inputs }) => {
        const box = ensureBoxData(inputs.box);
        if (!box) {
          return {
            cornerA: null,
            cornerB: null,
            cornerC: null,
            cornerD: null,
            cornerE: null,
            cornerF: null,
            cornerG: null,
            cornerH: null,
          };
        }
        const plane = box.plane ?? defaultPlane();
        const min = box.localMin ?? new THREE.Vector3();
        const max = box.localMax ?? new THREE.Vector3();
        const corners = [
          applyPlane(plane, min.x, min.y, min.z),
          applyPlane(plane, max.x, min.y, min.z),
          applyPlane(plane, max.x, max.y, min.z),
          applyPlane(plane, min.x, max.y, min.z),
          applyPlane(plane, min.x, min.y, max.z),
          applyPlane(plane, max.x, min.y, max.z),
          applyPlane(plane, max.x, max.y, max.z),
          applyPlane(plane, min.x, max.y, max.z),
        ];
        return {
          cornerA: corners[0],
          cornerB: corners[1],
          cornerC: corners[2],
          cornerD: corners[3],
          cornerE: corners[4],
          cornerF: corners[5],
          cornerG: corners[6],
          cornerH: corners[7],
        };
      },
    });

    register('{aa1dc107-70de-473e-9636-836030160fc3}', {
      type: 'surface',
      pinMap: {
        inputs: {
          S: 'surface', Surface: 'surface', surface: 'surface',
          uv: 'uv', UV: 'uv', Point: 'uv', point: 'uv',
        },
        outputs: {
          P: 'point', Point: 'point', point: 'point',
          N: 'normal', Normal: 'normal', normal: 'normal',
          F: 'frame', Frame: 'frame', frame: 'frame',
        },
      },
      eval: ({ inputs }) => {
        const data = evaluateSurfaceSample(inputs.surface, inputs.uv);
        const point = data.point ? data.point.clone() : null;
        const normal = data.normal ? data.normal.clone() : null;
        const frame = cloneFrame(data.frame);
        return { point, normal, frame };
      },
    });

    register('{ab766b01-a3f5-4257-831a-fc84d7b288b4}', {
      type: 'geometry',
      pinMap: {
        inputs: { B: 'brep', Brep: 'brep', brep: 'brep' },
        outputs: { A: 'area', Area: 'area', C: 'centroid', Centroid: 'centroid' },
      },
      eval: ({ inputs }) => {
        const { area, centroid } = estimateGeometryAreaAndCentroid(inputs.brep);
        return { area, centroid };
      },
    });

    register('{ac750e41-2450-4f98-9658-98fef97b01b2}', {
      type: 'surface',
      pinMap: {
        inputs: {
          B: 'brep', Brep: 'brep', brep: 'brep',
          D: 'density', Density: 'density', density: 'density',
        },
        outputs: { W: 'wireframe', Wireframe: 'wireframe', wireframe: 'wireframe' },
      },
      eval: ({ inputs }) => {
        const surfaces = ensureSurfaceEntries(inputs.brep);
        const density = Math.max(0, Math.round(ensureNumeric(inputs.density, 3)));
        const wireframe = [];
        const signatures = new Set();
        for (const entry of surfaces) {
          const curves = generateSurfaceIsoCurves(entry.surface, density || 3, 32);
          for (const curve of curves) {
            const signature = createPolylineSignature(curve.points);
            if (!signatures.has(signature)) {
              signatures.add(signature);
              wireframe.push(curve);
            }
          }
        }
        return { wireframe };
      },
    });

    register('{af9cdb9d-9617-4827-bb3c-9efd88c76a70}', {
      type: 'geometry',
      pinMap: {
        inputs: { B: 'box', Box: 'box', box: 'box' },
        outputs: {
          C: 'center', Center: 'center', center: 'center',
          D: 'diagonal', Diagonal: 'diagonal', diagonal: 'diagonal',
          A: 'area', Area: 'area',
          V: 'volume', Volume: 'volume',
          d: 'degeneracy', Degeneracy: 'degeneracy', degeneracy: 'degeneracy',
        },
      },
      eval: ({ inputs }) => {
        const box = ensureBoxData(inputs.box);
        if (!box) {
          return { center: null, diagonal: new THREE.Vector3(), area: 0, volume: 0, degeneracy: 3 };
        }
        const metrics = computeBoxMetrics(box);
        const plane = metrics.plane;
        const min = metrics.localMin;
        const max = metrics.localMax;
        const minWorld = applyPlane(plane, min.x, min.y, min.z);
        const maxWorld = applyPlane(plane, max.x, max.y, max.z);
        const diagonal = maxWorld.clone().sub(minWorld);
        const degeneracy = [metrics.size.x, metrics.size.y, metrics.size.z].filter((value) => Math.abs(value) <= EPSILON).length;
        return {
          center: metrics.center.clone(),
          diagonal,
          area: metrics.area,
          volume: metrics.volume,
          degeneracy,
        };
      },
    });

    register('{b799b7c0-76df-4bdb-b3cc-401b1d021aa5}', {
      type: 'surface',
      pinMap: {
        inputs: {
          S: 'surface', Surface: 'surface', surface: 'surface',
          uv: 'uv', UV: 'uv', Point: 'uv', point: 'uv',
        },
        outputs: {
          P: 'point', Point: 'point', point: 'point',
          C1: 'circle1', 'First circle': 'circle1',
          C2: 'circle2', 'Second circle': 'circle2',
        },
      },
      eval: ({ inputs }) => {
        const data = evaluateSurfaceSample(inputs.surface, inputs.uv);
        const curvature = data.surface
          ? computeSurfaceCurvatureData(data.surface, data.uv.u, data.uv.v)
          : null;
        const principal = curvature?.principalCurvatures ?? [];
        const first = principal[0] ?? null;
        const second = principal[1] ?? null;
        const circle1 = first ? createOsculatingCircle(data.point, data.normal, first.direction, first.value) : null;
        const circle2 = second ? createOsculatingCircle(data.point, data.normal, second.direction, second.value) : null;
        return {
          point: data.point ? data.point.clone() : null,
          circle1,
          circle2,
        };
      },
    });

    register('{c72d0184-bb99-4af4-a629-4662e1c3d428}', {
      type: 'geometry',
      pinMap: {
        inputs: { B: 'brep', Brep: 'brep', brep: 'brep' },
        outputs: { V: 'volume', Volume: 'volume', C: 'centroid', Centroid: 'centroid' },
      },
      eval: ({ inputs }) => {
        const { volume, centroid } = estimateGeometryVolumeAndCentroid(inputs.brep);
        return { volume, centroid };
      },
    });

    register('{c98c1666-5f29-4bb8-aafd-bb5a708e8a95}', {
      type: 'geometry',
      pinMap: {
        inputs: { G: 'geometry', Geometry: 'geometry', geometry: 'geometry' },
        outputs: {
          A: 'area', Area: 'area',
          C: 'centroid', Centroid: 'centroid',
          I: 'inertia', Inertia: 'inertia',
          'I±': 'inertiaError', 'Inertia (error)': 'inertiaError',
          S: 'secondary', Secondary: 'secondary',
          'S±': 'secondaryError', 'Secondary (error)': 'secondaryError',
          G: 'gyration', Gyration: 'gyration',
        },
      },
      eval: ({ inputs }) => {
        const { area, centroid } = estimateGeometryAreaAndCentroid(inputs.geometry);
        const moments = computeMomentsForGeometry(inputs.geometry, { massOverride: area });
        return {
          area,
          centroid,
          inertia: moments.inertia.clone(),
          inertiaError: moments.inertiaError.clone(),
          secondary: moments.secondary.clone(),
          secondaryError: moments.secondaryError.clone(),
          gyration: moments.gyration.clone(),
        };
      },
    });

    register('{cdd5d441-3bad-4f19-a370-6cf180b6f0fa}', {
      type: 'surface',
      pinMap: {
        inputs: {
          P: 'point', Point: 'point', point: 'point',
          B: 'brep', Brep: 'brep', brep: 'brep',
        },
        outputs: {
          P: 'closestPoint', Point: 'closestPoint', point: 'closestPoint',
          D: 'distance', Distance: 'distance', distance: 'distance',
        },
      },
      eval: ({ inputs }) => {
        const target = inputs.point ? ensurePoint(inputs.point, null) : null;
        const surfaces = ensureSurfaceEntries(inputs.brep);
        let result = target ? closestPointOnSurfaceEntries(surfaces, target) : null;
        if (!result && target) {
          const fallbackBox = ensureBoxData(inputs.brep) ?? computeBoundingBoxForContent(inputs.brep);
          if (fallbackBox) {
            const center = fallbackBox.center?.clone() ?? applyPlane(fallbackBox.plane ?? defaultPlane(), 0, 0, 0);
            result = {
              point: center.clone(),
              distance: center.distanceTo(target),
            };
          }
        }
        if (!result) {
          return { closestPoint: null, distance: 0 };
        }
        return {
          closestPoint: result.point.clone(),
          distance: result.distance ?? 0,
        };
      },
    });

    register('{d4bc9653-c770-4bee-a31d-d120cbb75b39}', {
      type: 'surface',
      pinMap: {
        inputs: {
          S: 'surface', Surface: 'surface', surface: 'surface',
          I: 'interior', Interior: 'interior', interior: 'interior',
        },
        outputs: {
          F: 'planar', Planar: 'planar', planar: 'planar',
          P: 'plane', Plane: 'plane', plane: 'plane',
        },
      },
      eval: ({ inputs }) => {
        const entry = primarySurfaceEntry(inputs.surface);
        if (!entry) {
          return { planar: false, plane: cloneFrame(null) };
        }
        const planarity = evaluateSurfacePlanarity(entry.surface);
        return {
          planar: Boolean(planarity.planar),
          plane: cloneFrame(planarity.plane),
        };
      },
    });

    register('{db7d83b1-2898-4ef9-9be5-4e94b4e2048d}', {
      type: 'geometry',
      pinMap: {
        inputs: { B: 'box', Box: 'box', box: 'box' },
        outputs: {
          P: 'plane', Plane: 'plane', plane: 'plane',
          X: 'xSize', x: 'xSize',
          Y: 'ySize', y: 'ySize',
          Z: 'zSize', z: 'zSize',
        },
      },
      eval: ({ inputs }) => {
        const box = ensureBoxData(inputs.box);
        if (!box) {
          return { plane: cloneFrame(null), xSize: 0, ySize: 0, zSize: 0 };
        }
        const metrics = computeBoxMetrics(box);
        return {
          plane: cloneFrame(metrics.plane),
          xSize: metrics.size.x,
          ySize: metrics.size.y,
          zSize: metrics.size.z,
        };
      },
    });

    register('{e03561f8-0e66-41d3-afde-62049f152443}', {
      type: 'surface',
      pinMap: {
        inputs: {
          B: 'brep', Brep: 'brep', brep: 'brep',
          P: 'point', Point: 'point', point: 'point',
          S: 'strict', Strict: 'strict', strict: 'strict',
        },
        outputs: { I: 'inside', Inside: 'inside', inside: 'inside' },
      },
      eval: ({ inputs }) => {
        const point = inputs.point ? ensurePoint(inputs.point, null) : null;
        const box = ensureBoxData(inputs.brep) ?? computeBoundingBoxForContent(inputs.brep);
        const strict = ensureBoolean(inputs.strict, false);
        const inside = point && box ? computeBoxInclusion(box, point, { strict }) : false;
        return { inside };
      },
    });

    register('{f241e42e-8983-4ed3-b869-621c07630b00}', {
      type: 'surface',
      pinMap: {
        inputs: { S: 'surface', Surface: 'surface', surface: 'surface' },
        outputs: {
          U: 'dimensionU', 'U dimension': 'dimensionU',
          V: 'dimensionV', 'V dimension': 'dimensionV',
        },
      },
      eval: ({ inputs }) => {
        const entry = primarySurfaceEntry(inputs.surface);
        if (!entry) {
          return { dimensionU: 0, dimensionV: 0 };
        }
        const dimensions = computeSurfaceDimensions(entry.surface);
        return {
          dimensionU: dimensions.u,
          dimensionV: dimensions.v,
        };
      },
    });

    register('{f881810b-96de-4668-a95a-f9a6d683e65c}', {
      type: 'surface',
      pinMap: {
        inputs: {
          S: 'surface', Surface: 'surface', surface: 'surface',
          P: 'uv', UV: 'uv', 'UV Point': 'uv',
        },
        outputs: { I: 'inclusion', Inclusion: 'inclusion', inclusion: 'inclusion' },
      },
      eval: ({ inputs }) => {
        const entry = primarySurfaceEntry(inputs.surface);
        if (!entry) {
          return { inclusion: false };
        }
        const uv = ensureUVPoint(inputs.uv, entry.surface);
        const domainU = entry.surface.domainU ?? createDomain(0, 1);
        const domainV = entry.surface.domainV ?? createDomain(0, 1);
        const tolerance = 1e-6;
        const inclusion = uv.u >= domainU.min - tolerance
          && uv.u <= domainU.max + tolerance
          && uv.v >= domainV.min - tolerance
          && uv.v <= domainV.max + tolerance;
        return { inclusion };
      },
    });

    register('{ffdfcfc5-3933-4c38-b680-8bb530e243ff}', {
      type: 'geometry',
      pinMap: {
        inputs: { B: 'brep', Brep: 'brep', brep: 'brep' },
        outputs: {
          V: 'volume', Volume: 'volume',
          C: 'centroid', Centroid: 'centroid',
          I: 'inertia', Inertia: 'inertia',
          'I±': 'inertiaError', 'Inertia (error)': 'inertiaError',
          S: 'secondary', Secondary: 'secondary',
          'S±': 'secondaryError', 'Secondary (error)': 'secondaryError',
          G: 'gyration', Gyration: 'gyration',
        },
      },
      eval: ({ inputs }) => {
        const moments = computeMomentsForGeometry(inputs.brep);
        return {
          volume: moments.volume,
          centroid: moments.centroid.clone(),
          inertia: moments.inertia.clone(),
          inertiaError: moments.inertiaError.clone(),
          secondary: moments.secondary.clone(),
          secondaryError: moments.secondaryError.clone(),
          gyration: moments.gyration.clone(),
        };
      },
    });
  }

  const SUBD_DEFAULT_TOLERANCE = 1e-6;
  const SUBD_DEFAULT_PIPE_SEGMENTS = 8;

  function parseEdgeTagValue(value, fallback = 'smooth') {
    if (value === undefined || value === null) {
      return fallback;
    }
    if (typeof value === 'string') {
      const normalized = value.trim().toLowerCase();
      if (!normalized) {
        return fallback;
      }
      if (['crease', 'sharp', 'hard', 'c'].includes(normalized)) {
        return 'crease';
      }
      if (['smooth', 'soft', 's'].includes(normalized)) {
        return 'smooth';
      }
      return normalized;
    }
    if (typeof value === 'number') {
      return value !== 0 ? 'crease' : 'smooth';
    }
    if (typeof value === 'object') {
      if ('tag' in value) {
        return parseEdgeTagValue(value.tag, fallback);
      }
      if ('value' in value) {
        return parseEdgeTagValue(value.value, fallback);
      }
      if ('type' in value) {
        return parseEdgeTagValue(value.type, fallback);
      }
    }
    return fallback;
  }

  function parseVertexTagValue(value, fallback = 'smooth') {
    if (value === undefined || value === null) {
      return fallback;
    }
    if (typeof value === 'string') {
      const normalized = value.trim().toLowerCase();
      if (!normalized) {
        return fallback;
      }
      if (['smooth', 's'].includes(normalized)) {
        return 'smooth';
      }
      if (['crease', 'c', 'sharp'].includes(normalized)) {
        return 'crease';
      }
      if (['corner', 'l'].includes(normalized)) {
        return 'corner';
      }
      if (['dart', 'd'].includes(normalized)) {
        return 'dart';
      }
      return normalized;
    }
    if (typeof value === 'number') {
      const rounded = Math.round(value);
      if (rounded === 1) {
        return 'crease';
      }
      if (rounded === 2) {
        return 'corner';
      }
      if (rounded === 3) {
        return 'dart';
      }
      return 'smooth';
    }
    if (typeof value === 'object') {
      if ('tag' in value) {
        return parseVertexTagValue(value.tag, fallback);
      }
      if ('value' in value) {
        return parseVertexTagValue(value.value, fallback);
      }
      if ('type' in value) {
        return parseVertexTagValue(value.type, fallback);
      }
    }
    return fallback;
  }

  function createEdgeKey(a, b) {
    const first = Math.min(a, b);
    const second = Math.max(a, b);
    return `${first}|${second}`;
  }

  function createVertexKey(point) {
    const vector = cloneVector(point, new THREE.Vector3());
    const precision = 6;
    return [vector.x, vector.y, vector.z]
      .map((value) => Number.parseFloat(value).toFixed(precision))
      .join('|');
  }

  function createEdgeCoordinateKey(pointA, pointB) {
    const keyA = createVertexKey(pointA);
    const keyB = createVertexKey(pointB);
    return [keyA, keyB].sort().join('->');
  }

  function cloneVector(value, fallback = new THREE.Vector3()) {
    return ensurePoint(value, fallback.clone());
  }

  function collectIndices(input) {
    const indices = [];
    function visit(value) {
      if (value === undefined || value === null) {
        return;
      }
      if (Array.isArray(value)) {
        for (const entry of value) {
          visit(entry);
        }
        return;
      }
      if (typeof value === 'object') {
        if ('value' in value) {
          visit(value.value);
          return;
        }
        if ('id' in value) {
          visit(value.id);
          return;
        }
        if ('index' in value) {
          visit(value.index);
          return;
        }
      }
      const numeric = ensureNumeric(value, Number.NaN);
      if (Number.isFinite(numeric)) {
        indices.push(Math.round(numeric));
      }
    }
    visit(input);
    return indices;
  }

  function computeFaceCentroid(vertexIds, vertices) {
    if (!vertexIds.length) {
      return new THREE.Vector3();
    }
    const centroid = new THREE.Vector3();
    vertexIds.forEach((id) => {
      const vertex = vertices[id];
      if (vertex) {
        centroid.add(vertex.point);
      }
    });
    centroid.multiplyScalar(1 / vertexIds.length);
    return centroid;
  }

  function buildEdgesForFaces(faces, vertices, existingTags = new Map()) {
    const edgesMap = new Map();
    faces.forEach((face) => {
      const ids = face.vertices ?? [];
      for (let i = 0; i < ids.length; i += 1) {
        const a = ids[i];
        const b = ids[(i + 1) % ids.length];
        if (a === undefined || b === undefined) {
          continue;
        }
        const key = createEdgeKey(a, b);
        if (!edgesMap.has(key)) {
          edgesMap.set(key, {
            id: edgesMap.size,
            vertices: [a, b],
            faces: [],
            tag: existingTags.get(key) ?? 'smooth',
          });
        }
        const edge = edgesMap.get(key);
        if (!edge.faces.includes(face.id)) {
          edge.faces.push(face.id);
        }
      }
    });
    return Array.from(edgesMap.values());
  }

  function createSubDFromPolygons(polygons, { metadata = {}, defaultEdgeTag = 'smooth', defaultVertexTag = 'smooth' } = {}) {
    if (!Array.isArray(polygons) || !polygons.length) {
      return {
        type: 'subd',
        vertices: [],
        edges: [],
        faces: [],
        metadata: { ...metadata },
      };
    }
    const toleranceSq = SUBD_DEFAULT_TOLERANCE * SUBD_DEFAULT_TOLERANCE;
    const vertices = [];
    const vertexLookup = [];
    function getVertexId(point) {
      for (let i = 0; i < vertices.length; i += 1) {
        if (vertices[i].point.distanceToSquared(point) <= toleranceSq) {
          return vertices[i].id;
        }
      }
      const id = vertices.length;
      const entry = {
        id,
        point: point.clone(),
        tag: defaultVertexTag,
      };
      vertices.push(entry);
      vertexLookup[id] = entry;
      return id;
    }
    const faces = [];
    polygons.forEach((polygon, faceIndex) => {
      if (!Array.isArray(polygon) || polygon.length < 3) {
        return;
      }
      const vertexIds = polygon.map((point) => getVertexId(cloneVector(point)));
      const face = {
        id: faces.length,
        vertices: vertexIds,
        edges: [],
        centroid: computeFaceCentroid(vertexIds, vertexLookup),
      };
      faces.push(face);
    });
    const edges = buildEdgesForFaces(faces, vertexLookup);
    edges.forEach((edge) => {
      edge.tag = edge.tag ?? defaultEdgeTag;
    });
    faces.forEach((face) => {
      const ids = face.vertices ?? [];
      const faceEdges = [];
      for (let i = 0; i < ids.length; i += 1) {
        const a = ids[i];
        const b = ids[(i + 1) % ids.length];
        const key = createEdgeKey(a, b);
        const edge = edges.find((entry) => createEdgeKey(entry.vertices[0], entry.vertices[1]) === key);
        if (edge) {
          faceEdges.push(edge.id);
        }
      }
      face.edges = faceEdges;
    });
    return {
      type: 'subd',
      vertices: vertices.map((vertex) => ({
        id: vertex.id,
        point: vertex.point.clone(),
        tag: vertex.tag ?? defaultVertexTag,
      })),
      edges: edges.map((edge) => ({
        id: edge.id,
        vertices: edge.vertices.slice(),
        faces: edge.faces.slice(),
        tag: parseEdgeTagValue(edge.tag, defaultEdgeTag),
      })),
      faces: faces.map((face) => ({
        id: face.id,
        vertices: face.vertices.slice(),
        edges: face.edges.slice(),
        centroid: face.centroid.clone(),
      })),
      metadata: { ...metadata },
    };
  }

  function cloneSubD(subd) {
    if (!subd || subd.type !== 'subd') {
      return null;
    }
    return {
      type: 'subd',
      vertices: (subd.vertices ?? []).map((vertex, index) => ({
        id: index,
        point: cloneVector(vertex.point ?? vertex.position ?? vertex),
        tag: parseVertexTagValue(vertex.tag ?? vertex.type ?? 'smooth'),
      })),
      edges: (subd.edges ?? []).map((edge, index) => ({
        id: index,
        vertices: Array.isArray(edge.vertices) ? edge.vertices.map((value) => Number(value)) : [],
        faces: Array.isArray(edge.faces) ? edge.faces.map((value) => Number(value)) : [],
        tag: parseEdgeTagValue(edge.tag ?? edge.type ?? 'smooth'),
      })),
      faces: (subd.faces ?? []).map((face, index) => ({
        id: index,
        vertices: Array.isArray(face.vertices) ? face.vertices.map((value) => Number(value)) : [],
        edges: Array.isArray(face.edges) ? face.edges.map((value) => Number(value)) : [],
        centroid: cloneVector(face.centroid ?? new THREE.Vector3()),
      })),
      metadata: { ...(subd.metadata ?? {}) },
    };
  }

  function normalizeSubD(subd) {
    if (!subd) {
      return null;
    }
    if (subd.type === 'subd' && Array.isArray(subd.vertices) && Array.isArray(subd.faces)) {
      const cloned = cloneSubD(subd);
      const facePolygons = cloned.faces.map((face) => face.vertices.map((id) => cloned.vertices[id]?.point ?? new THREE.Vector3()));
      const normalized = createSubDFromPolygons(facePolygons, { metadata: { ...(cloned.metadata ?? {}) } });
      const edgeTagMap = new Map();
      (subd.edges ?? []).forEach((edge) => {
        if (!Array.isArray(edge.vertices) || edge.vertices.length < 2) {
          return;
        }
        const a = subd.vertices?.[edge.vertices[0]];
        const b = subd.vertices?.[edge.vertices[1]];
        if (!a || !b) {
          return;
        }
        const key = createEdgeCoordinateKey(a.point ?? a.position ?? a, b.point ?? b.position ?? b);
        edgeTagMap.set(key, parseEdgeTagValue(edge.tag ?? 'smooth'));
      });
      normalized.edges.forEach((edge) => {
        const start = normalized.vertices[edge.vertices[0]];
        const end = normalized.vertices[edge.vertices[1]];
        if (!start || !end) {
          return;
        }
        const key = createEdgeCoordinateKey(start.point, end.point);
        if (edgeTagMap.has(key)) {
          edge.tag = edgeTagMap.get(key);
        }
      });
      const vertexTagMap = new Map();
      (subd.vertices ?? []).forEach((vertex) => {
        const key = createVertexKey(vertex.point ?? vertex.position ?? vertex);
        vertexTagMap.set(key, parseVertexTagValue(vertex.tag ?? 'smooth'));
      });
      normalized.vertices.forEach((vertex) => {
        const key = createVertexKey(vertex.point);
        if (vertexTagMap.has(key)) {
          vertex.tag = vertexTagMap.get(key);
        }
      });
      normalized.metadata = { ...(cloned.metadata ?? {}) };
      return normalized;
    }
    if (subd.subd) {
      return normalizeSubD(subd.subd);
    }
    return null;
  }

  function ensureSubD(input) {
    if (!input) {
      return null;
    }
    if (input.type === 'subd' || input.subd) {
      return normalizeSubD(input);
    }
    const meshSubD = createSubDFromMesh(input);
    if (meshSubD) {
      return meshSubD;
    }
    return null;
  }

  function applyEdgeTags(subd, ids, tag) {
    if (!subd) {
      return null;
    }
    const normalized = normalizeSubD(subd);
    if (!normalized) {
      return null;
    }
    const tagValue = parseEdgeTagValue(tag, 'smooth');
    const idSet = new Set(ids);
    normalized.edges.forEach((edge, index) => {
      if (idSet.has(index) || idSet.has(edge.id)) {
        edge.tag = tagValue;
      }
    });
    return normalized;
  }

  function applyVertexTags(subd, ids, tag) {
    if (!subd) {
      return null;
    }
    const normalized = normalizeSubD(subd);
    if (!normalized) {
      return null;
    }
    const tagValue = parseVertexTagValue(tag, 'smooth');
    const idSet = new Set(ids);
    normalized.vertices.forEach((vertex, index) => {
      if (idSet.has(index) || idSet.has(vertex.id)) {
        vertex.tag = tagValue;
      }
    });
    return normalized;
  }

  function subdFacesToPolygons(subd) {
    if (!subd) {
      return [];
    }
    const normalized = normalizeSubD(subd);
    if (!normalized) {
      return [];
    }
    return normalized.faces.map((face) => face.vertices.map((id) => normalized.vertices[id]?.point.clone() ?? new THREE.Vector3()));
  }

  function computeSubDBox(subd) {
    const normalized = normalizeSubD(subd);
    if (!normalized || !normalized.vertices.length) {
      return null;
    }
    const points = normalized.vertices.map((vertex) => vertex.point.clone());
    const box3 = new THREE.Box3();
    box3.setFromPoints(points);
    if (!Number.isFinite(box3.min.x) || !Number.isFinite(box3.max.x)) {
      return null;
    }
    return createBoxDataFromPlaneExtents({ plane: defaultPlane(), min: box3.min, max: box3.max });
  }

  function mergeSubDs(subdA, subdB) {
    const polygons = [...subdFacesToPolygons(subdA), ...subdFacesToPolygons(subdB)];
    return createSubDFromPolygons(polygons, {
      metadata: {
        merged: true,
        sources: [subdA?.metadata ?? null, subdB?.metadata ?? null].filter(Boolean),
      },
    });
  }

  function subtractSubD(base, subtractor) {
    const baseNormalized = normalizeSubD(base);
    if (!baseNormalized) {
      return null;
    }
    const subtractBox = computeSubDBox(subtractor);
    if (!subtractBox) {
      return cloneSubD(baseNormalized);
    }
    const polygons = [];
    baseNormalized.faces.forEach((face) => {
      const centroid = face.centroid ?? computeFaceCentroid(face.vertices, baseNormalized.vertices);
      const include = !computeBoxInclusion(subtractBox, centroid, { strict: false });
      if (include) {
        const polygon = face.vertices.map((id) => baseNormalized.vertices[id]?.point.clone() ?? new THREE.Vector3());
        polygons.push(polygon);
      }
    });
    return createSubDFromPolygons(polygons, {
      metadata: {
        difference: true,
        source: baseNormalized.metadata ?? null,
        subtract: subtractor?.metadata ?? null,
      },
    });
  }

  function intersectSubDs(subdA, subdB) {
    const boxA = computeSubDBox(subdA);
    const boxB = computeSubDBox(subdB);
    if (!boxA || !boxB) {
      return mergeSubDs(subdA, subdB);
    }
    const combinedA = new THREE.Box3(boxA.box3?.min ?? boxA.localMin, boxA.box3?.max ?? boxA.localMax);
    const combinedB = new THREE.Box3(boxB.box3?.min ?? boxB.localMin, boxB.box3?.max ?? boxB.localMax);
    const intersection = combinedA.clone().intersect(combinedB);
    if (intersection.isEmpty()) {
      return createSubDFromPolygons([], { metadata: { empty: true } });
    }
    const plane = defaultPlane();
    return createSubDFromPolygons([
      [
        applyPlane(plane, intersection.min.x, intersection.min.y, intersection.min.z),
        applyPlane(plane, intersection.max.x, intersection.min.y, intersection.min.z),
        applyPlane(plane, intersection.max.x, intersection.max.y, intersection.min.z),
        applyPlane(plane, intersection.min.x, intersection.max.y, intersection.min.z),
      ],
      [
        applyPlane(plane, intersection.min.x, intersection.min.y, intersection.max.z),
        applyPlane(plane, intersection.max.x, intersection.min.y, intersection.max.z),
        applyPlane(plane, intersection.max.x, intersection.max.y, intersection.max.z),
        applyPlane(plane, intersection.min.x, intersection.max.y, intersection.max.z),
      ],
      [
        applyPlane(plane, intersection.min.x, intersection.min.y, intersection.min.z),
        applyPlane(plane, intersection.max.x, intersection.min.y, intersection.min.z),
        applyPlane(plane, intersection.max.x, intersection.min.y, intersection.max.z),
        applyPlane(plane, intersection.min.x, intersection.min.y, intersection.max.z),
      ],
      [
        applyPlane(plane, intersection.max.x, intersection.min.y, intersection.min.z),
        applyPlane(plane, intersection.max.x, intersection.max.y, intersection.min.z),
        applyPlane(plane, intersection.max.x, intersection.max.y, intersection.max.z),
        applyPlane(plane, intersection.max.x, intersection.min.y, intersection.max.z),
      ],
      [
        applyPlane(plane, intersection.max.x, intersection.max.y, intersection.min.z),
        applyPlane(plane, intersection.min.x, intersection.max.y, intersection.min.z),
        applyPlane(plane, intersection.min.x, intersection.max.y, intersection.max.z),
        applyPlane(plane, intersection.max.x, intersection.max.y, intersection.max.z),
      ],
      [
        applyPlane(plane, intersection.min.x, intersection.max.y, intersection.min.z),
        applyPlane(plane, intersection.min.x, intersection.min.y, intersection.min.z),
        applyPlane(plane, intersection.min.x, intersection.min.y, intersection.max.z),
        applyPlane(plane, intersection.min.x, intersection.max.y, intersection.max.z),
      ],
    ], {
      metadata: {
        intersection: true,
        sources: [subdA?.metadata ?? null, subdB?.metadata ?? null].filter(Boolean),
      },
    });
  }

  function collectVertexNeighbors(subd) {
    const neighbors = new Map();
    (subd.edges ?? []).forEach((edge) => {
      const [a, b] = edge.vertices ?? [];
      if (!Number.isFinite(a) || !Number.isFinite(b)) {
        return;
      }
      if (!neighbors.has(a)) {
        neighbors.set(a, new Set());
      }
      if (!neighbors.has(b)) {
        neighbors.set(b, new Set());
      }
      neighbors.get(a).add(b);
      neighbors.get(b).add(a);
    });
    return neighbors;
  }

  function smoothSubD(subd, steps = 0) {
    const normalized = normalizeSubD(subd);
    if (!normalized || steps <= 0) {
      return normalized;
    }
    const result = cloneSubD(normalized);
    const neighbors = collectVertexNeighbors(result);
    for (let step = 0; step < steps; step += 1) {
      const newPositions = result.vertices.map((vertex) => vertex.point.clone());
      result.vertices.forEach((vertex, index) => {
        const adjacency = neighbors.get(index);
        const isBoundary = (result.edges ?? []).some((edge) => {
          if (!edge.vertices.includes(index)) {
            return false;
          }
          return edge.tag === 'crease' || edge.faces.length <= 1;
        });
        if (vertex.tag === 'corner' || vertex.tag === 'dart' || isBoundary || !adjacency || !adjacency.size) {
          return;
        }
        const average = new THREE.Vector3();
        adjacency.forEach((neighborId) => {
          const neighbor = result.vertices[neighborId];
          if (neighbor) {
            average.add(neighbor.point);
          }
        });
        average.multiplyScalar(1 / adjacency.size);
        newPositions[index] = vertex.point.clone().lerp(average, 0.5);
      });
      result.vertices.forEach((vertex, index) => {
        vertex.point.copy(newPositions[index]);
      });
    }
    return result;
  }

  function edgeToLineSegment(edge, vertices) {
    if (!edge || !Array.isArray(edge.vertices) || edge.vertices.length < 2) {
      return null;
    }
    const start = vertices[edge.vertices[0]]?.point ?? new THREE.Vector3();
    const end = vertices[edge.vertices[1]]?.point ?? new THREE.Vector3(1, 0, 0);
    const direction = end.clone().sub(start);
    const length = direction.length();
    const safeDirection = length > EPSILON ? direction.clone().divideScalar(length) : new THREE.Vector3(1, 0, 0);
    return {
      type: 'line',
      start: start.clone(),
      end: end.clone(),
      length,
      direction: safeDirection,
    };
  }

  function edgeToPolyline(edge, vertices) {
    if (!edge || !Array.isArray(edge.vertices) || edge.vertices.length < 2) {
      return null;
    }
    const points = edge.vertices.map((id) => vertices[id]?.point.clone() ?? new THREE.Vector3());
    return {
      type: 'polyline',
      points,
      closed: false,
      length: computePolylineLength(points, false),
      tag: edge.tag ?? 'smooth',
    };
  }

  function createMeshFromSubD(subd, { triangulate = true, metadata = {} } = {}) {
    const normalized = normalizeSubD(subd);
    if (!normalized) {
      return null;
    }
    const vertices = normalized.vertices.map((vertex) => vertex.point.clone());
    const faces = [];
    normalized.faces.forEach((face) => {
      if (!face.vertices || face.vertices.length < 3) {
        return;
      }
      if (!triangulate) {
        faces.push(face.vertices.slice());
        return;
      }
      for (let i = 1; i < face.vertices.length - 1; i += 1) {
        faces.push([face.vertices[0], face.vertices[i], face.vertices[i + 1]]);
      }
    });
    let geometry = null;
    if (vertices.length) {
      const positionArray = new Float32Array(vertices.length * 3);
      vertices.forEach((vertex, index) => {
        positionArray[index * 3] = vertex.x;
        positionArray[index * 3 + 1] = vertex.y;
        positionArray[index * 3 + 2] = vertex.z;
      });
      geometry = new THREE.BufferGeometry();
      geometry.setAttribute('position', new THREE.BufferAttribute(positionArray, 3));
      if (faces.length) {
        const indexArray = new Uint32Array(faces.length * 3);
        faces.forEach((face, faceIndex) => {
          indexArray[faceIndex * 3] = face[0];
          indexArray[faceIndex * 3 + 1] = face[1];
          indexArray[faceIndex * 3 + 2] = face[2];
        });
        geometry.setIndex(new THREE.BufferAttribute(indexArray, 1));
        geometry.computeVertexNormals();
      }
    }
    return {
      type: 'mesh',
      vertices,
      faces,
      geometry,
      metadata: { ...metadata, source: 'subd' },
    };
  }

  function createControlPolygonFromSubD(subd) {
    const normalized = normalizeSubD(subd);
    if (!normalized) {
      return null;
    }
    const edges = normalized.edges.map((edge) => ({
      id: edge.id,
      line: edgeToLineSegment(edge, normalized.vertices),
      tag: edge.tag,
    }));
    return {
      type: 'mesh',
      edges,
      vertices: normalized.vertices.map((vertex) => vertex.point.clone()),
      metadata: { ...(normalized.metadata ?? {}), controlPolygon: true },
    };
  }

  function isMeshCandidate(input) {
    if (!input || typeof input !== 'object') {
      return false;
    }
    if (input.type === 'mesh') {
      return true;
    }
    if (input.isBufferGeometry || input.isGeometry) {
      return true;
    }
    if (Array.isArray(input.vertices) && Array.isArray(input.faces)) {
      return true;
    }
    if (input.geometry) {
      return isMeshCandidate(input.geometry);
    }
    return false;
  }

  function extractMeshPolygons(input) {
    if (!input) {
      return [];
    }
    if (input.type === 'mesh' && Array.isArray(input.vertices) && Array.isArray(input.faces)) {
      return input.faces.map((face) => face.map((id) => cloneVector(input.vertices[id])));
    }
    if (input.geometry) {
      return extractMeshPolygons(input.geometry);
    }
    if (input.isBufferGeometry && input.attributes?.position) {
      const position = input.attributes.position;
      const points = [];
      for (let i = 0; i < position.count; i += 1) {
        points.push(new THREE.Vector3(position.getX(i), position.getY(i), position.getZ(i)));
      }
      const polygons = [];
      if (input.index) {
        const indexArray = input.index.array;
        for (let i = 0; i < indexArray.length; i += 3) {
          polygons.push([
            points[indexArray[i]].clone(),
            points[indexArray[i + 1]].clone(),
            points[indexArray[i + 2]].clone(),
          ]);
        }
      } else {
        for (let i = 0; i < points.length; i += 3) {
          polygons.push([
            points[i].clone(),
            points[i + 1] ? points[i + 1].clone() : points[i].clone(),
            points[i + 2] ? points[i + 2].clone() : points[i].clone(),
          ]);
        }
      }
      return polygons;
    }
    if (input.isGeometry && Array.isArray(input.faces) && Array.isArray(input.vertices)) {
      return input.faces.map((face) => {
        const ids = [face.a, face.b, face.c].filter((id) => Number.isFinite(id));
        return ids.map((id) => cloneVector(input.vertices[id]));
      });
    }
    if (Array.isArray(input)) {
      return input
        .map((face) => (Array.isArray(face) ? face.map((pt) => cloneVector(pt)) : null))
        .filter(Boolean);
    }
    return [];
  }

  function createSubDFromMesh(input, options = {}) {
    if (!isMeshCandidate(input)) {
      return null;
    }
    const polygons = extractMeshPolygons(input);
    if (!polygons.length) {
      return null;
    }
    const subd = createSubDFromPolygons(polygons, options);
    return subd;
  }

  function createSubDBox(boxInput, { density = 1, creases = false } = {}) {
    const box = ensureBoxData(boxInput);
    if (!box) {
      return null;
    }
    const plane = box.plane ?? defaultPlane();
    const localMin = box.localMin ?? new THREE.Vector3();
    const localMax = box.localMax ?? new THREE.Vector3(1, 1, 1);
    const segments = Math.max(1, Math.round(density));
    const xs = [];
    const ys = [];
    const zs = [];
    for (let i = 0; i <= segments; i += 1) {
      const t = i / segments;
      xs.push(localMin.x + (localMax.x - localMin.x) * t);
      ys.push(localMin.y + (localMax.y - localMin.y) * t);
      zs.push(localMin.z + (localMax.z - localMin.z) * t);
    }
    function toWorld(x, y, z) {
      return applyPlane(plane, x, y, z);
    }
    const polygons = [];
    for (let iy = 0; iy < ys.length - 1; iy += 1) {
      for (let ix = 0; ix < xs.length - 1; ix += 1) {
        polygons.push([
          toWorld(xs[ix], ys[iy], localMin.z),
          toWorld(xs[ix + 1], ys[iy], localMin.z),
          toWorld(xs[ix + 1], ys[iy + 1], localMin.z),
          toWorld(xs[ix], ys[iy + 1], localMin.z),
        ]);
        polygons.push([
          toWorld(xs[ix], ys[iy], localMax.z),
          toWorld(xs[ix + 1], ys[iy], localMax.z),
          toWorld(xs[ix + 1], ys[iy + 1], localMax.z),
          toWorld(xs[ix], ys[iy + 1], localMax.z),
        ]);
      }
    }
    for (let iz = 0; iz < zs.length - 1; iz += 1) {
      for (let ix = 0; ix < xs.length - 1; ix += 1) {
        polygons.push([
          toWorld(xs[ix], localMin.y, zs[iz]),
          toWorld(xs[ix + 1], localMin.y, zs[iz]),
          toWorld(xs[ix + 1], localMin.y, zs[iz + 1]),
          toWorld(xs[ix], localMin.y, zs[iz + 1]),
        ]);
        polygons.push([
          toWorld(xs[ix], localMax.y, zs[iz]),
          toWorld(xs[ix + 1], localMax.y, zs[iz]),
          toWorld(xs[ix + 1], localMax.y, zs[iz + 1]),
          toWorld(xs[ix], localMax.y, zs[iz + 1]),
        ]);
      }
    }
    for (let iz = 0; iz < zs.length - 1; iz += 1) {
      for (let iy = 0; iy < ys.length - 1; iy += 1) {
        polygons.push([
          toWorld(localMin.x, ys[iy], zs[iz]),
          toWorld(localMin.x, ys[iy + 1], zs[iz]),
          toWorld(localMin.x, ys[iy + 1], zs[iz + 1]),
          toWorld(localMin.x, ys[iy], zs[iz + 1]),
        ]);
        polygons.push([
          toWorld(localMax.x, ys[iy], zs[iz]),
          toWorld(localMax.x, ys[iy + 1], zs[iz]),
          toWorld(localMax.x, ys[iy + 1], zs[iz + 1]),
          toWorld(localMax.x, ys[iy], zs[iz + 1]),
        ]);
      }
    }
    const subd = createSubDFromPolygons(polygons, {
      metadata: {
        type: 'box',
        density: segments,
        creases,
      },
    });
    if (creases) {
      const idSet = new Set();
      subd.edges.forEach((edge) => {
        if (edge.faces.length <= 1) {
          idSet.add(edge.id);
        }
      });
      idSet.forEach((edgeId) => {
        const edge = subd.edges[edgeId];
        if (edge) {
          edge.tag = 'crease';
        }
      });
    }
    return subd;
  }

  function collectPipePath(curveInput, segments = 16) {
    const sample = sampleCurvePoints(curveInput, segments);
    if (!sample.points.length) {
      return null;
    }
    return sample;
  }

  function determineNodeSizeMap(points, nodeSizeInput, sizePointsInput) {
    const defaultRadius = Math.max(Math.abs(ensureNumeric(nodeSizeInput, 1)), EPSILON);
    const map = new Map();
    points.forEach((point, index) => {
      map.set(index, defaultRadius);
    });
    const explicitSizes = ensureArray(nodeSizeInput);
    const sizePoints = ensureArray(sizePointsInput);
    if (explicitSizes.length && sizePoints.length && explicitSizes.length === sizePoints.length) {
      const tolerance = SUBD_DEFAULT_TOLERANCE * SUBD_DEFAULT_TOLERANCE;
      sizePoints.forEach((sizePoint, index) => {
        const point = cloneVector(sizePoint);
        let bestIndex = -1;
        let bestDistance = Number.POSITIVE_INFINITY;
        points.forEach((candidate, candidateIndex) => {
          const distance = candidate.distanceToSquared(point);
          if (distance < bestDistance) {
            bestDistance = distance;
            bestIndex = candidateIndex;
          }
        });
        if (bestIndex >= 0 && bestDistance <= tolerance) {
          const value = ensureNumeric(explicitSizes[index], defaultRadius);
          map.set(bestIndex, Math.max(Math.abs(value), EPSILON));
        }
      });
    }
    return { map, defaultRadius };
  }

  function createPipeSubD(curvesInput, options = {}) {
    const curves = ensureArray(curvesInput);
    const polygons = [];
    const metadata = { type: 'pipe', options };
    for (const curve of curves) {
      const sample = collectPipePath(curve, options.segments ?? DEFAULT_CURVE_SEGMENTS);
      if (!sample) {
        continue;
      }
      const frames = createFramesAlongPath(sample.points, sample.points.length >= 2 ? {
        origin: sample.points[0],
        xAxis: new THREE.Vector3(1, 0, 0),
        yAxis: new THREE.Vector3(0, 1, 0),
        zAxis: new THREE.Vector3(0, 0, 1),
      } : defaultPlane(), { closed: sample.closed });
      const nodeData = determineNodeSizeMap(sample.points, options.nodeSize ?? options.radius ?? 1, options.sizePoints);
      const segments = Math.max(3, Math.round(options.circleSegments ?? SUBD_DEFAULT_PIPE_SEGMENTS));
      const ringPoints = [];
      for (let i = 0; i < sample.points.length; i += 1) {
        const frame = frames[i] ?? frames[frames.length - 1];
        const radius = nodeData.map.get(i) ?? nodeData.defaultRadius;
        const circle = [];
        for (let segment = 0; segment < segments; segment += 1) {
          const angle = (segment / segments) * Math.PI * 2;
          const offset = frame.xAxis.clone().multiplyScalar(Math.cos(angle) * radius)
            .add(frame.yAxis.clone().multiplyScalar(Math.sin(angle) * radius));
          circle.push(sample.points[i].clone().add(offset));
        }
        ringPoints.push(circle);
      }
      for (let i = 0; i < ringPoints.length - 1; i += 1) {
        const ringA = ringPoints[i];
        const ringB = ringPoints[i + 1];
        for (let segment = 0; segment < segments; segment += 1) {
          const next = (segment + 1) % segments;
          polygons.push([
            ringA[segment],
            ringA[next],
            ringB[next],
            ringB[segment],
          ]);
        }
      }
      if (options.caps && options.caps > 0 && ringPoints.length) {
        const startRing = ringPoints[0];
        const endRing = ringPoints[ringPoints.length - 1];
        const startCenter = sample.points[0].clone();
        const endCenter = sample.points[sample.points.length - 1].clone();
        const startPolygon = startRing.slice().reverse();
        const endPolygon = endRing.slice();
        polygons.push(startPolygon);
        polygons.push(endPolygon);
        polygons.push(startRing.map((pt) => startCenter.clone()));
        polygons.push(endRing.map((pt) => endCenter.clone()));
      }
    }
    if (!polygons.length) {
      return null;
    }
    return createSubDFromPolygons(polygons, { metadata });
  }

  function registerSubDComponents() {
    register('{048b219e-284a-49f2-ae40-a60465b08447}', {
      type: 'subd',
      pinMap: {
        inputs: {
          S: 'subd', SubD: 'subd', subd: 'subd',
          T: 'tag', Tag: 'tag', tag: 'tag', 'Edge Tag': 'tag', edgeTag: 'tag',
          E: 'edgeIds', 'Edge IDs': 'edgeIds', edgeIds: 'edgeIds',
        },
        outputs: { S: 'subd', SubD: 'subd', subd: 'subd' },
      },
      eval: ({ inputs }) => {
        const base = ensureSubD(inputs.subd);
        if (!base) {
          return {};
        }
        const ids = collectIndices(inputs.edgeIds);
        if (!ids.length) {
          return { subd: base };
        }
        const updated = applyEdgeTags(base, ids, inputs.tag ?? 'smooth');
        return updated ? { subd: updated } : { subd: base };
      },
    });

    register('{10487e4e-a405-48b5-b188-5a8a6328418b}', {
      type: 'subd',
      pinMap: {
        inputs: {
          B: 'box', Box: 'box', box: 'box',
          D: 'density', Density: 'density', density: 'density',
          C: 'crease', Creases: 'crease', creases: 'crease',
        },
        outputs: { S: 'subd', SubD: 'subd', subd: 'subd' },
      },
      eval: ({ inputs }) => {
        const density = Math.max(1, Math.round(ensureNumeric(inputs.density, 1)));
        const creases = ensureBoolean(inputs.crease, false);
        const subd = createSubDBox(inputs.box, { density, creases });
        return subd ? { subd } : {};
      },
    });

    register('{2183c4c6-b5b3-45d2-9261-2096c9357f92}', {
      type: 'subd',
      pinMap: {
        inputs: { S: 'subd', SubD: 'subd', subd: 'subd' },
        outputs: {
          L: 'lines', Line: 'lines', lines: 'lines',
          E: 'edges', Edge: 'edges', edges: 'edges',
          T: 'tags', Tag: 'tags', tags: 'tags',
          I: 'ids', Id: 'ids', ids: 'ids',
        },
      },
      eval: ({ inputs }) => {
        const subd = ensureSubD(inputs.subd);
        if (!subd) {
          return { lines: [], edges: [], tags: [], ids: [] };
        }
        const lines = [];
        const curves = [];
        const tags = [];
        const ids = [];
        (subd.edges ?? []).forEach((edge) => {
          const line = edgeToLineSegment(edge, subd.vertices);
          const curve = edgeToPolyline(edge, subd.vertices);
          if (line) {
            lines.push(line);
          }
          if (curve) {
            curves.push(curve);
          }
          tags.push(edge.tag ?? 'smooth');
          ids.push(edge.id);
        });
        return { lines, edges: curves, tags, ids };
      },
    });

    register('{264b4aa6-4915-4a67-86a7-22a5c4acf565}', {
      type: 'subd',
      pinMap: {
        inputs: {
          A: 'subdA', 'SubD A': 'subdA', subdA: 'subdA',
          B: 'subdB', 'SubD B': 'subdB', subdB: 'subdB',
          O: 'option', Option: 'option', option: 'option', 'Boolean Option': 'option',
          S: 'smoothing', Smoothing: 'smoothing', smoothing: 'smoothing',
        },
        outputs: { F: 'subd', Fuse: 'subd', subd: 'subd' },
      },
      eval: ({ inputs }) => {
        const a = ensureSubD(inputs.subdA);
        const b = ensureSubD(inputs.subdB);
        if (!a && !b) {
          return {};
        }
        const option = Math.max(0, Math.min(3, Math.round(ensureNumeric(inputs.option, 0)))) || 0;
        let result = null;
        if (option === 0) {
          result = mergeSubDs(a ?? b, b ?? a);
        } else if (option === 1) {
          result = intersectSubDs(a ?? b, b ?? a);
        } else if (option === 2) {
          result = subtractSubD(a ?? b, b ?? null) ?? a ?? b;
        } else if (option === 3) {
          result = subtractSubD(b ?? a, a ?? null) ?? b ?? a;
        }
        const smoothing = Math.max(0, Math.round(ensureNumeric(inputs.smoothing, 0)));
        const fused = smoothing > 0 ? smoothSubD(result, smoothing) : result;
        return fused ? { subd: fused } : {};
      },
    });

    function registerMultiPipe(guid, pinMapInputs) {
      register(guid, {
        type: 'subd',
        pinMap: {
          inputs: pinMapInputs,
          outputs: { P: 'subd', Pipe: 'subd', subd: 'subd' },
        },
        eval: ({ inputs }) => {
          const curves = ensureArray(inputs.curves);
          if (!curves.length) {
            return {};
          }
          const options = {
            nodeSize: inputs.nodeSize,
            sizePoints: inputs.sizePoints,
            endOffset: ensureNumeric(inputs.endOffset, 0),
            strutSize: ensureNumeric(inputs.strutSize, 1),
            segment: ensureNumeric(inputs.segment, 0),
            kinkAngle: ensureNumeric(inputs.kinkAngle, 0),
            cubeFit: ensureNumeric(inputs.cubeFit, 0),
            caps: ensureNumeric(inputs.caps, 0),
          };
          const subd = createPipeSubD(curves, options);
          return subd ? { subd } : {};
        },
      });
    }

    registerMultiPipe('{4bfe1bf6-fbc9-4ad2-bf28-a7402e1392ee}', {
      Curves: 'curves', C: 'curves', curves: 'curves',
      NodeSize: 'nodeSize', N: 'nodeSize', nodeSize: 'nodeSize',
      SizePoints: 'sizePoints', SP: 'sizePoints', sizePoints: 'sizePoints',
      EndOffset: 'endOffset', E: 'endOffset', endOffset: 'endOffset',
      StrutSize: 'strutSize', SS: 'strutSize', strutSize: 'strutSize',
      Segment: 'segment', S: 'segment', segment: 'segment',
      KinkAngle: 'kinkAngle', KA: 'kinkAngle', kinkAngle: 'kinkAngle',
      CubeFit: 'cubeFit', CF: 'cubeFit', cubeFit: 'cubeFit',
      Caps: 'caps', cap: 'caps', caps: 'caps',
    });

    registerMultiPipe('{f1b75016-5818-4ece-be56-065253a2357d}', {
      C: 'curves', Curves: 'curves', curves: 'curves',
      N: 'nodeSize', NodeSize: 'nodeSize', nodeSize: 'nodeSize',
      SP: 'sizePoints', SizePoints: 'sizePoints', sizePoints: 'sizePoints',
      E: 'endOffset', EndOffset: 'endOffset', endOffset: 'endOffset',
      SS: 'strutSize', StrutSize: 'strutSize', strutSize: 'strutSize',
      S: 'segment', Segment: 'segment', segment: 'segment',
      KA: 'kinkAngle', KinkAngle: 'kinkAngle', kinkAngle: 'kinkAngle',
      CF: 'cubeFit', CubeFit: 'cubeFit', cubeFit: 'cubeFit',
      Caps: 'caps', cap: 'caps', caps: 'caps',
    });

    register('{83c81431-17bc-4bff-bb85-be0a846bd044}', {
      type: 'subd',
      pinMap: {
        inputs: { S: 'subd', SubD: 'subd', subd: 'subd' },
        outputs: {
          P: 'points', Point: 'points', points: 'points',
          C: 'counts', Count: 'counts', counts: 'counts',
          E: 'edges', Edges: 'edges', edges: 'edges',
          V: 'vertices', Vertices: 'vertices', vertices: 'vertices',
        },
      },
      eval: ({ inputs }) => {
        const subd = ensureSubD(inputs.subd);
        if (!subd) {
          return { points: [], counts: [], edges: [], vertices: [] };
        }
        const points = [];
        const counts = [];
        const edges = [];
        const vertices = [];
        subd.faces.forEach((face) => {
          points.push(face.centroid.clone());
          counts.push(face.vertices.length);
          edges.push(face.edges.slice());
          vertices.push(face.vertices.slice());
        });
        return { points, counts, edges, vertices };
      },
    });

    register('{855a2c73-31c0-41d2-b061-57d54229d11b}', {
      type: 'subd',
      pinMap: {
        inputs: {
          M: 'mesh', Mesh: 'mesh', mesh: 'mesh',
          Cr: 'creases', Creases: 'creases', creases: 'creases',
          Co: 'corners', Corners: 'corners', corners: 'corners',
          I: 'interpolate', Interpolate: 'interpolate', interpolate: 'interpolate',
        },
        outputs: { S: 'subd', SubD: 'subd', subd: 'subd' },
      },
      eval: ({ inputs }) => {
        const subd = createSubDFromMesh(inputs.mesh, {
          metadata: {
            interpolate: ensureBoolean(inputs.interpolate, false),
          },
        });
        if (!subd) {
          return {};
        }
        if (ensureBoolean(inputs.creases, false)) {
          subd.edges.forEach((edge) => {
            if (edge.faces.length <= 1) {
              edge.tag = 'crease';
            }
          });
        }
        if (ensureBoolean(inputs.corners, false)) {
          const boundaryVertices = new Set();
          subd.edges.forEach((edge) => {
            if (edge.faces.length <= 1) {
              edge.vertices.forEach((id) => boundaryVertices.add(id));
            }
          });
          subd.vertices.forEach((vertex) => {
            if (boundaryVertices.has(vertex.id)) {
              vertex.tag = 'corner';
            }
          });
        }
        return { subd };
      },
    });

    register('{954a8963-bb2c-4847-9012-69ff34acddd5}', {
      type: 'subd',
      pinMap: {
        inputs: {
          S: 'subd', SubD: 'subd', subd: 'subd',
          T: 'tag', Tag: 'tag', tag: 'tag', 'Vertex Tag': 'tag', vertexTag: 'tag',
          V: 'vertexIds', 'Vertex IDs': 'vertexIds', vertexIds: 'vertexIds',
        },
        outputs: { S: 'subd', SubD: 'subd', subd: 'subd' },
      },
      eval: ({ inputs }) => {
        const base = ensureSubD(inputs.subd);
        if (!base) {
          return {};
        }
        const ids = collectIndices(inputs.vertexIds);
        if (!ids.length) {
          return { subd: base };
        }
        const updated = applyVertexTags(base, ids, inputs.tag ?? 'smooth');
        return updated ? { subd: updated } : { subd: base };
      },
    });

    register('{c0b3c6e9-d05d-4c51-a0df-1ce2678c7a33}', {
      type: 'mesh',
      pinMap: {
        inputs: { S: 'subd', SubD: 'subd', subd: 'subd', D: 'density', Density: 'density', density: 'density' },
        outputs: { M: 'mesh', Mesh: 'mesh', mesh: 'mesh' },
      },
      eval: ({ inputs }) => {
        const subd = ensureSubD(inputs.subd);
        if (!subd) {
          return {};
        }
        const mesh = createMeshFromSubD(subd, {
          metadata: { density: Math.max(0, Math.round(ensureNumeric(inputs.density, 0))) },
        });
        return mesh ? { mesh } : {};
      },
    });

    register('{c1a57c2a-11c5-4f77-851e-0a7dffef848e}', {
      type: 'mesh',
      pinMap: {
        inputs: { S: 'subd', SubD: 'subd', subd: 'subd' },
        outputs: { M: 'mesh', Mesh: 'mesh', mesh: 'mesh' },
      },
      eval: ({ inputs }) => {
        const subd = ensureSubD(inputs.subd);
        if (!subd) {
          return {};
        }
        const mesh = createControlPolygonFromSubD(subd);
        return mesh ? { mesh } : {};
      },
    });

    register('{cd9efa8f-0084-4d52-ab13-ad88ff22dc46}', {
      type: 'subd',
      pinMap: {
        inputs: { S: 'subd', SubD: 'subd', subd: 'subd' },
        outputs: {
          P: 'points', Point: 'points', points: 'points',
          I: 'ids', Id: 'ids', ids: 'ids',
        },
      },
      eval: ({ inputs }) => {
        const subd = ensureSubD(inputs.subd);
        if (!subd) {
          return { points: [], ids: [] };
        }
        const points = subd.vertices.map((vertex) => vertex.point.clone());
        const ids = subd.vertices.map((vertex) => vertex.id);
        return { points, ids };
      },
    });

    register('{fc8ad805-2cbf-4447-b41b-50c0be591fcd}', {
      type: 'subd',
      pinMap: {
        inputs: { S: 'subd', SubD: 'subd', subd: 'subd' },
        outputs: {
          P: 'points', Point: 'points', points: 'points',
          I: 'ids', Id: 'ids', ids: 'ids',
          T: 'tags', Tag: 'tags', tags: 'tags',
        },
      },
      eval: ({ inputs }) => {
        const subd = ensureSubD(inputs.subd);
        if (!subd) {
          return { points: [], ids: [], tags: [] };
        }
        const points = subd.vertices.map((vertex) => vertex.point.clone());
        const ids = subd.vertices.map((vertex) => vertex.id);
        const tags = subd.vertices.map((vertex) => vertex.tag ?? 'smooth');
        return { points, ids, tags };
      },
    });
  }

  function registerFreeformComponents() {
    register('{45f19d16-1c9f-4b0f-a9a6-45a77f3d206c}', {
      type: 'surface',
      pinMap: {
        inputs: {
          Cls: 'closed', Closed: 'closed', closed: 'closed',
          Adj: 'adjust', Adjust: 'adjust', adjust: 'adjust',
          Rbd: 'rebuild', Rebuild: 'rebuild', rebuild: 'rebuild',
          Rft: 'refit', Refit: 'refit', refit: 'refit',
          T: 'type', Type: 'type', type: 'type',
        },
        outputs: { O: 'options', Options: 'options', options: 'options' },
      },
      eval: ({ inputs }) => {
        const closed = ensureBoolean(inputs.closed, false);
        const adjust = ensureBoolean(inputs.adjust, false);
        const rebuild = Math.max(0, Math.round(ensureNumeric(inputs.rebuild, 0)));
        const refit = Math.max(0, ensureNumeric(inputs.refit, 0));
        const loftType = Math.max(0, Math.min(5, Math.round(ensureNumeric(inputs.type, 0))));
        const options = { closed, adjust, rebuild, refit, type: loftType };
        return { options };
      },
    });

    register('{a7a41d0a-2188-4f7a-82cc-1a2c4e4ec850}', {
      type: 'surface',
      pinMap: {
        inputs: {
          C: 'curves', Curves: 'curves', curves: 'curves',
          O: 'options', Options: 'options', options: 'options',
        },
        outputs: { L: 'loft', Loft: 'loft', S: 'loft', Surface: 'loft', surface: 'loft' },
      },
      eval: ({ inputs }) => {
        const curves = ensureArray(inputs.curves);
        const sections = [];
        for (const curve of curves) {
          const section = sampleCurvePoints(curve, DEFAULT_CURVE_SEGMENTS);
          if (section.points.length >= 2) {
            sections.push(section);
          }
        }
        if (!sections.length) {
          return {};
        }
        const providedOptions = inputs.options?.options ?? inputs.options;
        const normalizedOptions = providedOptions && typeof providedOptions === 'object'
          ? {
            closed: ensureBoolean(providedOptions.closed, false),
            adjust: ensureBoolean(providedOptions.adjust, false),
            rebuild: Math.max(0, Math.round(ensureNumeric(providedOptions.rebuild, 0))),
            refit: Math.max(0, ensureNumeric(providedOptions.refit, 0)),
            type: Math.max(0, Math.min(5, Math.round(ensureNumeric(providedOptions.type, 0)))),
          }
          : { closed: false, adjust: false, rebuild: 0, refit: 0, type: 0 };
        const surface = createLoftSurfaceFromSections(sections, {
          metadata: { type: 'loft', options: normalizedOptions },
          closed: normalizedOptions.closed,
        });
        if (!surface) {
          return {};
        }
        return {
          loft: wrapSurface(surface, {
            sections: sections.map((section) => section.points.map((pt) => pt.clone())),
          }),
        };
      },
    });

    register('{342aa574-1327-4bc2-8daf-203da2a45676}', {
      type: 'surface',
      pinMap: {
        inputs: {
          C: 'curves', Curves: 'curves', curves: 'curves',
          Nu: 'countU', 'Count U': 'countU', countU: 'countU',
          Du: 'degreeU', 'Degree U': 'degreeU', degreeU: 'degreeU',
          Dv: 'degreeV', 'Degree V': 'degreeV', degreeV: 'degreeV',
        },
        outputs: { S: 'surface', Surface: 'surface', surface: 'surface' },
      },
      eval: ({ inputs }) => {
        const curves = ensureArray(inputs.curves);
        const sections = [];
        for (const curve of curves) {
          const section = sampleCurvePoints(curve, DEFAULT_CURVE_SEGMENTS);
          if (section.points.length >= 2) {
            sections.push(section);
          }
        }
        if (!sections.length) {
          return {};
        }
        const countU = Math.max(2, Math.round(ensureNumeric(inputs.countU, sections[0].points.length)));
        const resampled = sections.map((section) => ({
          points: resamplePolyline(section.points, countU, { closed: section.closed }),
          closed: section.closed,
        }));
        const degreeU = Math.max(1, Math.round(ensureNumeric(inputs.degreeU, 3)));
        const degreeV = Math.max(1, Math.round(ensureNumeric(inputs.degreeV, 3)));
        const surface = createLoftSurfaceFromSections(resampled, {
          metadata: { type: 'fit-loft', countU, degreeU, degreeV },
          closed: resampled.some((section) => section.closed),
        });
        if (!surface) {
          return {};
        }
        return {
          surface: wrapSurface(surface, {
            sections: resampled.map((section) => section.points.map((pt) => pt.clone())),
          }),
        };
      },
    });

    register('{5c270622-ee80-45a4-b07a-bd8ffede92a2}', {
      type: 'surface',
      pinMap: {
        inputs: {
          C: 'curves', Curves: 'curves', curves: 'curves',
          D: 'degree', Degree: 'degree', degree: 'degree',
        },
        outputs: { S: 'surface', Surface: 'surface', surface: 'surface' },
      },
      eval: ({ inputs }) => {
        const curves = ensureArray(inputs.curves);
        const sections = [];
        for (const curve of curves) {
          const section = sampleCurvePoints(curve, DEFAULT_CURVE_SEGMENTS);
          if (section.points.length >= 2) {
            sections.push(section);
          }
        }
        if (!sections.length) {
          return {};
        }
        const degree = Math.max(1, Math.round(ensureNumeric(inputs.degree, 3)));
        const surface = createLoftSurfaceFromSections(sections, {
          metadata: { type: 'control-point-loft', degree },
          closed: sections.some((section) => section.closed),
        });
        if (!surface) {
          return {};
        }
        return {
          surface: wrapSurface(surface, {
            sections: sections.map((section) => section.points.map((pt) => pt.clone())),
          }),
        };
      },
    });

    register('{36132830-e2ef-4476-8ea1-6a43922344f0}', {
      type: 'surface',
      pinMap: {
        inputs: {
          A: 'curveA', 'Curve A': 'curveA', CurveA: 'curveA', curveA: 'curveA',
          B: 'curveB', 'Curve B': 'curveB', CurveB: 'curveB', curveB: 'curveB',
          C: 'curveC', 'Curve C': 'curveC', CurveC: 'curveC', curveC: 'curveC',
          D: 'curveD', 'Curve D': 'curveD', CurveD: 'curveD', curveD: 'curveD',
        },
        outputs: { S: 'surface', Surface: 'surface', surface: 'surface' },
      },
      eval: ({ inputs }) => {
        const candidates = [inputs.curveA, inputs.curveB, inputs.curveC, inputs.curveD];
        const sections = [];
        for (const candidate of candidates) {
          if (!candidate) continue;
          const section = sampleCurvePoints(candidate, DEFAULT_CURVE_SEGMENTS);
          if (section.points.length >= 2) {
            sections.push(section);
          }
        }
        if (sections.length < 2) {
          return {};
        }
        const surface = createLoftSurfaceFromSections(sections, {
          metadata: { type: 'edge-surface' },
          closed: sections.some((section) => section.closed),
        });
        if (!surface) {
          return {};
        }
        return {
          surface: wrapSurface(surface, {
            sections: sections.map((section) => section.points.map((pt) => pt.clone())),
          }),
        };
      },
    });

    register('{4b04a1e1-cddf-405d-a7db-335aaa940541}', {
      type: 'surface',
      pinMap: {
        inputs: {
          P: 'points', Points: 'points', points: 'points',
          U: 'countU', 'U Count': 'countU', countU: 'countU',
          I: 'interpolate', Interpolate: 'interpolate', interpolate: 'interpolate',
        },
        outputs: { S: 'surface', Surface: 'surface', surface: 'surface' },
      },
      eval: ({ inputs }) => {
        const pointList = collectPoints(inputs.points);
        if (!pointList.length) {
          return {};
        }
        const countU = Math.max(2, Math.round(ensureNumeric(inputs.countU, Math.round(Math.sqrt(pointList.length)) || 2)));
        const interpolate = ensureBoolean(inputs.interpolate, false);
        const surface = createSurfaceFromPointGrid(pointList, countU, {
          metadata: { type: 'surface-from-points', interpolate },
        });
        if (!surface) {
          return {};
        }
        return {
          surface: wrapSurface(surface, { points: pointList.map((pt) => pt.clone()) }),
        };
      },
    });

    register('{57b2184c-8931-4e70-9220-612ec5b3809a}', {
      type: 'surface',
      pinMap: {
        inputs: {
          C: 'curves', Curves: 'curves', curves: 'curves',
          P: 'points', Points: 'points', points: 'points',
          S: 'spans', Spans: 'spans', spans: 'spans',
          F: 'flexibility', Flexibility: 'flexibility', flexibility: 'flexibility',
          T: 'trim', Trim: 'trim', trim: 'trim',
        },
        outputs: { P: 'patch', Patch: 'patch', S: 'patch', Surface: 'patch' },
      },
      eval: ({ inputs }) => {
        const curvePoints = [];
        const curves = ensureArray(inputs.curves);
        for (const curve of curves) {
          const section = sampleCurvePoints(curve, DEFAULT_CURVE_SEGMENTS);
          if (section.points.length) {
            curvePoints.push(...section.points);
          }
        }
        const extraPoints = collectPoints(inputs.points);
        const allPoints = [...curvePoints, ...extraPoints];
        if (!allPoints.length) {
          return {};
        }
        const plane = allPoints.length >= 3
          ? planeFromPoints(allPoints[0], allPoints[1], allPoints[2])
          : defaultPlane();
        const coords = allPoints.map((pt) => planeCoordinates(pt, plane));
        const xs = coords.map((coord) => coord.x);
        const ys = coords.map((coord) => coord.y);
        const minX = Math.min(...xs);
        const maxX = Math.max(...xs);
        const minY = Math.min(...ys);
        const maxY = Math.max(...ys);
        const surface = createPlanarSurfaceFromBounds(plane, minX, maxX, minY, maxY);
        if (!surface) {
          return {};
        }
        const spans = Math.max(1, Math.round(ensureNumeric(inputs.spans, 1)));
        const flexibility = Math.max(0, ensureNumeric(inputs.flexibility, 1));
        const trim = ensureBoolean(inputs.trim, false);
        return {
          patch: wrapSurface(surface, {
            spans,
            flexibility,
            trim,
            supportPoints: allPoints.map((pt) => pt.clone()),
          }),
        };
      },
    });

    register('{5e33c760-adcd-4235-b1dd-05cf72eb7a38}', {
      type: 'surface',
      pinMap: {
        inputs: {
          A: 'curveA', CurveA: 'curveA', 'Curve A': 'curveA', curveA: 'curveA',
          B: 'curveB', CurveB: 'curveB', 'Curve B': 'curveB', curveB: 'curveB',
        },
        outputs: { S: 'surface', Surface: 'surface', surface: 'surface' },
      },
      eval: ({ inputs }) => {
        const curveA = sampleCurvePoints(inputs.curveA, DEFAULT_CURVE_SEGMENTS);
        const curveB = sampleCurvePoints(inputs.curveB, DEFAULT_CURVE_SEGMENTS);
        if (!curveA.points.length || !curveB.points.length) {
          return {};
        }
        const origin = curveA.points[0].clone().add(curveB.points[0]);
        const rows = curveB.points.map((pb) => curveA.points.map((pa) => pa.clone().add(pb).sub(origin)));
        const surface = createGridSurface(rows, {
          metadata: { type: 'sum-surface' },
          closedU: curveA.closed,
          closedV: curveB.closed,
        });
        if (!surface) {
          return {};
        }
        return {
          surface: wrapSurface(surface, {
            curveA: curveA.points.map((pt) => pt.clone()),
            curveB: curveB.points.map((pt) => pt.clone()),
          }),
        };
      },
    });

    register('{6e5de495-ba76-42d0-9985-a5c265e9aeca}', {
      type: 'surface',
      pinMap: {
        inputs: {
          A: 'curveA', CurveA: 'curveA', 'Curve A': 'curveA', curveA: 'curveA',
          B: 'curveB', CurveB: 'curveB', 'Curve B': 'curveB', curveB: 'curveB',
        },
        outputs: { S: 'surface', Surface: 'surface', surface: 'surface' },
      },
      eval: ({ inputs }) => {
        const curveA = sampleCurvePoints(inputs.curveA, DEFAULT_CURVE_SEGMENTS);
        const curveB = sampleCurvePoints(inputs.curveB, DEFAULT_CURVE_SEGMENTS);
        if (!curveA.points.length || !curveB.points.length) {
          return {};
        }
        const surface = createLoftSurfaceFromSections([curveA, curveB], {
          metadata: { type: 'ruled-surface' },
          closed: curveA.closed && curveB.closed,
        });
        if (!surface) {
          return {};
        }
        return {
          surface: wrapSurface(surface, {
            startCurve: curveA.points.map((pt) => pt.clone()),
            endCurve: curveB.points.map((pt) => pt.clone()),
          }),
        };
      },
    });

    register('{71506fa8-9bf0-432d-b897-b2e0c5ac316c}', {
      type: 'surface',
      pinMap: {
        inputs: {
          U: 'curvesU', CurvesU: 'curvesU', 'Curves U': 'curvesU', curvesU: 'curvesU',
          V: 'curvesV', CurvesV: 'curvesV', 'Curves V': 'curvesV', curvesV: 'curvesV',
          C: 'continuity', Continuity: 'continuity', continuity: 'continuity',
        },
        outputs: { S: 'surface', Surface: 'surface', surface: 'surface' },
      },
      eval: ({ inputs }) => {
        const curvesU = ensureArray(inputs.curvesU).map((curve) => sampleCurvePoints(curve, DEFAULT_CURVE_SEGMENTS)).filter((section) => section.points.length >= 2);
        const curvesV = ensureArray(inputs.curvesV).map((curve) => sampleCurvePoints(curve, DEFAULT_CURVE_SEGMENTS)).filter((section) => section.points.length >= 2);
        if (!curvesU.length && !curvesV.length) {
          return {};
        }
        const continuity = Math.max(0, Math.round(ensureNumeric(inputs.continuity, 0)));
        let surface = null;
        if (curvesU.length >= 2) {
          surface = createLoftSurfaceFromSections(curvesU, {
            metadata: { type: 'network-surface', direction: 'u', continuity, curvesV: curvesV.length },
            closed: curvesU.some((section) => section.closed),
          });
        } else if (curvesV.length >= 2) {
          surface = createLoftSurfaceFromSections(curvesV, {
            metadata: { type: 'network-surface', direction: 'v', continuity, curvesU: curvesU.length },
            closed: curvesV.some((section) => section.closed),
          });
        }
        if (!surface) {
          return {};
        }
        return {
          surface: wrapSurface(surface, {
            curvesU: curvesU.map((section) => section.points.map((pt) => pt.clone())),
            curvesV: curvesV.map((section) => section.points.map((pt) => pt.clone())),
            continuity,
          }),
        };
      },
    });

    register('{75164624-395a-4d24-b60b-6bf91cab0194}', {
      type: 'surface',
      pinMap: {
        inputs: {
          'R¹': 'rail1', R1: 'rail1', rail1: 'rail1', 'Rail 1': 'rail1',
          'R²': 'rail2', R2: 'rail2', rail2: 'rail2', 'Rail 2': 'rail2',
          S: 'sections', Sections: 'sections', sections: 'sections',
          H: 'sameHeight', 'Same Height': 'sameHeight', sameHeight: 'sameHeight',
        },
        outputs: { S: 'breps', Brep: 'breps', Breps: 'breps' },
      },
      eval: ({ inputs }) => {
        const sectionCurves = ensureArray(inputs.sections)
          .map((curve) => sampleCurvePoints(curve, DEFAULT_CURVE_SEGMENTS))
          .filter((section) => section.points.length >= 2);
        if (!sectionCurves.length) {
          return {};
        }
        const surface = createLoftSurfaceFromSections(sectionCurves, {
          metadata: {
            type: 'sweep2',
            rails: [inputs.rail1 ?? null, inputs.rail2 ?? null],
            sameHeight: ensureBoolean(inputs.sameHeight, false),
          },
          closed: sectionCurves.some((section) => section.closed),
        });
        if (!surface) {
          return {};
        }
        return {
          breps: [wrapSurface(surface, {
            sections: sectionCurves.map((section) => section.points.map((pt) => pt.clone())),
          })],
        };
      },
    });

    register('{bb6666e7-d0f4-41ec-a257-df2371619f13}', {
      type: 'surface',
      pinMap: {
        inputs: {
          R: 'rail', Rail: 'rail', rail: 'rail',
          S: 'sections', Sections: 'sections', sections: 'sections',
          M: 'miter', Miter: 'miter', miter: 'miter',
        },
        outputs: { S: 'breps', Brep: 'breps', Breps: 'breps' },
      },
      eval: ({ inputs }) => {
        const sectionCurves = ensureArray(inputs.sections)
          .map((curve) => sampleCurvePoints(curve, DEFAULT_CURVE_SEGMENTS))
          .filter((section) => section.points.length >= 2);
        if (!sectionCurves.length) {
          return {};
        }
        const surface = createLoftSurfaceFromSections(sectionCurves, {
          metadata: {
            type: 'sweep1',
            rail: inputs.rail ?? null,
            miter: Math.max(0, Math.round(ensureNumeric(inputs.miter, 0))),
          },
          closed: sectionCurves.some((section) => section.closed),
        });
        if (!surface) {
          return {};
        }
        return {
          breps: [wrapSurface(surface, {
            sections: sectionCurves.map((section) => section.points.map((pt) => pt.clone())),
          })],
        };
      },
    });

    register('{38a5638b-6d01-4417-bf11-976d925f8a71}', {
      type: 'surface',
      pinMap: {
        inputs: {
          B: 'base', Base: 'base', base: 'base',
          C: 'curve', Curve: 'curve', curve: 'curve',
        },
        outputs: { E: 'extrusion', Extrusion: 'extrusion', extrusion: 'extrusion' },
      },
      eval: ({ inputs }) => {
        const profile = extractProfileData(inputs.base);
        if (!profile.coords.length) {
          return {};
        }
        const pathSample = sampleCurvePoints(inputs.curve, DEFAULT_CURVE_SEGMENTS);
        if (!pathSample.points.length) {
          return {};
        }
        const pathPoints = resamplePolyline(pathSample.points, Math.max(pathSample.points.length, 16), { closed: pathSample.closed });
        const frames = createFramesAlongPath(pathPoints, profile.plane, { closed: pathSample.closed });
        const surface = createExtrusionSurface(profile, frames, {
          metadata: { type: 'extrude-along' },
          closedPath: pathSample.closed,
        });
        if (!surface) {
          return {};
        }
        return {
          extrusion: wrapSurface(surface, {
            path: pathPoints.map((pt) => pt.clone()),
            closed: pathSample.closed,
          }),
        };
      },
    });

    register('{8efd5eb9-a896-486e-9f98-d8d1a07a49f3}', {
      type: 'surface',
      pinMap: {
        inputs: {
          P: 'profile', Profile: 'profile', profile: 'profile',
          Po: 'orientationProfile', 'Orientation (P)': 'orientationProfile',
          A: 'axis', Axis: 'axis', axis: 'axis',
          Ao: 'orientationAxis', 'Orientation (A)': 'orientationAxis',
        },
        outputs: { E: 'extrusion', Extrusion: 'extrusion', extrusion: 'extrusion' },
      },
      eval: ({ inputs }) => {
        let profile = extractProfileData(inputs.profile);
        if (inputs.orientationProfile) {
          const orientPlane = ensurePlane(inputs.orientationProfile);
          const coords = profile.points.map((pt) => planeCoordinates(pt, orientPlane));
          profile = {
            plane: orientPlane,
            coords: coords.map((coord) => new THREE.Vector2(coord.x, coord.y)),
            points: profile.points.map((pt) => pt.clone()),
            centroid: profile.centroid.clone(),
            closed: profile.closed,
          };
        }
        if (!profile.coords.length) {
          return {};
        }
        const axisData = parseAxisInput(inputs.axis, profile.plane.origin.clone(), profile.plane.zAxis.clone());
        const pathPoints = createLinearPath(axisData.origin.clone(), axisData.vector.clone(), 16);
        const frames = createFramesAlongPath(pathPoints, profile.plane);
        const surface = createExtrusionSurface(profile, frames, {
          metadata: {
            type: 'extrude-linear',
            axis: axisData,
            orientationAxis: inputs.orientationAxis ?? null,
          },
        });
        if (!surface) {
          return {};
        }
        return {
          extrusion: wrapSurface(surface, {
            path: pathPoints.map((pt) => pt.clone()),
            axis: axisData,
          }),
        };
      },
    });

    register('{962034e9-cc27-4394-afc4-5c16e3447cf9}', {
      type: 'surface',
      pinMap: {
        inputs: {
          B: 'base', Base: 'base', base: 'base',
          D: 'direction', Direction: 'direction', direction: 'direction',
        },
        outputs: { E: 'extrusion', Extrusion: 'extrusion', extrusion: 'extrusion' },
      },
      eval: ({ inputs }) => {
        const profile = extractProfileData(inputs.base);
        if (!profile.coords.length) {
          return {};
        }
        const directionVector = ensurePoint(inputs.direction, profile.plane.zAxis.clone());
        if (directionVector.lengthSq() <= EPSILON) {
          return {};
        }
        const pathPoints = createLinearPath(profile.plane.origin.clone(), directionVector.clone(), 16);
        const frames = createFramesAlongPath(pathPoints, profile.plane);
        const surface = createExtrusionSurface(profile, frames, {
          metadata: { type: 'extrude-vector', direction: directionVector.clone() },
        });
        if (!surface) {
          return {};
        }
        return {
          extrusion: wrapSurface(surface, {
            path: pathPoints.map((pt) => pt.clone()),
            direction: directionVector.clone(),
          }),
        };
      },
    });

    register('{be6636b2-2f1a-4d42-897b-fdef429b6f17}', {
      type: 'surface',
      pinMap: {
        inputs: {
          B: 'base', Base: 'base', base: 'base',
          P: 'point', Point: 'point', point: 'point',
        },
        outputs: { E: 'extrusion', Extrusion: 'extrusion', extrusion: 'extrusion' },
      },
      eval: ({ inputs }) => {
        const profile = extractProfileData(inputs.base);
        if (!profile.coords.length) {
          return {};
        }
        const tip = ensurePoint(inputs.point, profile.plane.origin.clone().add(profile.plane.zAxis.clone()));
        const vector = tip.clone().sub(profile.plane.origin);
        if (vector.lengthSq() <= EPSILON) {
          return {};
        }
        const pathPoints = createLinearPath(profile.plane.origin.clone(), vector.clone(), 16);
        const frames = createFramesAlongPath(pathPoints, profile.plane);
        const surface = createExtrusionSurface(profile, frames, {
          metadata: { type: 'extrude-point', tip },
        });
        if (!surface) {
          return {};
        }
        return {
          extrusion: wrapSurface(surface, {
            path: pathPoints.map((pt) => pt.clone()),
            tip,
          }),
        };
      },
    });

    register('{ae57e09b-a1e4-4d05-8491-abd232213bc9}', {
      type: 'surface',
      pinMap: {
        inputs: {
          P: 'polyline', Polyline: 'polyline', polyline: 'polyline',
          Hb: 'baseHeight', 'Base height': 'baseHeight', baseHeight: 'baseHeight',
          Ht: 'topHeight', 'Top height': 'topHeight', topHeight: 'topHeight',
          A: 'angles', Angles: 'angles', angles: 'angles',
        },
        outputs: { S: 'shape', Shape: 'shape', shape: 'shape' },
      },
      eval: ({ inputs }) => {
        const profile = extractProfileData(inputs.polyline);
        if (!profile.coords.length) {
          return {};
        }
        const baseHeight = ensureNumeric(inputs.baseHeight, 0);
        const topHeight = ensureNumeric(inputs.topHeight, 0);
        const totalHeight = baseHeight + topHeight;
        const vector = profile.plane.zAxis.clone().multiplyScalar(totalHeight || 1);
        const pathPoints = createLinearPath(profile.plane.origin.clone(), vector, 4);
        const frames = createFramesAlongPath(pathPoints, profile.plane);
        const surface = createExtrusionSurface(profile, frames, {
          metadata: {
            type: 'extrude-angled',
            baseHeight,
            topHeight,
            angles: ensureArray(inputs.angles).map((value) => ensureNumeric(value, 0)),
          },
        });
        if (!surface) {
          return {};
        }
        return {
          shape: wrapSurface(surface, {
            path: pathPoints.map((pt) => pt.clone()),
            baseHeight,
            topHeight,
          }),
        };
      },
    });

    register('{888f9c3c-f1e1-4344-94b0-5ee6a45aee11}', {
      type: 'surface',
      pinMap: {
        inputs: {
          C: 'curve', Curve: 'curve', curve: 'curve',
          t: 'parameters', T: 'parameters', Parameters: 'parameters', parameters: 'parameters',
          R: 'radii', Radii: 'radii', radii: 'radii',
          E: 'caps', Caps: 'caps', caps: 'caps',
        },
        outputs: { P: 'pipe', Pipe: 'pipe', pipe: 'pipe' },
      },
      eval: ({ inputs }) => {
        const pathSample = sampleCurvePoints(inputs.curve, DEFAULT_CURVE_SEGMENTS);
        if (!pathSample.points.length) {
          return {};
        }
        const basePlane = pathSample.points.length >= 3
          ? planeFromPoints(pathSample.points[0], pathSample.points[1], pathSample.points[2])
          : defaultPlane();
        const rawParameters = ensureArray(inputs.parameters)
          .map((value) => ensureNumeric(value, Number.NaN))
          .filter((value) => Number.isFinite(value))
          .map((value) => clamp01(value));
        const rawRadii = ensureArray(inputs.radii)
          .map((value) => Math.abs(ensureNumeric(value, Number.NaN)))
          .filter((value) => Number.isFinite(value));
        const segmentCount = Math.max(pathSample.points.length, rawParameters.length, rawRadii.length, 24);
        const pathPoints = resamplePolyline(pathSample.points, segmentCount, { closed: pathSample.closed });
        const frames = createFramesAlongPath(pathPoints, basePlane, { closed: pathSample.closed });
        const samples = frames.map((_, index) => {
          if (!rawParameters.length || rawParameters.length !== rawRadii.length) {
            const sourceIndex = Math.min(index, rawRadii.length - 1);
            return Math.max(ensureNumeric(rawRadii[sourceIndex] ?? rawRadii[0] ?? 1, 1), EPSILON);
          }
          const t = frames.length <= 1 ? 0 : index / (frames.length - 1);
          let lowerIndex = 0;
          let upperIndex = rawParameters.length - 1;
          for (let i = 0; i < rawParameters.length; i += 1) {
            if (rawParameters[i] <= t) {
              lowerIndex = i;
            }
            if (rawParameters[i] >= t) {
              upperIndex = i;
              break;
            }
          }
          const lowerParam = rawParameters[lowerIndex] ?? 0;
          const upperParam = rawParameters[upperIndex] ?? 1;
          const lowerRadius = rawRadii[lowerIndex] ?? rawRadii[0] ?? 1;
          const upperRadius = rawRadii[upperIndex] ?? rawRadii[rawRadii.length - 1] ?? lowerRadius;
          if (Math.abs(upperParam - lowerParam) <= EPSILON) {
            return Math.max(lowerRadius, EPSILON);
          }
          const factor = (t - lowerParam) / (upperParam - lowerParam);
          return Math.max(lowerRadius + (upperRadius - lowerRadius) * factor, EPSILON);
        });
        const surface = createPipeSurface(frames, samples, {
          metadata: {
            type: 'pipe-variable',
            caps: ensureNumeric(inputs.caps, 0),
          },
          closed: pathSample.closed,
        });
        if (!surface) {
          return {};
        }
        return {
          pipe: wrapSurface(surface, {
            path: pathPoints.map((pt) => pt.clone()),
            radii: samples,
            caps: ensureNumeric(inputs.caps, 0),
          }),
        };
      },
    });

    register('{c277f778-6fdf-4890-8f78-347efb23c406}', {
      type: 'surface',
      pinMap: {
        inputs: {
          C: 'curve', Curve: 'curve', curve: 'curve',
          R: 'radius', Radius: 'radius', radius: 'radius',
          E: 'caps', Caps: 'caps', caps: 'caps',
        },
        outputs: { P: 'pipe', Pipe: 'pipe', pipe: 'pipe' },
      },
      eval: ({ inputs }) => {
        const pathSample = sampleCurvePoints(inputs.curve, DEFAULT_CURVE_SEGMENTS);
        if (!pathSample.points.length) {
          return {};
        }
        const basePlane = pathSample.points.length >= 3
          ? planeFromPoints(pathSample.points[0], pathSample.points[1], pathSample.points[2])
          : defaultPlane();
        const radius = Math.max(Math.abs(ensureNumeric(inputs.radius, 1)), EPSILON);
        const pathPoints = resamplePolyline(pathSample.points, Math.max(pathSample.points.length, 24), { closed: pathSample.closed });
        const frames = createFramesAlongPath(pathPoints, basePlane, { closed: pathSample.closed });
        const surface = createPipeSurface(frames, frames.map(() => radius), {
          metadata: { type: 'pipe', caps: ensureNumeric(inputs.caps, 0) },
          closed: pathSample.closed,
        });
        if (!surface) {
          return {};
        }
        return {
          pipe: wrapSurface(surface, {
            path: pathPoints.map((pt) => pt.clone()),
            radius,
            caps: ensureNumeric(inputs.caps, 0),
          }),
        };
      },
    });

    register('{c77a8b3b-c569-4d81-9b59-1c27299a1c45}', {
      type: 'surface',
      pinMap: {
        inputs: {
          A: 'cornerA', CornerA: 'cornerA', 'Corner A': 'cornerA', cornerA: 'cornerA',
          B: 'cornerB', CornerB: 'cornerB', 'Corner B': 'cornerB', cornerB: 'cornerB',
          C: 'cornerC', CornerC: 'cornerC', 'Corner C': 'cornerC', cornerC: 'cornerC',
          D: 'cornerD', CornerD: 'cornerD', 'Corner D': 'cornerD', cornerD: 'cornerD',
        },
        outputs: { S: 'surface', Surface: 'surface', surface: 'surface' },
      },
      eval: ({ inputs }) => {
        const cornerA = ensurePoint(inputs.cornerA, new THREE.Vector3());
        const cornerB = ensurePoint(inputs.cornerB, cornerA.clone().add(new THREE.Vector3(1, 0, 0)));
        const cornerC = ensurePoint(inputs.cornerC, cornerA.clone().add(new THREE.Vector3(0, 1, 0)));
        const cornerD = inputs.cornerD ? ensurePoint(inputs.cornerD, cornerA.clone()) : cornerA.clone().add(cornerC.clone().sub(cornerB));
        const rows = [
          [cornerA.clone(), cornerB.clone()],
          [cornerD.clone(), cornerC.clone()],
        ];
        const surface = createGridSurface(rows, {
          metadata: { type: 'four-point-surface' },
        });
        if (!surface) {
          return {};
        }
        return {
          surface: wrapSurface(surface, {
            corners: [cornerA, cornerB, cornerC, cornerD],
          }),
        };
      },
    });

    register('{cb56b26c-2595-4d03-bdb2-eb2e6aeba82d}', {
      type: 'surface',
      pinMap: {
        inputs: { B: 'boundary', Boundary: 'boundary', boundary: 'boundary' },
        outputs: { P: 'patch', Patch: 'patch', patch: 'patch' },
      },
      eval: ({ inputs }) => {
        const surface = createBoundarySurfaceFromCurve(inputs.boundary);
        if (!surface) {
          return {};
        }
        return { patch: wrapSurface(surface, { boundary: inputs.boundary ?? null }) };
      },
    });

    register('{cdee962f-4202-456b-a1b4-f3ed9aa0dc29}', {
      type: 'surface',
      pinMap: {
        inputs: {
          P: 'profile', Curve: 'profile', profile: 'profile',
          A: 'axis', Axis: 'axis', axis: 'axis',
          D: 'domain', Domain: 'domain', domain: 'domain',
        },
        outputs: { S: 'surface', Surface: 'surface', surface: 'surface' },
      },
      eval: ({ inputs }) => {
        const profileSample = sampleCurvePoints(inputs.profile, DEFAULT_CURVE_SEGMENTS);
        if (!profileSample.points.length) {
          return {};
        }
        const axisData = parseAxisInput(inputs.axis, profileSample.points[0] ?? new THREE.Vector3(), new THREE.Vector3(0, 0, 1));
        const domain = inputs.domain && typeof inputs.domain === 'object'
          ? {
            start: ensureNumeric(inputs.domain.start ?? inputs.domain.min ?? inputs.domain[0], 0),
            end: ensureNumeric(inputs.domain.end ?? inputs.domain.max ?? inputs.domain[1], Math.PI * 2),
          }
          : null;
        const surface = createRevolutionSurface(profileSample.points, axisData, {
          domain,
          metadata: { type: 'revolution' },
        });
        if (!surface) {
          return {};
        }
        return {
          surface: wrapSurface(surface, {
            profile: profileSample.points.map((pt) => pt.clone()),
            axis: axisData,
            domain,
          }),
        };
      },
    });

    register('{d8d68c35-f869-486d-adf3-69ee3cc2d501}', {
      type: 'surface',
      pinMap: {
        inputs: {
          P: 'profile', Curve: 'profile', profile: 'profile',
          R: 'rail', Rail: 'rail', rail: 'rail',
          A: 'axis', Axis: 'axis', axis: 'axis',
          S: 'scale', Scale: 'scale', scale: 'scale',
        },
        outputs: { S: 'surface', Surface: 'surface', surface: 'surface' },
      },
      eval: ({ inputs }) => {
        const profileSample = sampleCurvePoints(inputs.profile, DEFAULT_CURVE_SEGMENTS);
        if (!profileSample.points.length) {
          return {};
        }
        const axisData = parseAxisInput(inputs.axis, profileSample.points[0] ?? new THREE.Vector3(), new THREE.Vector3(0, 0, 1));
        const surface = createRevolutionSurface(profileSample.points, axisData, {
          metadata: {
            type: 'rail-revolution',
            rail: inputs.rail ?? null,
            scale: ensureNumeric(inputs.scale, 1),
          },
        });
        if (!surface) {
          return {};
        }
        return {
          surface: wrapSurface(surface, {
            profile: profileSample.points.map((pt) => pt.clone()),
            axis: axisData,
            rail: inputs.rail ?? null,
            scale: ensureNumeric(inputs.scale, 1),
          }),
        };
      },
    });

    register('{d51e9b65-aa4e-4fd6-976c-cef35d421d05}', {
      type: 'surface',
      pinMap: {
        inputs: { E: 'edges', Edges: 'edges', edges: 'edges' },
        outputs: { S: 'surfaces', Surfaces: 'surfaces', surfaces: 'surfaces' },
      },
      eval: ({ inputs }) => {
        const edges = ensureArray(inputs.edges);
        const surfaces = [];
        for (const edge of edges) {
          const surface = createBoundarySurfaceFromCurve(edge);
          if (surface) {
            surfaces.push(wrapSurface(surface, { boundary: edge ?? null }));
          }
        }
        if (!surfaces.length) {
          return {};
        }
        return { surfaces };
      },
    });
  }
  if (!freeformOnly) {
  register('{0373008a-80ee-45be-887d-ab5a244afc29}', {
    type: 'surface',
    pinMap: {
      inputs: {
        B: 'base', base: 'base', Base: 'base', P: 'base',
        R: 'radius', Radius: 'radius', radius: 'radius',
        L: 'height', Length: 'height', H: 'height', height: 'height',
      },
      outputs: { C: 'cylinder', Cylinder: 'cylinder', surface: 'cylinder' },
    },
    eval: ({ inputs }) => {
      const plane = ensurePlane(inputs.base);
      const radius = ensureNumeric(inputs.radius, 1);
      const height = ensureNumeric(inputs.height, 1);
      const surface = createCylinderSurface(plane, radius, height);
      return {
        cylinder: wrapSurface(surface, {
          plane: surface.plane,
          radius: Math.max(Math.abs(radius), EPSILON),
          height,
        }),
      };
    },
  });

  function createConeResult(inputs) {
    const plane = ensurePlane(inputs.base);
    const radius = ensureNumeric(inputs.radius, 1);
    const height = ensureNumeric(inputs.height, 1);
    const surface = createConeSurface(plane, radius, height);
    const tip = applyPlane(surface.plane, 0, 0, height);
    return {
      cone: wrapSurface(surface, {
        plane: surface.plane,
        radius: Math.max(Math.abs(radius), EPSILON),
        height,
      }),
      tip,
    };
  }

  register('{03e331ed-c4d1-4a23-afa2-f57b87d2043c}', {
    type: 'surface',
    pinMap: {
      inputs: {
        B: 'base', base: 'base', Base: 'base',
        R: 'radius', Radius: 'radius', radius: 'radius',
        L: 'height', Length: 'height', H: 'height', height: 'height',
      },
      outputs: { C: 'cone', Cone: 'cone', surface: 'cone', T: 'tip', Tip: 'tip' },
    },
    eval: ({ inputs }) => createConeResult(inputs),
  });

  register('{22e61c07-c02f-4c53-b567-c821a164fd92}', {
    type: 'surface',
    pinMap: {
      inputs: {
        B: 'base', base: 'base', Base: 'base',
        R: 'radius', Radius: 'radius', radius: 'radius',
        L: 'height', Length: 'height', H: 'height', height: 'height',
      },
      outputs: { C: 'cone', Cone: 'cone', surface: 'cone', T: 'tip', Tip: 'tip' },
    },
    eval: ({ inputs }) => createConeResult(inputs),
  });

  register('{28061aae-04fb-4cb5-ac45-16f3b66bc0a4}', {
    type: 'geometry',
    pinMap: {
      inputs: {
        B: 'base', base: 'base', Base: 'base',
        X: 'sizeX', x: 'sizeX', Width: 'sizeX', width: 'sizeX',
        Y: 'sizeY', y: 'sizeY', Height: 'sizeY', height: 'sizeY',
        Z: 'sizeZ', z: 'sizeZ', Depth: 'sizeZ', depth: 'sizeZ',
      },
      outputs: { B: 'box', Box: 'box', geometry: 'box' },
    },
    eval: ({ inputs }) => {
      const plane = ensurePlane(inputs.base);
      const sizeX = ensureNumeric(inputs.sizeX, 1);
      const sizeY = ensureNumeric(inputs.sizeY, 1);
      const sizeZ = ensureNumeric(inputs.sizeZ, 1);
      const box = createBoxFromPlaneDimensions(plane, sizeX, sizeY, sizeZ);
      return { box };
    },
  });

  register('{2a43ef96-8f87-4892-8b94-237a47e8d3cf}', {
    type: 'geometry',
    pinMap: {
      inputs: {
        A: 'pointA', a: 'pointA', 'Point A': 'pointA',
        B: 'pointB', b: 'pointB', 'Point B': 'pointB',
        P: 'plane', plane: 'plane', Plane: 'plane',
      },
      outputs: { B: 'box', Box: 'box', geometry: 'box' },
    },
    eval: ({ inputs }) => {
      const plane = ensurePlane(inputs.plane ?? defaultPlane());
      const pointA = ensurePoint(inputs.pointA, plane.origin.clone());
      const pointB = ensurePoint(inputs.pointB, plane.origin.clone().add(plane.xAxis));
      const coordA = planeCoordinates(pointA, plane);
      const coordB = planeCoordinates(pointB, plane);
      const min = new THREE.Vector3(
        Math.min(coordA.x, coordB.x),
        Math.min(coordA.y, coordB.y),
        Math.min(coordA.z, coordB.z),
      );
      const max = new THREE.Vector3(
        Math.max(coordA.x, coordB.x),
        Math.max(coordA.y, coordB.y),
        Math.max(coordA.z, coordB.z),
      );
      const box = createBoxDataFromPlaneExtents({ plane, min, max });
      return { box };
    },
  });

  register('{361790d6-9d66-4808-8c5a-8de9c218c227}', {
    type: 'surface',
    pinMap: {
      inputs: { B: 'base', base: 'base', Base: 'base', R: 'radius', Radius: 'radius', radius: 'radius' },
      outputs: { S: 'sphere', Sphere: 'sphere', surface: 'sphere' },
    },
    eval: ({ inputs }) => {
      const plane = ensurePlane(inputs.base);
      const radius = ensureNumeric(inputs.radius, 1);
      const center = plane.origin.clone();
      const surface = createSphereSurface(center, plane, radius);
      return {
        sphere: wrapSurface(surface, {
          center,
          radius: Math.max(Math.abs(radius), EPSILON),
        }),
      };
    },
  });
  register('{439a55a5-2f9e-4f66-9de2-32f24fec2ef5}', {
    type: 'surface',
    pinMap: {
      inputs: {
        P: 'plane', plane: 'plane', Plane: 'plane',
        X: 'sizeX', x: 'sizeX', Width: 'sizeX', width: 'sizeX',
        Y: 'sizeY', y: 'sizeY', Height: 'sizeY', height: 'sizeY',
      },
      outputs: { P: 'surface', S: 'surface', Surface: 'surface', surface: 'surface' },
    },
    eval: ({ inputs }) => {
      const plane = ensurePlane(inputs.plane);
      const sizeX = ensureNumeric(inputs.sizeX, 1);
      const sizeY = ensureNumeric(inputs.sizeY, 1);
      const surface = createPlanarSurfaceFromSize(plane, sizeX, sizeY);
      return { surface: wrapSurface(surface) };
    },
  });

  register('{0bb3d234-9097-45db-9998-621639c87d3b}', {
    type: 'geometry',
    pinMap: {
      inputs: { C: 'content', content: 'content', Content: 'content', P: 'plane', plane: 'plane', Plane: 'plane' },
      outputs: {
        B: 'worldBoxes', Box: 'worldBoxes',
        'world box': 'worldBoxes',
        'plane box': 'planeBoxes',
        Plane: 'planeBoxes',
      },
    },
    eval: ({ inputs }) => {
      const plane = inputs.plane ? ensurePlane(inputs.plane) : null;
      const { worldBoxes, planeBoxes } = computeBoxesForContent(inputs.content, { plane });
      return {
        worldBoxes,
        planeBoxes,
      };
    },
  });

  register('{6aa8da2e-6f25-4585-8b37-aa44609beb46}', {
    type: 'geometry',
    pinMap: {
      inputs: { C: 'content', content: 'content', Content: 'content', U: 'union', Union: 'union' },
      outputs: { B: 'worldBoxes', Box: 'worldBoxes' },
    },
    eval: ({ inputs }) => {
      const union = ensureBoolean(inputs.union, false);
      const { worldBoxes } = computeBoxesForContent(inputs.content, { union });
      return { worldBoxes };
    },
  });

  register('{79aa7f47-397c-4d3f-9761-aaf421bb7f5f}', {
    type: 'geometry',
    pinMap: {
      inputs: {
        B: 'base', base: 'base', Base: 'base',
        X: 'domainX', domainX: 'domainX', 'X Domain': 'domainX',
        Y: 'domainY', domainY: 'domainY', 'Y Domain': 'domainY',
        Z: 'domainZ', domainZ: 'domainZ', 'Z Domain': 'domainZ',
      },
      outputs: { B: 'box', Box: 'box' },
    },
    eval: ({ inputs }) => {
      const plane = ensurePlane(inputs.base);
      const domainX = ensureDomainInput(inputs.domainX, -0.5, 0.5);
      const domainY = ensureDomainInput(inputs.domainY, -0.5, 0.5);
      const domainZ = ensureDomainInput(inputs.domainZ, -0.5, 0.5);
      const box = createBoxFromDomains(plane, domainX, domainY, domainZ);
      return { box };
    },
  });

  register('{87df35c8-6e1d-4e2a-821a-7c1066714409}', {
    type: 'geometry',
    pinMap: {
      inputs: {
        C: 'content', content: 'content', Content: 'content',
        P: 'plane', plane: 'plane', Plane: 'plane',
        U: 'union', Union: 'union',
      },
      outputs: {
        B: 'worldBoxes', Box: 'worldBoxes',
        'plane box': 'planeBoxes', Plane: 'planeBoxes',
      },
    },
    eval: ({ inputs }) => {
      const plane = inputs.plane ? ensurePlane(inputs.plane) : null;
      const union = ensureBoolean(inputs.union, false);
      const { worldBoxes, planeBoxes } = computeBoxesForContent(inputs.content, { plane, union });
      return { worldBoxes, planeBoxes };
    },
  });

  register('{9aef6eb4-98c3-4b0e-b875-1a7cb1bb1038}', {
    type: 'geometry',
    pinMap: {
      inputs: { A: 'pointA', a: 'pointA', 'Point A': 'pointA', B: 'pointB', b: 'pointB', 'Point B': 'pointB' },
      outputs: { B: 'box', Box: 'box' },
    },
    eval: ({ inputs }) => {
      const pointA = ensurePoint(inputs.pointA, new THREE.Vector3());
      const pointB = ensurePoint(inputs.pointB, new THREE.Vector3(1, 1, 1));
      const points = [pointA, pointB];
      const box = createAxisAlignedBoxFromPoints(points);
      return { box };
    },
  });

  register('{9d375779-649d-49f1-baaf-04560a51cd3d}', {
    type: 'geometry',
    pinMap: {
      inputs: { C: 'content', content: 'content', Content: 'content' },
      outputs: { B: 'worldBoxes', Box: 'worldBoxes' },
    },
    eval: ({ inputs }) => {
      const { worldBoxes } = computeBoxesForContent(inputs.content);
      return { worldBoxes };
    },
  });
  register('{b083c06d-9a71-4f40-b354-1d80bba1e858}', {
    type: 'surface',
    pinMap: {
      inputs: { P1: 'p1', P2: 'p2', P3: 'p3', P4: 'p4', 'Point 1': 'p1', 'Point 2': 'p2', 'Point 3': 'p3', 'Point 4': 'p4' },
      outputs: { C: 'center', Center: 'center', R: 'radius', Radius: 'radius', S: 'sphere', Sphere: 'sphere', surface: 'sphere' },
    },
    eval: ({ inputs }) => {
      const points = [inputs.p1, inputs.p2, inputs.p3, inputs.p4]
        .map((value, index) => ensurePoint(value, index === 0 ? new THREE.Vector3() : new THREE.Vector3(index, 0, 0)));
      const sphere = computeSphereFromPoints(points);
      if (!sphere) {
        return {};
      }
      const orientationPlane = planeFromPoints(points[0], points[1], points[2]);
      const plane = normalizePlaneAxes(sphere.center.clone(), orientationPlane.xAxis.clone(), orientationPlane.yAxis.clone(), orientationPlane.zAxis.clone());
      const surface = createSphereSurface(sphere.center, plane, sphere.radius);
      return {
        center: sphere.center,
        radius: sphere.radius,
        sphere: wrapSurface(surface, { center: sphere.center, radius: sphere.radius }),
      };
    },
  });

  register('{d0a56c9e-2483-45e7-ab98-a450b97f1bc0}', {
    type: 'geometry',
    pinMap: {
      inputs: { R: 'rectangle', rectangle: 'rectangle', Rectangle: 'rectangle', H: 'height', height: 'height', Height: 'height' },
      outputs: { B: 'box', Box: 'box' },
    },
    eval: ({ inputs }) => {
      const rectangleData = extractRectangleData(inputs.rectangle);
      const height = ensureNumeric(inputs.height, 1);
      if (!rectangleData) {
        return {};
      }
      const box = createBoxFromRectangle(rectangleData, height);
      return { box };
    },
  });

  register('{d8698126-0e91-4ae7-ba05-2490258573ea}', {
    type: 'surface',
    pinMap: {
      inputs: { P: 'plane', plane: 'plane', Plane: 'plane', S: 'shape', shape: 'shape', Shape: 'shape', I: 'inflate', Inflate: 'inflate', inflate: 'inflate' },
      outputs: { S: 'surface', Surface: 'surface', surface: 'surface' },
    },
    eval: ({ inputs }) => {
      const plane = ensurePlane(inputs.plane);
      const points = collectPoints(inputs.shape);
      if (!points.length) {
        return {};
      }
      const bounds = projectPointsToPlaneBounds(points, plane);
      if (!bounds) {
        return {};
      }
      const inflate = Math.abs(ensureNumeric(inputs.inflate, 0));
      const expanded = expandPlaneBounds(bounds, inflate, inflate);
      const surface = createPlanarSurfaceFromBounds(plane, expanded.min.x, expanded.max.x, expanded.min.y, expanded.max.y);
      return { surface: wrapSurface(surface) };
    },
  });

  register('{dabc854d-f50e-408a-b001-d043c7de151d}', {
    type: 'surface',
    pinMap: {
      inputs: { B: 'base', base: 'base', Base: 'base', R: 'radius', Radius: 'radius', radius: 'radius' },
      outputs: { S: 'sphere', Sphere: 'sphere', surface: 'sphere' },
    },
    eval: ({ inputs }) => {
      const plane = ensurePlane(inputs.base);
      const radius = ensureNumeric(inputs.radius, 1);
      const center = plane.origin.clone();
      const surface = createSphereSurface(center, plane, radius);
      return {
        sphere: wrapSurface(surface, { center, radius: Math.max(Math.abs(radius), EPSILON) }),
      };
    },
  });

  register('{e7ffb3af-2d77-4804-a260-755308bf8285}', {
    type: 'surface',
    pinMap: {
      inputs: { P: 'points', points: 'points', Points: 'points' },
      outputs: { C: 'center', Center: 'center', R: 'radius', Radius: 'radius', S: 'sphere', Sphere: 'sphere', surface: 'sphere' },
    },
    eval: ({ inputs }) => {
      const points = collectPoints(inputs.points);
      const sphere = computeSphereFromPoints(points);
      if (!sphere) {
        return {};
      }
      let plane = null;
      if (points.length >= 3) {
        plane = planeFromPoints(points[0], points[1], points[2]);
      } else {
        plane = defaultPlane();
      }
      const orientedPlane = normalizePlaneAxes(sphere.center.clone(), plane.xAxis.clone(), plane.yAxis.clone(), plane.zAxis.clone());
      const surface = createSphereSurface(sphere.center, orientedPlane, sphere.radius);
      return {
        center: sphere.center,
        radius: sphere.radius,
        sphere: wrapSurface(surface, { center: sphere.center, radius: sphere.radius }),
      };
    },
  });

  register('{f565fd67-5a98-4b48-9ea9-2e184a9ef0e6}', {
    type: 'surface',
    pinMap: {
      inputs: { P: 'plane', plane: 'plane', Plane: 'plane', B: 'box', box: 'box', Box: 'box', I: 'inflate', Inflate: 'inflate', inflate: 'inflate' },
      outputs: { S: 'surface', Surface: 'surface', surface: 'surface' },
    },
    eval: ({ inputs }) => {
      const plane = ensurePlane(inputs.plane);
      const points = collectPoints(inputs.box);
      if (!points.length) {
        return {};
      }
      const bounds = projectPointsToPlaneBounds(points, plane);
      if (!bounds) {
        return {};
      }
      const inflate = Math.abs(ensureNumeric(inputs.inflate, 0));
      const expanded = expandPlaneBounds(bounds, inflate, inflate);
      const surface = createPlanarSurfaceFromBounds(plane, expanded.min.x, expanded.max.x, expanded.min.y, expanded.max.y);
      return { surface: wrapSurface(surface) };
    },
  });
  }

  registerAnalysisComponents();
}

export function registerSurfaceFreeformComponents(args) {
  registerSurfacePrimitiveComponents({ ...args, mode: REGISTER_SURFACE_FREEFORM_ONLY });
}

export function registerSurfaceAnalysisComponents(args) {
  registerSurfacePrimitiveComponents({ ...args, mode: REGISTER_SURFACE_ANALYSIS_ONLY });
}

export function registerSurfaceSubDComponents(args) {
  registerSurfacePrimitiveComponents({ ...args, mode: REGISTER_SURFACE_SUBD_ONLY });
}

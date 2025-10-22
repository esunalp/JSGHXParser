import * as THREE from 'three';

const REGISTER_SURFACE_FREEFORM_ONLY = Symbol('register-surface-freeform-only');

export function registerSurfacePrimitiveComponents({
  register,
  toNumber,
  toVector3,
  mode = null,
  includeFreeform = false,
}) {
  const freeformOnly = mode === REGISTER_SURFACE_FREEFORM_ONLY;
  const shouldRegisterFreeform = freeformOnly || includeFreeform;
  if (typeof register !== 'function') {
    throw new Error('register function is required to register surface primitive components.');
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
}

export function registerSurfaceFreeformComponents(args) {
  registerSurfacePrimitiveComponents({ ...args, mode: REGISTER_SURFACE_FREEFORM_ONLY });
}

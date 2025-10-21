import * as THREE from 'three';

export function registerSurfacePrimitiveComponents({ register, toNumber, toVector3 }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register surface primitive components.');
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

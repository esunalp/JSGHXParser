import * as THREE from 'three';

export function registerVectorPointComponents({ register, toNumber, toVector3 }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register vector point components.');
  }
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register vector point components.');
  }
  if (typeof toVector3 !== 'function') {
    throw new Error('toVector3 function is required to register vector point components.');
  }

  const EPSILON = 1e-9;

  function ensureNumber(value, fallback = 0) {
    const numeric = toNumber(value, Number.NaN);
    return Number.isFinite(numeric) ? numeric : fallback;
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

  function ensureArray(value) {
    if (value === undefined || value === null) {
      return [];
    }
    return Array.isArray(value) ? value : [value];
  }

  function ensurePoint(value, fallback = new THREE.Vector3()) {
    const point = toVector3(value, null);
    if (point) {
      return point;
    }
    return fallback.clone();
  }

  function collectNumbers(input) {
    const result = [];
    const stack = [input];
    const visited = new Set();
    while (stack.length) {
      const current = stack.pop();
      if (current === undefined || current === null) {
        continue;
      }
      if (typeof current === 'object' && current !== null) {
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
        result.push(current.x, current.y, current.z);
        continue;
      }
      if (typeof current === 'object') {
        if (Object.prototype.hasOwnProperty.call(current, 'value')) {
          stack.push(current.value);
          continue;
        }
        if (Object.prototype.hasOwnProperty.call(current, 'values')) {
          stack.push(current.values);
          continue;
        }
      }
      const numeric = toNumber(current, Number.NaN);
      if (Number.isFinite(numeric)) {
        result.push(numeric);
      }
    }
    return result;
  }

  function collectPoints(input) {
    const result = [];
    const stack = [input];
    const visited = new Set();
    while (stack.length) {
      const current = stack.pop();
      if (current === undefined || current === null) {
        continue;
      }
      if (typeof current === 'object' && current !== null) {
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
        result.push(current.clone());
        continue;
      }
      if (typeof current === 'object') {
        if (Object.prototype.hasOwnProperty.call(current, 'point')) {
          stack.push(current.point);
        }
        if (Object.prototype.hasOwnProperty.call(current, 'points')) {
          stack.push(current.points);
        }
        if (Object.prototype.hasOwnProperty.call(current, 'position')) {
          stack.push(current.position);
        }
        if (Object.prototype.hasOwnProperty.call(current, 'value')) {
          stack.push(current.value);
        }
        const vector = toVector3(current, null);
        if (vector) {
          result.push(vector);
        }
        continue;
      }
      const numeric = toNumber(current, Number.NaN);
      if (Number.isFinite(numeric)) {
        result.push(new THREE.Vector3(numeric, 0, 0));
      }
    }
    return result;
  }

  function parseMask(maskInput, fallback = ['x', 'y', 'z']) {
    if (maskInput === undefined || maskInput === null) {
      return fallback.slice();
    }
    if (Array.isArray(maskInput)) {
      const entries = [];
      for (const entry of maskInput) {
        const subMask = parseMask(entry, []);
        if (subMask.length) {
          entries.push(...subMask);
        }
      }
      return entries.length ? entries : fallback.slice();
    }
    if (typeof maskInput === 'object') {
      if (Object.prototype.hasOwnProperty.call(maskInput, 'mask')) {
        return parseMask(maskInput.mask, fallback);
      }
      if (Object.prototype.hasOwnProperty.call(maskInput, 'value')) {
        return parseMask(maskInput.value, fallback);
      }
    }
    const text = String(maskInput ?? '').trim().toLowerCase();
    if (!text) {
      return fallback.slice();
    }
    const axes = [];
    for (const char of text) {
      if (char === 'x' || char === 'y' || char === 'z') {
        axes.push(char);
      }
    }
    return axes.length ? axes : fallback.slice();
  }

  function defaultPlane() {
    return {
      origin: new THREE.Vector3(0, 0, 0),
      xAxis: new THREE.Vector3(1, 0, 0),
      yAxis: new THREE.Vector3(0, 1, 0),
      zAxis: new THREE.Vector3(0, 0, 1),
    };
  }

  function clonePlaneData(plane) {
    return {
      origin: plane.origin.clone(),
      xAxis: plane.xAxis.clone(),
      yAxis: plane.yAxis.clone(),
      zAxis: plane.zAxis.clone(),
    };
  }

  function normalizeVector(vector, fallback = new THREE.Vector3(1, 0, 0)) {
    const result = vector.clone();
    if (!Number.isFinite(result.x) || !Number.isFinite(result.y) || !Number.isFinite(result.z)) {
      return fallback.clone();
    }
    if (result.lengthSq() < EPSILON) {
      return fallback.clone();
    }
    return result.normalize();
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

  function normalizePlaneAxes(origin, xAxis, yAxis, zAxis) {
    const z = normalizeVector(zAxis, new THREE.Vector3(0, 0, 1));
    let x = xAxis.clone();
    if (x.lengthSq() < EPSILON) {
      x = orthogonalVector(z);
    } else {
      x.normalize();
    }
    let y = yAxis.clone();
    if (y.lengthSq() < EPSILON) {
      y = z.clone().cross(x).normalize();
    } else {
      y.normalize();
    }
    x = y.clone().cross(z).normalize();
    y = z.clone().cross(x).normalize();
    return {
      origin: origin.clone(),
      xAxis: x,
      yAxis: y,
      zAxis: z,
    };
  }

  function planeFromPoints(a, b, c) {
    const origin = ensurePoint(a, new THREE.Vector3());
    const ab = ensurePoint(b, origin.clone()).sub(origin.clone());
    const ac = ensurePoint(c, origin.clone()).sub(origin.clone());
    const normal = ab.clone().cross(ac);
    if (normal.lengthSq() < EPSILON) {
      return normalizePlaneAxes(origin, new THREE.Vector3(1, 0, 0), new THREE.Vector3(0, 1, 0), new THREE.Vector3(0, 0, 1));
    }
    const xAxis = ab.lengthSq() < EPSILON ? orthogonalVector(normal) : ab.clone().normalize();
    const yAxis = normal.clone().cross(xAxis).normalize();
    const zAxis = normal.clone().normalize();
    return normalizePlaneAxes(origin, xAxis, yAxis, zAxis);
  }

  function hasPlaneProperties(value) {
    if (!value || typeof value !== 'object') {
      return false;
    }
    if (value.isPlane) {
      return true;
    }
    let score = 0;
    if (Object.prototype.hasOwnProperty.call(value, 'origin')) score += 1;
    if (Object.prototype.hasOwnProperty.call(value, 'normal')) score += 1;
    if (Object.prototype.hasOwnProperty.call(value, 'zAxis')) score += 1;
    if (Object.prototype.hasOwnProperty.call(value, 'xAxis')) score += 1;
    if (Object.prototype.hasOwnProperty.call(value, 'yAxis')) score += 1;
    if (Object.prototype.hasOwnProperty.call(value, 'plane')) score += 2;
    return score >= 2;
  }

  function ensurePlane(input, fallback = defaultPlane()) {
    if (input === undefined || input === null) {
      return clonePlaneData(fallback);
    }
    if (input?.isPlane) {
      const normal = input.normal.clone();
      const origin = input.coplanarPoint ? input.coplanarPoint(new THREE.Vector3()) : new THREE.Vector3();
      if (origin && origin.isVector3) {
        const xAxis = orthogonalVector(normal);
        const yAxis = normal.clone().cross(xAxis).normalize();
        return normalizePlaneAxes(origin, xAxis, yAxis, normal);
      }
    }
    if (Array.isArray(input)) {
      const points = collectPoints(input);
      if (points.length >= 3) {
        return planeFromPoints(points[0], points[1], points[2]);
      }
      if (points.length === 2) {
        const origin = points[0];
        const direction = points[1].clone().sub(points[0]);
        if (direction.lengthSq() < EPSILON) {
          return clonePlaneData(fallback);
        }
        const xAxis = direction.clone().normalize();
        const normal = orthogonalVector(direction);
        const yAxis = normal.clone().cross(xAxis).normalize();
        return normalizePlaneAxes(origin, xAxis, yAxis, normal);
      }
      if (points.length === 1) {
        const origin = points[0];
        return normalizePlaneAxes(origin, new THREE.Vector3(1, 0, 0), new THREE.Vector3(0, 1, 0), new THREE.Vector3(0, 0, 1));
      }
    }
    if (typeof input === 'object') {
      if (Object.prototype.hasOwnProperty.call(input, 'plane')) {
        return ensurePlane(input.plane, fallback);
      }
      if (Object.prototype.hasOwnProperty.call(input, 'origin') && Object.prototype.hasOwnProperty.call(input, 'xAxis') && Object.prototype.hasOwnProperty.call(input, 'yAxis')) {
        const origin = ensurePoint(input.origin, new THREE.Vector3());
        const xAxis = normalizeVector(ensurePoint(input.xAxis, new THREE.Vector3(1, 0, 0)), new THREE.Vector3(1, 0, 0));
        const yAxis = normalizeVector(ensurePoint(input.yAxis, new THREE.Vector3(0, 1, 0)), new THREE.Vector3(0, 1, 0));
        const zAxis = normalizeVector(Object.prototype.hasOwnProperty.call(input, 'zAxis') ? ensurePoint(input.zAxis, new THREE.Vector3(0, 0, 1)) : xAxis.clone().cross(yAxis), new THREE.Vector3(0, 0, 1));
        return normalizePlaneAxes(origin, xAxis, yAxis, zAxis);
      }
      if (Object.prototype.hasOwnProperty.call(input, 'origin') && Object.prototype.hasOwnProperty.call(input, 'normal')) {
        const origin = ensurePoint(input.origin, new THREE.Vector3());
        const normal = normalizeVector(ensurePoint(input.normal, new THREE.Vector3(0, 0, 1)), new THREE.Vector3(0, 0, 1));
        const xAxis = orthogonalVector(normal);
        const yAxis = normal.clone().cross(xAxis).normalize();
        return normalizePlaneAxes(origin, xAxis, yAxis, normal);
      }
      if (Object.prototype.hasOwnProperty.call(input, 'point') && Object.prototype.hasOwnProperty.call(input, 'normal')) {
        const origin = ensurePoint(input.point, new THREE.Vector3());
        const normal = normalizeVector(ensurePoint(input.normal, new THREE.Vector3(0, 0, 1)), new THREE.Vector3(0, 0, 1));
        const xAxis = orthogonalVector(normal);
        const yAxis = normal.clone().cross(xAxis).normalize();
        return normalizePlaneAxes(origin, xAxis, yAxis, normal);
      }
    }
    if (hasPlaneProperties(input)) {
      const origin = ensurePoint(input.origin ?? input.point ?? new THREE.Vector3(), new THREE.Vector3());
      const normal = normalizeVector(ensurePoint(input.normal ?? input.zAxis ?? new THREE.Vector3(0, 0, 1), new THREE.Vector3(0, 0, 1)), new THREE.Vector3(0, 0, 1));
      const xAxis = normalizeVector(ensurePoint(input.xAxis ?? new THREE.Vector3(1, 0, 0), new THREE.Vector3(1, 0, 0)), new THREE.Vector3(1, 0, 0));
      const yAxis = normalizeVector(ensurePoint(input.yAxis ?? new THREE.Vector3(0, 1, 0), new THREE.Vector3(0, 1, 0)), new THREE.Vector3(0, 1, 0));
      return normalizePlaneAxes(origin, xAxis, yAxis, normal);
    }
    if (typeof input === 'object' && (Object.prototype.hasOwnProperty.call(input, 'x') || Object.prototype.hasOwnProperty.call(input, 'y') || Object.prototype.hasOwnProperty.call(input, 'z'))) {
      const origin = ensurePoint(input, new THREE.Vector3());
      const plane = defaultPlane();
      plane.origin.copy(origin);
      return plane;
    }
    if (typeof input === 'object' && Object.prototype.hasOwnProperty.call(input, 'value')) {
      return ensurePlane(input.value, fallback);
    }
    const origin = ensurePoint(input, new THREE.Vector3());
    const plane = defaultPlane();
    plane.origin.copy(origin);
    return plane;
  }

  function planeCoordinates(point, plane) {
    const relative = point.clone().sub(plane.origin);
    return {
      x: relative.dot(plane.xAxis),
      y: relative.dot(plane.yAxis),
      z: relative.dot(plane.zAxis),
    };
  }

  function pointFromPlaneCoordinates(plane, u, v, w = 0) {
    return plane.origin.clone()
      .add(plane.xAxis.clone().multiplyScalar(u))
      .add(plane.yAxis.clone().multiplyScalar(v))
      .add(plane.zAxis.clone().multiplyScalar(w));
  }

  function evaluateCurvePoint(curve, t) {
    if (!curve) {
      return null;
    }
    if (typeof curve.getPointAt === 'function') {
      const pt = curve.getPointAt(t);
      if (pt?.isVector3) {
        return pt.clone();
      }
      if (pt && typeof pt === 'object') {
        return new THREE.Vector3(ensureNumber(pt.x, 0), ensureNumber(pt.y, 0), ensureNumber(pt.z, 0));
      }
    }
    if (curve.path && typeof curve.path.getPointAt === 'function') {
      const pt = curve.path.getPointAt(t);
      if (pt?.isVector3) {
        return pt.clone();
      }
      if (pt && typeof pt === 'object') {
        return new THREE.Vector3(ensureNumber(pt.x, 0), ensureNumber(pt.y, 0), ensureNumber(pt.z, 0));
      }
    }
    if (typeof curve.getPoint === 'function') {
      const pt = curve.getPoint(t);
      if (pt?.isVector3) {
        return pt.clone();
      }
      if (pt && typeof pt === 'object') {
        return new THREE.Vector3(ensureNumber(pt.x, 0), ensureNumber(pt.y, 0), ensureNumber(pt.z, 0));
      }
    }
    return null;
  }

  function approximateClosestParameterOnCurve(curve, point, { samples = 128, refinement = 4 } = {}) {
    if (!curve || typeof point?.clone !== 'function') {
      return null;
    }
    const safeSamples = Math.max(8, samples);
    let bestT = 0;
    let bestDistanceSq = Number.POSITIVE_INFINITY;
    for (let i = 0; i <= safeSamples; i += 1) {
      const t = i / safeSamples;
      const curvePoint = evaluateCurvePoint(curve, t);
      if (!curvePoint) {
        continue;
      }
      const distanceSq = curvePoint.distanceToSquared(point);
      if (distanceSq < bestDistanceSq) {
        bestDistanceSq = distanceSq;
        bestT = t;
      }
    }
    let searchCenter = bestT;
    let searchRadius = 1 / safeSamples;
    for (let iteration = 0; iteration < refinement; iteration += 1) {
      const start = Math.max(0, searchCenter - searchRadius);
      const end = Math.min(1, searchCenter + searchRadius);
      const steps = 10;
      for (let i = 0; i <= steps; i += 1) {
        const t = start + ((end - start) * (i / steps));
        const curvePoint = evaluateCurvePoint(curve, t);
        if (!curvePoint) {
          continue;
        }
        const distanceSq = curvePoint.distanceToSquared(point);
        if (distanceSq < bestDistanceSq) {
          bestDistanceSq = distanceSq;
          bestT = t;
        }
      }
      searchCenter = bestT;
      searchRadius *= 0.5;
    }
    const bestPoint = evaluateCurvePoint(curve, bestT);
    if (!bestPoint) {
      return null;
    }
    return {
      t: bestT,
      point: bestPoint,
      distanceSq: bestDistanceSq,
      distance: Math.sqrt(bestDistanceSq),
    };
  }

  function toCurveParameter(curve, tNormalized) {
    const domain = curve?.domain;
    if (!domain || typeof tNormalized !== 'number') {
      return tNormalized;
    }
    const start = ensureNumber(domain.start ?? domain.min ?? domain.t0 ?? domain.a ?? domain.from ?? 0, 0);
    const end = ensureNumber(domain.end ?? domain.max ?? domain.t1 ?? domain.b ?? domain.to ?? 1, 1);
    return start + (end - start) * tNormalized;
  }

  function projectPointOntoPlane(point, plane) {
    const relative = point.clone().sub(plane.origin);
    const distance = relative.dot(plane.zAxis);
    const projected = point.clone().sub(plane.zAxis.clone().multiplyScalar(distance));
    return { point: projected, distance };
  }

  function intersectRayWithPlane(point, direction, plane) {
    const normal = plane.zAxis.clone();
    const denominator = normal.dot(direction);
    if (Math.abs(denominator) < EPSILON) {
      return null;
    }
    const difference = plane.origin.clone().sub(point);
    const t = difference.dot(normal) / denominator;
    const intersectionPoint = point.clone().add(direction.clone().multiplyScalar(t));
    const distance = Math.abs(t) * direction.length();
    return { point: intersectionPoint, parameter: t, distance };
  }

  function createDataTree(branches = []) {
    return { type: 'tree', branches };
  }

  function toBranchesFromGroups(groups, mapper) {
    return groups.map((group, index) => ({
      path: [index],
      values: group.map(mapper),
    }));
  }

  function normalizeList(value) {
    if (value === undefined || value === null) {
      return [];
    }
    return Array.isArray(value) ? value : [value];
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

  function toText(value, fallback = '') {
    if (value === undefined || value === null) {
      return fallback;
    }
    if (Array.isArray(value)) {
      if (!value.length) {
        return fallback;
      }
      return toText(value[0], fallback);
    }
    return String(value ?? fallback);
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
      try {
        const color = new THREE.Color(text);
        return color;
      } catch (error) {
        return fallback ? fallback.clone() : null;
      }
    }
    if (typeof input === 'object') {
      if (Object.prototype.hasOwnProperty.call(input, 'color')) {
        return parseColor(input.color, fallback);
      }
      const r = ensureNumber(input.r ?? input.red, Number.NaN);
      const g = ensureNumber(input.g ?? input.green, Number.NaN);
      const b = ensureNumber(input.b ?? input.blue, Number.NaN);
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

  function ensureTagPlane(input) {
    if (hasPlaneProperties(input)) {
      return ensurePlane(input);
    }
    const origin = ensurePoint(input, new THREE.Vector3());
    const plane = defaultPlane();
    plane.origin.copy(origin);
    return plane;
  }

  function createTagEntry(location, text, size, color) {
    const plane = clonePlaneData(ensureTagPlane(location));
    const resolvedText = toText(text, '');
    const resolvedSize = Math.max(ensureNumber(size, 1), 0);
    const resolvedColor = parseColor(color, null);
    return {
      type: 'text-tag',
      plane,
      point: plane.origin.clone(),
      text: resolvedText,
      size: resolvedSize,
      color: resolvedColor,
    };
  }

  function parseGridSize(value, fallback = { x: 1, y: 1 }) {
    if (value === undefined || value === null) {
      return { ...fallback };
    }
    if (Array.isArray(value)) {
      if (!value.length) {
        return { ...fallback };
      }
      if (value.length === 1) {
        const numeric = ensureNumber(value[0], fallback.x);
        return { x: numeric, y: numeric };
      }
      const x = ensureNumber(value[0], fallback.x);
      const y = ensureNumber(value[1], fallback.y);
      return { x, y };
    }
    if (typeof value === 'object') {
      const x = ensureNumber(value.x ?? value.X ?? value.width ?? value.Width ?? value[0], fallback.x);
      const y = ensureNumber(value.y ?? value.Y ?? value.height ?? value.Height ?? value[1], fallback.y);
      return { x, y };
    }
    const numeric = ensureNumber(value, fallback.x);
    return { x: numeric, y: numeric };
  }

  function axialToWorld(plane, q, r, size) {
    const x = size * (Math.sqrt(3) * q + (Math.sqrt(3) / 2) * r);
    const y = size * (1.5 * r);
    return pointFromPlaneCoordinates(plane, x, y, 0);
  }

  function buildHexGrid(basePlane, radius, size) {
    const points = [];
    const cells = [];
    const centers = [];
    const axialRadius = Math.max(0, Math.floor(radius));
    for (let q = -axialRadius; q <= axialRadius; q += 1) {
      const rMin = Math.max(-axialRadius, -q - axialRadius);
      const rMax = Math.min(axialRadius, -q + axialRadius);
      for (let r = rMin; r <= rMax; r += 1) {
        const centerPoint = axialToWorld(basePlane, q, r, size);
        points.push(centerPoint.clone());
        const plane = clonePlaneData(basePlane);
        plane.origin.copy(centerPoint);
        centers.push(plane);
        const corners = [];
        for (let i = 0; i < 6; i += 1) {
          const angle = (Math.PI / 3) * i + Math.PI / 6;
          const offset = basePlane.xAxis.clone().multiplyScalar(size * Math.cos(angle))
            .add(basePlane.yAxis.clone().multiplyScalar(size * Math.sin(angle)));
          const corner = centerPoint.clone().add(offset);
          corners.push(corner);
        }
        corners.push(corners[0].clone());
        cells.push(corners);
      }
    }
    return { points, cells, centers };
  }

  function buildRectangularGrid(basePlane, xCount, yCount, sizeX, sizeY) {
    const gridPoints = [];
    const cells = [];
    const centers = [];
    for (let ix = 0; ix < xCount; ix += 1) {
      for (let iy = 0; iy < yCount; iy += 1) {
        gridPoints.push(pointFromPlaneCoordinates(basePlane, ix * sizeX, iy * sizeY, 0));
      }
    }
    if (xCount > 1 && yCount > 1) {
      for (let ix = 0; ix < xCount - 1; ix += 1) {
        for (let iy = 0; iy < yCount - 1; iy += 1) {
          const bottomLeft = pointFromPlaneCoordinates(basePlane, ix * sizeX, iy * sizeY, 0);
          const bottomRight = pointFromPlaneCoordinates(basePlane, (ix + 1) * sizeX, iy * sizeY, 0);
          const topRight = pointFromPlaneCoordinates(basePlane, (ix + 1) * sizeX, (iy + 1) * sizeY, 0);
          const topLeft = pointFromPlaneCoordinates(basePlane, ix * sizeX, (iy + 1) * sizeY, 0);
          cells.push([bottomLeft, bottomRight, topRight, topLeft, bottomLeft.clone()]);
          centers.push(pointFromPlaneCoordinates(basePlane, (ix + 0.5) * sizeX, (iy + 0.5) * sizeY, 0));
        }
      }
    }
    return { gridPoints, cells, centers };
  }

  function pickClosestCandidate(candidates, preferForward = true) {
    if (!candidates.length) {
      return null;
    }
    const sorted = [...candidates].sort((a, b) => {
      const aForward = preferForward ? a.parameter >= -EPSILON : true;
      const bForward = preferForward ? b.parameter >= -EPSILON : true;
      if (aForward && !bForward) {
        return -1;
      }
      if (!aForward && bForward) {
        return 1;
      }
      return a.distance - b.distance;
    });
    return sorted[0];
  }

  function resolveCount(value, fallback = 1) {
    return Math.max(1, Math.round(ensureNumber(value, fallback)));
  }

  function toUniquePoints(points, tolerance) {
    const unique = [];
    const indices = [];
    const valence = [];
    const toleranceSq = tolerance * tolerance;
    points.forEach((point, index) => {
      let foundIndex = -1;
      for (let i = 0; i < unique.length; i += 1) {
        if (unique[i].distanceToSquared(point) <= toleranceSq + EPSILON) {
          foundIndex = i;
          break;
        }
      }
      if (foundIndex === -1) {
        unique.push(point.clone());
        indices.push(index);
        valence.push(1);
      } else {
        valence[foundIndex] += 1;
      }
    });
    return { unique, indices, valence };
  }

  function groupNearbyPoints(points, threshold) {
    const groups = [];
    const visited = new Array(points.length).fill(false);
    const thresholdSq = threshold * threshold;
    for (let i = 0; i < points.length; i += 1) {
      if (visited[i]) {
        continue;
      }
      const queue = [i];
      visited[i] = true;
      const indices = [];
      const values = [];
      while (queue.length) {
        const index = queue.shift();
        indices.push(index);
        values.push(points[index].clone());
        for (let j = 0; j < points.length; j += 1) {
          if (visited[j]) {
            continue;
          }
          if (points[index].distanceToSquared(points[j]) <= thresholdSq + EPSILON) {
            visited[j] = true;
            queue.push(j);
          }
        }
      }
      groups.push({ indices, values });
    }
    return groups;
  }

  function geometryToPlaneCandidates(geometry) {
    const candidates = [];
    const stack = ensureArray(geometry);
    for (const entry of stack) {
      if (hasPlaneProperties(entry)) {
        candidates.push(ensurePlane(entry));
        continue;
      }
      if (entry && typeof entry === 'object' && Object.prototype.hasOwnProperty.call(entry, 'plane')) {
        candidates.push(ensurePlane(entry.plane));
      }
    }
    return candidates;
  }

  function geometryToCurveCandidates(geometry) {
    const candidates = [];
    const stack = ensureArray(geometry);
    for (const entry of stack) {
      if (!entry || typeof entry !== 'object') {
        continue;
      }
      if (typeof entry.getPointAt === 'function' || (entry.path && typeof entry.path.getPointAt === 'function')) {
        candidates.push(entry);
        continue;
      }
      if (Object.prototype.hasOwnProperty.call(entry, 'curve')) {
        candidates.push(entry.curve);
      }
    }
    return candidates;
  }

  function registerNumbersToPoints() {
    register(['{0ae07da9-951b-4b9b-98ca-d312c252374d}', 'numbers to points', 'num2pt'], {
      type: 'point',
      pinMap: {
        inputs: { N: 'numbers', Numbers: 'numbers', numbers: 'numbers', M: 'mask', Mask: 'mask', mask: 'mask' },
        outputs: { P: 'points', Points: 'points', points: 'points' },
      },
      eval: ({ inputs }) => {
        const numbers = collectNumbers(inputs.numbers ?? inputs.N);
        const mask = parseMask(inputs.mask ?? inputs.M);
        const chunkSize = Math.max(1, mask.length);
        const points = [];
        for (let index = 0; index + chunkSize - 1 < numbers.length; index += chunkSize) {
          let x = 0;
          let y = 0;
          let z = 0;
          for (let offset = 0; offset < chunkSize; offset += 1) {
            const value = numbers[index + offset];
            const axis = mask[offset];
            if (axis === 'x') {
              x = value;
            } else if (axis === 'y') {
              y = value;
            } else if (axis === 'z') {
              z = value;
            }
          }
          points.push(new THREE.Vector3(x, y, z));
        }
        return { points };
      },
    });
  }

  function registerTextTagComponents() {
    register([
      '{18564c36-5652-4c63-bb6f-f0e1273666dd}',
      '{ebf4d987-09b9-4825-a735-cac3d4770c19}',
      'text tag 3d',
      'tag 3d',
      'text tag3d',
    ], {
      type: 'text-tag',
      pinMap: {
        inputs: {
          L: 'locations', Location: 'locations', locations: 'locations',
          T: 'texts', Text: 'texts', texts: 'texts',
          S: 'sizes', Size: 'sizes', sizes: 'sizes',
          C: 'colours', Colour: 'colours', Color: 'colours', colours: 'colours', colors: 'colours',
        },
        outputs: { Tag: 'tags', Tags: 'tags', tags: 'tags' },
      },
      eval: ({ inputs }) => {
        const locations = normalizeList(inputs.locations ?? inputs.location ?? inputs.L);
        const texts = normalizeList(inputs.texts ?? inputs.text ?? inputs.T);
        const sizes = normalizeList(inputs.sizes ?? inputs.size ?? inputs.S);
        const colours = normalizeList(inputs.colours ?? inputs.colors ?? inputs.colour ?? inputs.color ?? inputs.C);
        const count = Math.max(locations.length, texts.length, sizes.length, colours.length, 1);
        const tags = [];
        for (let index = 0; index < count; index += 1) {
          const location = getListValue(locations, index, locations[0]);
          const text = getListValue(texts, index, texts[0]);
          const size = getListValue(sizes, index, sizes[0]);
          const colour = getListValue(colours, index, colours[0]);
          tags.push(createTagEntry(location, text, size, colour));
        }
        return { tags };
      },
    });

    register(['{4b3d38d3-0620-42e5-9ae8-0d4d9ad914cd}', 'text tag', 'tag'], {
      type: 'text-tag',
      pinMap: {
        inputs: { L: 'locations', Location: 'locations', locations: 'locations', T: 'texts', Text: 'texts', texts: 'texts' },
        outputs: { Tag: 'tags', Tags: 'tags', tags: 'tags' },
      },
      eval: ({ inputs }) => {
        const locations = normalizeList(inputs.locations ?? inputs.location ?? inputs.L);
        const texts = normalizeList(inputs.texts ?? inputs.text ?? inputs.T);
        const count = Math.max(locations.length, texts.length, 1);
        const tags = [];
        for (let index = 0; index < count; index += 1) {
          const location = getListValue(locations, index, locations[0]);
          const text = getListValue(texts, index, texts[0]);
          tags.push(createTagEntry(location, text, 1, null));
        }
        return { tags };
      },
    });
  }

  function registerPointConstructionComponents() {
    register(['{3581f42a-9592-4549-bd6b-1c0fc39d067b}', 'construct point', 'point xyz', 'pt'], {
      type: 'point',
      pinMap: {
        inputs: { X: 'x', x: 'x', Y: 'y', y: 'y', Z: 'z', z: 'z' },
        outputs: { Pt: 'point', point: 'point', Point: 'point' },
      },
      eval: ({ inputs }) => {
        const x = ensureNumber(inputs.x ?? inputs.X, 0);
        const y = ensureNumber(inputs.y ?? inputs.Y, 0);
        const z = ensureNumber(inputs.z ?? inputs.Z, 0);
        return { point: new THREE.Vector3(x, y, z) };
      },
    });

    register(['{8a5aae11-8775-4ee5-b4fc-db3a1bd89c2f}', 'construct point oriented', 'point oriented'], {
      type: 'point',
      pinMap: {
        inputs: {
          X: 'x', x: 'x',
          Y: 'y', y: 'y',
          Z: 'z', z: 'z',
          S: 'system', System: 'system', system: 'system',
        },
        outputs: { Pt: 'point', point: 'point', Point: 'point' },
      },
      eval: ({ inputs }) => {
        const plane = ensurePlane(inputs.system ?? inputs.S);
        const x = ensureNumber(inputs.x ?? inputs.X, 0);
        const y = ensureNumber(inputs.y ?? inputs.Y, 0);
        const z = ensureNumber(inputs.z ?? inputs.Z, 0);
        return { point: pointFromPlaneCoordinates(plane, x, y, z) };
      },
    });

    register(['{aa333235-5922-424c-9002-1e0b866a854b}', 'point oriented', 'point uvw'], {
      type: 'point',
      pinMap: {
        inputs: {
          P: 'plane', Plane: 'plane', plane: 'plane',
          U: 'u', u: 'u',
          V: 'v', v: 'v',
          W: 'w', w: 'w',
        },
        outputs: { Pt: 'point', point: 'point', Point: 'point' },
      },
      eval: ({ inputs }) => {
        const plane = ensurePlane(inputs.plane ?? inputs.P);
        const u = ensureNumber(inputs.u ?? inputs.U, 0);
        const v = ensureNumber(inputs.v ?? inputs.V, 0);
        const w = ensureNumber(inputs.w ?? inputs.W, 0);
        return { point: pointFromPlaneCoordinates(plane, u, v, w) };
      },
    });

    register(['{23603075-be64-4d86-9294-c3c125a12104}', 'point cylindrical', 'point cylinder'], {
      type: 'point',
      pinMap: {
        inputs: {
          P: 'plane', Plane: 'plane', plane: 'plane',
          A: 'angle', angle: 'angle',
          R: 'radius', radius: 'radius',
          E: 'elevation', elevation: 'elevation',
        },
        outputs: { Pt: 'point', point: 'point', Point: 'point' },
      },
      eval: ({ inputs }) => {
        const plane = ensurePlane(inputs.plane ?? inputs.P);
        const angle = ensureNumber(inputs.angle ?? inputs.A, 0);
        const radius = ensureNumber(inputs.radius ?? inputs.R, 0);
        const elevation = ensureNumber(inputs.elevation ?? inputs.E, 0);
        const x = Math.cos(angle) * radius;
        const y = Math.sin(angle) * radius;
        return { point: pointFromPlaneCoordinates(plane, x, y, elevation) };
      },
    });

    register(['{a435f5c8-28a2-43e8-a52a-0b6e73c2e300}', 'point polar', 'point spherical'], {
      type: 'point',
      pinMap: {
        inputs: {
          P: 'plane', Plane: 'plane', plane: 'plane',
          xy: 'phi', Phi: 'phi', phi: 'phi',
          z: 'theta', Theta: 'theta', theta: 'theta',
          d: 'distance', Distance: 'distance', distance: 'distance',
        },
        outputs: { Pt: 'point', point: 'point', Point: 'point' },
      },
      eval: ({ inputs }) => {
        const plane = ensurePlane(inputs.plane ?? inputs.P);
        const phi = ensureNumber(inputs.phi ?? inputs.xy ?? inputs.Phi, 0);
        const theta = ensureNumber(inputs.theta ?? inputs.z ?? inputs.Theta, 0);
        const distance = ensureNumber(inputs.distance ?? inputs.d ?? inputs.Distance, 0);
        const horizontal = distance * Math.cos(theta);
        const x = Math.cos(phi) * horizontal;
        const y = Math.sin(phi) * horizontal;
        const z = Math.sin(theta) * distance;
        return { point: pointFromPlaneCoordinates(plane, x, y, z) };
      },
    });

    register(['{9adffd61-f5d1-4e9e-9572-e8d9145730dc}', 'barycentric point', 'barycentric'], {
      type: 'point',
      pinMap: {
        inputs: {
          A: 'a', a: 'a',
          B: 'b', b: 'b',
          C: 'c', c: 'c',
          U: 'u', u: 'u',
          V: 'v', v: 'v',
          W: 'w', w: 'w',
        },
        outputs: { P: 'point', Point: 'point', point: 'point' },
      },
      eval: ({ inputs }) => {
        const pointA = ensurePoint(inputs.a ?? inputs.A, new THREE.Vector3());
        const pointB = ensurePoint(inputs.b ?? inputs.B, new THREE.Vector3());
        const pointC = ensurePoint(inputs.c ?? inputs.C, new THREE.Vector3());
        const u = ensureNumber(inputs.u ?? inputs.U, 0);
        const v = ensureNumber(inputs.v ?? inputs.V, 0);
        let w = ensureNumber(inputs.w ?? inputs.W, Number.NaN);
        if (!Number.isFinite(w)) {
          w = 1 - u - v;
        }
        const point = new THREE.Vector3();
        point.add(pointA.clone().multiplyScalar(u));
        point.add(pointB.clone().multiplyScalar(v));
        point.add(pointC.clone().multiplyScalar(w));
        return { point };
      },
    });
  }

  function registerPointAnalysisComponents() {
    register(['{571ca323-6e55-425a-bf9e-ee103c7ba4b9}', 'closest point', 'cp'], {
      type: 'point',
      pinMap: {
        inputs: { P: 'point', Point: 'point', point: 'point', C: 'cloud', Cloud: 'cloud', cloud: 'cloud' },
        outputs: {
          P: 'closestPoint', Point: 'closestPoint', 'Closest Point': 'closestPoint', closestPoint: 'closestPoint',
          i: 'index', I: 'index', 'CP Index': 'index', index: 'index',
          D: 'distance', Distance: 'distance', distance: 'distance',
        },
      },
      eval: ({ inputs }) => {
        const target = ensurePoint(inputs.point ?? inputs.P, new THREE.Vector3());
        const candidates = collectPoints(inputs.cloud ?? inputs.C);
        if (!candidates.length) {
          return { closestPoint: null, index: -1, distance: 0 };
        }
        let bestIndex = 0;
        let bestDistanceSq = Number.POSITIVE_INFINITY;
        candidates.forEach((candidate, index) => {
          const distanceSq = candidate.distanceToSquared(target);
          if (distanceSq < bestDistanceSq) {
            bestDistanceSq = distanceSq;
            bestIndex = index;
          }
        });
        return {
          closestPoint: candidates[bestIndex].clone(),
          index: bestIndex,
          distance: Math.sqrt(bestDistanceSq),
        };
      },
    });

    register(['{446014c4-c11c-45a7-8839-c45dc60950d6}', 'closest points', 'cps'], {
      type: 'point',
      pinMap: {
        inputs: {
          P: 'point', Point: 'point', point: 'point',
          C: 'cloud', Cloud: 'cloud', cloud: 'cloud',
          N: 'count', Count: 'count', count: 'count',
        },
        outputs: {
          P: 'points', Points: 'points', 'Closest Point': 'points',
          i: 'indices', I: 'indices', 'CP Index': 'indices', indices: 'indices',
          D: 'distances', Distance: 'distances', distances: 'distances',
        },
      },
      eval: ({ inputs }) => {
        const target = ensurePoint(inputs.point ?? inputs.P, new THREE.Vector3());
        const candidates = collectPoints(inputs.cloud ?? inputs.C);
        if (!candidates.length) {
          return { points: [], indices: [], distances: [] };
        }
        const count = resolveCount(inputs.count ?? inputs.N, 1);
        const entries = candidates.map((candidate, index) => ({
          point: candidate.clone(),
          index,
          distance: candidate.distanceTo(target),
        }));
        entries.sort((a, b) => a.distance - b.distance);
        const limited = entries.slice(0, Math.min(count, entries.length));
        return {
          points: limited.map((entry) => entry.point.clone()),
          indices: limited.map((entry) => entry.index),
          distances: limited.map((entry) => entry.distance),
        };
      },
    });

    register(['{59aaebf8-6654-46b7-8386-89223c773978}', 'sort along curve', 'along curve'], {
      type: 'point',
      pinMap: {
        inputs: { P: 'points', Points: 'points', points: 'points', C: 'curve', Curve: 'curve', curve: 'curve' },
        outputs: { P: 'points', Points: 'points', points: 'points', I: 'indices', indices: 'indices', 'Point Index': 'indices' },
      },
      eval: ({ inputs }) => {
        const points = collectPoints(inputs.points ?? inputs.P);
        const curve = inputs.curve ?? inputs.C;
        if (!points.length || !curve) {
          return { points, indices: points.map((_, index) => index) };
        }
        const entries = points.map((point, index) => {
          const closest = approximateClosestParameterOnCurve(curve, point, { samples: 200, refinement: 4 });
          return {
            point: point.clone(),
            index,
            parameter: closest ? toCurveParameter(curve, closest.t) : Number.POSITIVE_INFINITY,
          };
        });
        entries.sort((a, b) => a.parameter - b.parameter);
        return {
          points: entries.map((entry) => entry.point),
          indices: entries.map((entry) => entry.index),
        };
      },
    });

    register(['{4e86ba36-05e2-4cc0-a0f5-3ad57c91f04e}', 'sort points', 'sort pt'], {
      type: 'point',
      pinMap: {
        inputs: { P: 'points', Points: 'points', points: 'points' },
        outputs: { P: 'points', Points: 'points', points: 'points', I: 'indices', indices: 'indices', Indices: 'indices' },
      },
      eval: ({ inputs }) => {
        const points = collectPoints(inputs.points ?? inputs.P);
        const entries = points.map((point, index) => ({ point: point.clone(), index }));
        entries.sort((a, b) => {
          if (Math.abs(a.point.x - b.point.x) > EPSILON) {
            return a.point.x - b.point.x;
          }
          if (Math.abs(a.point.y - b.point.y) > EPSILON) {
            return a.point.y - b.point.y;
          }
          if (Math.abs(a.point.z - b.point.z) > EPSILON) {
            return a.point.z - b.point.z;
          }
          return a.index - b.index;
        });
        return {
          points: entries.map((entry) => entry.point),
          indices: entries.map((entry) => entry.index),
        };
      },
    });

    register(['{93b8e93d-f932-402c-b435-84be04d87666}', 'distance', 'point distance'], {
      type: 'point',
      pinMap: {
        inputs: { A: 'a', a: 'a', B: 'b', b: 'b' },
        outputs: { D: 'distance', Distance: 'distance', distance: 'distance' },
      },
      eval: ({ inputs }) => {
        const a = ensurePoint(inputs.a ?? inputs.A, new THREE.Vector3());
        const b = ensurePoint(inputs.b ?? inputs.B, new THREE.Vector3());
        return { distance: a.distanceTo(b) };
      },
    });

    register(['{61647ba2-31eb-4921-9632-df81e3286f7d}', 'to polar', 'point to polar'], {
      type: 'point',
      pinMap: {
        inputs: { P: 'point', Point: 'point', point: 'point', S: 'system', System: 'system', system: 'system' },
        outputs: { P: 'phi', Phi: 'phi', phi: 'phi', T: 'theta', Theta: 'theta', theta: 'theta', R: 'radius', Radius: 'radius', radius: 'radius' },
      },
      eval: ({ inputs }) => {
        const point = ensurePoint(inputs.point ?? inputs.P, new THREE.Vector3());
        const plane = ensurePlane(inputs.system ?? inputs.S);
        const coords = planeCoordinates(point, plane);
        const radius = Math.sqrt(coords.x ** 2 + coords.y ** 2 + coords.z ** 2);
        const horizontal = Math.sqrt(coords.x ** 2 + coords.y ** 2);
        const phi = Math.atan2(coords.y, coords.x);
        const theta = Math.atan2(coords.z, horizontal);
        return { phi, theta, radius };
      },
    });

    register(['{670fcdba-da07-4eb4-b1c1-bfa0729d767d}', 'deconstruct point', 'depoint'], {
      type: 'point',
      pinMap: {
        inputs: { P: 'point', Point: 'point', point: 'point', S: 'system', System: 'system', system: 'system' },
        outputs: { X: 'x', x: 'x', Y: 'y', y: 'y', Z: 'z', z: 'z' },
      },
      eval: ({ inputs }) => {
        const point = ensurePoint(inputs.point ?? inputs.P, new THREE.Vector3());
        const plane = ensurePlane(inputs.system ?? inputs.S);
        const coords = planeCoordinates(point, plane);
        return { x: coords.x, y: coords.y, z: coords.z };
      },
    });

    register(['{9abae6b7-fa1d-448c-9209-4a8155345841}', 'deconstruct', 'pdecon'], {
      type: 'point',
      pinMap: {
        inputs: { P: 'point', Point: 'point', point: 'point' },
        outputs: { X: 'x', x: 'x', Y: 'y', y: 'y', Z: 'z', z: 'z' },
      },
      eval: ({ inputs }) => {
        const point = ensurePoint(inputs.point ?? inputs.P, new THREE.Vector3());
        return { x: point.x, y: point.y, z: point.z };
      },
    });

    register(['{6eaffbb2-3392-441a-8556-2dc126aa8910}', 'cull duplicates', 'cull pt'], {
      type: 'point',
      pinMap: {
        inputs: { P: 'points', Points: 'points', points: 'points', T: 'tolerance', Tolerance: 'tolerance', tolerance: 'tolerance' },
        outputs: { P: 'points', Points: 'points', points: 'points', I: 'indices', indices: 'indices', Indices: 'indices', V: 'valence', Valence: 'valence', valence: 'valence' },
      },
      eval: ({ inputs }) => {
        const points = collectPoints(inputs.points ?? inputs.P);
        const tolerance = Math.max(ensureNumber(inputs.tolerance ?? inputs.T, 0.001), 0);
        const { unique, indices, valence } = toUniquePoints(points, tolerance);
        return { points: unique, indices, valence };
      },
    });

    register(['{81f6afc9-22d9-49f0-8579-1fd7e0df6fa6}', 'point groups', 'pgroups'], {
      type: 'point',
      pinMap: {
        inputs: { P: 'points', Points: 'points', points: 'points', D: 'distance', Distance: 'distance', distance: 'distance' },
        outputs: { G: 'groups', Groups: 'groups', groups: 'groups', I: 'indices', Indices: 'indices', indices: 'indices' },
      },
      eval: ({ inputs }) => {
        const points = collectPoints(inputs.points ?? inputs.P);
        if (!points.length) {
          return {
            groups: createDataTree(),
            indices: createDataTree(),
          };
        }
        const distance = Math.max(ensureNumber(inputs.distance ?? inputs.D, 0.1), 0);
        if (distance <= EPSILON) {
          const { unique, indices } = toUniquePoints(points, 0);
          const groupBranches = unique.map((point, index) => ({ path: [index], values: [point.clone()] }));
          const indexBranches = indices.map((value, index) => ({ path: [index], values: [value] }));
          return {
            groups: createDataTree(groupBranches),
            indices: createDataTree(indexBranches),
          };
        }
        const groups = groupNearbyPoints(points, distance);
        const groupBranches = groups.map((group, index) => ({ path: [index], values: group.values }));
        const indexBranches = groups.map((group, index) => ({ path: [index], values: group.indices }));
        return {
          groups: createDataTree(groupBranches),
          indices: createDataTree(indexBranches),
        };
      },
    });
  }

  function registerPointConversionComponents() {
    register(['{d24169cc-9922-4923-92bc-b9222efc413f}', 'points to numbers', 'pt2num'], {
      type: 'point',
      pinMap: {
        inputs: { P: 'points', Points: 'points', points: 'points', M: 'mask', Mask: 'mask', mask: 'mask' },
        outputs: { N: 'numbers', Numbers: 'numbers', numbers: 'numbers' },
      },
      eval: ({ inputs }) => {
        const points = collectPoints(inputs.points ?? inputs.P);
        const mask = parseMask(inputs.mask ?? inputs.M);
        const numbers = [];
        points.forEach((point) => {
          for (const axis of mask) {
            if (axis === 'x') {
              numbers.push(point.x);
            } else if (axis === 'y') {
              numbers.push(point.y);
            } else if (axis === 'z') {
              numbers.push(point.z);
            }
          }
        });
        return { numbers };
      },
    });
  }

  function registerPointProjectionComponents() {
    register(['{5184b8cb-b71e-4def-a590-cd2c9bc58906}', 'project point', 'project'], {
      type: 'point',
      pinMap: {
        inputs: {
          P: 'point', Point: 'point', point: 'point',
          D: 'direction', Direction: 'direction', direction: 'direction',
          G: 'geometry', Geometry: 'geometry', geometry: 'geometry',
        },
        outputs: { P: 'point', Point: 'point', point: 'point', I: 'index', Index: 'index', index: 'index' },
      },
      eval: ({ inputs }) => {
        const point = ensurePoint(inputs.point ?? inputs.P, new THREE.Vector3());
        const direction = ensurePoint(inputs.direction ?? inputs.D, null);
        if (!direction || direction.lengthSq() < EPSILON) {
          return { point: null, index: -1 };
        }
        const planes = geometryToPlaneCandidates(inputs.geometry ?? inputs.G);
        const rayDirection = direction.clone();
        const candidates = [];
        planes.forEach((plane, index) => {
          const intersection = intersectRayWithPlane(point, rayDirection, plane);
          if (intersection) {
            candidates.push({ ...intersection, index });
          }
        });
        const best = pickClosestCandidate(candidates, true);
        if (!best) {
          return { point: null, index: -1 };
        }
        return { point: best.point, index: best.index };
      },
    });

    register([
      '{902289da-28dc-454b-98d4-b8f8aa234516}',
      '{cf3a0865-4882-46bd-91a1-d512acf95be4}',
      'pull point',
      'pull',
    ], {
      type: 'point',
      pinMap: {
        inputs: {
          P: 'point', Point: 'point', point: 'point',
          G: 'geometry', Geometry: 'geometry', geometry: 'geometry',
          C: 'closest', Closest: 'closest', closest: 'closest',
        },
        outputs: {
          P: 'closestPoint', Point: 'closestPoint', 'Closest Point': 'closestPoint', closestPoint: 'closestPoint',
          D: 'distance', Distance: 'distance', distance: 'distance',
        },
      },
      eval: ({ inputs }) => {
        const point = ensurePoint(inputs.point ?? inputs.P, new THREE.Vector3());
        const preferClosest = ensureBoolean(inputs.closest ?? inputs.C, true);
        const geometry = inputs.geometry ?? inputs.G;
        const pointCandidates = collectPoints(geometry);
        const planes = geometryToPlaneCandidates(geometry);
        const curves = geometryToCurveCandidates(geometry);
        const candidates = [];
        pointCandidates.forEach((candidate) => {
          candidates.push({ point: candidate.clone(), distance: candidate.distanceTo(point), parameter: 0 });
        });
        planes.forEach((plane) => {
          const projection = projectPointOntoPlane(point, plane);
          candidates.push({ point: projection.point, distance: Math.abs(projection.distance), parameter: 0 });
        });
        curves.forEach((curve) => {
          const closest = approximateClosestParameterOnCurve(curve, point, { samples: 200, refinement: 4 });
          if (closest) {
            candidates.push({ point: closest.point, distance: closest.distance, parameter: closest.t });
          }
        });
        if (!candidates.length) {
          return { closestPoint: point.clone(), distance: 0 };
        }
        candidates.sort((a, b) => a.distance - b.distance);
        const best = preferClosest ? candidates[0] : candidates[candidates.length - 1];
        return { closestPoint: best.point.clone(), distance: best.distance };
      },
    });
  }

  function registerGridComponents() {
    register(['{8ce6a747-6d36-4bd4-8af0-9a1081df417d}', 'grid hexagonal obsolete', 'hexgrid obsolete'], {
      type: 'point',
      pinMap: {
        inputs: {
          P: 'plane', Plane: 'plane', plane: 'plane',
          R: 'radius', Radius: 'radius', radius: 'radius',
          S: 'size', Size: 'size', size: 'size',
        },
        outputs: {
          G: 'grid', Grid: 'grid', grid: 'grid',
          C: 'cells', Cells: 'cells', cells: 'cells',
          M: 'centers', Centers: 'centers', centers: 'centers',
        },
      },
      eval: ({ inputs }) => {
        const plane = ensurePlane(inputs.plane ?? inputs.P);
        const radius = Math.max(0, ensureNumber(inputs.radius ?? inputs.R, 3));
        const size = Math.max(ensureNumber(inputs.size ?? inputs.S, 1), EPSILON);
        const grid = buildHexGrid(plane, radius, size);
        return { grid: grid.points, cells: grid.cells, centers: grid.centers };
      },
    });

    register(['{99f1e47c-978d-468f-bb3d-a3df44552a8e}', 'grid rectangular obsolete', 'rectangular grid obsolete'], {
      type: 'point',
      pinMap: {
        inputs: {
          P: 'plane', Plane: 'plane', plane: 'plane',
          X: 'xCount', x: 'xCount',
          Y: 'yCount', y: 'yCount',
          S: 'size', Size: 'size', size: 'size',
        },
        outputs: {
          G: 'grid', Grid: 'grid', grid: 'grid',
          C: 'cells', Cells: 'cells', cells: 'cells',
          M: 'centers', Centers: 'centers', centers: 'centers',
        },
      },
      eval: ({ inputs }) => {
        const plane = ensurePlane(inputs.plane ?? inputs.P);
        const xCount = Math.max(1, Math.round(ensureNumber(inputs.xCount ?? inputs.X, 3)));
        const yCount = Math.max(1, Math.round(ensureNumber(inputs.yCount ?? inputs.Y, 3)));
        const size = parseGridSize(inputs.size ?? inputs.S, { x: 1, y: 1 });
        const grid = buildRectangularGrid(plane, xCount, yCount, Math.max(size.x, EPSILON), Math.max(size.y, EPSILON));
        return { grid: grid.gridPoints, cells: grid.cells, centers: grid.centers };
      },
    });
  }

  registerNumbersToPoints();
  registerTextTagComponents();
  registerPointConstructionComponents();
  registerPointAnalysisComponents();
  registerPointConversionComponents();
  registerPointProjectionComponents();
  registerGridComponents();
}

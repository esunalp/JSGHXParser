import * as THREE from 'three';

function createVectorComponentRegistrar({ register, toNumber, toVector3 }) {
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
    if (fallback === null || fallback === undefined) {
      return fallback;
    }
    if (typeof fallback.clone === 'function') {
      return fallback.clone();
    }
    const fallbackPoint = toVector3(fallback, null);
    if (fallbackPoint) {
      return fallbackPoint;
    }
    return new THREE.Vector3();
  }

  function ensureVector(value, fallback = new THREE.Vector3()) {
    return ensurePoint(value, fallback);
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

  function collectPlanes(input) {
    const planes = [];
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
      if (hasPlaneProperties(current)) {
        planes.push(ensurePlane(current));
        continue;
      }
      if (typeof current === 'object') {
        if (Object.prototype.hasOwnProperty.call(current, 'plane')) {
          planes.push(ensurePlane(current.plane));
          continue;
        }
        if (Object.prototype.hasOwnProperty.call(current, 'value')) {
          stack.push(current.value);
          continue;
        }
      }
    }
    return planes;
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

  function ensureLine(input) {
    if (input === undefined || input === null) {
      const start = new THREE.Vector3();
      const end = new THREE.Vector3(1, 0, 0);
      return { start, end, direction: end.clone().sub(start) };
    }
    if (Array.isArray(input)) {
      if (input.length >= 2) {
        const start = toVector3(input[0], new THREE.Vector3());
        const end = toVector3(input[1], start.clone().add(new THREE.Vector3(1, 0, 0)));
        let direction = end.clone().sub(start);
        if (direction.lengthSq() < EPSILON && input.length > 2) {
          direction = toVector3(input[2], new THREE.Vector3());
        }
        if (direction.lengthSq() < EPSILON) {
          direction.set(1, 0, 0);
        }
        return { start, end, direction };
      }
      if (input.length === 1) {
        return ensureLine(input[0]);
      }
    }
    if (typeof input === 'object') {
      if (Object.prototype.hasOwnProperty.call(input, 'line')) {
        return ensureLine(input.line);
      }
      const start = toVector3(
        input.start
          ?? input.from
          ?? input.a
          ?? input.A
          ?? input.origin
          ?? input.p0
          ?? input.point0
          ?? input.pointA
          ?? input.point,
        new THREE.Vector3(),
      );
      let endCandidate = input.end ?? input.to ?? input.b ?? input.B ?? input.p1 ?? input.point1 ?? input.pointB;
      let directionCandidate = input.direction ?? input.dir ?? input.tangent ?? input.vector;
      if (directionCandidate !== undefined && directionCandidate !== null) {
        const direction = toVector3(directionCandidate, new THREE.Vector3(1, 0, 0));
        if (direction.lengthSq() >= EPSILON) {
          const normalizedDirection = direction.clone();
          if (endCandidate === undefined || endCandidate === null) {
            endCandidate = start.clone().add(normalizedDirection);
          }
          return {
            start,
            end: toVector3(endCandidate, start.clone().add(normalizedDirection)),
            direction: normalizedDirection,
          };
        }
      }
      const end = toVector3(
        endCandidate,
        start.clone().add(new THREE.Vector3(1, 0, 0)),
      );
      const direction = end.clone().sub(start);
      if (direction.lengthSq() < EPSILON) {
        direction.set(1, 0, 0);
      }
      return { start, end, direction };
    }
    const start = toVector3(input, new THREE.Vector3());
    const end = start.clone().add(new THREE.Vector3(1, 0, 0));
    return { start, end, direction: end.clone().sub(start) };
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

  function jacobiEigenDecomposition(matrix) {
    const m = [
      [matrix.xx, matrix.xy, matrix.xz],
      [matrix.xy, matrix.yy, matrix.yz],
      [matrix.xz, matrix.yz, matrix.zz],
    ];
    const eigenVectors = [
      [1, 0, 0],
      [0, 1, 0],
      [0, 0, 1],
    ];
    const tolerance = 1e-10;
    const maxIterations = 32;
    for (let iteration = 0; iteration < maxIterations; iteration += 1) {
      let p = 0;
      let q = 1;
      if (Math.abs(m[0][1]) < Math.abs(m[0][2])) {
        p = 0;
        q = 2;
      }
      if (Math.abs(m[p][q]) < Math.abs(m[1][2])) {
        p = 1;
        q = 2;
      }
      if (Math.abs(m[p][q]) < tolerance) {
        break;
      }
      const app = m[p][p];
      const aqq = m[q][q];
      const apq = m[p][q];
      const angle = 0.5 * Math.atan2(2 * apq, aqq - app);
      const c = Math.cos(angle);
      const s = Math.sin(angle);
      for (let k = 0; k < 3; k += 1) {
        if (k === p || k === q) {
          continue;
        }
        const mkp = m[k][p];
        const mkq = m[k][q];
        m[k][p] = c * mkp - s * mkq;
        m[p][k] = m[k][p];
        m[k][q] = c * mkq + s * mkp;
        m[q][k] = m[k][q];
      }
      m[p][p] = c * c * app - 2 * s * c * apq + s * s * aqq;
      m[q][q] = s * s * app + 2 * s * c * apq + c * c * aqq;
      m[p][q] = 0;
      m[q][p] = 0;
      for (let k = 0; k < 3; k += 1) {
        const vip = eigenVectors[k][p];
        const viq = eigenVectors[k][q];
        eigenVectors[k][p] = c * vip - s * viq;
        eigenVectors[k][q] = s * vip + c * viq;
      }
    }
    const eigenValues = [m[0][0], m[1][1], m[2][2]];
    return { eigenValues, eigenVectors };
  }

  function fitPlaneToPoints(points) {
    if (!points.length) {
      return { plane: defaultPlane(), deviation: 0 };
    }
    if (points.length === 1) {
      const plane = defaultPlane();
      plane.origin.copy(points[0]);
      return { plane, deviation: 0 };
    }
    if (points.length === 2) {
      const origin = points[0].clone();
      let xAxis = points[1].clone().sub(points[0]);
      if (xAxis.lengthSq() < EPSILON) {
        xAxis = new THREE.Vector3(1, 0, 0);
      }
      xAxis.normalize();
      const normal = orthogonalVector(xAxis);
      const yAxis = normal.clone().cross(xAxis).normalize();
      return { plane: normalizePlaneAxes(origin, xAxis, yAxis, normal), deviation: 0 };
    }
    const centroid = new THREE.Vector3();
    points.forEach((point) => centroid.add(point));
    centroid.divideScalar(points.length);
    let xx = 0;
    let xy = 0;
    let xz = 0;
    let yy = 0;
    let yz = 0;
    let zz = 0;
    points.forEach((point) => {
      const dx = point.x - centroid.x;
      const dy = point.y - centroid.y;
      const dz = point.z - centroid.z;
      xx += dx * dx;
      xy += dx * dy;
      xz += dx * dz;
      yy += dy * dy;
      yz += dy * dz;
      zz += dz * dz;
    });
    const { eigenValues, eigenVectors } = jacobiEigenDecomposition({ xx, xy, xz, yy, yz, zz });
    let minIndex = 0;
    if (eigenValues[1] < eigenValues[minIndex]) minIndex = 1;
    if (eigenValues[2] < eigenValues[minIndex]) minIndex = 2;
    let normal = new THREE.Vector3(
      eigenVectors[0][minIndex],
      eigenVectors[1][minIndex],
      eigenVectors[2][minIndex],
    );
    if (normal.lengthSq() < EPSILON) {
      normal = new THREE.Vector3(0, 0, 1);
    } else {
      normal.normalize();
    }
    let xAxis = points[0].clone().sub(centroid);
    xAxis.sub(normal.clone().multiplyScalar(xAxis.dot(normal)));
    if (xAxis.lengthSq() < EPSILON) {
      xAxis = orthogonalVector(normal);
    } else {
      xAxis.normalize();
    }
    const yAxis = normal.clone().cross(xAxis).normalize();
    xAxis = yAxis.clone().cross(normal).normalize();
    const plane = normalizePlaneAxes(centroid, xAxis, yAxis, normal);
    let deviation = 0;
    points.forEach((point) => {
      const coords = planeCoordinates(point, plane);
      deviation = Math.max(deviation, Math.abs(coords.z));
    });
    return { plane, deviation };
  }

  function planeFromLineAndPoint(line, point) {
    const origin = line.start.clone();
    let xAxis = line.direction.clone();
    if (xAxis.lengthSq() < EPSILON) {
      xAxis = line.end.clone().sub(line.start);
    }
    if (xAxis.lengthSq() < EPSILON) {
      xAxis = new THREE.Vector3(1, 0, 0);
    }
    xAxis.normalize();
    let offset = point.clone().sub(origin);
    offset.sub(xAxis.clone().multiplyScalar(offset.dot(xAxis)));
    if (offset.lengthSq() < EPSILON) {
      offset = orthogonalVector(xAxis);
    } else {
      offset.normalize();
    }
    let normal = xAxis.clone().cross(offset);
    if (normal.lengthSq() < EPSILON) {
      const fallback = orthogonalVector(xAxis);
      offset = fallback.clone();
      normal = xAxis.clone().cross(offset);
    }
    normal.normalize();
    const yAxis = normal.clone().cross(xAxis).normalize();
    return normalizePlaneAxes(origin, xAxis, yAxis, normal);
  }

  function planeFromLines(lineA, lineB) {
    const origin = lineA.start.clone();
    let xAxis = lineA.direction.clone();
    if (xAxis.lengthSq() < EPSILON) {
      xAxis = lineA.end.clone().sub(lineA.start);
    }
    if (xAxis.lengthSq() < EPSILON) {
      xAxis = new THREE.Vector3(1, 0, 0);
    }
    xAxis.normalize();
    let reference = lineB.direction.clone();
    if (reference.lengthSq() < EPSILON) {
      reference = lineB.end.clone().sub(lineB.start);
    }
    if (reference.lengthSq() < EPSILON) {
      reference = lineB.start.clone().sub(origin);
    }
    if (reference.lengthSq() < EPSILON) {
      reference = orthogonalVector(xAxis);
    }
    let normal = xAxis.clone().cross(reference);
    if (normal.lengthSq() < EPSILON) {
      normal = xAxis.clone().cross(lineB.start.clone().sub(origin));
    }
    if (normal.lengthSq() < EPSILON) {
      normal = orthogonalVector(xAxis);
    }
    normal.normalize();
    const yAxis = normal.clone().cross(xAxis).normalize();
    return normalizePlaneAxes(origin, xAxis, yAxis, normal);
  }

  function alignPlaneToReference(reference, plane) {
    const target = clonePlaneData(plane);
    if (target.zAxis.dot(reference.zAxis) < 0) {
      target.zAxis.multiplyScalar(-1);
      target.xAxis.multiplyScalar(-1);
      target.yAxis.multiplyScalar(-1);
    }
    const candidates = [
      {
        origin: target.origin.clone(),
        xAxis: target.xAxis.clone(),
        yAxis: target.yAxis.clone(),
        zAxis: target.zAxis.clone(),
      },
      {
        origin: target.origin.clone(),
        xAxis: target.xAxis.clone().multiplyScalar(-1),
        yAxis: target.yAxis.clone().multiplyScalar(-1),
        zAxis: target.zAxis.clone(),
      },
    ];
    let best = candidates[0];
    let bestScore = Number.NEGATIVE_INFINITY;
    candidates.forEach((candidate) => {
      const score = candidate.xAxis.dot(reference.xAxis) + candidate.yAxis.dot(reference.yAxis);
      if (score > bestScore) {
        best = candidate;
        bestScore = score;
      }
    });
    return normalizePlaneAxes(best.origin, best.xAxis, best.yAxis, best.zAxis);
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

  function ensureColor(value, fallback = new THREE.Color()) {
    const fallbackColor = fallback ?? new THREE.Color();
    return parseColor(value, fallbackColor) ?? fallbackColor.clone();
  }

  function clamp01(value) {
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
  }

  function clampColor(color) {
    const result = color ?? new THREE.Color();
    result.r = clamp01(result.r);
    result.g = clamp01(result.g);
    result.b = clamp01(result.b);
    return result;
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

  function createSeededRandom(seedInput) {
    const numericSeed = Math.floor(ensureNumber(seedInput, Number.NaN));
    if (!Number.isFinite(numericSeed)) {
      return () => Math.random();
    }
    let state = numericSeed % 2147483647;
    if (state <= 0) {
      state += 2147483646;
    }
    return () => {
      state = (state * 16807) % 2147483647;
      return state / 2147483647;
    };
  }

  function computeBoundingBoxFromPoints(points) {
    if (!Array.isArray(points) || !points.length) {
      return null;
    }
    const box = new THREE.Box3();
    let valid = false;
    for (const point of points) {
      if (point?.isVector3) {
        box.expandByPoint(point);
        valid = true;
      }
    }
    return valid ? box : null;
  }

  function randomPointInAxisAlignedBox(box, rng = Math.random) {
    if (!box || !box.isBox3) {
      return new THREE.Vector3();
    }
    const u = rng();
    const v = rng();
    const w = rng();
    const x = THREE.MathUtils.lerp(box.min.x, box.max.x, u);
    const y = THREE.MathUtils.lerp(box.min.y, box.max.y, v);
    const z = THREE.MathUtils.lerp(box.min.z, box.max.z, w);
    return new THREE.Vector3(x, y, z);
  }

  function randomPointInRectangle(section, rng = Math.random) {
    const u = rng();
    const v = rng();
    const x = THREE.MathUtils.lerp(section.minX, section.maxX, u);
    const y = THREE.MathUtils.lerp(section.minY, section.maxY, v);
    return pointFromPlaneCoordinates(section.plane, x, y, 0);
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

  function buildRectangularGrid(basePlane, xCount, yCount, sizeX, sizeY, offset = { x: 0, y: 0 }) {
    const gridPoints = [];
    const cells = [];
    const centers = [];
    const offsetX = ensureNumber(offset.x, 0);
    const offsetY = ensureNumber(offset.y, 0);
    for (let ix = 0; ix < xCount; ix += 1) {
      for (let iy = 0; iy < yCount; iy += 1) {
        gridPoints.push(pointFromPlaneCoordinates(basePlane, ix * sizeX + offsetX, iy * sizeY + offsetY, 0));
      }
    }
    if (xCount > 1 && yCount > 1) {
      for (let ix = 0; ix < xCount - 1; ix += 1) {
        for (let iy = 0; iy < yCount - 1; iy += 1) {
          const bottomLeft = pointFromPlaneCoordinates(basePlane, ix * sizeX + offsetX, iy * sizeY + offsetY, 0);
          const bottomRight = pointFromPlaneCoordinates(basePlane, (ix + 1) * sizeX + offsetX, iy * sizeY + offsetY, 0);
          const topRight = pointFromPlaneCoordinates(basePlane, (ix + 1) * sizeX + offsetX, (iy + 1) * sizeY + offsetY, 0);
          const topLeft = pointFromPlaneCoordinates(basePlane, ix * sizeX + offsetX, (iy + 1) * sizeY + offsetY, 0);
          cells.push([bottomLeft, bottomRight, topRight, topLeft, bottomLeft.clone()]);
          centers.push(pointFromPlaneCoordinates(basePlane, (ix + 0.5) * sizeX + offsetX, (iy + 0.5) * sizeY + offsetY, 0));
        }
      }
    }
    return { gridPoints, cells, centers };
  }

  function createGridPointTree(points, xCount, yCount) {
    if (!Array.isArray(points) || !points.length) {
      return [];
    }
    const rows = [];
    for (let iy = 0; iy < yCount; iy += 1) {
      const row = [];
      for (let ix = 0; ix < xCount; ix += 1) {
        const index = ix * yCount + iy;
        row.push(points[index].clone());
      }
      rows.push(row);
    }
    return rows;
  }

  function buildHexGridByExtents(basePlane, size, countX, countY) {
    const rows = [];
    const cells = [];
    const localRows = [];
    let minX = Number.POSITIVE_INFINITY;
    let minY = Number.POSITIVE_INFINITY;
    let maxX = Number.NEGATIVE_INFINITY;
    let maxY = Number.NEGATIVE_INFINITY;
    const stepX = Math.sqrt(3) * size;
    const stepY = 1.5 * size;
    for (let row = 0; row < countY; row += 1) {
      const localRow = [];
      const rowOffset = (row % 2) * (stepX / 2);
      for (let col = 0; col < countX; col += 1) {
        const x = col * stepX + rowOffset;
        const y = row * stepY;
        localRow.push({ x, y });
        if (x < minX) minX = x;
        if (x > maxX) maxX = x;
        if (y < minY) minY = y;
        if (y > maxY) maxY = y;
      }
      localRows.push(localRow);
    }
    if (!Number.isFinite(minX) || !Number.isFinite(minY) || !Number.isFinite(maxX) || !Number.isFinite(maxY)) {
      return { points: [], cells: [] };
    }
    const offsetX = (minX + maxX) / 2;
    const offsetY = (minY + maxY) / 2;
    for (let row = 0; row < countY; row += 1) {
      const pointRow = [];
      for (let col = 0; col < countX; col += 1) {
        const { x, y } = localRows[row][col];
        const center = pointFromPlaneCoordinates(basePlane, x - offsetX, y - offsetY, 0);
        pointRow.push(center);
        const corners = [];
        for (let i = 0; i < 6; i += 1) {
          const angle = (Math.PI / 3) * i + Math.PI / 6;
          corners.push(pointFromPlaneCoordinates(
            basePlane,
            x - offsetX + size * Math.cos(angle),
            y - offsetY + size * Math.sin(angle),
            0,
          ));
        }
        corners.push(corners[0].clone());
        cells.push(corners);
      }
      rows.push(pointRow);
    }
    return { points: rows.map((row) => row.map((pt) => pt.clone())), cells };
  }

  function buildRadialGrid(basePlane, radiusStep, radialCount, polarCount) {
    const rings = [];
    const cells = [];
    const normalizedPolar = Math.max(3, Math.round(polarCount));
    const angleStep = (Math.PI * 2) / normalizedPolar;
    rings.push([pointFromPlaneCoordinates(basePlane, 0, 0, 0)]);
    for (let ring = 1; ring <= radialCount; ring += 1) {
      const radius = radiusStep * ring;
      const ringPoints = [];
      for (let segment = 0; segment < normalizedPolar; segment += 1) {
        const angle = segment * angleStep;
        ringPoints.push(pointFromPlaneCoordinates(basePlane, radius * Math.cos(angle), radius * Math.sin(angle), 0));
      }
      rings.push(ringPoints);
    }
    for (let ring = 0; ring < radialCount; ring += 1) {
      const innerRadius = radiusStep * ring;
      const outerRadius = radiusStep * (ring + 1);
      for (let segment = 0; segment < normalizedPolar; segment += 1) {
        const angleA = segment * angleStep;
        const angleB = (segment + 1) * angleStep;
        const corners = [
          pointFromPlaneCoordinates(basePlane, innerRadius * Math.cos(angleA), innerRadius * Math.sin(angleA), 0),
          pointFromPlaneCoordinates(basePlane, outerRadius * Math.cos(angleA), outerRadius * Math.sin(angleA), 0),
          pointFromPlaneCoordinates(basePlane, outerRadius * Math.cos(angleB), outerRadius * Math.sin(angleB), 0),
          pointFromPlaneCoordinates(basePlane, innerRadius * Math.cos(angleB), innerRadius * Math.sin(angleB), 0),
        ];
        corners.push(corners[0].clone());
        cells.push(corners);
      }
    }
    return { rings: rings.map((ring) => ring.map((pt) => pt.clone())), cells };
  }

  function buildTriangularGrid(basePlane, edgeLength, countX, countY) {
    const height = edgeLength * Math.sqrt(3) / 2;
    const localRows = [];
    let minX = Number.POSITIVE_INFINITY;
    let minY = Number.POSITIVE_INFINITY;
    let maxX = Number.NEGATIVE_INFINITY;
    let maxY = Number.NEGATIVE_INFINITY;
    for (let row = 0; row <= countY; row += 1) {
      const localRow = [];
      for (let col = 0; col <= countX; col += 1) {
        const x = (col + (row / 2)) * edgeLength;
        const y = row * height;
        localRow.push({ x, y });
        if (x < minX) minX = x;
        if (x > maxX) maxX = x;
        if (y < minY) minY = y;
        if (y > maxY) maxY = y;
      }
      localRows.push(localRow);
    }
    if (!Number.isFinite(minX) || !Number.isFinite(maxX) || !Number.isFinite(minY) || !Number.isFinite(maxY)) {
      return { points: [], cells: [] };
    }
    const offsetX = (minX + maxX) / 2;
    const offsetY = (minY + maxY) / 2;
    const rows = localRows.map((row) => row.map(({ x, y }) => pointFromPlaneCoordinates(basePlane, x - offsetX, y - offsetY, 0)));
    const cells = [];
    for (let row = 0; row < countY; row += 1) {
      for (let col = 0; col < countX; col += 1) {
        const p00 = rows[row][col];
        const p10 = rows[row][col + 1];
        const p01 = rows[row + 1][col];
        const p11 = rows[row + 1][col + 1];
        if ((row + col) % 2 === 0) {
          cells.push([p00.clone(), p10.clone(), p11.clone(), p00.clone()]);
          cells.push([p00.clone(), p11.clone(), p01.clone(), p00.clone()]);
        } else {
          cells.push([p00.clone(), p10.clone(), p01.clone(), p00.clone()]);
          cells.push([p10.clone(), p11.clone(), p01.clone(), p10.clone()]);
        }
      }
    }
    return { points: rows.map((row) => row.map((pt) => pt.clone())), cells };
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

  function zeroSymmetricMatrix() {
    return {
      xx: 0,
      xy: 0,
      xz: 0,
      yy: 0,
      yz: 0,
      zz: 0,
    };
  }

  function accumulateSymmetricMatrix(target, matrix) {
    if (!target || !matrix) {
      return;
    }
    target.xx += Number.isFinite(matrix.xx) ? matrix.xx : 0;
    target.xy += Number.isFinite(matrix.xy) ? matrix.xy : 0;
    target.xz += Number.isFinite(matrix.xz) ? matrix.xz : 0;
    target.yy += Number.isFinite(matrix.yy) ? matrix.yy : 0;
    target.yz += Number.isFinite(matrix.yz) ? matrix.yz : 0;
    target.zz += Number.isFinite(matrix.zz) ? matrix.zz : 0;
  }

  function createTensorContribution(direction, weight = 1) {
    if (!direction) {
      return zeroSymmetricMatrix();
    }
    const vector = direction.clone ? direction.clone() : ensureVector(direction, new THREE.Vector3());
    if (!vector || !vector.isVector3) {
      return zeroSymmetricMatrix();
    }
    const length = vector.length();
    if (!(length > EPSILON)) {
      return zeroSymmetricMatrix();
    }
    const normalized = vector.clone().divideScalar(length);
    const magnitude = Math.max(0, Number.isFinite(weight) ? weight : 0);
    return {
      xx: normalized.x * normalized.x * magnitude,
      xy: normalized.x * normalized.y * magnitude,
      xz: normalized.x * normalized.z * magnitude,
      yy: normalized.y * normalized.y * magnitude,
      yz: normalized.y * normalized.z * magnitude,
      zz: normalized.z * normalized.z * magnitude,
    };
  }

  function finalizeTensorMatrix(matrix) {
    const sanitized = {
      xx: Number.isFinite(matrix.xx) ? matrix.xx : 0,
      xy: Number.isFinite(matrix.xy) ? matrix.xy : 0,
      xz: Number.isFinite(matrix.xz) ? matrix.xz : 0,
      yy: Number.isFinite(matrix.yy) ? matrix.yy : 0,
      yz: Number.isFinite(matrix.yz) ? matrix.yz : 0,
      zz: Number.isFinite(matrix.zz) ? matrix.zz : 0,
    };
    const nearZero = Math.abs(sanitized.xx) < EPSILON
      && Math.abs(sanitized.xy) < EPSILON
      && Math.abs(sanitized.xz) < EPSILON
      && Math.abs(sanitized.yy) < EPSILON
      && Math.abs(sanitized.yz) < EPSILON
      && Math.abs(sanitized.zz) < EPSILON;
    if (nearZero) {
      return { matrix: sanitized, principal: [], magnitude: 0 };
    }
    const { eigenValues, eigenVectors } = jacobiEigenDecomposition(sanitized);
    const indices = [0, 1, 2].sort((a, b) => Math.abs(eigenValues[b]) - Math.abs(eigenValues[a]));
    const principal = indices.map((index) => {
      const direction = new THREE.Vector3(
        eigenVectors[0][index],
        eigenVectors[1][index],
        eigenVectors[2][index],
      );
      if (direction.lengthSq() < EPSILON) {
        direction.set(0, 0, 0);
      } else {
        direction.normalize();
      }
      return {
        direction,
        magnitude: eigenValues[index],
      };
    });
    const magnitude = Math.sqrt(Math.max(0, sanitized.xx + sanitized.yy + sanitized.zz));
    return { matrix: sanitized, principal, magnitude };
  }

  function createFieldSource({ type = 'custom', bounds = null, meta = {}, evaluate }) {
    if (typeof evaluate !== 'function') {
      throw new Error('Field source requires an evaluate function.');
    }
    return {
      type,
      bounds: bounds ?? null,
      meta: meta ? { ...meta } : {},
      evaluate,
    };
  }

  function isField(value) {
    return Boolean(value && typeof value === 'object' && value.isField && Array.isArray(value.sources));
  }

  function createField(sources = [], metadata = {}) {
    const normalizedSources = [];
    sources.forEach((source) => {
      if (!source || typeof source.evaluate !== 'function') {
        return;
      }
      normalizedSources.push({
        type: source.type ?? 'custom',
        bounds: source.bounds ?? null,
        meta: source.meta ? { ...source.meta } : {},
        evaluate: source.evaluate,
      });
    });
    return {
      type: 'field',
      isField: true,
      sources: normalizedSources,
      bounds: metadata.bounds ?? null,
      meta: metadata.meta ? { ...metadata.meta } : {},
    };
  }

  function mergeFields(fieldInputs, metadata = {}) {
    const sources = [];
    const boundsCollection = [];
    (fieldInputs ?? []).forEach((entry) => {
      const field = ensureField(entry);
      if (!field) {
        return;
      }
      field.sources.forEach((source) => {
        if (!source || typeof source.evaluate !== 'function') {
          return;
        }
        sources.push({
          type: source.type ?? 'custom',
          bounds: source.bounds ?? null,
          meta: source.meta ? { ...source.meta } : {},
          evaluate: source.evaluate,
        });
      });
      if (field.bounds !== undefined && field.bounds !== null) {
        boundsCollection.push(field.bounds);
      }
    });
    let combinedBounds = metadata.bounds ?? null;
    if (combinedBounds === null && boundsCollection.length) {
      combinedBounds = boundsCollection.length === 1 ? boundsCollection[0] : boundsCollection;
    }
    return createField(sources, { ...metadata, bounds: combinedBounds });
  }

  function ensureField(input) {
    if (input === undefined || input === null) {
      return null;
    }
    if (isField(input)) {
      return input;
    }
    if (Array.isArray(input)) {
      const fields = input.map((entry) => ensureField(entry)).filter(Boolean);
      if (fields.length === 1) {
        return fields[0];
      }
      if (fields.length > 1) {
        return mergeFields(fields);
      }
      return null;
    }
    if (typeof input === 'object') {
      if (Object.prototype.hasOwnProperty.call(input, 'field')) {
        return ensureField(input.field);
      }
      if (Object.prototype.hasOwnProperty.call(input, 'value')) {
        return ensureField(input.value);
      }
      if (Array.isArray(input.sources)) {
        return createField(input.sources, { bounds: input.bounds, meta: input.meta });
      }
    }
    return null;
  }

  function collectFields(input) {
    if (input === undefined || input === null) {
      return [];
    }
    if (Array.isArray(input)) {
      const fields = [];
      input.forEach((entry) => {
        fields.push(...collectFields(entry));
      });
      return fields;
    }
    const field = ensureField(input);
    return field ? [field] : [];
  }

  function evaluateField(fieldInput, pointInput) {
    const field = ensureField(fieldInput);
    const point = ensurePoint(pointInput, new THREE.Vector3());
    const zeroResult = {
      point: point.clone(),
      vector: new THREE.Vector3(),
      magnitude: 0,
      strength: 0,
      direction: new THREE.Vector3(),
      tensor: { matrix: zeroSymmetricMatrix(), principal: [], magnitude: 0 },
      contributions: [],
    };
    if (!field || !field.sources.length) {
      return zeroResult;
    }
    const totalVector = new THREE.Vector3();
    let totalStrength = 0;
    const matrix = zeroSymmetricMatrix();
    const contributions = [];
    field.sources.forEach((source) => {
      if (!source || typeof source.evaluate !== 'function') {
        return;
      }
      const result = source.evaluate(point.clone(), { field, source });
      if (!result) {
        return;
      }
      let vector = null;
      if (result.vector?.isVector3) {
        vector = result.vector.clone();
      } else if (result.vector) {
        vector = ensureVector(result.vector, null);
      }
      if (!vector) {
        vector = new THREE.Vector3();
      }
      const strength = Number.isFinite(result.strength) ? result.strength : vector.length();
      totalVector.add(vector);
      totalStrength += Math.abs(strength);
      if (result.tensor) {
        accumulateSymmetricMatrix(matrix, result.tensor);
      } else if (vector.lengthSq() > EPSILON) {
        accumulateSymmetricMatrix(matrix, createTensorContribution(vector.clone(), Math.abs(strength)));
      }
      contributions.push({
        sourceType: source.type ?? 'custom',
        vector: vector.clone(),
        strength: Math.abs(strength),
      });
    });
    const magnitude = totalVector.length();
    const direction = magnitude > EPSILON ? totalVector.clone().divideScalar(magnitude) : new THREE.Vector3();
    const tensor = finalizeTensorMatrix(matrix);
    return {
      point: point.clone(),
      vector: totalVector,
      magnitude,
      strength: totalStrength,
      direction,
      tensor,
      contributions,
    };
  }

  function parseSampleCounts(input, fallback = { x: 10, y: 10 }) {
    if (input === undefined || input === null) {
      return { x: fallback.x, y: fallback.y };
    }
    if (Array.isArray(input)) {
      if (input.length >= 2) {
        const x = ensureNumber(input[0], Number.NaN);
        const y = ensureNumber(input[1], Number.NaN);
        return {
          x: Math.max(1, Math.round(Number.isFinite(x) ? x : fallback.x)),
          y: Math.max(1, Math.round(Number.isFinite(y) ? y : fallback.y)),
        };
      }
      if (input.length === 1) {
        const value = ensureNumber(input[0], fallback.x);
        const count = Math.max(1, Math.round(Number.isFinite(value) ? value : fallback.x));
        return { x: count, y: count };
      }
    }
    if (typeof input === 'object') {
      if (Object.prototype.hasOwnProperty.call(input, 'samples')) {
        return parseSampleCounts(input.samples, fallback);
      }
      if (Object.prototype.hasOwnProperty.call(input, 'value')) {
        return parseSampleCounts(input.value, fallback);
      }
      const x = ensureNumber(
        input.x ?? input.X ?? input.u ?? input.U ?? input.columns ?? input.width ?? input.count ?? input.n ?? input.N,
        Number.NaN,
      );
      const y = ensureNumber(
        input.y ?? input.Y ?? input.v ?? input.V ?? input.rows ?? input.height ?? input.count ?? input.n ?? input.N,
        Number.NaN,
      );
      if (Number.isFinite(x) && Number.isFinite(y)) {
        return { x: Math.max(1, Math.round(x)), y: Math.max(1, Math.round(y)) };
      }
      if (Number.isFinite(x)) {
        const count = Math.max(1, Math.round(x));
        return { x: count, y: count };
      }
      if (Number.isFinite(y)) {
        const count = Math.max(1, Math.round(y));
        return { x: count, y: count };
      }
    }
    const value = ensureNumber(input, Number.NaN);
    const count = Number.isFinite(value) ? Math.max(1, Math.round(value)) : fallback.x;
    return { x: count, y: count };
  }

  function extractDomain(domainInput, fallbackMin, fallbackMax) {
    if (domainInput === undefined || domainInput === null) {
      return { min: fallbackMin, max: fallbackMax };
    }
    if (Array.isArray(domainInput)) {
      if (domainInput.length >= 2) {
        const min = ensureNumber(domainInput[0], Number.NaN);
        const max = ensureNumber(domainInput[1], Number.NaN);
        if (Number.isFinite(min) && Number.isFinite(max)) {
          return { min, max };
        }
      }
      if (domainInput.length === 1) {
        return extractDomain(domainInput[0], fallbackMin, fallbackMax);
      }
    }
    if (typeof domainInput === 'object') {
      const min = ensureNumber(
        domainInput.min ?? domainInput.start ?? domainInput.from ?? domainInput.a ?? domainInput.A ?? domainInput[0],
        Number.NaN,
      );
      const max = ensureNumber(
        domainInput.max ?? domainInput.end ?? domainInput.to ?? domainInput.b ?? domainInput.B ?? domainInput[1],
        Number.NaN,
      );
      if (Number.isFinite(min) && Number.isFinite(max)) {
        return { min, max };
      }
      if (Number.isFinite(min)) {
        return { min, max: fallbackMax };
      }
      if (Number.isFinite(max)) {
        return { min: fallbackMin, max };
      }
      const length = ensureNumber(domainInput.length ?? domainInput.span ?? domainInput.size, Number.NaN);
      if (Number.isFinite(length)) {
        const half = Math.abs(length) / 2;
        return { min: -half, max: half };
      }
    }
    const numeric = ensureNumber(domainInput, Number.NaN);
    if (Number.isFinite(numeric)) {
      const half = Math.abs(numeric) / 2;
      return { min: -half, max: half };
    }
    return { min: fallbackMin, max: fallbackMax };
  }

  function extractRectangleSection(rectangleInput) {
    if (!rectangleInput) {
      const plane = defaultPlane();
      return {
        plane: clonePlaneData(plane),
        minX: -0.5,
        maxX: 0.5,
        minY: -0.5,
        maxY: 0.5,
      };
    }
    if (Array.isArray(rectangleInput) && rectangleInput.length === 1) {
      return extractRectangleSection(rectangleInput[0]);
    }
    if (rectangleInput.rectangle) {
      return extractRectangleSection(rectangleInput.rectangle);
    }
    let corners = [];
    if (Array.isArray(rectangleInput)) {
      corners = rectangleInput;
    } else if (Array.isArray(rectangleInput.corners)) {
      corners = rectangleInput.corners;
    } else if (Array.isArray(rectangleInput.points)) {
      corners = rectangleInput.points;
    }
    const parsedCorners = corners
      .map((corner) => ensurePoint(corner, null))
      .filter((corner) => corner && corner.isVector3);
    let plane;
    if (parsedCorners.length >= 3) {
      plane = planeFromPoints(parsedCorners[0], parsedCorners[1], parsedCorners[2]);
    } else if (rectangleInput.plane) {
      plane = ensurePlane(rectangleInput.plane);
    } else if (hasPlaneProperties(rectangleInput)) {
      plane = ensurePlane(rectangleInput);
    } else {
      plane = defaultPlane();
    }
    let minX = -0.5;
    let maxX = 0.5;
    let minY = -0.5;
    let maxY = 0.5;
    if (parsedCorners.length >= 3) {
      minX = Number.POSITIVE_INFINITY;
      maxX = Number.NEGATIVE_INFINITY;
      minY = Number.POSITIVE_INFINITY;
      maxY = Number.NEGATIVE_INFINITY;
      parsedCorners.forEach((corner) => {
        const coord = planeCoordinates(corner, plane);
        if (coord.x < minX) minX = coord.x;
        if (coord.x > maxX) maxX = coord.x;
        if (coord.y < minY) minY = coord.y;
        if (coord.y > maxY) maxY = coord.y;
      });
      if (!Number.isFinite(minX) || !Number.isFinite(maxX) || !Number.isFinite(minY) || !Number.isFinite(maxY)) {
        minX = -0.5;
        maxX = 0.5;
        minY = -0.5;
        maxY = 0.5;
      }
    } else {
      const width = ensureNumber(
        rectangleInput.width
          ?? rectangleInput.xSize
          ?? rectangleInput.sizeX
          ?? rectangleInput.widthX
          ?? rectangleInput.X
          ?? rectangleInput.x,
        Number.NaN,
      );
      const height = ensureNumber(
        rectangleInput.height
          ?? rectangleInput.ySize
          ?? rectangleInput.sizeY
          ?? rectangleInput.heightY
          ?? rectangleInput.Y
          ?? rectangleInput.y,
        Number.NaN,
      );
      const domainX = extractDomain(
        rectangleInput.domainX
          ?? rectangleInput.xDomain
          ?? rectangleInput.XDomain
          ?? rectangleInput.intervalX
          ?? rectangleInput.xInterval
          ?? rectangleInput.XInterval,
        -0.5,
        0.5,
      );
      const domainY = extractDomain(
        rectangleInput.domainY
          ?? rectangleInput.yDomain
          ?? rectangleInput.YDomain
          ?? rectangleInput.intervalY
          ?? rectangleInput.yInterval
          ?? rectangleInput.YInterval,
        -0.5,
        0.5,
      );
      if (Number.isFinite(width)) {
        const half = Math.abs(width) / 2;
        minX = -half;
        maxX = half;
      } else {
        minX = domainX.min;
        maxX = domainX.max;
      }
      if (Number.isFinite(height)) {
        const half = Math.abs(height) / 2;
        minY = -half;
        maxY = half;
      } else {
        minY = domainY.min;
        maxY = domainY.max;
      }
    }
    const sectionPlane = clonePlaneData(plane);
    return {
      plane: sectionPlane,
      minX,
      maxX,
      minY,
      maxY,
    };
  }

  function extractBoxRegion(regionInput) {
    const fallback = {
      plane: clonePlaneData(defaultPlane()),
      min: new THREE.Vector3(-0.5, -0.5, -0.5),
      max: new THREE.Vector3(0.5, 0.5, 0.5),
    };
    if (regionInput === undefined || regionInput === null) {
      return fallback;
    }
    if (Array.isArray(regionInput)) {
      if (regionInput.length === 1) {
        return extractBoxRegion(regionInput[0]);
      }
      const points = collectPoints(regionInput);
      const box = computeBoundingBoxFromPoints(points);
      if (box) {
        return { plane: clonePlaneData(defaultPlane()), min: box.min.clone(), max: box.max.clone() };
      }
    }
    if (regionInput.isBox3) {
      return { plane: clonePlaneData(defaultPlane()), min: regionInput.min.clone(), max: regionInput.max.clone() };
    }
    if (regionInput.box) {
      return extractBoxRegion(regionInput.box);
    }
    if (regionInput.region) {
      return extractBoxRegion(regionInput.region);
    }
    if (regionInput.bounds) {
      return extractBoxRegion(regionInput.bounds);
    }
    const plane = hasPlaneProperties(regionInput.plane)
      ? ensurePlane(regionInput.plane)
      : hasPlaneProperties(regionInput)
        ? ensurePlane(regionInput)
        : defaultPlane();
    const toLocalVector = (value) => {
      if (!value && value !== 0) {
        return null;
      }
      if (value?.isVector3) {
        const coord = planeCoordinates(value, plane);
        return new THREE.Vector3(coord.x, coord.y, coord.z);
      }
      if (typeof value === 'object') {
        if (
          Object.prototype.hasOwnProperty.call(value, 'x')
          || Object.prototype.hasOwnProperty.call(value, 'y')
          || Object.prototype.hasOwnProperty.call(value, 'z')
        ) {
          const x = ensureNumber(value.x ?? value.X ?? value[0], Number.NaN);
          const y = ensureNumber(value.y ?? value.Y ?? value[1], Number.NaN);
          const z = ensureNumber(value.z ?? value.Z ?? value[2], Number.NaN);
          if (Number.isFinite(x) && Number.isFinite(y) && Number.isFinite(z)) {
            return new THREE.Vector3(x, y, z);
          }
        }
      }
      const point = ensurePoint(value, null);
      if (point) {
        const coord = planeCoordinates(point, plane);
        return new THREE.Vector3(coord.x, coord.y, coord.z);
      }
      return null;
    };
    let localMin = toLocalVector(regionInput.localMin ?? regionInput.min ?? null);
    let localMax = toLocalVector(regionInput.localMax ?? regionInput.max ?? null);
    if (!localMin || !localMax) {
      const centerLocal = toLocalVector(regionInput.center ?? null) ?? new THREE.Vector3();
      const sizeVector = toLocalVector(regionInput.size ?? regionInput.dimensions ?? null);
      if (sizeVector) {
        localMin = centerLocal.clone().sub(sizeVector.clone().multiplyScalar(0.5));
        localMax = centerLocal.clone().add(sizeVector.clone().multiplyScalar(0.5));
      }
    }
    if (!localMin || !localMax) {
      const points = collectPoints(regionInput.points ?? regionInput.corners ?? regionInput.vertices ?? regionInput.locations);
      const box = computeBoundingBoxFromPoints(points);
      if (box) {
        const minCoord = planeCoordinates(box.min, plane);
        const maxCoord = planeCoordinates(box.max, plane);
        localMin = new THREE.Vector3(
          Math.min(minCoord.x, maxCoord.x),
          Math.min(minCoord.y, maxCoord.y),
          Math.min(minCoord.z, maxCoord.z),
        );
        localMax = new THREE.Vector3(
          Math.max(minCoord.x, maxCoord.x),
          Math.max(minCoord.y, maxCoord.y),
          Math.max(minCoord.z, maxCoord.z),
        );
      }
    }
    if (!localMin || !localMax) {
      return fallback;
    }
    const min = new THREE.Vector3(
      Math.min(localMin.x, localMax.x),
      Math.min(localMin.y, localMax.y),
      Math.min(localMin.z, localMax.z),
    );
    const max = new THREE.Vector3(
      Math.max(localMin.x, localMax.x),
      Math.max(localMin.y, localMax.y),
      Math.max(localMin.z, localMax.z),
    );
    return { plane: clonePlaneData(plane), min, max };
  }

  function randomPointInBoxRegion(region, rng = Math.random) {
    const u = rng();
    const v = rng();
    const w = rng();
    const x = THREE.MathUtils.lerp(region.min.x, region.max.x, u);
    const y = THREE.MathUtils.lerp(region.min.y, region.max.y, v);
    const z = THREE.MathUtils.lerp(region.min.z, region.max.z, w);
    return pointFromPlaneCoordinates(region.plane, x, y, z);
  }

  function createDiscreteSampler(points) {
    const entries = points.map((pt) => pt.clone());
    return (rng) => {
      if (!entries.length) {
        return new THREE.Vector3();
      }
      const scaled = rng() * entries.length;
      const index = Math.max(0, Math.min(entries.length - 1, Math.floor(scaled)));
      return entries[index].clone();
    };
  }

  function gatherGeometrySamplers(geometryInput) {
    const stack = ensureArray(geometryInput);
    const visited = new Set();
    const samplers = [];
    const collectedPoints = [];
    while (stack.length) {
      const current = stack.pop();
      if (current === undefined || current === null) {
        continue;
      }
      if (current?.isVector3) {
        collectedPoints.push(current.clone());
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
      if (typeof current === 'number') {
        collectedPoints.push(new THREE.Vector3(current, 0, 0));
        continue;
      }
      if (typeof current === 'object') {
        if (current.geometry && current.geometry !== current) stack.push(current.geometry);
        if (current.surface && current.surface !== current) stack.push(current.surface);
        if (current.curve && current.curve !== current) stack.push(current.curve);
        if (current.mesh && current.mesh !== current) stack.push(current.mesh);
        if (current.value && current.value !== current) stack.push(current.value);
        if (current.boundary && current.boundary !== current) stack.push(current.boundary);
        if (current.edges && current.edges !== current) stack.push(current.edges);
        if (current.faces && current.faces !== current) stack.push(current.faces);
        if (current.objects && current.objects !== current) stack.push(current.objects);
        if (current.children && current.children !== current) stack.push(current.children);

        if (typeof current.getPointAt === 'function') {
          const domain = current.domain ?? current.parameterDomain ?? createDomain(0, 1);
          samplers.push((rng) => {
            const t = THREE.MathUtils.lerp(domain.min ?? 0, domain.max ?? 1, rng());
            const denominator = (domain.max ?? 1) - (domain.min ?? 0);
            const normalized = denominator !== 0 ? (t - (domain.min ?? 0)) / denominator : 0;
            const point = current.getPointAt(normalized);
            return ensurePoint(point, new THREE.Vector3());
          });
        } else if (typeof current.getPoint === 'function') {
          samplers.push((rng) => {
            const target = new THREE.Vector3();
            current.getPoint(rng(), target);
            return target.clone();
          });
        } else if (typeof current.evaluate === 'function') {
          const domainU = current.domainU ?? createDomain(0, 1);
          const domainV = current.domainV ?? createDomain(0, 1);
          samplers.push((rng) => {
            const u = THREE.MathUtils.lerp(domainU.min ?? 0, domainU.max ?? 1, rng());
            const v = THREE.MathUtils.lerp(domainV.min ?? 0, domainV.max ?? 1, rng());
            const point = current.evaluate(u, v);
            return ensurePoint(point, new THREE.Vector3());
          });
        }

        if (current.points || current.corners || current.vertices || current.positions) {
          const dataPoints = collectPoints(current.points ?? current.corners ?? current.vertices ?? current.positions);
          if (dataPoints.length) {
            const clones = dataPoints.map((pt) => pt.clone());
            collectedPoints.push(...clones);
            samplers.push(createDiscreteSampler(clones));
          }
        }
        if (current.center?.isVector3) {
          collectedPoints.push(current.center.clone());
        }
        if (current.origin?.isVector3) {
          collectedPoints.push(current.origin.clone());
        }
      }
    }
    const boundingBox = computeBoundingBoxFromPoints(collectedPoints);
    if (!samplers.length && boundingBox) {
      samplers.push((rng) => randomPointInAxisAlignedBox(boundingBox, rng));
    }
    if (!samplers.length && collectedPoints.length) {
      samplers.push(createDiscreteSampler(collectedPoints));
    }
    return { samplers, fallbackPoints: collectedPoints, boundingBox };
  }

  function clamp01(value) {
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
  }

  function createDomain(min = 0, max = 1) {
    return { min, max, span: max - min };
  }

  function computePolylineLength(points) {
    let length = 0;
    for (let i = 1; i < points.length; i += 1) {
      length += points[i].distanceTo(points[i - 1]);
    }
    return length;
  }

  function closestPointOnSegment(point, start, end) {
    const segment = end.clone().sub(start);
    const lengthSq = segment.lengthSq();
    if (lengthSq < EPSILON) {
      return start.clone();
    }
    const t = clamp01(point.clone().sub(start).dot(segment) / lengthSq);
    return start.clone().add(segment.multiplyScalar(t));
  }

  function createPolylineCurve(pointsInput) {
    const points = pointsInput.map((pt) => pt.clone());
    if (!points.length) {
      return {
        type: 'polyline',
        points: [],
        segments: 0,
        length: 0,
        closed: false,
        domain: createDomain(0, 1),
        getPointAt: () => new THREE.Vector3(),
        getTangentAt: () => new THREE.Vector3(),
      };
    }
    if (points.length === 1) {
      const onlyPoint = points[0].clone();
      return {
        type: 'polyline',
        points: [onlyPoint],
        segments: 0,
        length: 0,
        closed: false,
        domain: createDomain(0, 1),
        getPointAt: () => onlyPoint.clone(),
        getTangentAt: () => new THREE.Vector3(),
      };
    }
    const segments = points.length - 1;
    const length = computePolylineLength(points);
    const curve = {
      type: 'polyline',
      points,
      segments,
      length,
      closed: false,
      domain: createDomain(0, 1),
    };
    curve.getPointAt = (t) => {
      const clamped = clamp01(t);
      if (segments === 0) {
        return points[0].clone();
      }
      const scaled = clamped * segments;
      const index = Math.min(Math.floor(scaled), segments - 1);
      const localT = scaled - index;
      const start = points[index];
      const end = points[index + 1];
      return start.clone().lerp(end, localT);
    };
    curve.getTangentAt = (t) => {
      if (segments === 0) {
        return new THREE.Vector3();
      }
      const clamped = clamp01(t);
      if (clamped <= EPSILON) {
        return points[1].clone().sub(points[0]).normalize();
      }
      if (clamped >= 1 - EPSILON) {
        return points[segments].clone().sub(points[segments - 1]).normalize();
      }
      const delta = 1 / Math.max(segments * 4, 32);
      const p0 = curve.getPointAt(Math.max(0, clamped - delta));
      const p1 = curve.getPointAt(Math.min(1, clamped + delta));
      const tangent = p1.clone().sub(p0);
      if (tangent.lengthSq() < EPSILON) {
        const index = Math.min(Math.floor(clamped * segments), segments - 1);
        return points[index + 1].clone().sub(points[index]).normalize();
      }
      return tangent.normalize();
    };
    return curve;
  }

  function sampleFieldSection(fieldInput, sectionInput, samplesInput, iterator) {
    const section = extractRectangleSection(sectionInput);
    const sampleCounts = parseSampleCounts(samplesInput);
    const result = {
      section,
      samples: { x: Math.max(1, sampleCounts.x), y: Math.max(1, sampleCounts.y) },
    };
    const field = ensureField(fieldInput);
    if (!field || !field.sources.length || typeof iterator !== 'function') {
      return result;
    }
    const xCount = result.samples.x;
    const yCount = result.samples.y;
    for (let ix = 0; ix < xCount; ix += 1) {
      const u = xCount === 1 ? 0.5 : ix / (xCount - 1);
      const x = THREE.MathUtils.lerp(section.minX, section.maxX, u);
      for (let iy = 0; iy < yCount; iy += 1) {
        const v = yCount === 1 ? 0.5 : iy / (yCount - 1);
        const y = THREE.MathUtils.lerp(section.minY, section.maxY, v);
        const point = pointFromPlaneCoordinates(section.plane, x, y, 0);
        const evaluation = evaluateField(field, point);
        iterator({
          point,
          evaluation,
          index: { x: ix, y: iy },
          uv: { u, v },
          planeCoordinates: { x, y },
          section,
        });
      }
    }
    return result;
  }

  function createFieldDisplayPayload({ field, sectionInput, samplesInput, mode, mapper }) {
    const entries = [];
    const sampled = sampleFieldSection(field, sectionInput, samplesInput, (payload) => {
      if (typeof mapper !== 'function') {
        return;
      }
      const entry = mapper({
        point: payload.point.clone(),
        evaluation: payload.evaluation,
        index: payload.index,
        uv: payload.uv,
        planeCoordinates: payload.planeCoordinates,
        section: payload.section,
      });
      if (entry !== undefined && entry !== null) {
        entries.push(entry);
      }
    });
    const corners = [
      pointFromPlaneCoordinates(sampled.section.plane, sampled.section.minX, sampled.section.minY, 0),
      pointFromPlaneCoordinates(sampled.section.plane, sampled.section.maxX, sampled.section.minY, 0),
      pointFromPlaneCoordinates(sampled.section.plane, sampled.section.maxX, sampled.section.maxY, 0),
      pointFromPlaneCoordinates(sampled.section.plane, sampled.section.minX, sampled.section.maxY, 0),
    ];
    return {
      type: 'field-display',
      mode,
      section: {
        plane: sampled.section.plane,
        minX: sampled.section.minX,
        maxX: sampled.section.maxX,
        minY: sampled.section.minY,
        maxY: sampled.section.maxY,
        width: sampled.section.maxX - sampled.section.minX,
        height: sampled.section.maxY - sampled.section.minY,
        corners,
      },
      samples: sampled.samples,
      entries,
    };
  }

  function computeFieldDirection(field, point) {
    const evaluation = evaluateField(field, point);
    if (!evaluation || !(evaluation.magnitude > EPSILON)) {
      return null;
    }
    const speed = THREE.MathUtils.clamp(evaluation.magnitude, 0.1, 5);
    const direction = evaluation.direction.clone();
    if (direction.lengthSq() < EPSILON) {
      return null;
    }
    return direction.multiplyScalar(speed);
  }

  function advanceFieldPoint(field, point, stepSize, order = 4) {
    const step = Math.max(stepSize, EPSILON);
    const k1 = computeFieldDirection(field, point);
    if (!k1) {
      return null;
    }
    if (order <= 1) {
      return point.clone().add(k1.clone().multiplyScalar(step));
    }
    const mid1 = point.clone().add(k1.clone().multiplyScalar(step / 2));
    const k2 = computeFieldDirection(field, mid1) ?? k1.clone();
    if (order === 2) {
      const direction = k1.clone().add(k2).multiplyScalar(0.5);
      if (direction.lengthSq() < EPSILON) {
        return null;
      }
      return point.clone().add(direction.multiplyScalar(step));
    }
    const mid2 = point.clone().add(k2.clone().multiplyScalar(step / 2));
    const k3 = computeFieldDirection(field, mid2) ?? k2.clone();
    if (order === 3) {
      const direction = k1.clone().add(k2).add(k3).multiplyScalar(1 / 3);
      if (direction.lengthSq() < EPSILON) {
        return null;
      }
      return point.clone().add(direction.multiplyScalar(step));
    }
    const endPoint = point.clone().add(k3.clone().multiplyScalar(step));
    const k4 = computeFieldDirection(field, endPoint) ?? k3.clone();
    const direction = k1.clone()
      .add(k2.clone().multiplyScalar(2))
      .add(k3.clone().multiplyScalar(2))
      .add(k4)
      .multiplyScalar(1 / 6);
    if (direction.lengthSq() < EPSILON) {
      return null;
    }
    return point.clone().add(direction.multiplyScalar(step));
  }

  function integrateFieldLine(fieldInput, startPointInput, { steps = 25, stepSize = 0.5, method = 4 } = {}) {
    const field = ensureField(fieldInput);
    const startPoint = ensurePoint(startPointInput, new THREE.Vector3());
    if (!field || !field.sources.length) {
      return [startPoint.clone()];
    }
    const result = [startPoint.clone()];
    let current = startPoint.clone();
    for (let i = 0; i < steps; i += 1) {
      const next = advanceFieldPoint(field, current, stepSize, method);
      if (!next) {
        break;
      }
      if (next.distanceToSquared(current) < EPSILON * EPSILON) {
        break;
      }
      current = next;
      result.push(current.clone());
    }
    return result;
  }

  function createSpinForceFieldSource(planeInput, strengthInput, radiusInput, decayInput, bounds) {
    const plane = ensurePlane(planeInput);
    const strength = ensureNumber(strengthInput, 1);
    const radius = Math.max(ensureNumber(radiusInput, 1), EPSILON);
    const decay = Math.max(ensureNumber(decayInput, 1), 0);
    return createFieldSource({
      type: 'spin-force',
      bounds,
      meta: { plane: clonePlaneData(plane), strength, radius, decay },
      evaluate(point) {
        const coords = planeCoordinates(point, plane);
        const radial = Math.hypot(coords.x, coords.y);
        const falloff = 1 / Math.pow(1 + radial / radius, decay + 1);
        const verticalFalloff = 1 / (1 + Math.abs(coords.z) / radius);
        const tangential = plane.xAxis.clone().multiplyScalar(-coords.y)
          .add(plane.yAxis.clone().multiplyScalar(coords.x));
        if (tangential.lengthSq() < EPSILON) {
          return { vector: new THREE.Vector3(), strength: 0, tensor: zeroSymmetricMatrix() };
        }
        tangential.normalize();
        const magnitude = strength * falloff * verticalFalloff;
        const vector = tangential.multiplyScalar(magnitude);
        const weight = Math.abs(magnitude);
        return { vector, strength: weight, tensor: createTensorContribution(vector.clone(), weight) };
      },
    });
  }

  function createPointChargeFieldSource(positionInput, chargeInput, decayInput, bounds) {
    const position = ensurePoint(positionInput, new THREE.Vector3());
    const charge = ensureNumber(chargeInput, 1);
    const decay = Math.max(ensureNumber(decayInput, 2), 0);
    return createFieldSource({
      type: 'point-charge',
      bounds,
      meta: { position: position.clone(), charge, decay },
      evaluate(point) {
        const offset = point.clone().sub(position);
        const distanceSq = offset.lengthSq();
        if (!(distanceSq > EPSILON)) {
          return { vector: new THREE.Vector3(), strength: Math.abs(charge), tensor: zeroSymmetricMatrix() };
        }
        const distance = Math.sqrt(distanceSq);
        const direction = offset.clone().divideScalar(distance);
        const magnitude = charge / Math.pow(distance + EPSILON, decay);
        const vector = direction.clone().multiplyScalar(magnitude);
        const weight = Math.abs(magnitude);
        return { vector, strength: weight, tensor: createTensorContribution(direction, weight) };
      },
    });
  }

  function createLineChargeFieldSource(lineInput, chargeInput, bounds) {
    const line = ensureLine(lineInput);
    const charge = ensureNumber(chargeInput, 1);
    const start = line.start.clone();
    const end = line.end.clone();
    const direction = end.clone().sub(start);
    const length = direction.length();
    if (!(length > EPSILON)) {
      return createPointChargeFieldSource(start, charge, 2, bounds);
    }
    const segments = Math.max(8, Math.round(length * 4));
    return createFieldSource({
      type: 'line-charge',
      bounds,
      meta: { start: start.clone(), end: end.clone(), charge },
      evaluate(point) {
        const totalVector = new THREE.Vector3();
        let totalStrength = 0;
        const tensorMatrix = zeroSymmetricMatrix();
        for (let i = 0; i < segments; i += 1) {
          const t = (i + 0.5) / segments;
          const samplePoint = start.clone().lerp(end, t);
          const offset = point.clone().sub(samplePoint);
          const distanceSq = offset.lengthSq();
          if (!(distanceSq > EPSILON)) {
            continue;
          }
          const distance = Math.sqrt(distanceSq);
          const directionVector = offset.clone().divideScalar(distance);
          const magnitude = (charge / segments) / (distanceSq + EPSILON);
          const contribution = directionVector.clone().multiplyScalar(magnitude);
          totalVector.add(contribution);
          const weight = Math.abs(magnitude);
          totalStrength += weight;
          accumulateSymmetricMatrix(tensorMatrix, createTensorContribution(directionVector, weight));
        }
        return { vector: totalVector, strength: totalStrength, tensor: tensorMatrix };
      },
    });
  }

  function createVectorForceFieldSource(lineInput, bounds) {
    const line = ensureLine(lineInput);
    const start = line.start.clone();
    const end = line.end.clone();
    const axis = end.clone().sub(start);
    let length = axis.length();
    if (!(length > EPSILON)) {
      axis.set(1, 0, 0);
      length = 1;
    }
    axis.normalize();
    return createFieldSource({
      type: 'vector-force',
      bounds,
      meta: { start: start.clone(), end: end.clone() },
      evaluate(point) {
        const closest = closestPointOnSegment(point, start, end);
        const offset = point.clone().sub(closest);
        const distanceSq = offset.lengthSq();
        const axial = axis.clone().multiplyScalar(1 / (1 + distanceSq));
        let radial = new THREE.Vector3();
        if (distanceSq > EPSILON) {
          radial = offset.clone().divideScalar(Math.sqrt(distanceSq)).multiplyScalar(0.5 / (1 + distanceSq));
        }
        const vector = axial.add(radial);
        const strength = vector.length();
        return { vector, strength, tensor: createTensorContribution(vector.clone(), strength) };
      },
    });
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

  function registerVectorComputationComponents() {
    const TWO_PI = Math.PI * 2;

    const clampToUnit = (value) => Math.max(-1, Math.min(1, value));

    function safeNormalized(vector) {
      const length = vector.length();
      if (length < EPSILON) {
        return { valid: false, vector: new THREE.Vector3(), length: 0 };
      }
      return { valid: true, vector: vector.clone().divideScalar(length), length };
    }

    function sumVectors(vectors, unitize) {
      const sum = new THREE.Vector3();
      vectors.forEach((vector) => {
        if (!vector) {
          return;
        }
        if (unitize) {
          const { valid, vector: normalized } = safeNormalized(vector.clone());
          if (valid) {
            sum.add(normalized);
          }
        } else {
          sum.add(vector.clone());
        }
      });
      return sum;
    }

    function computeAngle3D(a, b) {
      const lengthA = a.length();
      const lengthB = b.length();
      if (lengthA < EPSILON || lengthB < EPSILON) {
        return { angle: 0, reflex: 0 };
      }
      const normalizedDot = clampToUnit(a.dot(b) / (lengthA * lengthB));
      const angle = Math.acos(normalizedDot);
      return { angle, reflex: TWO_PI - angle };
    }

    function computeAngleOnPlane(a, b, plane) {
      const projectedA = {
        x: a.dot(plane.xAxis),
        y: a.dot(plane.yAxis),
      };
      const projectedB = {
        x: b.dot(plane.xAxis),
        y: b.dot(plane.yAxis),
      };
      const magA = Math.hypot(projectedA.x, projectedA.y);
      const magB = Math.hypot(projectedB.x, projectedB.y);
      if (magA < EPSILON || magB < EPSILON) {
        return computeAngle3D(a, b);
      }
      const angleA = Math.atan2(projectedA.y, projectedA.x);
      const angleB = Math.atan2(projectedB.y, projectedB.x);
      let delta = angleB - angleA;
      while (delta < 0) delta += TWO_PI;
      while (delta >= TWO_PI) delta -= TWO_PI;
      const reflex = delta <= EPSILON ? 0 : TWO_PI - delta;
      return { angle: delta, reflex };
    }

    function ensureVectorList(value) {
      return collectPoints(value ?? []);
    }

    function divideVector(vector, scalar) {
      if (Math.abs(scalar) < EPSILON) {
        return new THREE.Vector3();
      }
      return vector.clone().divideScalar(scalar);
    }

    function multiplyVector(vector, scalar) {
      return vector.clone().multiplyScalar(scalar);
    }

    function rotateVector(vector, axis, angle) {
      const axisClone = axis.clone();
      if (axisClone.lengthSq() < EPSILON) {
        return vector.clone();
      }
      const quaternion = new THREE.Quaternion();
      quaternion.setFromAxisAngle(axisClone.normalize(), angle);
      return vector.clone().applyQuaternion(quaternion);
    }

    function parseGeoLocation(value) {
      if (value === undefined || value === null) {
        return { latitude: 0, longitude: 0 };
      }
      if (Array.isArray(value)) {
        if (value.length >= 2) {
          return {
            longitude: ensureNumber(value[0], 0),
            latitude: ensureNumber(value[1], 0),
          };
        }
        if (value.length === 1) {
          return { latitude: ensureNumber(value[0], 0), longitude: 0 };
        }
      }
      if (value?.isVector3) {
        return {
          longitude: ensureNumber(value.x, 0),
          latitude: ensureNumber(value.y, 0),
        };
      }
      if (typeof value === 'object') {
        const longitude = ensureNumber(
          value.longitude ?? value.lon ?? value.lng ?? value.Longitude ?? value.Long ?? value.x ?? value.X,
          0,
        );
        const latitude = ensureNumber(value.latitude ?? value.lat ?? value.Latitude ?? value.Lat ?? value.y ?? value.Y, 0);
        return { latitude, longitude };
      }
      const numeric = ensureNumber(value, Number.NaN);
      if (Number.isFinite(numeric)) {
        return { latitude: numeric, longitude: 0 };
      }
      return { latitude: 0, longitude: 0 };
    }

    function ensureDateValue(value) {
      if (value instanceof Date && !Number.isNaN(value.getTime())) {
        return new Date(value.getTime());
      }
      if (typeof value === 'number') {
        const date = new Date(value);
        if (!Number.isNaN(date.getTime())) {
          return date;
        }
      }
      if (typeof value === 'string') {
        const date = new Date(value);
        if (!Number.isNaN(date.getTime())) {
          return date;
        }
      }
      if (typeof value === 'object' && value !== null) {
        const year = ensureNumber(value.year ?? value.Year, Number.NaN);
        const month = ensureNumber(value.month ?? value.Month, Number.NaN);
        const day = ensureNumber(value.day ?? value.Day, Number.NaN);
        if (Number.isFinite(year) && Number.isFinite(month) && Number.isFinite(day)) {
          const hour = ensureNumber(value.hour ?? value.Hour, 0);
          const minute = ensureNumber(value.minute ?? value.Minute, 0);
          const second = ensureNumber(value.second ?? value.Second, 0);
          const constructed = new Date(Date.UTC(year, Math.max(0, Math.round(month) - 1), Math.round(day), Math.round(hour), Math.round(minute), Math.round(second)));
          if (!Number.isNaN(constructed.getTime())) {
            return constructed;
          }
        }
      }
      return new Date();
    }

    function computeSolarData(date, location, plane) {
      const latRad = THREE.MathUtils.degToRad(location.latitude);
      const longitudeDeg = location.longitude;
      const timezoneHours = -date.getTimezoneOffset() / 60;

      const startOfYear = Date.UTC(date.getUTCFullYear(), 0, 1);
      const currentDay = Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), date.getUTCDate());
      const dayOfYear = Math.floor((currentDay - startOfYear) / 86400000) + 1;
      const minutes = date.getHours() * 60 + date.getMinutes() + date.getSeconds() / 60;
      const gamma = (2 * Math.PI / 365) * (dayOfYear - 1 + (minutes / 60 - 12) / 24);

      const equationOfTime = 229.18 * (
        0.000075
        + 0.001868 * Math.cos(gamma)
        - 0.032077 * Math.sin(gamma)
        - 0.014615 * Math.cos(2 * gamma)
        - 0.040849 * Math.sin(2 * gamma)
      );

      const declination = (
        0.006918
        - 0.399912 * Math.cos(gamma)
        + 0.070257 * Math.sin(gamma)
        - 0.006758 * Math.cos(2 * gamma)
        + 0.000907 * Math.sin(2 * gamma)
        - 0.002697 * Math.cos(3 * gamma)
        + 0.00148 * Math.sin(3 * gamma)
      );

      const timeOffset = equationOfTime + 4 * longitudeDeg - 60 * timezoneHours;
      let trueSolarTime = minutes + timeOffset;
      trueSolarTime = ((trueSolarTime % 1440) + 1440) % 1440;
      let hourAngleDeg = trueSolarTime / 4 - 180;
      if (hourAngleDeg < -180) {
        hourAngleDeg += 360;
      }
      const hourAngle = THREE.MathUtils.degToRad(hourAngleDeg);

      const cosZenith = clampToUnit(
        Math.sin(latRad) * Math.sin(declination)
        + Math.cos(latRad) * Math.cos(declination) * Math.cos(hourAngle),
      );
      const zenith = Math.acos(cosZenith);
      const elevation = Math.PI / 2 - zenith;

      let azimuth = 0;
      const sinZenith = Math.sin(zenith);
      if (sinZenith >= EPSILON) {
        const azimuthCos = clampToUnit(
          (Math.sin(latRad) * Math.cos(zenith) - Math.sin(declination))
          / (Math.cos(latRad) * sinZenith),
        );
        azimuth = Math.acos(azimuthCos);
        if (trueSolarTime > 720) {
          azimuth = TWO_PI - azimuth;
        }
      }

      const east = Math.sin(azimuth) * Math.cos(elevation);
      const north = Math.cos(azimuth) * Math.cos(elevation);
      const up = Math.sin(elevation);

      const direction = plane.xAxis.clone().multiplyScalar(east)
        .add(plane.yAxis.clone().multiplyScalar(north))
        .add(plane.zAxis.clone().multiplyScalar(up))
        .normalize();

      return { direction, elevation, horizon: elevation > 0 };
    }

    function colorForElevation(elevation) {
      const color = new THREE.Color();
      if (!(elevation > 0)) {
        color.setRGB(0.08, 0.09, 0.15);
        return color;
      }
      const normalized = Math.min(1, Math.max(0, elevation / (Math.PI / 3)));
      const hue = Math.min(1, Math.max(0, 0.12 - 0.05 * normalized));
      const saturation = Math.min(1, 0.75 + 0.15 * (1 - normalized));
      const lightness = Math.min(1, 0.35 + 0.25 * normalized);
      color.setHSL(hue, saturation, lightness);
      return color;
    }

    register(['{152a264e-fc74-40e5-88cc-d1a681cd09c3}', 'vector angle', 'angle'], {
      type: 'vector',
      pinMap: {
        inputs: { A: 'a', a: 'a', B: 'b', b: 'b' },
        outputs: { A: 'angle', Angle: 'angle', angle: 'angle', R: 'reflex', Reflex: 'reflex', reflex: 'reflex' },
      },
      eval: ({ inputs }) => {
        const a = ensureVector(inputs.a ?? inputs.A, new THREE.Vector3(1, 0, 0));
        const b = ensureVector(inputs.b ?? inputs.B, new THREE.Vector3(0, 1, 0));
        return computeAngle3D(a, b);
      },
    });

    register(['{b464fccb-50e7-41bd-9789-8438db9bea9f}', 'vector angle plane', 'angle plane'], {
      type: 'vector',
      pinMap: {
        inputs: {
          A: 'a', a: 'a',
          B: 'b', b: 'b',
          P: 'plane', Plane: 'plane', plane: 'plane',
        },
        outputs: { A: 'angle', Angle: 'angle', angle: 'angle', R: 'reflex', Reflex: 'reflex', reflex: 'reflex' },
      },
      eval: ({ inputs }) => {
        const a = ensureVector(inputs.a ?? inputs.A, new THREE.Vector3(1, 0, 0));
        const b = ensureVector(inputs.b ?? inputs.B, new THREE.Vector3(0, 1, 0));
        const plane = ensurePlane(inputs.plane ?? inputs.P, defaultPlane());
        return computeAngleOnPlane(a, b, plane);
      },
    });

    register(['{2a5cfb31-028a-4b34-b4e1-9b20ae15312e}', 'cross product', 'xprod'], {
      type: 'vector',
      pinMap: {
        inputs: { A: 'a', a: 'a', B: 'b', b: 'b', U: 'unitize', Unitize: 'unitize', unitize: 'unitize' },
        outputs: { V: 'vector', Vector: 'vector', vector: 'vector', L: 'length', Length: 'length', length: 'length' },
      },
      eval: ({ inputs }) => {
        const a = ensureVector(inputs.a ?? inputs.A, new THREE.Vector3());
        const b = ensureVector(inputs.b ?? inputs.B, new THREE.Vector3());
        const unitize = ensureBoolean(inputs.unitize ?? inputs.U, false);
        const cross = a.clone().cross(b);
        const length = cross.length();
        if (unitize) {
          if (length > EPSILON) {
            cross.divideScalar(length);
          } else {
            cross.set(0, 0, 0);
          }
        }
        return { vector: cross, length };
      },
    });

    register(['{310e1065-d03a-4858-bcd1-809d39c042af}', 'vector divide', 'vdiv'], {
      type: 'vector',
      pinMap: {
        inputs: { V: 'vector', vector: 'vector', Vector: 'vector', F: 'factor', f: 'factor', factor: 'factor', Factor: 'factor' },
        outputs: { V: 'vector', vector: 'vector', Vector: 'vector', L: 'length', Length: 'length', length: 'length' },
      },
      eval: ({ inputs }) => {
        const vector = ensureVector(inputs.vector ?? inputs.V, new THREE.Vector3());
        const factor = ensureNumber(inputs.factor ?? inputs.F, 1);
        const result = divideVector(vector, factor);
        return { vector: result, length: result.length() };
      },
    });

    register(['{56b92eab-d121-43f7-94d3-6cd8f0ddead8}', 'vector xyz', 'vec'], {
      type: 'vector',
      pinMap: {
        inputs: { X: 'x', x: 'x', Y: 'y', y: 'y', Z: 'z', z: 'z' },
        outputs: { V: 'vector', Vector: 'vector', vector: 'vector', L: 'length', Length: 'length', length: 'length' },
      },
      eval: ({ inputs }) => {
        const x = ensureNumber(inputs.x ?? inputs.X, 0);
        const y = ensureNumber(inputs.y ?? inputs.Y, 0);
        const z = ensureNumber(inputs.z ?? inputs.Z, 0);
        const vector = new THREE.Vector3(x, y, z);
        return { vector, length: vector.length() };
      },
    });

    register(['{675e31bf-1775-48d7-bb8d-76b77786dd53}', 'vector length', 'vlen'], {
      type: 'vector',
      pinMap: {
        inputs: { V: 'vector', vector: 'vector', Vector: 'vector' },
        outputs: { L: 'length', Length: 'length', length: 'length' },
      },
      eval: ({ inputs }) => {
        const vector = ensureVector(inputs.vector ?? inputs.V, new THREE.Vector3());
        return { length: vector.length() };
      },
    });

    register(['{6ec39468-dae7-4ffa-a766-f2ab22a2c62e}', 'vector amplitude', 'amplitude'], {
      type: 'vector',
      pinMap: {
        inputs: { V: 'vector', vector: 'vector', Vector: 'vector', A: 'amplitude', a: 'amplitude', amplitude: 'amplitude', Amplitude: 'amplitude' },
        outputs: { V: 'vector', vector: 'vector', Vector: 'vector' },
      },
      eval: ({ inputs }) => {
        const vector = ensureVector(inputs.vector ?? inputs.V, new THREE.Vector3());
        const amplitude = ensureNumber(inputs.amplitude ?? inputs.A, vector.length());
        const { valid, vector: normalized } = safeNormalized(vector);
        if (!valid) {
          return { vector: new THREE.Vector3() };
        }
        return { vector: normalized.multiplyScalar(amplitude) };
      },
    });

    register(['{79f9fbb3-8f1d-4d9a-88a9-f7961b1012cd}', 'unit x', 'unit vector x'], {
      type: 'vector',
      pinMap: {
        inputs: { F: 'factor', factor: 'factor', Factor: 'factor', f: 'factor' },
        outputs: { V: 'vector', vector: 'vector', Vector: 'vector' },
      },
      eval: ({ inputs }) => {
        const factor = ensureNumber(inputs.factor ?? inputs.F, 1);
        return { vector: new THREE.Vector3(1, 0, 0).multiplyScalar(factor) };
      },
    });

    register(['{d3d195ea-2d59-4ffa-90b1-8b7ff3369f69}', 'unit y', 'unit vector y'], {
      type: 'vector',
      pinMap: {
        inputs: { F: 'factor', factor: 'factor', Factor: 'factor', f: 'factor' },
        outputs: { V: 'vector', vector: 'vector', Vector: 'vector' },
      },
      eval: ({ inputs }) => {
        const factor = ensureNumber(inputs.factor ?? inputs.F, 1);
        return { vector: new THREE.Vector3(0, 1, 0).multiplyScalar(factor) };
      },
    });

    register(['{9103c240-a6a9-4223-9b42-dbd19bf38e2b}', 'unit z', 'unit vector z'], {
      type: 'vector',
      pinMap: {
        inputs: { F: 'factor', factor: 'factor', Factor: 'factor', f: 'factor' },
        outputs: { V: 'vector', vector: 'vector', Vector: 'vector' },
      },
      eval: ({ inputs }) => {
        const factor = ensureNumber(inputs.factor ?? inputs.F, 1);
        return { vector: new THREE.Vector3(0, 0, 1).multiplyScalar(factor) };
      },
    });

    register(['{63fff845-7c61-4dfb-ba12-44d481b4bf0f}', 'vector multiply', 'vmul'], {
      type: 'vector',
      pinMap: {
        inputs: { V: 'vector', vector: 'vector', Vector: 'vector', F: 'factor', factor: 'factor', Factor: 'factor', f: 'factor' },
        outputs: { V: 'vector', vector: 'vector', Vector: 'vector', L: 'length', Length: 'length', length: 'length' },
      },
      eval: ({ inputs }) => {
        const vector = ensureVector(inputs.vector ?? inputs.V, new THREE.Vector3());
        const factor = ensureNumber(inputs.factor ?? inputs.F, 1);
        const result = multiplyVector(vector, factor);
        return { vector: result, length: result.length() };
      },
    });

    register(['{934ede4a-924a-4973-bb05-0dc4b36fae75}', 'vector 2pt', 'vec2pt'], {
      type: 'vector',
      pinMap: {
        inputs: { A: 'a', a: 'a', B: 'b', b: 'b', U: 'unitize', Unitize: 'unitize', unitize: 'unitize' },
        outputs: { V: 'vector', vector: 'vector', Vector: 'vector', L: 'length', Length: 'length', length: 'length' },
      },
      eval: ({ inputs }) => {
        const a = ensurePoint(inputs.a ?? inputs.A, new THREE.Vector3());
        const b = ensurePoint(inputs.b ?? inputs.B, new THREE.Vector3());
        const unitize = ensureBoolean(inputs.unitize ?? inputs.U, false);
        const vector = b.clone().sub(a);
        const length = vector.length();
        if (unitize) {
          if (length > EPSILON) {
            vector.divideScalar(length);
          } else {
            vector.set(0, 0, 0);
          }
        }
        return { vector, length };
      },
    });

    register(['{63f79e72-36c0-4489-a0c2-9ded0b9ca41f}', 'vector mass addition', 'mass addition', 'mass add'], {
      type: 'vector',
      pinMap: {
        inputs: { V: 'vectors', vectors: 'vectors', Vectors: 'vectors', U: 'unitize', Unitize: 'unitize', unitize: 'unitize' },
        outputs: { V: 'vector', vector: 'vector', Vector: 'vector', L: 'length', Length: 'length', length: 'length' },
      },
      eval: ({ inputs }) => {
        const vectors = ensureVectorList(inputs.vectors ?? inputs.V);
        const unitize = ensureBoolean(inputs.unitize ?? inputs.U, false);
        const vector = sumVectors(vectors, unitize);
        const length = vector.length();
        return { vector, length };
      },
    });

    register(['{b7f1178f-4222-47fd-9766-5d06e869362b}', 'mass addition total'], {
      type: 'vector',
      pinMap: {
        inputs: { V: 'vectors', vectors: 'vectors', Vectors: 'vectors', U: 'unitize', Unitize: 'unitize', unitize: 'unitize' },
        outputs: { V: 'vector', vector: 'vector', Vector: 'vector' },
      },
      eval: ({ inputs }) => {
        const vectors = ensureVectorList(inputs.vectors ?? inputs.V);
        const unitize = ensureBoolean(inputs.unitize ?? inputs.U, false);
        const vector = sumVectors(vectors, unitize);
        return { vector };
      },
    });

    register(['{d2da1306-259a-4994-85a4-672d8a4c7805}', 'unit vector', 'unitize vector', 'unit'], {
      type: 'vector',
      pinMap: {
        inputs: { V: 'vector', vector: 'vector', Vector: 'vector' },
        outputs: { V: 'vector', vector: 'vector', Vector: 'vector' },
      },
      eval: ({ inputs }) => {
        const vector = ensureVector(inputs.vector ?? inputs.V, new THREE.Vector3());
        const { valid, vector: normalized } = safeNormalized(vector);
        if (!valid) {
          return { vector: new THREE.Vector3() };
        }
        return { vector: normalized };
      },
    });

    register(['{d5788074-d75d-4021-b1a3-0bf992928584}', 'vector reverse', 'reverse'], {
      type: 'vector',
      pinMap: {
        inputs: { V: 'vector', vector: 'vector', Vector: 'vector' },
        outputs: { V: 'vector', vector: 'vector', Vector: 'vector' },
      },
      eval: ({ inputs }) => {
        const vector = ensureVector(inputs.vector ?? inputs.V, new THREE.Vector3());
        return { vector: vector.multiplyScalar(-1) };
      },
    });

    register(['{fb012ef9-4734-4049-84a0-b92b85bb09da}', 'vector addition', 'vadd'], {
      type: 'vector',
      pinMap: {
        inputs: { A: 'a', a: 'a', B: 'b', b: 'b', U: 'unitize', Unitize: 'unitize', unitize: 'unitize' },
        outputs: { V: 'vector', vector: 'vector', Vector: 'vector', L: 'length', Length: 'length', length: 'length' },
      },
      eval: ({ inputs }) => {
        const a = ensureVector(inputs.a ?? inputs.A, new THREE.Vector3());
        const b = ensureVector(inputs.b ?? inputs.B, new THREE.Vector3());
        const unitize = ensureBoolean(inputs.unitize ?? inputs.U, false);
        const vector = a.clone().add(b);
        const length = vector.length();
        if (unitize) {
          if (length > EPSILON) {
            vector.divideScalar(length);
          } else {
            vector.set(0, 0, 0);
          }
        }
        return { vector, length };
      },
    });

    register(['{43b9ea8f-f772-40f2-9880-011a9c3cbbb0}', 'dot product', 'dprod'], {
      type: 'vector',
      pinMap: {
        inputs: { A: 'a', a: 'a', B: 'b', b: 'b', U: 'unitize', Unitize: 'unitize', unitize: 'unitize' },
        outputs: { D: 'dot', dot: 'dot', Dot: 'dot' },
      },
      eval: ({ inputs }) => {
        const a = ensureVector(inputs.a ?? inputs.A, new THREE.Vector3());
        const b = ensureVector(inputs.b ?? inputs.B, new THREE.Vector3());
        const unitize = ensureBoolean(inputs.unitize ?? inputs.U, false);
        const vectorA = a.clone();
        const vectorB = b.clone();
        if (unitize) {
          const normalizedA = safeNormalized(vectorA);
          const normalizedB = safeNormalized(vectorB);
          if (!normalizedA.valid || !normalizedB.valid) {
            return { dot: 0 };
          }
          return { dot: normalizedA.vector.dot(normalizedB.vector) };
        }
        return { dot: vectorA.dot(vectorB) };
      },
    });

    register(['{a50fcd4a-cf42-4c3f-8616-022761e6cc93}', 'deconstruct vector', 'devec'], {
      type: 'vector',
      pinMap: {
        inputs: { V: 'vector', vector: 'vector', Vector: 'vector' },
        outputs: { X: 'x', x: 'x', Y: 'y', y: 'y', Z: 'z', z: 'z' },
      },
      eval: ({ inputs }) => {
        const vector = ensureVector(inputs.vector ?? inputs.V, new THREE.Vector3());
        return { x: vector.x, y: vector.y, z: vector.z };
      },
    });

    register(['{b6d7ba20-cf74-4191-a756-2216a36e30a7}', 'vector rotate', 'vrot'], {
      type: 'vector',
      pinMap: {
        inputs: {
          V: 'vector', vector: 'vector', Vector: 'vector',
          X: 'axis', axis: 'axis', Axis: 'axis',
          A: 'angle', angle: 'angle', Angle: 'angle',
        },
        outputs: { V: 'vector', vector: 'vector', Vector: 'vector' },
      },
      eval: ({ inputs }) => {
        const vector = ensureVector(inputs.vector ?? inputs.V, new THREE.Vector3());
        const axis = ensureVector(inputs.axis ?? inputs.X, new THREE.Vector3(0, 0, 1));
        const angle = ensureNumber(inputs.angle ?? inputs.A, 0);
        return { vector: rotateVector(vector, axis, angle) };
      },
    });

    register(['{59e1f848-38d4-4cbf-ad7f-40ffc52acdf5}', 'solar incidence', 'solar'], {
      type: 'vector',
      pinMap: {
        inputs: {
          L: 'location', location: 'location', Location: 'location',
          T: 'time', time: 'time', Time: 'time',
          P: 'plane', plane: 'plane', Plane: 'plane',
          Orientation: 'plane', orientation: 'plane',
        },
        outputs: {
          D: 'direction', direction: 'direction', Direction: 'direction',
          E: 'elevation', elevation: 'elevation', Elevation: 'elevation',
          H: 'horizon', horizon: 'horizon', Horizon: 'horizon',
          C: 'colour', colour: 'colour', Colour: 'colour', color: 'colour', Color: 'colour',
        },
      },
      eval: ({ inputs }) => {
        const location = parseGeoLocation(inputs.location ?? inputs.L);
        const plane = ensurePlane(inputs.plane ?? inputs.orientation ?? inputs.P, defaultPlane());
        const date = ensureDateValue(inputs.time ?? inputs.T);
        const { direction, elevation, horizon } = computeSolarData(date, location, plane);
        const colour = colorForElevation(elevation);
        return { direction, elevation, horizon, colour };
      },
    });
  }

  function registerColourComponents() {
    register([
      '{035bf8a7-b9e0-4e37-b031-4567bc60d047}',
      'colour multiplication',
      'color multiplication',
      'mul',
    ], {
      type: 'colour',
      pinMap: {
        inputs: {
          A: 'colourA',
          a: 'colourA',
          'Colour A': 'colourA',
          'Color A': 'colourA',
          colourA: 'colourA',
          colorA: 'colourA',
          B: 'colourB',
          b: 'colourB',
          'Colour B': 'colourB',
          'Color B': 'colourB',
          colourB: 'colourB',
          colorB: 'colourB',
        },
        outputs: {
          C: 'colour',
          c: 'colour',
          colour: 'colour',
          Colour: 'colour',
          color: 'colour',
          Color: 'colour',
        },
      },
      eval: ({ inputs }) => {
        const colourA = ensureColor(
          inputs.colourA ?? inputs.colorA ?? inputs.A ?? inputs['Colour A'] ?? inputs['Color A'],
          new THREE.Color(1, 1, 1),
        );
        const colourB = ensureColor(
          inputs.colourB ?? inputs.colorB ?? inputs.B ?? inputs['Colour B'] ?? inputs['Color B'],
          new THREE.Color(1, 1, 1),
        );
        const colour = clampColor(colourA.clone().multiply(colourB));
        return { colour };
      },
    });

    register([
      '{0c80d9c0-d8b3-4817-b8e1-6214d443704b}',
      'colour subtraction',
      'color subtraction',
      'sub',
    ], {
      type: 'colour',
      pinMap: {
        inputs: {
          A: 'colourA',
          a: 'colourA',
          'Colour A': 'colourA',
          'Color A': 'colourA',
          colourA: 'colourA',
          colorA: 'colourA',
          B: 'colourB',
          b: 'colourB',
          'Colour B': 'colourB',
          'Color B': 'colourB',
          colourB: 'colourB',
          colorB: 'colourB',
        },
        outputs: {
          C: 'colour',
          c: 'colour',
          colour: 'colour',
          Colour: 'colour',
          color: 'colour',
          Color: 'colour',
        },
      },
      eval: ({ inputs }) => {
        const colourA = ensureColor(
          inputs.colourA ?? inputs.colorA ?? inputs.A ?? inputs['Colour A'] ?? inputs['Color A'],
          new THREE.Color(),
        );
        const colourB = ensureColor(
          inputs.colourB ?? inputs.colorB ?? inputs.B ?? inputs['Colour B'] ?? inputs['Color B'],
          new THREE.Color(),
        );
        const colour = new THREE.Color(
          clamp01(colourA.r - colourB.r),
          clamp01(colourA.g - colourB.g),
          clamp01(colourA.b - colourB.b),
        );
        return { colour };
      },
    });

    register([
      '{8b4da37d-1124-436a-9de2-952e4224a220}',
      'blend colours',
      'blend colors',
      'blendcol',
    ], {
      type: 'colour',
      pinMap: {
        inputs: {
          A: 'colourA',
          a: 'colourA',
          'Colour A': 'colourA',
          'Color A': 'colourA',
          colourA: 'colourA',
          colorA: 'colourA',
          B: 'colourB',
          b: 'colourB',
          'Colour B': 'colourB',
          'Color B': 'colourB',
          colourB: 'colourB',
          colorB: 'colourB',
          F: 'factor',
          f: 'factor',
          factor: 'factor',
          Factor: 'factor',
        },
        outputs: {
          C: 'colour',
          c: 'colour',
          colour: 'colour',
          Colour: 'colour',
          color: 'colour',
          Color: 'colour',
        },
      },
      eval: ({ inputs }) => {
        const colourA = ensureColor(
          inputs.colourA ?? inputs.colorA ?? inputs.A ?? inputs['Colour A'] ?? inputs['Color A'],
          new THREE.Color(),
        );
        const colourB = ensureColor(
          inputs.colourB ?? inputs.colorB ?? inputs.B ?? inputs['Colour B'] ?? inputs['Color B'],
          new THREE.Color(1, 1, 1),
        );
        const factor = clamp01(ensureNumber(inputs.factor ?? inputs.F ?? inputs.f, 0.5));
        const colour = clampColor(colourA.clone().lerp(colourB, factor));
        return { colour };
      },
    });
  }

  function registerFieldComponents() {
    register(['{08619b6d-f9c4-4cb2-adcd-90959f08dc0d}', 'tensor display', 'ftensor'], {
      type: 'field',
      pinMap: {
        inputs: {
          F: 'field', field: 'field', Field: 'field',
          S: 'section', section: 'section', Section: 'section',
          N: 'samples', Samples: 'samples', samples: 'samples',
        },
        outputs: { D: 'display', Display: 'display', display: 'display' },
      },
      eval: ({ inputs }) => {
        const display = createFieldDisplayPayload({
          field: inputs.field ?? inputs.F,
          sectionInput: inputs.section ?? inputs.S,
          samplesInput: inputs.samples ?? inputs.N,
          mode: 'tensor',
          mapper: ({ point, evaluation, index, uv, planeCoordinates }) => ({
            point,
            index,
            uv,
            planeCoordinates,
            magnitude: evaluation.magnitude,
            strength: evaluation.strength,
            direction: evaluation.direction.clone(),
            principal: evaluation.tensor.principal.map((axis) => ({
              direction: axis.direction.clone(),
              magnitude: axis.magnitude,
            })),
            matrix: { ...evaluation.tensor.matrix },
          }),
        });
        return { display };
      },
    });

    register(['{55f9ce6a-490c-4f25-a536-a3d47b794752}', 'scalar display', 'fscalar'], {
      type: 'field',
      pinMap: {
        inputs: {
          F: 'field', field: 'field', Field: 'field',
          S: 'section', section: 'section', Section: 'section',
          N: 'samples', Samples: 'samples', samples: 'samples',
        },
        outputs: { D: 'display', Display: 'display', display: 'display' },
      },
      eval: ({ inputs }) => {
        const display = createFieldDisplayPayload({
          field: inputs.field ?? inputs.F,
          sectionInput: inputs.section ?? inputs.S,
          samplesInput: inputs.samples ?? inputs.N,
          mode: 'scalar',
          mapper: ({ point, evaluation, index, uv, planeCoordinates }) => ({
            point,
            index,
            uv,
            planeCoordinates,
            magnitude: evaluation.magnitude,
            strength: evaluation.strength,
            direction: evaluation.direction.clone(),
          }),
        });
        return { display };
      },
    });

    register(['{5ba20fab-6d71-48ea-a98f-cb034db6bbdc}', 'direction display', 'fdir'], {
      type: 'field',
      pinMap: {
        inputs: {
          F: 'field', field: 'field', Field: 'field',
          S: 'section', section: 'section', Section: 'section',
          N: 'samples', Samples: 'samples', samples: 'samples',
        },
        outputs: { D: 'display', Display: 'display', display: 'display' },
      },
      eval: ({ inputs }) => {
        const display = createFieldDisplayPayload({
          field: inputs.field ?? inputs.F,
          sectionInput: inputs.section ?? inputs.S,
          samplesInput: inputs.samples ?? inputs.N,
          mode: 'direction',
          mapper: ({ point, evaluation, index, uv, planeCoordinates }) => ({
            point,
            index,
            uv,
            planeCoordinates,
            magnitude: evaluation.magnitude,
            strength: evaluation.strength,
            direction: evaluation.direction.clone(),
          }),
        });
        return { display };
      },
    });

    register(['{bf106e4c-68f4-476f-b05b-9c15fb50e078}', 'perpendicular display', 'fperp'], {
      type: 'field',
      pinMap: {
        inputs: {
          F: 'field', field: 'field', Field: 'field',
          S: 'section', section: 'section', Section: 'section',
          N: 'samples', Samples: 'samples', samples: 'samples',
          'C+': 'positiveColour', 'Positive Colour': 'positiveColour', positiveColour: 'positiveColour',
          'C-': 'negativeColour', 'Negative Colour': 'negativeColour', negativeColour: 'negativeColour',
        },
        outputs: { D: 'display', Display: 'display', display: 'display' },
      },
      eval: ({ inputs }) => {
        const fallbackPositive = new THREE.Color(0.95, 0.45, 0.35);
        const fallbackNegative = new THREE.Color(0.35, 0.55, 0.95);
        const basePositive = parseColor(inputs.positiveColour ?? inputs['C+'], fallbackPositive) ?? fallbackPositive.clone();
        const baseNegative = parseColor(inputs.negativeColour ?? inputs['C-'], fallbackNegative) ?? fallbackNegative.clone();
        const display = createFieldDisplayPayload({
          field: inputs.field ?? inputs.F,
          sectionInput: inputs.section ?? inputs.S,
          samplesInput: inputs.samples ?? inputs.N,
          mode: 'perpendicular',
          mapper: ({ point, evaluation, index, uv, planeCoordinates, section }) => {
            const alignment = evaluation.direction.dot(section.plane.zAxis);
            const clamped = THREE.MathUtils.clamp(alignment, -1, 1);
            const factor = (clamped + 1) / 2;
            const color = baseNegative.clone().lerp(basePositive, factor);
            return {
              point,
              index,
              uv,
              planeCoordinates,
              alignment: clamped,
              magnitude: evaluation.magnitude,
              strength: evaluation.strength,
              direction: evaluation.direction.clone(),
              color,
            };
          },
        });
        return { display };
      },
    });

    register(['{4b59e893-d4ee-4e31-ae24-a489611d1088}', 'spin force', 'fspin'], {
      type: 'field',
      pinMap: {
        inputs: {
          P: 'plane', Plane: 'plane', plane: 'plane',
          S: 'strength', Strength: 'strength', strength: 'strength',
          R: 'radius', Radius: 'radius', radius: 'radius',
          D: 'decay', Decay: 'decay', decay: 'decay',
          B: 'bounds', Bounds: 'bounds', bounds: 'bounds',
        },
        outputs: { F: 'field', Field: 'field', field: 'field' },
      },
      eval: ({ inputs }) => {
        const bounds = inputs.bounds ?? inputs.B ?? null;
        const field = createField([
          createSpinForceFieldSource(
            inputs.plane ?? inputs.P,
            inputs.strength ?? inputs.S,
            inputs.radius ?? inputs.R,
            inputs.decay ?? inputs.D,
            bounds,
          ),
        ], { bounds });
        return { field };
      },
    });

    register(['{8cc9eb88-26a7-4baa-a896-13e5fc12416a}', 'line charge', 'lcharge'], {
      type: 'field',
      pinMap: {
        inputs: {
          L: 'line', Line: 'line', line: 'line',
          C: 'charge', Charge: 'charge', charge: 'charge',
          B: 'bounds', Bounds: 'bounds', bounds: 'bounds',
        },
        outputs: { F: 'field', Field: 'field', field: 'field' },
      },
      eval: ({ inputs }) => {
        const bounds = inputs.bounds ?? inputs.B ?? null;
        const field = createField([
          createLineChargeFieldSource(inputs.line ?? inputs.L ?? inputs.Line, inputs.charge ?? inputs.C, bounds),
        ], { bounds });
        return { field };
      },
    });

    register(['{cffdbaf3-8d33-4b38-9cad-c264af9fc3f4}', 'point charge', 'pcharge'], {
      type: 'field',
      pinMap: {
        inputs: {
          P: 'point', Point: 'point', point: 'point',
          C: 'charge', Charge: 'charge', charge: 'charge',
          D: 'decay', Decay: 'decay', decay: 'decay',
          B: 'bounds', Bounds: 'bounds', bounds: 'bounds',
        },
        outputs: { F: 'field', Field: 'field', field: 'field' },
      },
      eval: ({ inputs }) => {
        const bounds = inputs.bounds ?? inputs.B ?? null;
        const field = createField([
          createPointChargeFieldSource(
            inputs.point ?? inputs.P,
            inputs.charge ?? inputs.C,
            inputs.decay ?? inputs.D,
            bounds,
          ),
        ], { bounds });
        return { field };
      },
    });

    register(['{d27cc1ea-9ef7-47bf-8ee2-c6662da0e3d9}', 'vector force', 'fvector'], {
      type: 'field',
      pinMap: {
        inputs: {
          L: 'line', Line: 'line', line: 'line',
          B: 'bounds', Bounds: 'bounds', bounds: 'bounds',
        },
        outputs: { F: 'field', Field: 'field', field: 'field' },
      },
      eval: ({ inputs }) => {
        const bounds = inputs.bounds ?? inputs.B ?? null;
        const field = createField([
          createVectorForceFieldSource(inputs.line ?? inputs.L ?? inputs.Line, bounds),
        ], { bounds });
        return { field };
      },
    });

    register(['{a7c9f738-f8bd-4f64-8e7f-33341183e493}', 'evaluate field', 'evf'], {
      type: 'field',
      pinMap: {
        inputs: {
          F: 'field', field: 'field', Field: 'field',
          P: 'point', point: 'point', Point: 'point',
        },
        outputs: {
          T: 'tensor', Tensor: 'tensor', tensor: 'tensor',
          S: 'strength', Strength: 'strength', strength: 'strength',
        },
      },
      eval: ({ inputs }) => {
        const point = ensurePoint(inputs.point ?? inputs.P, new THREE.Vector3());
        const evaluation = evaluateField(inputs.field ?? inputs.F, point);
        const tensor = {
          point: evaluation.point.clone(),
          vector: evaluation.vector.clone(),
          direction: evaluation.direction.clone(),
          magnitude: evaluation.magnitude,
          strength: evaluation.strength,
          matrix: { ...evaluation.tensor.matrix },
          principal: evaluation.tensor.principal.map((axis) => ({
            direction: axis.direction.clone(),
            magnitude: axis.magnitude,
          })),
        };
        return { tensor, strength: evaluation.magnitude };
      },
    });

    register(['{add6be3e-c57f-4740-96e4-5680abaa9169}', 'field line', 'fline'], {
      type: 'field',
      pinMap: {
        inputs: {
          F: 'field', field: 'field', Field: 'field',
          P: 'point', point: 'point', Point: 'point',
          N: 'steps', Steps: 'steps', steps: 'steps',
          A: 'accuracy', Accuracy: 'accuracy', accuracy: 'accuracy',
          M: 'method', Method: 'method', method: 'method',
        },
        outputs: { C: 'curve', Curve: 'curve', curve: 'curve' },
      },
      eval: ({ inputs }) => {
        const startPoint = ensurePoint(inputs.point ?? inputs.P, new THREE.Vector3());
        const steps = resolveCount(inputs.steps ?? inputs.N, 25);
        const accuracy = Math.max(ensureNumber(inputs.accuracy ?? inputs.A, 0.5), EPSILON);
        const method = Math.max(1, Math.min(4, Math.round(ensureNumber(inputs.method ?? inputs.M, 4))));
        const points = integrateFieldLine(inputs.field ?? inputs.F, startPoint, {
          steps,
          stepSize: accuracy,
          method,
        });
        const curve = createPolylineCurve(points);
        return { curve };
      },
    });

    register(['{b27d53bc-e713-475d-81fd-71cdd8de2e58}', 'break field', 'breakf'], {
      type: 'field',
      pinMap: {
        inputs: { F: 'field', field: 'field', Field: 'field' },
        outputs: { F: 'fields', Fields: 'fields', fields: 'fields' },
      },
      eval: ({ inputs }) => {
        const field = ensureField(inputs.field ?? inputs.F);
        if (!field || !field.sources.length) {
          return { fields: [] };
        }
        const fields = field.sources.map((source) => createField([source], { bounds: source.bounds ?? null }));
        return { fields };
      },
    });

    register(['{d9a6fbd2-2e9f-472e-8147-33bf0233a115}', 'merge fields', 'mergef'], {
      type: 'field',
      pinMap: {
        inputs: { F: 'fields', Fields: 'fields', fields: 'fields' },
        outputs: { F: 'field', Field: 'field', field: 'field' },
      },
      eval: ({ inputs }) => {
        const fields = collectFields(inputs.fields ?? inputs.F);
        if (!fields.length) {
          return { field: createField() };
        }
        const field = mergeFields(fields);
        return { field };
      },
    });
  }

  function registerPlaneComponents() {
    register(['{17b7152b-d30d-4d50-b9ef-c9fe25576fc2}', 'xy plane', 'xy'], {
      type: 'plane',
      pinMap: {
        inputs: { O: 'origin', Origin: 'origin', origin: 'origin' },
        outputs: { P: 'plane', Plane: 'plane', plane: 'plane' },
      },
      eval: ({ inputs }) => {
        const origin = ensurePoint(inputs.origin, new THREE.Vector3());
        const plane = defaultPlane();
        plane.origin.copy(origin);
        return { plane };
      },
    });

    register(['{2318aee8-01fe-4ea8-9524-6966023fc622}', 'align planes'], {
      type: 'plane',
      pinMap: {
        inputs: {
          P: 'planes', Planes: 'planes', planes: 'planes',
          M: 'master', Master: 'master', master: 'master',
        },
        outputs: { P: 'planes', Planes: 'planes', planes: 'planes' },
      },
      eval: ({ inputs }) => {
        const planes = collectPlanes(inputs.planes);
        if (!planes.length) {
          return { planes: [] };
        }
        const master = inputs.master ? ensurePlane(inputs.master) : null;
        const result = [];
        let reference = master ? clonePlaneData(master) : clonePlaneData(planes[0]);
        planes.forEach((plane, index) => {
          if (index === 0 && !master) {
            const initial = clonePlaneData(plane);
            result.push(initial);
            reference = initial;
          } else {
            const aligned = alignPlaneToReference(reference, plane);
            result.push(aligned);
            reference = aligned;
          }
        });
        return { planes: result.map(clonePlaneData) };
      },
    });

    register(['{33bfc73c-19b2-480b-81e6-f3523a012ea6}', 'plane fit', 'plfit'], {
      type: 'plane',
      pinMap: {
        inputs: { P: 'points', Points: 'points', points: 'points' },
        outputs: {
          Pl: 'plane', Plane: 'plane', plane: 'plane',
          dx: 'deviation', Deviation: 'deviation', deviation: 'deviation',
        },
      },
      eval: ({ inputs }) => {
        const points = collectPoints(inputs.points);
        const { plane, deviation } = fitPlaneToPoints(points);
        return { plane, deviation };
      },
    });

    register(['{3a0c7bda-3d22-4588-8bab-03f57a52a6ea}', 'plane offset', 'pl offset'], {
      type: 'plane',
      pinMap: {
        inputs: { P: 'plane', Plane: 'plane', plane: 'plane', O: 'offset', Offset: 'offset', offset: 'offset' },
        outputs: { Pl: 'plane', Plane: 'plane', plane: 'plane' },
      },
      eval: ({ inputs }) => {
        const plane = ensurePlane(inputs.plane);
        const offset = ensureNumber(inputs.offset, 0);
        const result = clonePlaneData(plane);
        result.origin.add(result.zAxis.clone().multiplyScalar(offset));
        return { plane: result };
      },
    });

    register(['{3cd2949b-4ea8-4ffb-a70c-5c380f9f46ea}', 'deconstruct plane', 'deplane'], {
      type: 'plane',
      pinMap: {
        inputs: { P: 'plane', Plane: 'plane', plane: 'plane' },
        outputs: {
          O: 'origin', Origin: 'origin', origin: 'origin',
          X: 'xAxis', 'X-Axis': 'xAxis', xAxis: 'xAxis',
          Y: 'yAxis', 'Y-Axis': 'yAxis', yAxis: 'yAxis',
          Z: 'zAxis', 'Z-Axis': 'zAxis', zAxis: 'zAxis',
        },
      },
      eval: ({ inputs }) => {
        const plane = ensurePlane(inputs.plane);
        return {
          origin: plane.origin.clone(),
          xAxis: plane.xAxis.clone(),
          yAxis: plane.yAxis.clone(),
          zAxis: plane.zAxis.clone(),
        };
      },
    });

    register(['{5f127fa4-ca61-418e-bb2d-e3739d900f1f}', 'plane coordinates', 'plcoord'], {
      type: 'plane',
      pinMap: {
        inputs: {
          P: 'point', Point: 'point', point: 'point',
          S: 'system', System: 'system', system: 'system',
        },
        outputs: { X: 'x', x: 'x', Y: 'y', y: 'y', Z: 'z', z: 'z' },
      },
      eval: ({ inputs }) => {
        const point = ensurePoint(inputs.point, new THREE.Vector3());
        const plane = ensurePlane(inputs.system);
        const coords = planeCoordinates(point, plane);
        return { x: coords.x, y: coords.y, z: coords.z };
      },
    });

    register(['{75eec078-a905-47a1-b0d2-0934182b1e3d}', 'plane origin', 'pl origin'], {
      type: 'plane',
      pinMap: {
        inputs: {
          B: 'base', Base: 'base', base: 'base', P: 'base', Plane: 'base',
          O: 'origin', Origin: 'origin', origin: 'origin',
        },
        outputs: { Pl: 'plane', Plane: 'plane', plane: 'plane' },
      },
      eval: ({ inputs }) => {
        const plane = ensurePlane(inputs.base ?? inputs.P);
        const origin = ensurePoint(inputs.origin, plane.origin.clone());
        const result = clonePlaneData(plane);
        result.origin.copy(origin);
        return { plane: result };
      },
    });

    register(['{8cc3a196-f6a0-49ea-9ed9-0cb343a3ae64}', 'xz plane', 'xz'], {
      type: 'plane',
      pinMap: {
        inputs: { O: 'origin', Origin: 'origin', origin: 'origin' },
        outputs: { P: 'plane', Plane: 'plane', plane: 'plane' },
      },
      eval: ({ inputs }) => {
        const origin = ensurePoint(inputs.origin, new THREE.Vector3());
        const plane = normalizePlaneAxes(
          origin,
          new THREE.Vector3(1, 0, 0),
          new THREE.Vector3(0, 0, -1),
          new THREE.Vector3(0, 1, 0),
        );
        return { plane };
      },
    });

    register(['{9ce34996-d8c6-40d3-b442-1a7c8c093614}', 'adjust plane', 'padjust'], {
      type: 'plane',
      pinMap: {
        inputs: {
          P: 'plane', Plane: 'plane', plane: 'plane',
          N: 'normal', Normal: 'normal', normal: 'normal',
        },
        outputs: { P: 'plane', Plane: 'plane', plane: 'plane' },
      },
      eval: ({ inputs }) => {
        const plane = ensurePlane(inputs.plane);
        let normal = ensureVector(inputs.normal, plane.zAxis.clone());
        if (normal.lengthSq() < EPSILON) {
          normal = plane.zAxis.clone();
        } else {
          normal.normalize();
        }
        let xAxis = plane.xAxis.clone();
        xAxis.sub(normal.clone().multiplyScalar(xAxis.dot(normal)));
        if (xAxis.lengthSq() < EPSILON) {
          xAxis = plane.yAxis.clone();
          xAxis.sub(normal.clone().multiplyScalar(xAxis.dot(normal)));
        }
        if (xAxis.lengthSq() < EPSILON) {
          xAxis = orthogonalVector(normal);
        } else {
          xAxis.normalize();
        }
        const yAxis = normal.clone().cross(xAxis).normalize();
        const result = normalizePlaneAxes(plane.origin, xAxis, yAxis, normal.clone());
        return { plane: result };
      },
    });

    register(['{b075c065-efda-4c9f-9cc9-288362b1b4b9}', 'plane closest point', 'cp'], {
      type: 'plane',
      pinMap: {
        inputs: {
          S: 'point', Point: 'point', point: 'point',
          P: 'plane', Plane: 'plane', plane: 'plane',
        },
        outputs: {
          P: 'projected', Point: 'projected', projected: 'projected',
          uv: 'uv', 'UV Point': 'uv',
          D: 'distance', Distance: 'distance', distance: 'distance',
        },
      },
      eval: ({ inputs }) => {
        const point = ensurePoint(inputs.point, new THREE.Vector3());
        const plane = ensurePlane(inputs.plane);
        const coords = planeCoordinates(point, plane);
        const projected = pointFromPlaneCoordinates(plane, coords.x, coords.y, 0);
        const uv = new THREE.Vector2(coords.x, coords.y);
        return { projected, uv, distance: coords.z };
      },
    });

    register(['{bc3e379e-7206-4e7b-b63a-ff61f4b38a3e}', 'construct plane', 'pl'], {
      type: 'plane',
      pinMap: {
        inputs: {
          O: 'origin', Origin: 'origin', origin: 'origin',
          X: 'xAxis', 'X-Axis': 'xAxis', xAxis: 'xAxis',
          Y: 'yAxis', 'Y-Axis': 'yAxis', yAxis: 'yAxis',
        },
        outputs: { Pl: 'plane', Plane: 'plane', plane: 'plane' },
      },
      eval: ({ inputs }) => {
        const origin = ensurePoint(inputs.origin, new THREE.Vector3());
        let xAxis = ensureVector(inputs.xAxis, new THREE.Vector3(1, 0, 0));
        let yAxis = ensureVector(inputs.yAxis, new THREE.Vector3(0, 1, 0));
        if (xAxis.lengthSq() < EPSILON) {
          xAxis = new THREE.Vector3(1, 0, 0);
        }
        if (yAxis.lengthSq() < EPSILON) {
          yAxis = orthogonalVector(xAxis);
        }
        let zAxis = xAxis.clone().cross(yAxis);
        if (zAxis.lengthSq() < EPSILON) {
          yAxis = orthogonalVector(xAxis);
          zAxis = xAxis.clone().cross(yAxis);
        }
        const plane = normalizePlaneAxes(origin, xAxis, yAxis, zAxis);
        return { plane };
      },
    });

    register(['{c73e1ed0-82a2-40b0-b4df-8f10e445d60b}', 'flip plane', 'pflip'], {
      type: 'plane',
      pinMap: {
        inputs: {
          P: 'plane', Plane: 'plane', plane: 'plane',
          X: 'reverseX', 'Reverse X': 'reverseX', reverseX: 'reverseX',
          Y: 'reverseY', 'Reverse Y': 'reverseY', reverseY: 'reverseY',
          S: 'swap', 'Swap axes': 'swap', swap: 'swap',
        },
        outputs: { P: 'plane', Plane: 'plane', plane: 'plane' },
      },
      eval: ({ inputs }) => {
        const plane = ensurePlane(inputs.plane);
        const reverseX = ensureBoolean(inputs.reverseX, false);
        const reverseY = ensureBoolean(inputs.reverseY, false);
        const swap = ensureBoolean(inputs.swap, false);
        let xAxis = plane.xAxis.clone();
        let yAxis = plane.yAxis.clone();
        if (swap) {
          const temp = xAxis;
          xAxis = yAxis;
          yAxis = temp;
        }
        if (reverseX) {
          xAxis.multiplyScalar(-1);
        }
        if (reverseY) {
          yAxis.multiplyScalar(-1);
        }
        let zAxis = xAxis.clone().cross(yAxis);
        if (zAxis.lengthSq() < EPSILON) {
          zAxis = plane.zAxis.clone();
        }
        const result = normalizePlaneAxes(plane.origin, xAxis, yAxis, zAxis);
        return { plane: result };
      },
    });

    register(['{c98a6015-7a2f-423c-bc66-bdc505249b45}', 'plane 3pt', 'pl 3pt'], {
      type: 'plane',
      pinMap: {
        inputs: { A: 'a', a: 'a', B: 'b', b: 'b', C: 'c', c: 'c' },
        outputs: { Pl: 'plane', Plane: 'plane', plane: 'plane' },
      },
      eval: ({ inputs }) => {
        const a = ensurePoint(inputs.a ?? inputs.A, new THREE.Vector3());
        const b = ensurePoint(inputs.b ?? inputs.B, a.clone().add(new THREE.Vector3(1, 0, 0)));
        const c = ensurePoint(inputs.c ?? inputs.C, a.clone().add(new THREE.Vector3(0, 1, 0)));
        const plane = planeFromPoints(a, b, c);
        return { plane };
      },
    });

    register(['{ccc3f2ff-c9f6-45f8-aa30-8a924a9bda36}', 'line + pt', 'lnpt'], {
      type: 'plane',
      pinMap: {
        inputs: { L: 'line', Line: 'line', line: 'line', P: 'point', Point: 'point', point: 'point' },
        outputs: { Pl: 'plane', Plane: 'plane', plane: 'plane' },
      },
      eval: ({ inputs }) => {
        const line = ensureLine(inputs.line);
        const point = ensurePoint(inputs.point, line.start.clone().add(new THREE.Vector3(0, 1, 0)));
        const plane = planeFromLineAndPoint(line, point);
        return { plane };
      },
    });

    register(['{cfb6b17f-ca82-4f5d-b604-d4f69f569de3}', 'plane normal'], {
      type: 'plane',
      pinMap: {
        inputs: {
          O: 'origin', Origin: 'origin', origin: 'origin',
          Z: 'zAxis', 'Z-Axis': 'zAxis', zAxis: 'zAxis',
        },
        outputs: { P: 'plane', Plane: 'plane', plane: 'plane' },
      },
      eval: ({ inputs }) => {
        const origin = ensurePoint(inputs.origin, new THREE.Vector3());
        let zAxis = ensureVector(inputs.zAxis, new THREE.Vector3(0, 0, 1));
        if (zAxis.lengthSq() < EPSILON) {
          zAxis = new THREE.Vector3(0, 0, 1);
        } else {
          zAxis.normalize();
        }
        const xAxis = orthogonalVector(zAxis);
        const yAxis = zAxis.clone().cross(xAxis).normalize();
        const plane = normalizePlaneAxes(origin, xAxis, yAxis, zAxis.clone());
        return { plane };
      },
    });

    register(['{d788ad7f-6d68-4106-8b2f-9e55e6e107c0}', 'line + line', 'lnln'], {
      type: 'plane',
      pinMap: {
        inputs: { A: 'lineA', a: 'lineA', B: 'lineB', b: 'lineB', Line: 'lineA', 'Line A': 'lineA', 'Line B': 'lineB' },
        outputs: { Pl: 'plane', Plane: 'plane', plane: 'plane' },
      },
      eval: ({ inputs }) => {
        const lineA = ensureLine(inputs.lineA ?? inputs.A ?? inputs.a ?? inputs.Line);
        const lineB = ensureLine(inputs.lineB ?? inputs.B ?? inputs.b);
        const plane = planeFromLines(lineA, lineB);
        return { plane };
      },
    });

    register(['{e76040ec-3b91-41e1-8e00-c74c23b89391}', 'align plane', 'align plane direction'], {
      type: 'plane',
      pinMap: {
        inputs: {
          P: 'plane', Plane: 'plane', plane: 'plane',
          D: 'direction', Direction: 'direction', direction: 'direction',
        },
        outputs: { P: 'plane', Plane: 'plane', plane: 'plane', A: 'angle', Angle: 'angle', angle: 'angle' },
      },
      eval: ({ inputs }) => {
        const plane = ensurePlane(inputs.plane);
        const direction = ensureVector(inputs.direction, null);
        if (!direction || direction.lengthSq() < EPSILON) {
          return { plane: clonePlaneData(plane), angle: 0 };
        }
        const projected = direction.clone().sub(plane.zAxis.clone().multiplyScalar(direction.dot(plane.zAxis)));
        if (projected.lengthSq() < EPSILON) {
          return { plane: clonePlaneData(plane), angle: 0 };
        }
        const target = projected.normalize();
        const cosTheta = Math.max(-1, Math.min(1, plane.xAxis.dot(target)));
        const sinTheta = plane.yAxis.dot(target);
        const angle = Math.atan2(sinTheta, cosTheta);
        const rotation = new THREE.Quaternion().setFromAxisAngle(plane.zAxis.clone(), angle);
        const xAxis = plane.xAxis.clone().applyQuaternion(rotation);
        const yAxis = plane.yAxis.clone().applyQuaternion(rotation);
        const result = normalizePlaneAxes(plane.origin, xAxis, yAxis, plane.zAxis.clone());
        return { plane: result, angle };
      },
    });

    register(['{f6f14b09-6497-4564-8403-09e4eb5a6b82}', 'rotate plane', 'prot'], {
      type: 'plane',
      pinMap: {
        inputs: {
          P: 'plane', Plane: 'plane', plane: 'plane',
          A: 'angle', Angle: 'angle', angle: 'angle',
        },
        outputs: { P: 'plane', Plane: 'plane', plane: 'plane' },
      },
      eval: ({ inputs }) => {
        const plane = ensurePlane(inputs.plane);
        const angle = ensureNumber(inputs.angle, 0);
        if (Math.abs(angle) < EPSILON) {
          return { plane: clonePlaneData(plane) };
        }
        const rotation = new THREE.Quaternion().setFromAxisAngle(plane.zAxis.clone(), angle);
        const xAxis = plane.xAxis.clone().applyQuaternion(rotation);
        const yAxis = plane.yAxis.clone().applyQuaternion(rotation);
        const result = normalizePlaneAxes(plane.origin, xAxis, yAxis, plane.zAxis.clone());
        return { plane: result };
      },
    });

    register(['{fad344bc-09b1-4855-a2e6-437ef5715fe3}', 'yz plane', 'yz'], {
      type: 'plane',
      pinMap: {
        inputs: { O: 'origin', Origin: 'origin', origin: 'origin' },
        outputs: { P: 'plane', Plane: 'plane', plane: 'plane' },
      },
      eval: ({ inputs }) => {
        const origin = ensurePoint(inputs.origin, new THREE.Vector3());
        const plane = normalizePlaneAxes(
          origin,
          new THREE.Vector3(0, 1, 0),
          new THREE.Vector3(0, 0, 1),
          new THREE.Vector3(1, 0, 0),
        );
        return { plane };
      },
    });
  }

  function registerGridComponents() {
    function registerHexagonalGrid() {
      register(['{125dc122-8544-4617-945e-bb9a0c101c50}', 'hexagonal grid', 'hexgrid'], {
        type: 'point',
        pinMap: {
          inputs: {
            P: 'plane', Plane: 'plane', plane: 'plane',
            S: 'size', Size: 'size', size: 'size',
            'Extent X': 'extentX', 'extent x': 'extentX', Ex: 'extentX',
            'Extent Y': 'extentY', 'extent y': 'extentY', Ey: 'extentY',
          },
          outputs: {
            C: 'cells', Cells: 'cells', cells: 'cells',
            P: 'points', Points: 'points', points: 'points',
          },
        },
        eval: ({ inputs }) => {
          const plane = ensurePlane(inputs.plane);
          const size = Math.max(ensureNumber(inputs.size, 1), EPSILON);
          const extentX = Math.max(1, Math.round(ensureNumber(inputs.extentX, 6)));
          const extentY = Math.max(1, Math.round(ensureNumber(inputs.extentY, 6)));
          const grid = buildHexGridByExtents(plane, size, extentX, extentY);
          return {
            cells: grid.cells.map((cell) => cell.map((pt) => pt.clone())),
            points: grid.points.map((row) => row.map((pt) => pt.clone())),
          };
        },
      });
    }

    function registerRectangularGrids() {
      register([
        '{1a25aae0-0b56-497a-85b2-cc5bf7e4b96b}',
        '{fdedcd0a-ad40-4307-959d-d2891e2f533e}',
        'rectangular grid',
        'recgrid',
      ], {
        type: 'point',
        pinMap: {
          inputs: {
            P: 'plane', Plane: 'plane', plane: 'plane',
            'Size X': 'sizeX', 'size x': 'sizeX', Sx: 'sizeX',
            'Size Y': 'sizeY', 'size y': 'sizeY', Sy: 'sizeY',
            'Extent X': 'extentX', 'extent x': 'extentX', Ex: 'extentX',
            'Extent Y': 'extentY', 'extent y': 'extentY', Ey: 'extentY',
          },
          outputs: {
            C: 'cells', Cells: 'cells', cells: 'cells',
            P: 'points', Points: 'points', points: 'points',
          },
        },
        eval: ({ inputs }) => {
          const plane = ensurePlane(inputs.plane);
          const sizeX = Math.max(ensureNumber(inputs.sizeX, ensureNumber(inputs.sizeY, 1)), EPSILON);
          const sizeY = Math.max(ensureNumber(inputs.sizeY, sizeX), EPSILON);
          const cellsX = Math.max(1, Math.round(ensureNumber(inputs.extentX, 4)));
          const cellsY = Math.max(1, Math.round(ensureNumber(inputs.extentY, 4)));
          const pointCountX = cellsX + 1;
          const pointCountY = cellsY + 1;
          const offset = {
            x: -((pointCountX - 1) * sizeX) / 2,
            y: -((pointCountY - 1) * sizeY) / 2,
          };
          const grid = buildRectangularGrid(plane, pointCountX, pointCountY, sizeX, sizeY, offset);
          return {
            cells: grid.cells.map((cell) => cell.map((pt) => pt.clone())),
            points: createGridPointTree(grid.gridPoints, pointCountX, pointCountY),
          };
        },
      });
    }

    function registerSquareGrids() {
      register([
        '{40efea60-1902-4c28-8020-27abbb7a1449}',
        '{717a1e25-a075-4530-bc80-d43ecc2500d9}',
        'square grid',
        'sqgrid',
      ], {
        type: 'point',
        pinMap: {
          inputs: {
            P: 'plane', Plane: 'plane', plane: 'plane',
            S: 'size', Size: 'size', size: 'size',
            'Extent X': 'extentX', 'extent x': 'extentX', Ex: 'extentX',
            'Extent Y': 'extentY', 'extent y': 'extentY', Ey: 'extentY',
          },
          outputs: {
            C: 'cells', Cells: 'cells', cells: 'cells',
            P: 'points', Points: 'points', points: 'points',
          },
        },
        eval: ({ inputs }) => {
          const plane = ensurePlane(inputs.plane);
          const size = Math.max(ensureNumber(inputs.size, 1), EPSILON);
          const cellsX = Math.max(1, Math.round(ensureNumber(inputs.extentX, 4)));
          const cellsY = Math.max(1, Math.round(ensureNumber(inputs.extentY, 4)));
          const pointCountX = cellsX + 1;
          const pointCountY = cellsY + 1;
          const offset = {
            x: -((pointCountX - 1) * size) / 2,
            y: -((pointCountY - 1) * size) / 2,
          };
          const grid = buildRectangularGrid(plane, pointCountX, pointCountY, size, size, offset);
          return {
            cells: grid.cells.map((cell) => cell.map((pt) => pt.clone())),
            points: createGridPointTree(grid.gridPoints, pointCountX, pointCountY),
          };
        },
      });
    }

    function registerRadialGrids() {
      register([
        '{66eedc35-187d-4dab-b49b-408491b1255f}',
        '{773183d0-8c00-4fe4-a38c-f8d2408b7415}',
        'radial grid',
        'radgrid',
      ], {
        type: 'point',
        pinMap: {
          inputs: {
            P: 'plane', Plane: 'plane', plane: 'plane',
            S: 'size', Size: 'size', size: 'size',
            'Extent R': 'extentR', 'extent r': 'extentR', Er: 'extentR',
            'Extent P': 'extentP', 'extent p': 'extentP', Ep: 'extentP',
          },
          outputs: {
            C: 'cells', Cells: 'cells', cells: 'cells',
            P: 'points', Points: 'points', points: 'points',
          },
        },
        eval: ({ inputs }) => {
          const plane = ensurePlane(inputs.plane);
          const radiusStep = Math.max(ensureNumber(inputs.size, 1), EPSILON);
          const radialCount = Math.max(1, Math.round(ensureNumber(inputs.extentR, 4)));
          const polarCount = Math.max(3, Math.round(ensureNumber(inputs.extentP, 12)));
          const grid = buildRadialGrid(plane, radiusStep, radialCount, polarCount);
          return {
            cells: grid.cells.map((cell) => cell.map((pt) => pt.clone())),
            points: grid.rings.map((ring) => ring.map((pt) => pt.clone())),
          };
        },
      });
    }

    function registerTriangularGrid() {
      register(['{86a9944b-dea5-4126-9433-9e95ff07927a}', 'triangular grid', 'trigrid'], {
        type: 'point',
        pinMap: {
          inputs: {
            P: 'plane', Plane: 'plane', plane: 'plane',
            S: 'size', Size: 'size', size: 'size',
            'Extent X': 'extentX', 'extent x': 'extentX', Ex: 'extentX',
            'Extent Y': 'extentY', 'extent y': 'extentY', Ey: 'extentY',
          },
          outputs: {
            C: 'cells', Cells: 'cells', cells: 'cells',
            P: 'points', Points: 'points', points: 'points',
          },
        },
        eval: ({ inputs }) => {
          const plane = ensurePlane(inputs.plane);
          const edgeLength = Math.max(ensureNumber(inputs.size, 1), EPSILON);
          const cellsX = Math.max(1, Math.round(ensureNumber(inputs.extentX, 4)));
          const cellsY = Math.max(1, Math.round(ensureNumber(inputs.extentY, 4)));
          const grid = buildTriangularGrid(plane, edgeLength, cellsX, cellsY);
          return {
            cells: grid.cells.map((cell) => cell.map((pt) => pt.clone())),
            points: grid.points.map((row) => row.map((pt) => pt.clone())),
          };
        },
      });
    }

    function registerPopulateGeometryComponent() {
      register(['{c8cb6a5c-2ffd-4095-ba2a-5c35015e09e4}', 'populate geometry', 'popgeo'], {
        type: 'point',
        pinMap: {
          inputs: {
            G: 'geometry', Geometry: 'geometry', geometry: 'geometry',
            N: 'count', Count: 'count', count: 'count',
            S: 'seed', Seed: 'seed', seed: 'seed',
            P: 'existing', Points: 'existing', points: 'existing',
          },
          outputs: { P: 'population', Population: 'population', population: 'population' },
        },
        eval: ({ inputs }) => {
          const targetCount = resolveCount(inputs.count, 100);
          const rng = createSeededRandom(inputs.seed);
          const existing = collectPoints(inputs.existing).map((pt) => pt.clone());
          const { samplers, boundingBox, fallbackPoints } = gatherGeometrySamplers(inputs.geometry);
          const population = existing.slice(0, targetCount);
          let attempts = 0;
          while (population.length < targetCount && attempts < targetCount * 10) {
            attempts += 1;
            let point = null;
            if (samplers.length) {
              const index = Math.max(0, Math.min(samplers.length - 1, Math.floor(rng() * samplers.length)));
              point = samplers[index](rng);
            } else if (boundingBox) {
              point = randomPointInAxisAlignedBox(boundingBox, rng);
            }
            if (!point || !point.isVector3) {
              const fallbackPoint = ensurePoint(inputs.geometry, null) ?? fallbackPoints[0] ?? new THREE.Vector3();
              point = fallbackPoint.clone ? fallbackPoint.clone() : ensurePoint(fallbackPoint, new THREE.Vector3());
            }
            population.push(point.clone ? point.clone() : ensurePoint(point, new THREE.Vector3()));
            if (!samplers.length && !boundingBox) {
              break;
            }
          }
          while (population.length < targetCount) {
            const fallbackPoint = ensurePoint(inputs.geometry, null) ?? new THREE.Vector3();
            population.push(fallbackPoint.clone ? fallbackPoint.clone() : ensurePoint(fallbackPoint, new THREE.Vector3()));
            break;
          }
          return { population: population.slice(0, targetCount) };
        },
      });
    }

    function registerPopulate2DComponent() {
      register(['{e2d958e8-9f08-44f7-bf47-a684882d0b2a}', 'populate 2d', 'pop2d'], {
        type: 'point',
        pinMap: {
          inputs: {
            R: 'region', Region: 'region', region: 'region',
            N: 'count', Count: 'count', count: 'count',
            S: 'seed', Seed: 'seed', seed: 'seed',
            P: 'existing', Points: 'existing', points: 'existing',
          },
          outputs: { P: 'population', Population: 'population', population: 'population' },
        },
        eval: ({ inputs }) => {
          const section = extractRectangleSection(inputs.region);
          const targetCount = resolveCount(inputs.count, 100);
          const rng = createSeededRandom(inputs.seed);
          const existing = collectPoints(inputs.existing).map((pt) => pt.clone());
          const population = existing.slice(0, targetCount);
          let attempts = 0;
          while (population.length < targetCount && attempts < targetCount * 10) {
            attempts += 1;
            const point = randomPointInRectangle(section, rng);
            population.push(point);
            if (
              Math.abs(section.maxX - section.minX) < EPSILON
              && Math.abs(section.maxY - section.minY) < EPSILON
            ) {
              break;
            }
          }
          while (population.length < targetCount) {
            population.push(pointFromPlaneCoordinates(section.plane, section.minX, section.minY, 0));
            break;
          }
          return { population: population.slice(0, targetCount) };
        },
      });
    }

    function registerPopulate3DComponent() {
      register(['{e202025b-dc8e-4c51-ae19-4415b172886f}', 'populate 3d', 'pop3d'], {
        type: 'point',
        pinMap: {
          inputs: {
            R: 'region', Region: 'region', region: 'region',
            N: 'count', Count: 'count', count: 'count',
            S: 'seed', Seed: 'seed', seed: 'seed',
            P: 'existing', Points: 'existing', points: 'existing',
          },
          outputs: { P: 'population', Population: 'population', population: 'population' },
        },
        eval: ({ inputs }) => {
          const region = extractBoxRegion(inputs.region);
          const targetCount = resolveCount(inputs.count, 100);
          const rng = createSeededRandom(inputs.seed);
          const existing = collectPoints(inputs.existing).map((pt) => pt.clone());
          const population = existing.slice(0, targetCount);
          let attempts = 0;
          while (population.length < targetCount && attempts < targetCount * 10) {
            attempts += 1;
            const point = randomPointInBoxRegion(region, rng);
            population.push(point);
            if (
              Math.abs(region.max.x - region.min.x) < EPSILON
              && Math.abs(region.max.y - region.min.y) < EPSILON
              && Math.abs(region.max.z - region.min.z) < EPSILON
            ) {
              break;
            }
          }
          while (population.length < targetCount) {
            population.push(pointFromPlaneCoordinates(region.plane, region.min.x, region.min.y, region.min.z));
            break;
          }
          return { population: population.slice(0, targetCount) };
        },
      });
    }

    function registerFreeformCloudComponent() {
      register(['{f08233f1-9772-4514-8965-bde4948503df}', 'freeform cloud', 'ffcloud'], {
        type: 'point',
        pinMap: {
          inputs: {
            G: 'geometry', Guide: 'geometry', geometry: 'geometry', guide: 'geometry',
            N: 'count', Number: 'count', count: 'count',
            S: 'seed', Seed: 'seed', seed: 'seed',
          },
          outputs: { C: 'cloud', Cloud: 'cloud', cloud: 'cloud' },
        },
        eval: ({ inputs }) => {
          const targetCount = resolveCount(inputs.count, 100);
          const rng = createSeededRandom(inputs.seed);
          const { samplers, boundingBox, fallbackPoints } = gatherGeometrySamplers(inputs.geometry);
          const cloud = [];
          let attempts = 0;
          while (cloud.length < targetCount && attempts < targetCount * 10) {
            attempts += 1;
            let point = null;
            if (samplers.length) {
              const index = Math.max(0, Math.min(samplers.length - 1, Math.floor(rng() * samplers.length)));
              point = samplers[index](rng);
            } else if (boundingBox) {
              point = randomPointInAxisAlignedBox(boundingBox, rng);
            }
            if (!point || !point.isVector3) {
              const fallbackPoint = ensurePoint(inputs.geometry, null) ?? fallbackPoints[0] ?? new THREE.Vector3();
              point = fallbackPoint.clone ? fallbackPoint.clone() : ensurePoint(fallbackPoint, new THREE.Vector3());
            }
            cloud.push(point.clone ? point.clone() : ensurePoint(point, new THREE.Vector3()));
            if (!samplers.length && !boundingBox) {
              break;
            }
          }
          while (cloud.length < targetCount) {
            const fallbackPoint = ensurePoint(inputs.geometry, null) ?? new THREE.Vector3();
            cloud.push(fallbackPoint.clone ? fallbackPoint.clone() : ensurePoint(fallbackPoint, new THREE.Vector3()));
            break;
          }
          return { cloud: cloud.slice(0, targetCount) };
        },
      });
    }

    function registerSphericalCloudComponent() {
      register(['{fd68754e-6c60-44b2-9927-0a58146e0250}', 'spherical cloud', 'sphcloud'], {
        type: 'point',
        pinMap: {
          inputs: {
            C: 'center', Center: 'center', center: 'center',
            R: 'radius', Radius: 'radius', radius: 'radius',
            N: 'count', Count: 'count', count: 'count',
            S: 'seed', Seed: 'seed', seed: 'seed',
          },
          outputs: {
            C: 'cloud', Cloud: 'cloud', cloud: 'cloud',
            N: 'normals', Normals: 'normals', normals: 'normals',
          },
        },
        eval: ({ inputs }) => {
          const center = ensurePoint(inputs.center, new THREE.Vector3());
          const radius = Math.max(ensureNumber(inputs.radius, 1), EPSILON);
          const targetCount = resolveCount(inputs.count, 100);
          const rng = createSeededRandom(inputs.seed);
          const cloud = [];
          const normals = [];
          for (let i = 0; i < targetCount; i += 1) {
            const u = rng();
            const v = rng();
            const theta = 2 * Math.PI * u;
            const phi = Math.acos(2 * v - 1);
            const dir = new THREE.Vector3(
              Math.sin(phi) * Math.cos(theta),
              Math.sin(phi) * Math.sin(theta),
              Math.cos(phi),
            );
            const point = center.clone().add(dir.clone().multiplyScalar(radius));
            cloud.push(point);
            normals.push(dir);
          }
          return { cloud, normals };
        },
      });
    }

    function registerObsoleteGridComponents() {
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

    registerHexagonalGrid();
    registerRectangularGrids();
    registerSquareGrids();
    registerRadialGrids();
    registerTriangularGrid();
    registerPopulateGeometryComponent();
    registerPopulate2DComponent();
    registerPopulate3DComponent();
    registerFreeformCloudComponent();
    registerSphericalCloudComponent();
    registerObsoleteGridComponents();
  }

  return {
    registerPointCategory() {
      registerNumbersToPoints();
      registerTextTagComponents();
      registerPointConstructionComponents();
      registerPointAnalysisComponents();
      registerPointConversionComponents();
      registerPointProjectionComponents();
    },
    registerPlaneCategory() {
      registerPlaneComponents();
    },
    registerFieldCategory() {
      registerFieldComponents();
    },
    registerColourCategory() {
      registerColourComponents();
    },
    registerVectorCategory() {
      registerVectorComputationComponents();
    },
    registerGridCategory() {
      registerGridComponents();
    },
  };
}

export function registerVectorPointComponents(deps) {
  const { registerPointCategory } = createVectorComponentRegistrar(deps);
  registerPointCategory();
}

export function registerVectorPlaneComponents(deps) {
  const { registerPlaneCategory } = createVectorComponentRegistrar(deps);
  registerPlaneCategory();
}

export function registerVectorFieldComponents(deps) {
  const { registerFieldCategory } = createVectorComponentRegistrar(deps);
  registerFieldCategory();
}

export function registerVectorColourComponents(deps) {
  const { registerColourCategory } = createVectorComponentRegistrar(deps);
  registerColourCategory();
}

export function registerVectorVectorComponents(deps) {
  const { registerVectorCategory } = createVectorComponentRegistrar(deps);
  registerVectorCategory();
}

export function registerVectorGridComponents(deps) {
  const { registerGridCategory } = createVectorComponentRegistrar(deps);
  registerGridCategory();
}

import * as THREE from 'three';

function createTransformHelpers({ toNumber, toVector3 }) {
  const EPSILON = 1e-9;
  const SAMPLE_LIMIT = 2048;

  function identityMatrix() {
    const matrix = new THREE.Matrix4();
    matrix.identity();
    return matrix;
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
      if (!normalized) {
        return fallback;
      }
      if (['true', 'yes', 'y', '1', 'on'].includes(normalized)) {
        return true;
      }
      if (['false', 'no', 'n', '0', 'off'].includes(normalized)) {
        return false;
      }
      const numeric = Number(normalized);
      if (Number.isFinite(numeric)) {
        return numeric !== 0;
      }
      return fallback;
    }
    if (Array.isArray(value)) {
      if (!value.length) {
        return fallback;
      }
      return ensureBoolean(value[value.length - 1], fallback);
    }
    if (typeof value === 'object') {
      if ('value' in value) {
        return ensureBoolean(value.value, fallback);
      }
      if ('values' in value) {
        return ensureBoolean(value.values, fallback);
      }
    }
    return Boolean(value);
  }

  function ensurePoint(value, fallback = new THREE.Vector3()) {
    return toVector3(value, fallback.clone());
  }

  function ensureVector(value, fallback = new THREE.Vector3()) {
    const vector = toVector3(value, fallback.clone());
    if (vector.lengthSq() < EPSILON && fallback) {
      return fallback.clone();
    }
    return vector;
  }

  function ensureDirection(value, fallback = new THREE.Vector3(0, 0, 1)) {
    const vector = toVector3(value, fallback.clone());
    if (vector.lengthSq() < EPSILON) {
      return fallback.clone().normalize();
    }
    return vector.normalize();
  }

  function ensureUnitVector(vector, fallback) {
    const candidate = toVector3(vector, fallback.clone());
    if (candidate.lengthSq() < EPSILON) {
      return fallback.clone();
    }
    return candidate.normalize();
  }

  function defaultPlane() {
    return {
      origin: new THREE.Vector3(0, 0, 0),
      xAxis: new THREE.Vector3(1, 0, 0),
      yAxis: new THREE.Vector3(0, 1, 0),
      zAxis: new THREE.Vector3(0, 0, 1),
    };
  }

  function planeFromThreePlane(value) {
    const normal = value.normal?.clone?.() ?? new THREE.Vector3(0, 0, 1);
    if (normal.lengthSq() < EPSILON) {
      normal.set(0, 0, 1);
    } else {
      normal.normalize();
    }
    const origin = normal.clone().multiplyScalar(-(value.constant ?? 0));
    const xAxis = new THREE.Vector3(1, 0, 0);
    if (Math.abs(xAxis.dot(normal)) > 0.999) {
      xAxis.set(0, 1, 0);
    }
    xAxis.sub(normal.clone().multiplyScalar(xAxis.dot(normal))).normalize();
    const yAxis = normal.clone().cross(xAxis).normalize();
    return { origin, xAxis, yAxis, zAxis: normal.clone() };
  }

  function normalizePlaneAxes(origin, xAxis, yAxis, zAxisHint) {
    const x = ensureUnitVector(xAxis, new THREE.Vector3(1, 0, 0));
    let y = toVector3(yAxis, new THREE.Vector3(0, 1, 0));
    if (y.lengthSq() < EPSILON) {
      if (zAxisHint && zAxisHint.lengthSq() >= EPSILON) {
        y = zAxisHint.clone().cross(x);
        if (y.lengthSq() < EPSILON) {
          y = new THREE.Vector3(0, 1, 0);
        }
      } else {
        y = new THREE.Vector3(0, 1, 0);
      }
    }
    y.sub(x.clone().multiplyScalar(y.dot(x)));
    if (y.lengthSq() < EPSILON) {
      y = new THREE.Vector3(0, 1, 0);
    } else {
      y.normalize();
    }
    const z = (zAxisHint && zAxisHint.lengthSq() >= EPSILON)
      ? zAxisHint.clone().normalize()
      : x.clone().cross(y).normalize();
    const safeZ = z.lengthSq() < EPSILON ? new THREE.Vector3(0, 0, 1) : z;
    return {
      origin,
      xAxis: x,
      yAxis: y,
      zAxis: safeZ,
    };
  }

  function isPlaneLike(value) {
    if (!value) return false;
    if (value?.isPlane) return true;
    if (Array.isArray(value)) {
      if (value.length >= 3) return true;
      if (value.length === 1) return isPlaneLike(value[0]);
      return false;
    }
    if (typeof value === 'object') {
      if ('plane' in value) return true;
      if ('origin' in value || 'O' in value || 'o' in value) return true;
      if ('normal' in value) return true;
    }
    return false;
  }

  function ensurePlane(input) {
    if (!input) {
      return defaultPlane();
    }
    if (input?.isPlane) {
      return planeFromThreePlane(input);
    }
    if (Array.isArray(input)) {
      if (input.length >= 3) {
        const origin = toVector3(input[0], new THREE.Vector3());
        const xAxis = toVector3(input[1], new THREE.Vector3(1, 0, 0));
        const yAxis = toVector3(input[2], new THREE.Vector3(0, 1, 0));
        return normalizePlaneAxes(origin, xAxis, yAxis);
      }
      if (input.length === 1) {
        return ensurePlane(input[0]);
      }
    }
    if (typeof input === 'object') {
      if ('plane' in input) {
        return ensurePlane(input.plane);
      }
      if ('origin' in input || 'O' in input || 'o' in input) {
        const origin = toVector3(input.origin ?? input.O ?? input.o ?? new THREE.Vector3(), new THREE.Vector3());
        const xAxis = toVector3(
          input.xAxis ?? input.X ?? input.x ?? input.i ?? new THREE.Vector3(1, 0, 0),
          new THREE.Vector3(1, 0, 0),
        );
        const yAxis = toVector3(
          input.yAxis ?? input.Y ?? input.y ?? input.j ?? new THREE.Vector3(0, 1, 0),
          new THREE.Vector3(0, 1, 0),
        );
        const zAxis = input.zAxis ? toVector3(input.zAxis, new THREE.Vector3(0, 0, 1)) : undefined;
        return normalizePlaneAxes(origin, xAxis, yAxis, zAxis);
      }
      if ('normal' in input && 'point' in input) {
        const normal = ensureUnitVector(input.normal, new THREE.Vector3(0, 0, 1));
        const origin = toVector3(input.point, new THREE.Vector3());
        return normalizePlaneAxes(origin, new THREE.Vector3(1, 0, 0), new THREE.Vector3(0, 1, 0), normal);
      }
      if ('normal' in input && 'origin' in input) {
        const normal = ensureUnitVector(input.normal, new THREE.Vector3(0, 0, 1));
        const origin = toVector3(input.origin, new THREE.Vector3());
        return normalizePlaneAxes(origin, new THREE.Vector3(1, 0, 0), new THREE.Vector3(0, 1, 0), normal);
      }
      if ('normal' in input) {
        const normal = ensureUnitVector(input.normal, new THREE.Vector3(0, 0, 1));
        const origin = toVector3(input.point ?? input.center ?? input.origin ?? new THREE.Vector3(), new THREE.Vector3());
        return normalizePlaneAxes(origin, new THREE.Vector3(1, 0, 0), new THREE.Vector3(0, 1, 0), normal);
      }
    }
    return defaultPlane();
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
        const direction = end.clone().sub(start);
        if (direction.lengthSq() < EPSILON && input.length > 2) {
          direction.add(toVector3(input[2], new THREE.Vector3()));
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
      if ('line' in input) {
        return ensureLine(input.line);
      }
      const start = toVector3(
        input.start ?? input.from ?? input.a ?? input.A ?? input.origin ?? input.p0 ?? input.point0 ?? input.pointA ?? input.point,
        new THREE.Vector3(),
      );
      let end = input.end ?? input.to ?? input.b ?? input.B ?? input.p1 ?? input.point1 ?? input.pointB;
      let direction = input.direction ?? input.dir ?? input.tangent ?? input.vector;
      if (direction !== undefined) {
        direction = toVector3(direction, new THREE.Vector3(1, 0, 0));
        if (direction.lengthSq() < EPSILON) {
          direction = new THREE.Vector3(1, 0, 0);
        }
        direction = direction.clone();
        if (end === undefined) {
          end = start.clone().add(direction);
        }
      }
      const resolvedEnd = toVector3(
        end,
        direction ? start.clone().add(direction) : start.clone().add(new THREE.Vector3(1, 0, 0)),
      );
      const resolvedDirection = direction ? direction.clone() : resolvedEnd.clone().sub(start);
      if (resolvedDirection.lengthSq() < EPSILON) {
        resolvedDirection.set(1, 0, 0);
      }
      return { start, end: resolvedEnd, direction: resolvedDirection };
    }
    const start = toVector3(input, new THREE.Vector3());
    const end = start.clone().add(new THREE.Vector3(1, 0, 0));
    return { start, end, direction: end.clone().sub(start) };
  }

  function createPlaneMatrix(plane) {
    const matrix = new THREE.Matrix4();
    matrix.makeBasis(plane.xAxis.clone(), plane.yAxis.clone(), plane.zAxis.clone());
    matrix.setPosition(plane.origin.clone());
    return matrix;
  }

  function matrixFromPlaneToPlane(sourceInput, targetInput) {
    const source = ensurePlane(sourceInput);
    const target = ensurePlane(targetInput);
    const sourceMatrix = createPlaneMatrix(source);
    const targetMatrix = createPlaneMatrix(target);
    const inverseSource = sourceMatrix.clone();
    const determinant = inverseSource.determinant();
    if (Math.abs(determinant) < EPSILON) {
      inverseSource.identity();
    } else {
      inverseSource.invert();
    }
    return targetMatrix.multiply(inverseSource);
  }

  function transformDirectionVector(value, rotationMatrix, context) {
    if (value === undefined || value === null) {
      return value;
    }
    if (Array.isArray(value)) {
      return value.map((entry) => transformDirectionVector(entry, rotationMatrix, context));
    }
    if (value?.isVector3) {
      return value.clone().applyMatrix3(rotationMatrix);
    }
    if (typeof value === 'object') {
      if (context?.visited && context.visited.has(value)) {
        return context.visited.get(value);
      }
      if ('value' in value) {
        return transformDirectionVector(value.value, rotationMatrix, context);
      }
      if ('x' in value || 'y' in value || 'z' in value) {
        const x = toNumber(value.x, 0);
        const y = toNumber(value.y, 0);
        const z = toNumber(value.z, 0);
        const vector = new THREE.Vector3(x, y, z);
        return vector.applyMatrix3(rotationMatrix);
      }
    }
    return value;
  }

  function transformPlaneWithMatrix(planeInput, matrix, context) {
    const plane = ensurePlane(planeInput);
    const rotationMatrix = context?.rotationMatrix ?? new THREE.Matrix3().setFromMatrix4(matrix);
    const origin = plane.origin.clone().applyMatrix4(matrix);
    const xAxis = plane.xAxis.clone().applyMatrix3(rotationMatrix);
    const yAxis = plane.yAxis.clone().applyMatrix3(rotationMatrix);
    const zAxis = plane.zAxis.clone().applyMatrix3(rotationMatrix);
    return normalizePlaneAxes(origin, xAxis, yAxis, zAxis);
  }

  function transformGeometryStructure(value, matrix, context = {}) {
    if (value === undefined || value === null) {
      return value;
    }
    const rotationMatrix = context.rotationMatrix ?? new THREE.Matrix3().setFromMatrix4(matrix);
    const visited = context.visited ?? new Map();

    if (typeof value === 'object' || typeof value === 'function') {
      if (visited.has(value)) {
        return visited.get(value);
      }
    }

    if (value?.isVector3) {
      const result = value.clone();
      result.applyMatrix4(matrix);
      return result;
    }
    if (value?.isMatrix4) {
      return value.clone();
    }
    if (value?.isQuaternion) {
      return value.clone();
    }
    if (value?.isEuler) {
      return value.clone();
    }
    if (value?.isBufferGeometry) {
      const result = value.clone();
      result.applyMatrix4(matrix);
      return result;
    }
    if (value?.isGeometry) {
      const result = value.clone();
      result.applyMatrix4(matrix);
      return result;
    }
    if (value?.isPlane) {
      const result = value.clone();
      result.applyMatrix4(matrix);
      return result;
    }
    if (value?.isBox3) {
      const result = value.clone();
      result.applyMatrix4(matrix);
      return result;
    }
    if (value?.isMesh || value?.isObject3D) {
      const result = value.clone(true);
      visited.set(value, result);
      result.applyMatrix4(matrix);
      return result;
    }
    if (value?.isLine3) {
      const start = transformGeometryStructure(value.start, matrix, { rotationMatrix, visited });
      const end = transformGeometryStructure(value.end, matrix, { rotationMatrix, visited });
      const line = value.clone ? value.clone() : { start: start.clone(), end: end.clone() };
      line.start = start;
      line.end = end;
      line.delta = end.clone().sub(start);
      return line;
    }
    if (Array.isArray(value)) {
      const result = [];
      visited.set(value, result);
      for (const entry of value) {
        result.push(transformGeometryStructure(entry, matrix, { rotationMatrix, visited }));
      }
      return result;
    }
    if (typeof value === 'object') {
      if ('value' in value && Object.keys(value).length === 1) {
        return transformGeometryStructure(value.value, matrix, { rotationMatrix, visited });
      }
      const result = { ...value };
      visited.set(value, result);
      if ('geometry' in value) {
        result.geometry = transformGeometryStructure(value.geometry, matrix, { rotationMatrix, visited });
      }
      if ('geom' in value) {
        result.geom = transformGeometryStructure(value.geom, matrix, { rotationMatrix, visited });
      }
      if ('mesh' in value) {
        result.mesh = transformGeometryStructure(value.mesh, matrix, { rotationMatrix, visited });
      }
      if ('point' in value) {
        result.point = transformGeometryStructure(value.point, matrix, { rotationMatrix, visited });
      }
      if ('points' in value) {
        result.points = transformGeometryStructure(value.points, matrix, { rotationMatrix, visited });
      }
      if ('position' in value) {
        result.position = transformGeometryStructure(value.position, matrix, { rotationMatrix, visited });
      }
      if ('vertices' in value) {
        result.vertices = transformGeometryStructure(value.vertices, matrix, { rotationMatrix, visited });
      }
      if ('start' in value) {
        result.start = transformGeometryStructure(value.start, matrix, { rotationMatrix, visited });
      }
      if ('end' in value) {
        result.end = transformGeometryStructure(value.end, matrix, { rotationMatrix, visited });
      }
      if ('center' in value) {
        result.center = transformGeometryStructure(value.center, matrix, { rotationMatrix, visited });
      }
      if ('normal' in value) {
        result.normal = transformDirectionVector(value.normal, rotationMatrix, { rotationMatrix, visited });
      }
      if ('tangent' in value) {
        result.tangent = transformDirectionVector(value.tangent, rotationMatrix, { rotationMatrix, visited });
      }
      if ('binormal' in value) {
        result.binormal = transformDirectionVector(value.binormal, rotationMatrix, { rotationMatrix, visited });
      }
      if ('direction' in value) {
        result.direction = transformDirectionVector(value.direction, rotationMatrix, { rotationMatrix, visited });
      }
      if ('plane' in value && isPlaneLike(value.plane)) {
        result.plane = transformPlaneWithMatrix(value.plane, matrix, { rotationMatrix, visited });
      }
      if ('origin' in value && 'xAxis' in value && 'yAxis' in value) {
        const plane = transformPlaneWithMatrix(value, matrix, { rotationMatrix, visited });
        result.origin = plane.origin;
        result.xAxis = plane.xAxis;
        result.yAxis = plane.yAxis;
        result.zAxis = plane.zAxis;
      }
      if ('box3' in value && value.box3?.isBox3) {
        result.box3 = value.box3.clone().applyMatrix4(matrix);
      }
      if ('line' in value) {
        result.line = transformGeometryStructure(value.line, matrix, { rotationMatrix, visited });
      }
      return result;
    }
    return value;
  }

  function collectEntries(input) {
    const list = [];
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
        if ('values' in value) {
          visit(value.values);
          return;
        }
      }
      list.push(value);
    }
    visit(input);
    return list;
  }

  function collectPoints(value, limit = SAMPLE_LIMIT, result = [], visited = new Set()) {
    if (result.length >= limit) {
      return result;
    }
    if (value === undefined || value === null) {
      return result;
    }
    if (typeof value === 'object' || typeof value === 'function') {
      if (visited.has(value)) {
        return result;
      }
      visited.add(value);
    }
    if (value?.isVector3) {
      result.push(value.clone());
      return result;
    }
    if (value?.isBox3) {
      const min = value.min ?? new THREE.Vector3();
      const max = value.max ?? new THREE.Vector3();
      result.push(
        new THREE.Vector3(min.x, min.y, min.z),
        new THREE.Vector3(max.x, min.y, min.z),
        new THREE.Vector3(max.x, max.y, min.z),
        new THREE.Vector3(min.x, max.y, min.z),
        new THREE.Vector3(min.x, min.y, max.z),
        new THREE.Vector3(max.x, min.y, max.z),
        new THREE.Vector3(max.x, max.y, max.z),
        new THREE.Vector3(min.x, max.y, max.z),
      );
      return result;
    }
    if (value?.isBufferGeometry) {
      const position = value.getAttribute?.('position');
      if (position && position.count) {
        const vector = new THREE.Vector3();
        const remaining = Math.max(1, limit - result.length);
        const step = Math.max(1, Math.floor(position.count / remaining));
        for (let i = 0; i < position.count && result.length < limit; i += step) {
          vector.fromBufferAttribute(position, i);
          result.push(vector.clone());
        }
      }
      return result;
    }
    if (value?.isGeometry && Array.isArray(value.vertices)) {
      const remaining = Math.max(1, limit - result.length);
      const step = Math.max(1, Math.floor(value.vertices.length / remaining));
      for (let i = 0; i < value.vertices.length && result.length < limit; i += step) {
        const vertex = value.vertices[i];
        if (vertex?.isVector3) {
          result.push(vertex.clone());
        }
      }
      return result;
    }
    if (value?.isMesh) {
      if (value.geometry) {
        collectPoints(value.geometry, limit, result, visited);
      }
      return result;
    }
    if (Array.isArray(value)) {
      for (const entry of value) {
        if (result.length >= limit) break;
        collectPoints(entry, limit, result, visited);
      }
      return result;
    }
    if (typeof value === 'object') {
      if ('point' in value) {
        collectPoints(value.point, limit, result, visited);
      }
      if ('points' in value) {
        collectPoints(value.points, limit, result, visited);
      }
      if ('position' in value) {
        collectPoints(value.position, limit, result, visited);
      }
      if ('vertices' in value) {
        collectPoints(value.vertices, limit, result, visited);
      }
      if ('geometry' in value) {
        collectPoints(value.geometry, limit, result, visited);
      }
      if ('geom' in value) {
        collectPoints(value.geom, limit, result, visited);
      }
      if ('mesh' in value) {
        collectPoints(value.mesh, limit, result, visited);
      }
      if ('center' in value) {
        collectPoints(value.center, limit, result, visited);
      }
      if ('origin' in value) {
        collectPoints(value.origin, limit, result, visited);
      }
      if ('box3' in value) {
        collectPoints(value.box3, limit, result, visited);
      }
      if ('curve' in value && typeof value.curve?.getPoints === 'function') {
        const samples = value.curve.getPoints(32);
        collectPoints(samples, limit, result, visited);
      }
      if ('start' in value && 'end' in value) {
        collectPoints(value.start, limit, result, visited);
        collectPoints(value.end, limit, result, visited);
      }
      if ('line' in value) {
        collectPoints(value.line, limit, result, visited);
      }
      if ('x' in value || 'y' in value || 'z' in value) {
        const x = toNumber(value.x, Number.NaN);
        const y = toNumber(value.y, Number.NaN);
        const z = toNumber(value.z, Number.NaN);
        if (Number.isFinite(x) || Number.isFinite(y) || Number.isFinite(z)) {
          result.push(new THREE.Vector3(
            Number.isFinite(x) ? x : 0,
            Number.isFinite(y) ? y : 0,
            Number.isFinite(z) ? z : 0,
          ));
        }
      }
      return result;
    }
    if (typeof value === 'number' && Number.isFinite(value)) {
      result.push(new THREE.Vector3(value, 0, 0));
    }
    return result;
  }

  function computeBoundingBox(value) {
    const points = collectPoints(value);
    if (!points.length) {
      return null;
    }
    const box = new THREE.Box3();
    box.setFromPoints(points);
    if (Number.isNaN(box.min.x) || Number.isNaN(box.max.x)) {
      return null;
    }
    return box;
  }

  function computeCentroid(value) {
    const points = collectPoints(value);
    if (!points.length) {
      return null;
    }
    const centroid = new THREE.Vector3();
    for (const point of points) {
      centroid.add(point);
    }
    centroid.multiplyScalar(1 / points.length);
    return centroid;
  }

  function createAxisRotationMatrix(originInput, axisInput, angle) {
    const origin = ensurePoint(originInput, new THREE.Vector3());
    const axis = ensureDirection(axisInput, new THREE.Vector3(0, 0, 1));
    if (axis.lengthSq() < EPSILON || Math.abs(angle) < EPSILON) {
      return identityMatrix();
    }
    const rotation = new THREE.Matrix4().makeRotationAxis(axis, angle);
    const translateToOrigin = new THREE.Matrix4().makeTranslation(-origin.x, -origin.y, -origin.z);
    rotation.multiply(translateToOrigin);
    const translateBack = new THREE.Matrix4().makeTranslation(origin.x, origin.y, origin.z);
    rotation.premultiply(translateBack);
    return rotation;
  }

  function createDirectionRotationMatrix(centerInput, fromInput, toInput) {
    const center = ensurePoint(centerInput, new THREE.Vector3());
    const fromVector = ensureDirection(fromInput, new THREE.Vector3(1, 0, 0));
    const toVector = ensureDirection(toInput, new THREE.Vector3(1, 0, 0));
    if (fromVector.lengthSq() < EPSILON || toVector.lengthSq() < EPSILON) {
      return identityMatrix();
    }
    const quaternion = new THREE.Quaternion().setFromUnitVectors(fromVector.clone().normalize(), toVector.clone().normalize());
    const rotation = new THREE.Matrix4().makeRotationFromQuaternion(quaternion);
    const translateToOrigin = new THREE.Matrix4().makeTranslation(-center.x, -center.y, -center.z);
    rotation.multiply(translateToOrigin);
    const translateBack = new THREE.Matrix4().makeTranslation(center.x, center.y, center.z);
    rotation.premultiply(translateBack);
    return rotation;
  }

  function createMirrorMatrix(planeInput) {
    const plane = ensurePlane(planeInput);
    const normal = plane.zAxis.clone().normalize();
    const origin = plane.origin.clone();
    const nx = normal.x;
    const ny = normal.y;
    const nz = normal.z;
    const reflection = new THREE.Matrix4();
    reflection.set(
      1 - 2 * nx * nx, -2 * nx * ny, -2 * nx * nz, 0,
      -2 * ny * nx, 1 - 2 * ny * ny, -2 * ny * nz, 0,
      -2 * nz * nx, -2 * nz * ny, 1 - 2 * nz * nz, 0,
      0, 0, 0, 1,
    );
    const translateToOrigin = new THREE.Matrix4().makeTranslation(-origin.x, -origin.y, -origin.z);
    reflection.multiply(translateToOrigin);
    const translateBack = new THREE.Matrix4().makeTranslation(origin.x, origin.y, origin.z);
    reflection.premultiply(translateBack);
    return reflection;
  }

  function applyTransformToGeometry(geometryInput, matrix) {
    return transformGeometryStructure(geometryInput, matrix);
  }

  function planeFromPoints(aInput, bInput, cInput) {
    const fallback = defaultPlane();
    const a = ensurePoint(aInput, fallback.origin.clone());
    let b = ensurePoint(bInput, a.clone().add(new THREE.Vector3(1, 0, 0)));
    let c = ensurePoint(cInput, a.clone().add(new THREE.Vector3(0, 1, 0)));
    if (b.distanceToSquared(a) < EPSILON) {
      b = a.clone().add(new THREE.Vector3(1, 0, 0));
    }
    if (c.distanceToSquared(a) < EPSILON) {
      c = a.clone().add(new THREE.Vector3(0, 1, 0));
    }
    const ab = b.clone().sub(a);
    const ac = c.clone().sub(a);
    let xAxis = ab.clone();
    if (xAxis.lengthSq() < EPSILON) {
      xAxis = new THREE.Vector3(1, 0, 0);
    }
    xAxis.normalize();
    let normal = ab.clone().cross(ac);
    if (normal.lengthSq() < EPSILON) {
      normal = xAxis.clone().cross(new THREE.Vector3(0, 0, 1));
      if (normal.lengthSq() < EPSILON) {
        normal = new THREE.Vector3(0, 0, 1);
      }
    }
    normal.normalize();
    const yAxis = normal.clone().cross(xAxis).normalize();
    return normalizePlaneAxes(a.clone(), xAxis, yAxis, normal);
  }

  function planeCoordinates(pointInput, plane) {
    const point = ensurePoint(pointInput, plane.origin.clone());
    const offset = point.clone().sub(plane.origin);
    return new THREE.Vector3(
      offset.dot(plane.xAxis),
      offset.dot(plane.yAxis),
      offset.dot(plane.zAxis),
    );
  }

  function applyPlane(plane, x = 0, y = 0, z = 0) {
    const result = plane.origin.clone();
    result.add(plane.xAxis.clone().multiplyScalar(x));
    result.add(plane.yAxis.clone().multiplyScalar(y));
    result.add(plane.zAxis.clone().multiplyScalar(z));
    return result;
  }

  function matrixFromPoints(sourcePoints, targetPoints) {
    if (!Array.isArray(sourcePoints) || !Array.isArray(targetPoints)) {
      return identityMatrix();
    }
    if (sourcePoints.length < 4 || targetPoints.length < 4) {
      return identityMatrix();
    }
    const invalid = new THREE.Vector3(Number.NaN, Number.NaN, Number.NaN);
    const sp = sourcePoints.slice(0, 4).map((point) => ensurePoint(point, invalid.clone()));
    const tp = targetPoints.slice(0, 4).map((point) => ensurePoint(point, invalid.clone()));
    if (sp.some((p) => !Number.isFinite(p.x) || !Number.isFinite(p.y) || !Number.isFinite(p.z))) {
      return identityMatrix();
    }
    if (tp.some((p) => !Number.isFinite(p.x) || !Number.isFinite(p.y) || !Number.isFinite(p.z))) {
      return identityMatrix();
    }
    const sourceMatrix = new THREE.Matrix4().set(
      sp[0].x, sp[1].x, sp[2].x, sp[3].x,
      sp[0].y, sp[1].y, sp[2].y, sp[3].y,
      sp[0].z, sp[1].z, sp[2].z, sp[3].z,
      1, 1, 1, 1,
    );
    const targetMatrix = new THREE.Matrix4().set(
      tp[0].x, tp[1].x, tp[2].x, tp[3].x,
      tp[0].y, tp[1].y, tp[2].y, tp[3].y,
      tp[0].z, tp[1].z, tp[2].z, tp[3].z,
      1, 1, 1, 1,
    );
    const inverseSource = sourceMatrix.clone();
    const determinant = inverseSource.determinant();
    if (!Number.isFinite(determinant) || Math.abs(determinant) < EPSILON) {
      return identityMatrix();
    }
    inverseSource.invert();
    return targetMatrix.multiply(inverseSource);
  }

  function matrixIsIdentity(matrix) {
    if (!matrix) {
      return false;
    }
    const identity = identityMatrix();
    const { elements } = matrix;
    for (let i = 0; i < 16; i += 1) {
      if (Math.abs(elements[i] - identity.elements[i]) > 1e-9) {
        return false;
      }
    }
    return true;
  }

  function createProjectionMatrix(planeInput, directionInput) {
    const plane = ensurePlane(planeInput);
    const direction = directionInput ? ensureVector(directionInput, plane.zAxis.clone()) : plane.zAxis.clone();
    if (direction.lengthSq() < EPSILON) {
      return null;
    }
    const normal = plane.zAxis.clone().normalize();
    const dot = direction.dot(normal);
    if (Math.abs(dot) < EPSILON) {
      return null;
    }
    const c = -normal.dot(plane.origin);
    const scale = 1 / dot;
    const offset = direction.clone().multiplyScalar(-c * scale);
    const dx = direction.x;
    const dy = direction.y;
    const dz = direction.z;
    const nx = normal.x;
    const ny = normal.y;
    const nz = normal.z;
    const matrix = new THREE.Matrix4().set(
      1 - dx * nx * scale, -dx * ny * scale, -dx * nz * scale, offset.x,
      -dy * nx * scale, 1 - dy * ny * scale, -dy * nz * scale, offset.y,
      -dz * nx * scale, -dz * ny * scale, 1 - dz * nz * scale, offset.z,
      0, 0, 0, 1,
    );
    return matrix;
  }

  function extractDomain(domainInput, fallbackMin, fallbackMax) {
    if (domainInput === undefined || domainInput === null) {
      return { min: fallbackMin, max: fallbackMax };
    }
    if (Array.isArray(domainInput)) {
      if (domainInput.length >= 2) {
        const min = toNumber(domainInput[0], Number.NaN);
        const max = toNumber(domainInput[1], Number.NaN);
        if (Number.isFinite(min) && Number.isFinite(max)) {
          return { min, max };
        }
      }
      if (domainInput.length === 1) {
        return extractDomain(domainInput[0], fallbackMin, fallbackMax);
      }
    }
    if (typeof domainInput === 'object') {
      const min = toNumber(domainInput.min ?? domainInput.start ?? domainInput.from ?? domainInput.a ?? domainInput.A ?? domainInput[0], Number.NaN);
      const max = toNumber(domainInput.max ?? domainInput.end ?? domainInput.to ?? domainInput.b ?? domainInput.B ?? domainInput[1], Number.NaN);
      if (Number.isFinite(min) && Number.isFinite(max)) {
        return { min, max };
      }
      if (Number.isFinite(min)) {
        return { min, max: fallbackMax };
      }
      if (Number.isFinite(max)) {
        return { min: fallbackMin, max };
      }
      const length = toNumber(domainInput.length ?? domainInput.span ?? domainInput.size, Number.NaN);
      if (Number.isFinite(length)) {
        const half = Math.abs(length) / 2;
        return { min: -half, max: half };
      }
    }
    const numeric = toNumber(domainInput, Number.NaN);
    if (Number.isFinite(numeric)) {
      const half = Math.abs(numeric) / 2;
      return { min: -half, max: half };
    }
    return { min: fallbackMin, max: fallbackMax };
  }

  function extractRectangleFrame(rectangleInput) {
    if (!rectangleInput) {
      return null;
    }
    if (Array.isArray(rectangleInput) && rectangleInput.length === 1) {
      return extractRectangleFrame(rectangleInput[0]);
    }
    if (rectangleInput.rectangle) {
      return extractRectangleFrame(rectangleInput.rectangle);
    }
    let corners = [];
    if (Array.isArray(rectangleInput) && rectangleInput.length >= 3) {
      corners = rectangleInput;
    } else if (Array.isArray(rectangleInput.corners) && rectangleInput.corners.length >= 3) {
      corners = rectangleInput.corners;
    } else if (Array.isArray(rectangleInput.points) && rectangleInput.points.length >= 3) {
      corners = rectangleInput.points;
    }
    let plane;
    if (corners.length >= 3) {
      plane = planeFromPoints(corners[0], corners[1], corners[2]);
    } else if (rectangleInput.plane) {
      plane = ensurePlane(rectangleInput.plane);
    } else if (isPlaneLike(rectangleInput)) {
      plane = ensurePlane(rectangleInput);
    } else {
      plane = defaultPlane();
    }
    let minX = -0.5;
    let maxX = 0.5;
    let minY = -0.5;
    let maxY = 0.5;
    if (corners.length >= 1) {
      minX = Number.POSITIVE_INFINITY;
      maxX = Number.NEGATIVE_INFINITY;
      minY = Number.POSITIVE_INFINITY;
      maxY = Number.NEGATIVE_INFINITY;
      for (const corner of corners) {
        const coord = planeCoordinates(corner, plane);
        if (coord.x < minX) minX = coord.x;
        if (coord.x > maxX) maxX = coord.x;
        if (coord.y < minY) minY = coord.y;
        if (coord.y > maxY) maxY = coord.y;
      }
      if (!Number.isFinite(minX) || !Number.isFinite(maxX) || !Number.isFinite(minY) || !Number.isFinite(maxY)) {
        minX = -0.5;
        maxX = 0.5;
        minY = -0.5;
        maxY = 0.5;
      }
    } else {
      const width = toNumber(
        rectangleInput.width ?? rectangleInput.xSize ?? rectangleInput.sizeX ?? rectangleInput.widthX ?? rectangleInput.X ?? rectangleInput.x,
        Number.NaN,
      );
      const height = toNumber(
        rectangleInput.height ?? rectangleInput.ySize ?? rectangleInput.sizeY ?? rectangleInput.heightY ?? rectangleInput.Y ?? rectangleInput.y,
        Number.NaN,
      );
      const domainX = extractDomain(
        rectangleInput.domainX ?? rectangleInput.xDomain ?? rectangleInput.XDomain ?? rectangleInput.intervalX ?? rectangleInput.xInterval ?? rectangleInput.XInterval,
        -0.5,
        0.5,
      );
      const domainY = extractDomain(
        rectangleInput.domainY ?? rectangleInput.yDomain ?? rectangleInput.YDomain ?? rectangleInput.intervalY ?? rectangleInput.yInterval ?? rectangleInput.YInterval,
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
    return { plane, minX, maxX, minY, maxY };
  }

  function rectangleToMatrix(frame) {
    if (!frame) {
      return null;
    }
    const plane = frame.plane ?? defaultPlane();
    const minX = Number.isFinite(frame.minX) ? frame.minX : -0.5;
    const maxX = Number.isFinite(frame.maxX) ? frame.maxX : 0.5;
    const minY = Number.isFinite(frame.minY) ? frame.minY : -0.5;
    const maxY = Number.isFinite(frame.maxY) ? frame.maxY : 0.5;
    const basisX = plane.xAxis.clone().multiplyScalar(maxX - minX);
    const basisY = plane.yAxis.clone().multiplyScalar(maxY - minY);
    const normal = plane.zAxis.clone();
    const origin = applyPlane(plane, minX, minY, 0);
    const matrix = new THREE.Matrix4().set(
      basisX.x, basisY.x, normal.x, origin.x,
      basisX.y, basisY.y, normal.y, origin.y,
      basisX.z, basisY.z, normal.z, origin.z,
      0, 0, 0, 1,
    );
    return matrix;
  }

  function extractTriangleFrame(triangleInput) {
    if (!triangleInput) {
      return null;
    }
    if (Array.isArray(triangleInput) && triangleInput.length === 1) {
      return extractTriangleFrame(triangleInput[0]);
    }
    if (triangleInput.triangle) {
      return extractTriangleFrame(triangleInput.triangle);
    }
    const points = [];
    const invalid = new THREE.Vector3(Number.NaN, Number.NaN, Number.NaN);
    const pushPoint = (value) => {
      if (points.length >= 3 || value === undefined || value === null) return;
      const point = ensurePoint(value, invalid.clone());
      if (!Number.isFinite(point.x) || !Number.isFinite(point.y) || !Number.isFinite(point.z)) {
        return;
      }
      if (!points.some((existing) => existing.distanceToSquared(point) < EPSILON)) {
        points.push(point.clone());
      }
    };
    pushPoint(triangleInput.A ?? triangleInput.a ?? triangleInput.pointA ?? triangleInput.cornerA);
    pushPoint(triangleInput.B ?? triangleInput.b ?? triangleInput.pointB ?? triangleInput.cornerB);
    pushPoint(triangleInput.C ?? triangleInput.c ?? triangleInput.pointC ?? triangleInput.cornerC);
    if (points.length < 3 && Array.isArray(triangleInput.points)) {
      for (const entry of triangleInput.points) {
        if (points.length >= 3) break;
        pushPoint(entry);
      }
    }
    if (points.length < 3 && Array.isArray(triangleInput)) {
      for (const entry of triangleInput) {
        if (points.length >= 3) break;
        pushPoint(entry);
      }
    }
    if (points.length < 3) {
      const collected = collectPoints(triangleInput, 8);
      for (const entry of collected) {
        if (points.length >= 3) break;
        pushPoint(entry);
      }
    }
    if (points.length < 3) {
      return null;
    }
    const [a, b, c] = points;
    const plane = planeFromPoints(a, b, c);
    const xAxis = b.clone().sub(a);
    if (xAxis.lengthSq() < EPSILON) {
      xAxis.copy(plane.xAxis);
    }
    const yAxis = c.clone().sub(a);
    if (yAxis.lengthSq() < EPSILON) {
      yAxis.copy(plane.yAxis);
    }
    let normal = xAxis.clone().cross(yAxis);
    if (normal.lengthSq() < EPSILON) {
      normal = plane.zAxis.clone();
    } else {
      normal.normalize();
    }
    return { origin: a.clone(), xAxis, yAxis, normal };
  }

  function triangleToMatrix(frame) {
    if (!frame) {
      return null;
    }
    const origin = frame.origin ?? new THREE.Vector3();
    const basisX = frame.xAxis?.clone() ?? new THREE.Vector3(1, 0, 0);
    const basisY = frame.yAxis?.clone() ?? new THREE.Vector3(0, 1, 0);
    let normal = frame.normal?.clone() ?? basisX.clone().cross(basisY);
    if (basisX.lengthSq() < EPSILON) {
      basisX.set(1, 0, 0);
    }
    if (basisY.lengthSq() < EPSILON) {
      basisY.set(0, 1, 0);
    }
    if (normal.lengthSq() < EPSILON) {
      normal = basisX.clone().cross(basisY);
      if (normal.lengthSq() < EPSILON) {
        normal.set(0, 0, 1);
      }
    }
    normal.normalize();
    const matrix = new THREE.Matrix4().set(
      basisX.x, basisY.x, normal.x, origin.x,
      basisX.y, basisY.y, normal.y, origin.y,
      basisX.z, basisY.z, normal.z, origin.z,
      0, 0, 0, 1,
    );
    return matrix;
  }

  function extractBoxFrame(boxInput) {
    if (!boxInput) {
      return null;
    }
    if (Array.isArray(boxInput) && boxInput.length === 1) {
      return extractBoxFrame(boxInput[0]);
    }
    if (boxInput.box) {
      return extractBoxFrame(boxInput.box);
    }
    if (boxInput.type === 'box' && boxInput.plane && (boxInput.localMin || boxInput.localMax)) {
      const plane = normalizePlaneAxes(
        ensurePoint(boxInput.plane.origin ?? new THREE.Vector3(), new THREE.Vector3()),
        ensurePoint(boxInput.plane.xAxis ?? new THREE.Vector3(1, 0, 0), new THREE.Vector3(1, 0, 0)),
        ensurePoint(boxInput.plane.yAxis ?? new THREE.Vector3(0, 1, 0), new THREE.Vector3(0, 1, 0)),
        ensurePoint(boxInput.plane.zAxis ?? new THREE.Vector3(0, 0, 1), new THREE.Vector3(0, 0, 1)),
      );
      const min = ensurePoint(boxInput.localMin ?? new THREE.Vector3(), new THREE.Vector3());
      const max = ensurePoint(boxInput.localMax ?? new THREE.Vector3(1, 1, 1), new THREE.Vector3(1, 1, 1));
      return { plane, min, max };
    }
    if (boxInput.box3) {
      const plane = boxInput.plane ? ensurePlane(boxInput.plane) : defaultPlane();
      const min = ensurePoint(boxInput.box3.min ?? boxInput.min ?? new THREE.Vector3(), new THREE.Vector3());
      const max = ensurePoint(boxInput.box3.max ?? boxInput.max ?? new THREE.Vector3(1, 1, 1), new THREE.Vector3(1, 1, 1));
      return { plane, min, max };
    }
    if (boxInput.min || boxInput.max) {
      const plane = defaultPlane();
      const min = ensurePoint(boxInput.min ?? new THREE.Vector3(), new THREE.Vector3());
      const max = ensurePoint(boxInput.max ?? new THREE.Vector3(1, 1, 1), new THREE.Vector3(1, 1, 1));
      return { plane, min, max };
    }
    if (boxInput.center && boxInput.size) {
      const basePlane = boxInput.plane ? ensurePlane(boxInput.plane) : defaultPlane();
      const center = ensurePoint(boxInput.center, basePlane.origin.clone());
      const size = ensurePoint(boxInput.size, new THREE.Vector3(1, 1, 1));
      const half = size.clone().multiplyScalar(0.5);
      const plane = normalizePlaneAxes(center, basePlane.xAxis.clone(), basePlane.yAxis.clone(), basePlane.zAxis.clone());
      const min = new THREE.Vector3(-half.x, -half.y, -half.z);
      const max = new THREE.Vector3(half.x, half.y, half.z);
      return { plane, min, max };
    }
    const points = collectPoints(boxInput, 32);
    if (points.length) {
      const box = new THREE.Box3();
      box.setFromPoints(points);
      if (!Number.isNaN(box.min.x) && !Number.isNaN(box.max.x)) {
        return { plane: defaultPlane(), min: box.min.clone(), max: box.max.clone() };
      }
    }
    return null;
  }

  function boxToMatrix(box) {
    if (!box) {
      return null;
    }
    const plane = box.plane ?? defaultPlane();
    const min = box.min ?? new THREE.Vector3();
    const max = box.max ?? new THREE.Vector3(1, 1, 1);
    const basisX = plane.xAxis.clone().multiplyScalar(max.x - min.x);
    const basisY = plane.yAxis.clone().multiplyScalar(max.y - min.y);
    const basisZ = plane.zAxis.clone().multiplyScalar(max.z - min.z);
    const origin = applyPlane(plane, min.x, min.y, min.z);
    const matrix = new THREE.Matrix4().set(
      basisX.x, basisY.x, basisZ.x, origin.x,
      basisX.y, basisY.y, basisZ.y, origin.y,
      basisX.z, basisY.z, basisZ.z, origin.z,
      0, 0, 0, 1,
    );
    return matrix;
  }
  return {
    EPSILON,
    SAMPLE_LIMIT,
    identityMatrix,
    ensureBoolean,
    ensurePoint,
    ensureVector,
    ensureDirection,
    ensureUnitVector,
    defaultPlane,
    planeFromThreePlane,
    normalizePlaneAxes,
    isPlaneLike,
    ensurePlane,
    ensureLine,
    createPlaneMatrix,
    matrixFromPlaneToPlane,
    transformDirectionVector,
    transformPlaneWithMatrix,
    transformGeometryStructure,
    collectEntries,
    collectPoints,
    computeBoundingBox,
    computeCentroid,
    createAxisRotationMatrix,
    createDirectionRotationMatrix,
    createMirrorMatrix,
    applyTransformToGeometry,
    planeFromPoints,
    planeCoordinates,
    applyPlane,
    matrixFromPoints,
    matrixIsIdentity,
    createProjectionMatrix,
    extractRectangleFrame,
    rectangleToMatrix,
    extractTriangleFrame,
    triangleToMatrix,
    extractBoxFrame,
    boxToMatrix,
  };
}

export function registerTransformEuclideanComponents({ register, toNumber, toVector3 }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register transform components.');
  }
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register transform components.');
  }
  if (typeof toVector3 !== 'function') {
    throw new Error('toVector3 function is required to register transform components.');
  }


  const {
    EPSILON,
    SAMPLE_LIMIT,
    identityMatrix,
    ensureBoolean,
    ensurePoint,
    ensureVector,
    ensureDirection,
    ensureUnitVector,
    defaultPlane,
    planeFromThreePlane,
    normalizePlaneAxes,
    isPlaneLike,
    ensurePlane,
    ensureLine,
    createPlaneMatrix,
    matrixFromPlaneToPlane,
    transformDirectionVector,
    transformPlaneWithMatrix,
    transformGeometryStructure,
    collectEntries,
    collectPoints,
    computeBoundingBox,
    computeCentroid,
    createAxisRotationMatrix,
    createDirectionRotationMatrix,
    createMirrorMatrix,
    applyTransformToGeometry,
    planeFromPoints,
    planeCoordinates,
    applyPlane,
    matrixFromPoints,
    matrixIsIdentity,
    createProjectionMatrix,
    extractRectangleFrame,
    rectangleToMatrix,
    extractTriangleFrame,
    triangleToMatrix,
    extractBoxFrame,
    boxToMatrix,
  } = createTransformHelpers({ toNumber, toVector3 });

  register(['{03b3db66-d7e8-4d2d-bc0c-122913317254}', 'sanity xform', 'mwhahaha!!'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry' },
      outputs: { G: 'geometry', geometry: 'geometry', W: 'wackometry', wackometry: 'wackometry' },
    },
    eval: ({ inputs }) => {
      const entries = collectEntries(inputs.geometry);
      const sane = [];
      const wacky = [];
      for (const entry of entries) {
        const box = computeBoundingBox(entry);
        if (!box) {
          const clone = applyTransformToGeometry(entry, identityMatrix());
          sane.push(clone);
          wacky.push(applyTransformToGeometry(clone, identityMatrix()));
          continue;
        }
        const center = box.getCenter(new THREE.Vector3());
        const size = box.getSize(new THREE.Vector3());
        const maxExtent = Math.max(size.x, size.y, size.z, EPSILON);
        let scale = 1;
        if (maxExtent > 1000) {
          scale = 1000 / maxExtent;
        } else if (maxExtent < 0.01) {
          scale = 0.01 / maxExtent;
        }
        const translation = new THREE.Matrix4().makeTranslation(-center.x, -center.y, -center.z);
        let transform = translation.clone();
        if (Math.abs(scale - 1) > 1e-6) {
          const scaling = new THREE.Matrix4().makeScale(scale, scale, scale);
          transform = scaling.multiply(transform);
        }
        const sanitized = applyTransformToGeometry(entry, transform);
        const restore = transform.clone();
        if (Math.abs(scale - 1) > 1e-6 || center.lengthSq() > EPSILON) {
          restore.invert();
        } else {
          restore.identity();
        }
        const reinstated = applyTransformToGeometry(sanitized, restore);
        sane.push(sanitized);
        wacky.push(reinstated);
      }
      return { geometry: sane, wackometry: wacky };
    },
  });

  register(['{378d0690-9da0-4dd1-ab16-1d15246e7c22}', 'orient'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', A: 'source', Source: 'source', B: 'target', Target: 'target' },
      outputs: { G: 'geometry', geometry: 'geometry', X: 'transform', transform: 'transform' },
    },
    eval: ({ inputs }) => {
      const transform = matrixFromPlaneToPlane(inputs.source, inputs.target);
      const geometry = applyTransformToGeometry(inputs.geometry, transform);
      return { geometry, transform: transform.clone() };
    },
  });

  register(['{3ac8e589-37f5-477d-aa61-6699702c5728}', 'rotate axis', 'rotax'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', A: 'angle', Angle: 'angle', X: 'axis', Axis: 'axis' },
      outputs: { G: 'geometry', geometry: 'geometry', X: 'transform', transform: 'transform' },
    },
    eval: ({ inputs }) => {
      const angle = toNumber(inputs.angle, 0);
      const axisLine = ensureLine(inputs.axis);
      const transform = createAxisRotationMatrix(axisLine.start, axisLine.direction, angle);
      const geometry = applyTransformToGeometry(inputs.geometry, transform);
      return { geometry, transform: transform.clone() };
    },
  });

  register(['{3dfb9a77-6e05-4016-9f20-94f78607d672}', 'rotate 3d', 'rot3d'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', A: 'angle', Angle: 'angle', C: 'center', Center: 'center', X: 'axis', Axis: 'axis' },
      outputs: { G: 'geometry', geometry: 'geometry', X: 'transform', transform: 'transform' },
    },
    eval: ({ inputs }) => {
      const angle = toNumber(inputs.angle, 0);
      const center = ensurePoint(inputs.center, new THREE.Vector3());
      const axis = ensureDirection(inputs.axis, new THREE.Vector3(0, 0, 1));
      const transform = createAxisRotationMatrix(center, axis, angle);
      const geometry = applyTransformToGeometry(inputs.geometry, transform);
      return { geometry, transform: transform.clone() };
    },
  });

  register(['{4fe87ef8-49e4-4605-9859-87940d62e1de}', 'move to plane', 'movetoplane'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', P: 'plane', Plane: 'plane', A: 'above', Above: 'above', B: 'below', Below: 'below' },
      outputs: { G: 'geometry', geometry: 'geometry', X: 'transform', transform: 'transform' },
    },
    eval: ({ inputs }) => {
      const plane = ensurePlane(inputs.plane);
      const above = ensureBoolean(inputs.above, true);
      const below = ensureBoolean(inputs.below, true);
      const centroid = computeCentroid(inputs.geometry) ?? plane.origin.clone();
      const toOrigin = centroid.clone().sub(plane.origin);
      const distance = toOrigin.dot(plane.zAxis);
      let translation = new THREE.Vector3();
      if (distance > EPSILON && above) {
        translation = plane.zAxis.clone().multiplyScalar(-distance);
      } else if (distance < -EPSILON && below) {
        translation = plane.zAxis.clone().multiplyScalar(-distance);
      }
      if (translation.lengthSq() < EPSILON) {
        const geometry = applyTransformToGeometry(inputs.geometry, identityMatrix());
        return { geometry, transform: identityMatrix() };
      }
      const transform = new THREE.Matrix4().makeTranslation(translation.x, translation.y, translation.z);
      const geometry = applyTransformToGeometry(inputs.geometry, transform);
      return { geometry, transform: transform.clone() };
    },
  });

  register(['{55959599-0b44-4333-8427-a73564ea7ffb}', 'rotate axis'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', A: 'angle', Angle: 'angle', X: 'axis', Axis: 'axis' },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const angle = toNumber(inputs.angle, 0);
      const axisLine = ensureLine(inputs.axis);
      const transform = createAxisRotationMatrix(axisLine.start, axisLine.direction, angle);
      return { geometry: applyTransformToGeometry(inputs.geometry, transform) };
    },
  });

  register(['{5edaea74-32cb-4586-bd72-66694eb73160}', 'rotate direction'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', C: 'center', Center: 'center', F: 'from', From: 'from', T: 'to', To: 'to' },
      outputs: { G: 'geometry', geometry: 'geometry', X: 'transform', transform: 'transform' },
    },
    eval: ({ inputs }) => {
      const transform = createDirectionRotationMatrix(inputs.center, inputs.from, inputs.to);
      const geometry = applyTransformToGeometry(inputs.geometry, transform);
      return { geometry, transform: transform.clone() };
    },
  });

  register(['{955d887b-c83b-4c61-bf35-df5d4c4abd9b}', 'rotate 3d'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', A: 'angle', Angle: 'angle', C: 'center', Center: 'center', X: 'axis', Axis: 'axis' },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const angle = toNumber(inputs.angle, 0);
      const center = ensurePoint(inputs.center, new THREE.Vector3());
      const axis = ensureDirection(inputs.axis, new THREE.Vector3(0, 0, 1));
      const transform = createAxisRotationMatrix(center, axis, angle);
      return { geometry: applyTransformToGeometry(inputs.geometry, transform) };
    },
  });

  register(['{a35811bc-1034-4491-acb8-608a8cfa27b1}', 'orient'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', A: 'initial', Initial: 'initial', B: 'final', Final: 'final' },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const transform = matrixFromPlaneToPlane(inputs.initial, inputs.final);
      return { geometry: applyTransformToGeometry(inputs.geometry, transform) };
    },
  });

  register(['{a70bdac1-1ed2-40d3-b687-3437bc150af0}', 'mirror'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', P: 'plane', Plane: 'plane' },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const transform = createMirrorMatrix(inputs.plane);
      return { geometry: applyTransformToGeometry(inputs.geometry, transform) };
    },
  });

  register(['{b40f28a2-ba30-4ac2-afe5-a6ece7f985fc}', 'move'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', T: 'translation', Translation: 'translation', Motion: 'translation' },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const translation = ensureVector(inputs.translation, new THREE.Vector3());
      if (translation.lengthSq() < EPSILON) {
        return { geometry: applyTransformToGeometry(inputs.geometry, identityMatrix()) };
      }
      const transform = new THREE.Matrix4().makeTranslation(translation.x, translation.y, translation.z);
      return { geometry: applyTransformToGeometry(inputs.geometry, transform) };
    },
  });

  register(['{b661519d-43fd-4e5a-b244-d54d9fae2bde}', 'rotate'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', A: 'angle', Angle: 'angle', P: 'plane', Plane: 'plane' },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const plane = ensurePlane(inputs.plane);
      const angle = toNumber(inputs.angle, 0);
      const transform = createAxisRotationMatrix(plane.origin, plane.zAxis, angle);
      return { geometry: applyTransformToGeometry(inputs.geometry, transform) };
    },
  });

  register(['{b7798b74-037e-4f0c-8ac7-dc1043d093e0}', 'rotate'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', A: 'angle', Angle: 'angle', P: 'plane', Plane: 'plane' },
      outputs: { G: 'geometry', geometry: 'geometry', X: 'transform', transform: 'transform' },
    },
    eval: ({ inputs }) => {
      const plane = ensurePlane(inputs.plane);
      const angle = toNumber(inputs.angle, 0);
      const transform = createAxisRotationMatrix(plane.origin, plane.zAxis, angle);
      const geometry = applyTransformToGeometry(inputs.geometry, transform);
      return { geometry, transform: transform.clone() };
    },
  });

  register(['{bef50d22-e6b3-45fd-b7be-1c501502186b}', 'rotate axis'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', A: 'angle', Angle: 'angle', S: 'start', Start: 'start', E: 'end', End: 'end' },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const angle = toNumber(inputs.angle, 0);
      const start = ensurePoint(inputs.start, new THREE.Vector3());
      const end = ensurePoint(inputs.end, start.clone().add(new THREE.Vector3(0, 0, 1)));
      const direction = end.clone().sub(start);
      const transform = createAxisRotationMatrix(start, direction, angle);
      return { geometry: applyTransformToGeometry(inputs.geometry, transform) };
    },
  });

  register(['{dd9f597a-4db0-42b1-9cb2-5607ec97db09}', 'move away from', 'moveaway'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', E: 'emitter', Emitter: 'emitter', D: 'distance', Distance: 'distance' },
      outputs: { G: 'geometry', geometry: 'geometry', X: 'transform', transform: 'transform' },
    },
    eval: ({ inputs }) => {
      const distance = toNumber(inputs.distance, 0);
      const geometryCentroid = computeCentroid(inputs.geometry) ?? new THREE.Vector3();
      const emitterCentroid = computeCentroid(inputs.emitter) ?? new THREE.Vector3();
      const direction = geometryCentroid.clone().sub(emitterCentroid);
      if (direction.lengthSq() < EPSILON) {
        direction.set(0, 0, 1);
      }
      direction.normalize().multiplyScalar(distance);
      if (direction.lengthSq() < EPSILON) {
        return { geometry: applyTransformToGeometry(inputs.geometry, identityMatrix()), transform: identityMatrix() };
      }
      const transform = new THREE.Matrix4().makeTranslation(direction.x, direction.y, direction.z);
      const geometry = applyTransformToGeometry(inputs.geometry, transform);
      return { geometry, transform: transform.clone() };
    },
  });

  register(['{e9eb1dcf-92f6-4d4d-84ae-96222d60f56b}', 'move'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', T: 'motion', Motion: 'motion', Translation: 'motion' },
      outputs: { G: 'geometry', geometry: 'geometry', X: 'transform', transform: 'transform' },
    },
    eval: ({ inputs }) => {
      const translation = ensureVector(inputs.motion, new THREE.Vector3());
      if (translation.lengthSq() < EPSILON) {
        const geometry = applyTransformToGeometry(inputs.geometry, identityMatrix());
        return { geometry, transform: identityMatrix() };
      }
      const transform = new THREE.Matrix4().makeTranslation(translation.x, translation.y, translation.z);
      const geometry = applyTransformToGeometry(inputs.geometry, transform);
      return { geometry, transform: transform.clone() };
    },
  });

  register(['{f12daa2f-4fd5-48c1-8ac3-5dea476912ca}', 'mirror'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', P: 'plane', Plane: 'plane' },
      outputs: { G: 'geometry', geometry: 'geometry', X: 'transform', transform: 'transform' },
    },
    eval: ({ inputs }) => {
      const transform = createMirrorMatrix(inputs.plane);
      const geometry = applyTransformToGeometry(inputs.geometry, transform);
      return { geometry, transform: transform.clone() };
    },
  });
}

export function registerTransformAffineComponents({ register, toNumber, toVector3 }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register transform components.');
  }
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register transform components.');
  }
  if (typeof toVector3 !== 'function') {
    throw new Error('toVector3 function is required to register transform components.');
  }

  const {
    EPSILON,
    identityMatrix,
    ensurePoint,
    ensureVector,
    ensureDirection,
    ensurePlane,
    createPlaneMatrix,
    applyTransformToGeometry,
    matrixFromPoints,
    matrixIsIdentity,
    createProjectionMatrix,
    extractRectangleFrame,
    rectangleToMatrix,
    extractTriangleFrame,
    triangleToMatrix,
    extractBoxFrame,
    boxToMatrix,
  } = createTransformHelpers({ toNumber, toVector3 });

  function applyResult(inputs, matrix, includeTransform) {
    const geometry = applyTransformToGeometry(inputs.geometry, matrix);
    if (includeTransform) {
      return { geometry, transform: matrix.clone() };
    }
    return { geometry };
  }

  function identityResult(inputs, includeTransform) {
    const matrix = identityMatrix();
    return applyResult(inputs, matrix, includeTransform);
  }

  function buildRectangleMappingMatrix(sourceInput, targetInput) {
    const sourceFrame = extractRectangleFrame(sourceInput);
    const targetFrame = extractRectangleFrame(targetInput);
    if (!sourceFrame || !targetFrame) {
      return null;
    }
    const sourceMatrix = rectangleToMatrix(sourceFrame);
    const targetMatrix = rectangleToMatrix(targetFrame);
    if (!sourceMatrix || !targetMatrix) {
      return null;
    }
    const inverseSource = sourceMatrix.clone();
    const determinant = inverseSource.determinant();
    if (!Number.isFinite(determinant) || Math.abs(determinant) < EPSILON) {
      return null;
    }
    inverseSource.invert();
    return targetMatrix.clone().multiply(inverseSource);
  }

  function buildTriangleMappingMatrix(sourceInput, targetInput) {
    const sourceFrame = extractTriangleFrame(sourceInput);
    const targetFrame = extractTriangleFrame(targetInput);
    if (!sourceFrame || !targetFrame) {
      return null;
    }
    const sourceMatrix = triangleToMatrix(sourceFrame);
    const targetMatrix = triangleToMatrix(targetFrame);
    if (!sourceMatrix || !targetMatrix) {
      return null;
    }
    const inverseSource = sourceMatrix.clone();
    const determinant = inverseSource.determinant();
    if (!Number.isFinite(determinant) || Math.abs(determinant) < EPSILON) {
      return null;
    }
    inverseSource.invert();
    return targetMatrix.clone().multiply(inverseSource);
  }

  function buildBoxMappingMatrix(sourceInput, targetInput) {
    const sourceFrame = extractBoxFrame(sourceInput);
    const targetFrame = extractBoxFrame(targetInput);
    if (!sourceFrame || !targetFrame) {
      return null;
    }
    const sourceMatrix = boxToMatrix(sourceFrame);
    const targetMatrix = boxToMatrix(targetFrame);
    if (!sourceMatrix || !targetMatrix) {
      return null;
    }
    const inverseSource = sourceMatrix.clone();
    const determinant = inverseSource.determinant();
    if (!Number.isFinite(determinant) || Math.abs(determinant) < EPSILON) {
      return null;
    }
    inverseSource.invert();
    return targetMatrix.clone().multiply(inverseSource);
  }

  function createShearMatrix(planeInput, gripInput, targetInput) {
    const plane = ensurePlane(planeInput);
    const grip = ensurePoint(gripInput, plane.origin.clone());
    const target = ensurePoint(targetInput, grip.clone());
    const origin = plane.origin.clone();
    const basisX = origin.clone().add(plane.xAxis);
    const basisY = origin.clone().add(plane.yAxis);
    const basisZ = origin.clone().add(plane.zAxis);
    const attempts = [
      matrixFromPoints([origin, basisX, basisY, grip], [origin, basisX, basisY, target]),
      matrixFromPoints([origin, basisX, basisZ, grip], [origin, basisX, basisZ, target]),
      matrixFromPoints([origin, basisY, basisZ, grip], [origin, basisY, basisZ, target]),
    ];
    for (const candidate of attempts) {
      if (!matrixIsIdentity(candidate) || grip.distanceToSquared(target) < EPSILON) {
        return candidate;
      }
    }
    const offset = target.clone().sub(grip);
    if (offset.lengthSq() < EPSILON) {
      return identityMatrix();
    }
    return new THREE.Matrix4().makeTranslation(offset.x, offset.y, offset.z);
  }

  function createShearAngleMatrix(planeInput, angleXInput, angleYInput) {
    const plane = ensurePlane(planeInput);
    const angleX = toNumber(angleXInput, 0);
    const angleY = toNumber(angleYInput, 0);
    const shear = new THREE.Matrix4().set(
      1, 0, Math.tan(angleY), 0,
      0, 1, Math.tan(angleX), 0,
      0, 0, 1, 0,
      0, 0, 0, 1,
    );
    const orientation = createPlaneMatrix(plane);
    const inverse = orientation.clone().invert();
    return orientation.clone().multiply(shear).multiply(inverse);
  }

  function createScaleMatrix(planeInput, scaleXInput, scaleYInput, scaleZInput) {
    const plane = ensurePlane(planeInput);
    const scaleX = Number.isFinite(scaleXInput) ? scaleXInput : 1;
    const scaleY = Number.isFinite(scaleYInput) ? scaleYInput : 1;
    const scaleZ = Number.isFinite(scaleZInput) ? scaleZInput : 1;
    const orientation = createPlaneMatrix(plane);
    const inverse = orientation.clone().invert();
    const scale = new THREE.Matrix4().makeScale(scaleX, scaleY, scaleZ);
    return orientation.clone().multiply(scale).multiply(inverse);
  }

  function createUniformScaleMatrix(centerInput, factorInput) {
    const center = ensurePoint(centerInput, new THREE.Vector3());
    const factor = toNumber(factorInput, 1);
    const translateToOrigin = new THREE.Matrix4().makeTranslation(-center.x, -center.y, -center.z);
    const scale = new THREE.Matrix4().makeScale(factor, factor, factor);
    const translateBack = new THREE.Matrix4().makeTranslation(center.x, center.y, center.z);
    return translateBack.clone().multiply(scale).multiply(translateToOrigin);
  }

  function createOrientMatrix(pointAInput, directionAInput, pointBInput, directionBInput) {
    const pointA = ensurePoint(pointAInput, new THREE.Vector3());
    const pointB = ensurePoint(pointBInput, pointA.clone());
    const directionA = ensureDirection(directionAInput, new THREE.Vector3(1, 0, 0));
    const directionB = ensureDirection(directionBInput, directionA.clone());
    const quaternion = new THREE.Quaternion().setFromUnitVectors(directionA, directionB);
    const rotation = new THREE.Matrix4().makeRotationFromQuaternion(quaternion);
    const translateToOrigin = new THREE.Matrix4().makeTranslation(-pointA.x, -pointA.y, -pointA.z);
    const translateToTarget = new THREE.Matrix4().makeTranslation(pointB.x, pointB.y, pointB.z);
    return translateToTarget.clone().multiply(rotation).multiply(translateToOrigin);
  }

  function buildProjectionMatrix(planeInput, directionInput) {
    const matrix = createProjectionMatrix(planeInput, directionInput);
    return matrix ?? identityMatrix();
  }

  register(['{06d7bc4a-ba3e-4445-8ab5-079613b52f28}', 'project along', 'projecta'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', P: 'plane', Plane: 'plane', D: 'direction', Direction: 'direction' },
      outputs: { G: 'geometry', geometry: 'geometry', X: 'transform', transform: 'transform' },
    },
    eval: ({ inputs }) => {
      const matrix = buildProjectionMatrix(inputs.plane, inputs.direction);
      return applyResult(inputs, matrix, true);
    },
  });

  register(['{1602b2cc-007c-4b79-8926-0067c6184e44}', 'orient direction', 'orient'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        'Point A': 'pointA',
        pA: 'pointA',
        A: 'pointA',
        'Direction A': 'directionA',
        dA: 'directionA',
        'Point B': 'pointB',
        pB: 'pointB',
        B: 'pointB',
        'Direction B': 'directionB',
        dB: 'directionB',
      },
      outputs: { G: 'geometry', geometry: 'geometry', X: 'transform', transform: 'transform' },
    },
    eval: ({ inputs }) => {
      const matrix = createOrientMatrix(inputs.pointA, inputs.directionA, inputs.pointB, inputs.directionB);
      return applyResult(inputs, matrix, true);
    },
  });

  register(['{17d40004-489e-42d9-ad10-857f7b436801}', 'rectangle mapping', 'recmap'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', S: 'source', Source: 'source', T: 'target', Target: 'target' },
      outputs: { G: 'geometry', geometry: 'geometry', X: 'transform', transform: 'transform' },
    },
    eval: ({ inputs }) => {
      const matrix = buildRectangleMappingMatrix(inputs.source, inputs.target);
      if (!matrix) {
        return identityResult(inputs, true);
      }
      return applyResult(inputs, matrix, true);
    },
  });

  register(['{23285717-156c-468f-a691-b242488c06a6}', 'project'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', P: 'plane', Plane: 'plane' },
      outputs: { G: 'geometry', geometry: 'geometry', X: 'transform', transform: 'transform' },
    },
    eval: ({ inputs }) => {
      const matrix = buildProjectionMatrix(inputs.plane);
      return applyResult(inputs, matrix, true);
    },
  });

  register(['{24e913c9-7530-436d-b81d-bc3aa27296a4}', 'project'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', P: 'plane', Plane: 'plane' },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const matrix = buildProjectionMatrix(inputs.plane);
      return applyResult(inputs, matrix, false);
    },
  });

  register(['{290f418a-65ee-406a-a9d0-35699815b512}', 'scale nu', 'scale nu'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        P: 'plane',
        Plane: 'plane',
        X: 'scaleX',
        'Scale X': 'scaleX',
        Y: 'scaleY',
        'Scale Y': 'scaleY',
        Z: 'scaleZ',
        'Scale Z': 'scaleZ',
      },
      outputs: { G: 'geometry', geometry: 'geometry', X: 'transform', transform: 'transform' },
    },
    eval: ({ inputs }) => {
      const scaleX = toNumber(inputs.scaleX, 1);
      const scaleY = toNumber(inputs.scaleY, 1);
      const scaleZ = toNumber(inputs.scaleZ, 1);
      const matrix = createScaleMatrix(inputs.plane, scaleX, scaleY, scaleZ);
      return applyResult(inputs, matrix, true);
    },
  });

  register(['{3ae3a462-38fb-4d49-9f86-7558dfed7c3e}', 'shear'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        P: 'plane',
        Base: 'plane',
        Grip: 'grip',
        grip: 'grip',
        Target: 'target',
        T: 'target',
      },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const matrix = createShearMatrix(inputs.plane, inputs.grip, inputs.target);
      return applyResult(inputs, matrix, false);
    },
  });

  register(['{4041be93-6746-4cdb-aa95-929bff544fb0}', 'orient direction', 'orient'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        'Point A': 'pointA',
        pA: 'pointA',
        A: 'pointA',
        'Direction A': 'directionA',
        dA: 'directionA',
        'Point B': 'pointB',
        pB: 'pointB',
        B: 'pointB',
        'Direction B': 'directionB',
        dB: 'directionB',
      },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const matrix = createOrientMatrix(inputs.pointA, inputs.directionA, inputs.pointB, inputs.directionB);
      return applyResult(inputs, matrix, false);
    },
  });

  register(['{407e35c6-7c40-4652-bd80-fde1eb7ec034}', 'camera obscura', 'co'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', P: 'point', Point: 'point', F: 'factor', Factor: 'factor' },
      outputs: { G: 'geometry', geometry: 'geometry', X: 'transform', transform: 'transform' },
    },
    eval: ({ inputs }) => {
      const point = ensurePoint(inputs.point, new THREE.Vector3());
      const factor = toNumber(inputs.factor, 1);
      const translateToOrigin = new THREE.Matrix4().makeTranslation(-point.x, -point.y, -point.z);
      const scale = new THREE.Matrix4().makeScale(-factor, -factor, -factor);
      const translateBack = new THREE.Matrix4().makeTranslation(point.x, point.y, point.z);
      const matrix = translateBack.clone().multiply(scale).multiply(translateToOrigin);
      return applyResult(inputs, matrix, true);
    },
  });

  register(['{4d2a06bd-4b0f-4c65-9ee0-4220e4c01703}', 'scale'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', C: 'center', Center: 'center', F: 'factor', Factor: 'factor' },
      outputs: { G: 'geometry', geometry: 'geometry', X: 'transform', transform: 'transform' },
    },
    eval: ({ inputs }) => {
      const matrix = createUniformScaleMatrix(inputs.center, inputs.factor);
      return applyResult(inputs, matrix, true);
    },
  });

  register(['{4f0dfac8-6c61-40ef-ad41-aad84533f382}', 'scale'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', C: 'center', Center: 'center', F: 'factor', Factor: 'factor' },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const matrix = createUniformScaleMatrix(inputs.center, inputs.factor);
      return applyResult(inputs, matrix, false);
    },
  });

  register(['{5a27203a-e05f-4eea-b80f-a5f29a00fdf2}', 'shear'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        P: 'plane',
        Base: 'plane',
        Grip: 'grip',
        grip: 'grip',
        Target: 'target',
        T: 'target',
      },
      outputs: { G: 'geometry', geometry: 'geometry', X: 'transform', transform: 'transform' },
    },
    eval: ({ inputs }) => {
      const matrix = createShearMatrix(inputs.plane, inputs.grip, inputs.target);
      return applyResult(inputs, matrix, true);
    },
  });

  register(['{61d81100-c4d3-462d-8b51-d951c0ae32db}', 'triangle mapping', 'trimap'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', S: 'source', Source: 'source', T: 'target', Target: 'target' },
      outputs: { G: 'geometry', geometry: 'geometry', X: 'transform', transform: 'transform' },
    },
    eval: ({ inputs }) => {
      const matrix = buildTriangleMappingMatrix(inputs.source, inputs.target);
      if (!matrix) {
        return identityResult(inputs, true);
      }
      return applyResult(inputs, matrix, true);
    },
  });

  register(['{7753fb03-c1f1-4dbe-8557-f01e23aa3b20}', 'scale nu', 'scale nu'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        P: 'plane',
        Plane: 'plane',
        X: 'scaleX',
        Y: 'scaleY',
        Z: 'scaleZ',
      },
      outputs: { G: 'geometry', geometry: 'geometry', X: 'transform', transform: 'transform' },
    },
    eval: ({ inputs }) => {
      const scaleX = toNumber(inputs.scaleX, 1);
      const scaleY = toNumber(inputs.scaleY, 1);
      const scaleZ = toNumber(inputs.scaleZ, 1);
      const matrix = createScaleMatrix(inputs.plane, scaleX, scaleY, scaleZ);
      return applyResult(inputs, matrix, true);
    },
  });

  register(['{77bfb6a1-0305-4645-b309-cd6dbf1205d7}', 'shear angle', 'shear'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        P: 'plane',
        Base: 'plane',
        'Angle X': 'angleX',
        Ax: 'angleX',
        'Angle Y': 'angleY',
        Ay: 'angleY',
      },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const matrix = createShearAngleMatrix(inputs.plane, inputs.angleX, inputs.angleY);
      return applyResult(inputs, matrix, false);
    },
  });

  register(['{8465bcce-9e0a-4cf4-bbda-1a7ce5681e10}', 'box mapping', 'boxmap'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', S: 'source', Source: 'source', T: 'target', Target: 'target' },
      outputs: { G: 'geometry', geometry: 'geometry', X: 'transform', transform: 'transform' },
    },
    eval: ({ inputs }) => {
      const matrix = buildBoxMappingMatrix(inputs.source, inputs.target);
      if (!matrix) {
        return identityResult(inputs, true);
      }
      return applyResult(inputs, matrix, true);
    },
  });

  register(['{9025f4ca-159f-4c54-958b-0aad379dae77}', 'project'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', P: 'plane', Plane: 'plane' },
      outputs: { G: 'geometry', geometry: 'geometry', X: 'transform', transform: 'transform' },
    },
    eval: ({ inputs }) => {
      const matrix = buildProjectionMatrix(inputs.plane);
      return applyResult(inputs, matrix, true);
    },
  });

  register(['{f19ee36c-f21f-4e25-be4c-4ca4b30eda0d}', 'shear angle', 'shear'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        P: 'plane',
        Base: 'plane',
        'Angle X': 'angleX',
        Ax: 'angleX',
        'Angle Y': 'angleY',
        Ay: 'angleY',
      },
      outputs: { G: 'geometry', geometry: 'geometry', X: 'transform', transform: 'transform' },
    },
    eval: ({ inputs }) => {
      const matrix = createShearAngleMatrix(inputs.plane, inputs.angleX, inputs.angleY);
      return applyResult(inputs, matrix, true);
    },
  });
}

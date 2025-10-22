import * as THREE from 'three';

function createTransformHelpers({ toNumber, toVector3 }) {
  const EPSILON = 1e-9;
  const SAMPLE_LIMIT = 2048;
  const CANONICAL_TWISTED_BOX_CORNERS = [
    new THREE.Vector3(0, 0, 0),
    new THREE.Vector3(1, 0, 0),
    new THREE.Vector3(1, 1, 0),
    new THREE.Vector3(0, 1, 0),
    new THREE.Vector3(0, 0, 1),
    new THREE.Vector3(1, 0, 1),
    new THREE.Vector3(1, 1, 1),
    new THREE.Vector3(0, 1, 1),
  ];

  function identityMatrix() {
    const matrix = new THREE.Matrix4();
    matrix.identity();
    return matrix;
  }

  const TRANSFORM_METADATA = new WeakMap();
  const MATRIX_CLONE_PATCH = Symbol('ghx:matrixClonePatch');

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

  function cloneTransformMetadata(metadata) {
    if (!metadata) {
      return undefined;
    }
    const cloned = {};
    if (Array.isArray(metadata.fragments) && metadata.fragments.length) {
      cloned.fragments = metadata.fragments
        .map((fragment) => ensureMatrix4(fragment, identityMatrix()))
        .filter((fragment) => fragment);
    }
    return cloned;
  }

  function cloneMatrixWithMetadata() {
    const cloned = THREE.Matrix4.prototype.clone.call(this);
    const metadata = TRANSFORM_METADATA.get(this);
    if (metadata) {
      const clonedMetadata = cloneTransformMetadata(metadata);
      if (clonedMetadata && Object.keys(clonedMetadata).length) {
        setTransformMetadata(cloned, clonedMetadata);
      }
    }
    return cloned;
  }

  function setTransformMetadata(matrix, metadata) {
    if (!matrix?.isMatrix4) {
      return matrix;
    }
    if (metadata && typeof metadata === 'object' && Object.keys(metadata).length) {
      TRANSFORM_METADATA.set(matrix, metadata);
    } else {
      TRANSFORM_METADATA.delete(matrix);
    }
    if (!matrix[MATRIX_CLONE_PATCH]) {
      Object.defineProperty(matrix, MATRIX_CLONE_PATCH, {
        value: true,
        enumerable: false,
        configurable: false,
      });
      matrix.clone = cloneMatrixWithMetadata;
    }
    return matrix;
  }

  function getTransformMetadata(matrix) {
    if (!matrix?.isMatrix4) {
      return null;
    }
    return TRANSFORM_METADATA.get(matrix) ?? null;
  }

  function matrixFromArrayLike(arrayLike) {
    if (!arrayLike || arrayLike.length < 16) {
      return null;
    }
    const elements = [];
    for (let i = 0; i < 16; i += 1) {
      const numeric = toNumber(arrayLike[i], Number.NaN);
      if (!Number.isFinite(numeric)) {
        return null;
      }
      elements.push(numeric);
    }
    const matrix = new THREE.Matrix4();
    matrix.fromArray(elements);
    return matrix;
  }

  function ensureMatrix4(input, fallback = identityMatrix()) {
    if (input === undefined || input === null) {
      return fallback ? fallback.clone() : null;
    }
    if (input?.isMatrix4) {
      const matrix = THREE.Matrix4.prototype.clone.call(input);
      const metadata = getTransformMetadata(input);
      if (metadata) {
        const clonedMetadata = cloneTransformMetadata(metadata);
        if (clonedMetadata && Object.keys(clonedMetadata).length) {
          setTransformMetadata(matrix, clonedMetadata);
        }
      }
      return matrix;
    }
    if (typeof ArrayBuffer !== 'undefined' && typeof ArrayBuffer.isView === 'function' && ArrayBuffer.isView(input)) {
      const matrix = matrixFromArrayLike(input);
      if (matrix) {
        return matrix;
      }
    }
    if (Array.isArray(input)) {
      if (input.length === 16) {
        const matrix = matrixFromArrayLike(input);
        if (matrix) {
          return matrix;
        }
      }
      if (input.length === 4 && input.every((row) => Array.isArray(row) && row.length >= 4)) {
        const elements = [];
        for (let r = 0; r < 4; r += 1) {
          for (let c = 0; c < 4; c += 1) {
            const numeric = toNumber(input[r][c], Number.NaN);
            if (!Number.isFinite(numeric)) {
              return fallback ? fallback.clone() : null;
            }
            elements.push(numeric);
          }
        }
        const matrix = matrixFromArrayLike(elements);
        if (matrix) {
          return matrix;
        }
      }
      if (input.length === 1) {
        return ensureMatrix4(input[0], fallback);
      }
    }
    if (typeof input === 'object') {
      if (input === null) {
        return fallback ? fallback.clone() : null;
      }
      if (input.matrix || input.Matrix || input.transform || input.Transform) {
        const candidate = input.matrix ?? input.Matrix ?? input.transform ?? input.Transform;
        const matrix = ensureMatrix4(candidate, fallback);
        if (matrix && (input.fragments || input.transforms || input.values)) {
          const fragments = collectTransforms(input.fragments ?? input.transforms ?? input.values);
          if (fragments.length) {
            setTransformMetadata(matrix, { fragments: fragments.map((fragment) => fragment.clone()) });
          }
        }
        return matrix;
      }
      if ('value' in input) {
        return ensureMatrix4(input.value, fallback);
      }
      if ('values' in input) {
        return ensureMatrix4(input.values, fallback);
      }
      if (Array.isArray(input.elements) && input.elements.length >= 16) {
        const matrix = matrixFromArrayLike(input.elements);
        if (matrix) {
          return matrix;
        }
      }
      if (Array.isArray(input.data) && input.data.length >= 16) {
        const matrix = matrixFromArrayLike(input.data);
        if (matrix) {
          return matrix;
        }
      }
      if (
        'position' in input ||
        'translation' in input ||
        'quaternion' in input ||
        'rotation' in input ||
        'scale' in input ||
        'scaling' in input
      ) {
        const position = toVector3(
          input.position ?? input.translation ?? input.translate ?? input.origin ?? new THREE.Vector3(),
          new THREE.Vector3(),
        );
        let quaternion = null;
        if (input.quaternion?.isQuaternion) {
          quaternion = input.quaternion.clone();
        } else if (input.rotation?.isQuaternion) {
          quaternion = input.rotation.clone();
        }
        const rotationInput = input.rotation ?? input.euler ?? input.angles ?? null;
        if (!quaternion) {
          if (rotationInput?.isEuler) {
            quaternion = new THREE.Quaternion().setFromEuler(rotationInput);
          } else if (Array.isArray(rotationInput)) {
            const [rx, ry, rz, order] = rotationInput;
            const euler = new THREE.Euler(
              toNumber(rx, 0),
              toNumber(ry, 0),
              toNumber(rz, 0),
              typeof order === 'string' ? order : 'XYZ',
            );
            quaternion = new THREE.Quaternion().setFromEuler(euler);
          } else if (rotationInput && typeof rotationInput === 'object') {
            const euler = new THREE.Euler(
              toNumber(rotationInput.x ?? rotationInput[0], 0),
              toNumber(rotationInput.y ?? rotationInput[1], 0),
              toNumber(rotationInput.z ?? rotationInput[2], 0),
              rotationInput.order ?? 'XYZ',
            );
            quaternion = new THREE.Quaternion().setFromEuler(euler);
          } else if (Number.isFinite(rotationInput)) {
            quaternion = new THREE.Quaternion().setFromEuler(new THREE.Euler(0, 0, toNumber(rotationInput, 0)));
          }
        }
        if (!quaternion) {
          quaternion = new THREE.Quaternion();
        }
        const scaleInput = input.scale ?? input.scaling ?? input.size ?? input.dimensions;
        let scale = null;
        if (scaleInput?.isVector3) {
          scale = scaleInput.clone();
        } else if (Array.isArray(scaleInput)) {
          scale = new THREE.Vector3(
            toNumber(scaleInput[0], 1),
            toNumber(scaleInput[1], 1),
            toNumber(scaleInput[2], 1),
          );
        } else if (typeof scaleInput === 'object' && scaleInput) {
          scale = new THREE.Vector3(
            toNumber(scaleInput.x ?? scaleInput.width ?? scaleInput[0], 1),
            toNumber(scaleInput.y ?? scaleInput.height ?? scaleInput[1], 1),
            toNumber(scaleInput.z ?? scaleInput.depth ?? scaleInput[2], 1),
          );
        } else if (Number.isFinite(scaleInput)) {
          const uniform = toNumber(scaleInput, 1);
          scale = new THREE.Vector3(uniform, uniform, uniform);
        }
        if (!scale) {
          scale = new THREE.Vector3(1, 1, 1);
        }
        const matrix = new THREE.Matrix4();
        matrix.compose(position, quaternion, scale);
        return matrix;
      }
    }
    if (Number.isFinite(input)) {
      const factor = toNumber(input, 1);
      const matrix = new THREE.Matrix4().makeScale(factor, factor, factor);
      return matrix;
    }
    return fallback ? fallback.clone() : null;
  }

  function collectTransforms(input, visited = new Set()) {
    if (input === undefined || input === null) {
      return [];
    }
    if (input?.isMatrix4) {
      return [ensureMatrix4(input)];
    }
    if (typeof ArrayBuffer !== 'undefined' && typeof ArrayBuffer.isView === 'function' && ArrayBuffer.isView(input)) {
      const matrix = ensureMatrix4(input, null);
      return matrix ? [matrix] : [];
    }
    if (Array.isArray(input)) {
      if (visited.has(input)) {
        return [];
      }
      visited.add(input);
      const result = [];
      for (const entry of input) {
        const transforms = collectTransforms(entry, visited);
        result.push(...transforms);
      }
      visited.delete(input);
      return result;
    }
    if (typeof input === 'object') {
      if (visited.has(input)) {
        return [];
      }
      visited.add(input);
      try {
        if (input.matrix || input.Matrix || input.transform || input.Transform) {
          const candidate = input.matrix ?? input.Matrix ?? input.transform ?? input.Transform;
          const matrix = ensureMatrix4(candidate, null);
          if (!matrix) {
            return [];
          }
          if (input.fragments || input.transforms || input.values) {
            const fragments = collectTransforms(input.fragments ?? input.transforms ?? input.values, visited);
            if (fragments.length) {
              setTransformMetadata(matrix, { fragments: fragments.map((fragment) => fragment.clone()) });
            }
          }
          return [matrix];
        }
        if ('value' in input) {
          return collectTransforms(input.value, visited);
        }
        if ('values' in input) {
          return collectTransforms(input.values, visited);
        }
      } finally {
        visited.delete(input);
      }
    }
    const matrix = ensureMatrix4(input, null);
    return matrix ? [matrix] : [];
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

  function createTwistedBox(cornersInput = {}) {
    const labels = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'];
    const corners = {};
    const list = [];
    for (let i = 0; i < labels.length; i += 1) {
      const label = labels[i];
      const lower = label.toLowerCase();
      const value = Array.isArray(cornersInput)
        ? cornersInput[i]
        : cornersInput[label] ?? cornersInput[lower];
      const point = ensurePoint(value, CANONICAL_TWISTED_BOX_CORNERS[i].clone());
      corners[label] = point;
      list.push(point.clone());
    }
    const bottomCorners = [corners.A.clone(), corners.B.clone(), corners.C.clone(), corners.D.clone()];
    const topCorners = [corners.E.clone(), corners.F.clone(), corners.G.clone(), corners.H.clone()];
    const center = list.reduce((sum, point) => sum.add(point), new THREE.Vector3()).multiplyScalar(1 / list.length);
    const planes = {
      bottom: planeFromPoints(bottomCorners[0], bottomCorners[1], bottomCorners[3]),
      top: planeFromPoints(topCorners[0], topCorners[1], topCorners[3]),
    };
    return {
      type: 'twisted-box',
      corners,
      bottomCorners,
      topCorners,
      points: list,
      center,
      planes,
    };
  }

  function isTwistedBox(value) {
    return Boolean(value && value.type === 'twisted-box' && value.corners);
  }

  function ensureTwistedBox(input) {
    if (!input) {
      return createTwistedBox();
    }
    if (isTwistedBox(input)) {
      return createTwistedBox(input.corners);
    }
    if (input.corners) {
      return createTwistedBox(input.corners);
    }
    if (Array.isArray(input) && input.length >= 8) {
      return createTwistedBox(input);
    }
    if (typeof input === 'object') {
      const labels = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'];
      const corners = {};
      let found = false;
      for (let i = 0; i < labels.length; i += 1) {
        const label = labels[i];
        const lower = label.toLowerCase();
        const value = input[label] ?? input[lower] ?? input[i];
        if (value !== undefined) {
          found = true;
        }
        corners[label] = value;
      }
      if (found) {
        return createTwistedBox(corners);
      }
    }
    return createTwistedBox();
  }

  function bilinearInterpolation(corners, u, v) {
    if (!Array.isArray(corners) || corners.length < 4) {
      return new THREE.Vector3();
    }
    const ab = corners[0].clone().lerp(corners[1], THREE.MathUtils.clamp(u, 0, 1));
    const dc = corners[3].clone().lerp(corners[2], THREE.MathUtils.clamp(u, 0, 1));
    return ab.lerp(dc, THREE.MathUtils.clamp(v, 0, 1));
  }

  function bilinearDerivativeU(corners, u, v) {
    const clampedV = THREE.MathUtils.clamp(v, 0, 1);
    const term1 = corners[1].clone().sub(corners[0]).multiplyScalar(1 - clampedV);
    const term2 = corners[2].clone().sub(corners[3]).multiplyScalar(clampedV);
    return term1.add(term2);
  }

  function bilinearDerivativeV(corners, u) {
    const clampedU = THREE.MathUtils.clamp(u, 0, 1);
    const ab = corners[1].clone().sub(corners[0]);
    const dc = corners[2].clone().sub(corners[3]);
    const base = corners[3].clone().sub(corners[0]);
    const adjustment = dc.sub(ab).multiplyScalar(clampedU);
    return base.add(adjustment);
  }

  function evaluateTwistedBox(boxInput, u, v, w) {
    const box = ensureTwistedBox(boxInput);
    const clampedU = THREE.MathUtils.clamp(u, 0, 1);
    const clampedV = THREE.MathUtils.clamp(v, 0, 1);
    const clampedW = THREE.MathUtils.clamp(w, 0, 1);
    const bottom = bilinearInterpolation(box.bottomCorners, clampedU, clampedV);
    const top = bilinearInterpolation(box.topCorners, clampedU, clampedV);
    return bottom.lerp(top, clampedW);
  }

  function twistedBoxDerivatives(boxInput, u, v, w) {
    const box = ensureTwistedBox(boxInput);
    const clampedU = THREE.MathUtils.clamp(u, 0, 1);
    const clampedV = THREE.MathUtils.clamp(v, 0, 1);
    const clampedW = THREE.MathUtils.clamp(w, 0, 1);
    const bottom = bilinearInterpolation(box.bottomCorners, clampedU, clampedV);
    const top = bilinearInterpolation(box.topCorners, clampedU, clampedV);
    const dBottomDU = bilinearDerivativeU(box.bottomCorners, clampedU, clampedV);
    const dTopDU = bilinearDerivativeU(box.topCorners, clampedU, clampedV);
    const dBottomDV = bilinearDerivativeV(box.bottomCorners, clampedU);
    const dTopDV = bilinearDerivativeV(box.topCorners, clampedU);
    const du = dBottomDU.clone().multiplyScalar(1 - clampedW).add(dTopDU.clone().multiplyScalar(clampedW));
    const dv = dBottomDV.clone().multiplyScalar(1 - clampedW).add(dTopDV.clone().multiplyScalar(clampedW));
    const dw = top.clone().sub(bottom);
    const point = bottom.clone().lerp(top, clampedW);
    return { point, du, dv, dw, bottom, top };
  }

  function invertTwistedBox(boxInput, pointInput, initialGuess = {}) {
    const box = ensureTwistedBox(boxInput);
    const point = ensurePoint(pointInput, new THREE.Vector3());
    const canonicalBasis = [
      CANONICAL_TWISTED_BOX_CORNERS[0],
      CANONICAL_TWISTED_BOX_CORNERS[1],
      CANONICAL_TWISTED_BOX_CORNERS[3],
      CANONICAL_TWISTED_BOX_CORNERS[4],
    ];
    const actualBasis = [
      box.corners.A,
      box.corners.B,
      box.corners.D,
      box.corners.E,
    ];
    const toCanonical = matrixFromPoints(actualBasis, canonicalBasis);
    const estimated = point.clone().applyMatrix4(toCanonical);
    let u = Number.isFinite(initialGuess.u) ? initialGuess.u : estimated.x;
    let v = Number.isFinite(initialGuess.v) ? initialGuess.v : estimated.y;
    let w = Number.isFinite(initialGuess.w) ? initialGuess.w : estimated.z;
    if (!Number.isFinite(u)) u = 0.5;
    if (!Number.isFinite(v)) v = 0.5;
    if (!Number.isFinite(w)) w = 0.5;
    let success = false;
    let iterations = 0;
    for (; iterations < 25; iterations += 1) {
      const { point: evalPoint, du, dv, dw } = twistedBoxDerivatives(box, u, v, w);
      const error = evalPoint.clone().sub(point);
      if (error.lengthSq() < 1e-12) {
        success = true;
        break;
      }
      const jacobian = new THREE.Matrix3().set(
        du.x, dv.x, dw.x,
        du.y, dv.y, dw.y,
        du.z, dv.z, dw.z,
      );
      const determinant = jacobian.determinant();
      if (!Number.isFinite(determinant) || Math.abs(determinant) < EPSILON) {
        break;
      }
      const inverse = jacobian.clone().invert();
      const delta = error.clone().applyMatrix3(inverse);
      if (!Number.isFinite(delta.x) || !Number.isFinite(delta.y) || !Number.isFinite(delta.z)) {
        break;
      }
      u -= delta.x;
      v -= delta.y;
      w -= delta.z;
      if (Math.abs(delta.x) < 1e-9 && Math.abs(delta.y) < 1e-9 && Math.abs(delta.z) < 1e-9) {
        success = true;
        break;
      }
    }
    u = THREE.MathUtils.clamp(u, 0, 1);
    v = THREE.MathUtils.clamp(v, 0, 1);
    w = THREE.MathUtils.clamp(w, 0, 1);
    const mappedPoint = evaluateTwistedBox(box, u, v, w);
    const residual = mappedPoint.distanceTo(point);
    if (residual < 1e-6) {
      success = true;
    }
    return {
      u,
      v,
      w,
      point: mappedPoint,
      success,
      iterations,
      residual,
    };
  }

  function mapGeometryStructure(value, mapPoint, context = {}) {
    if (value === undefined || value === null) {
      return value;
    }
    if (typeof mapPoint !== 'function') {
      throw new Error('mapPoint function must be provided to map geometry structures.');
    }
    const visited = context.visited ?? new Map();
    if (typeof value === 'object' || typeof value === 'function') {
      if (visited.has(value)) {
        return visited.get(value);
      }
    }
    if (value?.isVector3) {
      return mapPoint(value.clone());
    }
    if (value?.isBufferGeometry) {
      const result = value.clone();
      visited.set(value, result);
      const position = result.getAttribute('position');
      if (position) {
        const vector = new THREE.Vector3();
        for (let i = 0; i < position.count; i += 1) {
          vector.fromBufferAttribute(position, i);
          const mapped = mapPoint(vector.clone());
          position.setXYZ(i, mapped.x, mapped.y, mapped.z);
        }
        position.needsUpdate = true;
      }
      if (typeof result.computeBoundingBox === 'function') {
        result.computeBoundingBox();
      }
      if (typeof result.computeBoundingSphere === 'function') {
        result.computeBoundingSphere();
      }
      return result;
    }
    if (value?.isGeometry) {
      const result = value.clone();
      visited.set(value, result);
      if (Array.isArray(result.vertices)) {
        result.vertices = result.vertices.map((vertex) => mapPoint(vertex.clone()));
      }
      if (typeof result.computeBoundingBox === 'function') {
        result.computeBoundingBox();
      }
      if (typeof result.computeBoundingSphere === 'function') {
        result.computeBoundingSphere();
      }
      return result;
    }
    if (value?.isPlane) {
      const origin = value.origin?.clone() ?? new THREE.Vector3();
      const xAxis = value.xAxis?.clone() ?? new THREE.Vector3(1, 0, 0);
      const yAxis = value.yAxis?.clone() ?? new THREE.Vector3(0, 1, 0);
      const zAxis = value.zAxis?.clone() ?? new THREE.Vector3(0, 0, 1);
      const mappedOrigin = mapPoint(origin.clone());
      const mappedX = mapPoint(origin.clone().add(xAxis)).sub(mappedOrigin);
      const mappedY = mapPoint(origin.clone().add(yAxis)).sub(mappedOrigin);
      const mappedZ = mapPoint(origin.clone().add(zAxis)).sub(mappedOrigin);
      return normalizePlaneAxes(mappedOrigin, mappedX, mappedY, mappedZ);
    }
    if (value?.isLine3) {
      const start = mapGeometryStructure(value.start, mapPoint, { visited });
      const end = mapGeometryStructure(value.end, mapPoint, { visited });
      const line = value.clone ? value.clone() : { start: start.clone(), end: end.clone() };
      line.start = start;
      line.end = end;
      line.delta = end.clone().sub(start);
      return line;
    }
    if (value?.isBox3) {
      const corners = [
        new THREE.Vector3(value.min.x, value.min.y, value.min.z),
        new THREE.Vector3(value.max.x, value.min.y, value.min.z),
        new THREE.Vector3(value.max.x, value.max.y, value.min.z),
        new THREE.Vector3(value.min.x, value.max.y, value.min.z),
        new THREE.Vector3(value.min.x, value.min.y, value.max.z),
        new THREE.Vector3(value.max.x, value.min.y, value.max.z),
        new THREE.Vector3(value.max.x, value.max.y, value.max.z),
        new THREE.Vector3(value.min.x, value.max.y, value.max.z),
      ];
      const mapped = corners.map((corner) => mapPoint(corner.clone()));
      const box = new THREE.Box3();
      box.setFromPoints(mapped);
      return box;
    }
    if (value?.isMesh || value?.isObject3D) {
      const result = value.clone(true);
      visited.set(value, result);
      if (value.geometry) {
        result.geometry = mapGeometryStructure(value.geometry, mapPoint, { visited });
      }
      if (Array.isArray(result.children)) {
        result.children = result.children.map((child) => mapGeometryStructure(child, mapPoint, { visited }));
      }
      if (result.position?.isVector3) {
        result.position.copy(mapPoint(result.position.clone()));
      }
      return result;
    }
    if (Array.isArray(value)) {
      const result = [];
      visited.set(value, result);
      for (const entry of value) {
        result.push(mapGeometryStructure(entry, mapPoint, { visited }));
      }
      return result;
    }
    if (typeof value === 'object') {
      if (isTwistedBox(value)) {
        const labels = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'];
        const corners = {};
        for (const label of labels) {
          corners[label] = mapGeometryStructure(value.corners[label], mapPoint, { visited });
        }
        return createTwistedBox(corners);
      }
      if ('value' in value && Object.keys(value).length === 1) {
        return mapGeometryStructure(value.value, mapPoint, { visited });
      }
      const result = Array.isArray(value) ? [] : { ...value };
      visited.set(value, result);
      if ('geometry' in value) {
        result.geometry = mapGeometryStructure(value.geometry, mapPoint, { visited });
      }
      if ('geom' in value) {
        result.geom = mapGeometryStructure(value.geom, mapPoint, { visited });
      }
      if ('mesh' in value) {
        result.mesh = mapGeometryStructure(value.mesh, mapPoint, { visited });
      }
      if ('point' in value) {
        result.point = mapGeometryStructure(value.point, mapPoint, { visited });
      }
      if ('points' in value) {
        result.points = mapGeometryStructure(value.points, mapPoint, { visited });
      }
      if ('position' in value) {
        result.position = mapGeometryStructure(value.position, mapPoint, { visited });
      }
      if ('vertices' in value) {
        result.vertices = mapGeometryStructure(value.vertices, mapPoint, { visited });
      }
      if ('start' in value) {
        result.start = mapGeometryStructure(value.start, mapPoint, { visited });
      }
      if ('end' in value) {
        result.end = mapGeometryStructure(value.end, mapPoint, { visited });
      }
      if ('center' in value) {
        result.center = mapGeometryStructure(value.center, mapPoint, { visited });
      }
      if ('origin' in value) {
        result.origin = mapGeometryStructure(value.origin, mapPoint, { visited });
      }
      if ('box3' in value) {
        result.box3 = mapGeometryStructure(value.box3, mapPoint, { visited });
      }
      if ('line' in value) {
        result.line = mapGeometryStructure(value.line, mapPoint, { visited });
      }
      return result;
    }
    return value;
  }

  function createRigidMorphMatrix(geometryInput, mapPoint) {
    if (typeof mapPoint !== 'function') {
      return null;
    }
    const centroid = computeCentroid(geometryInput);
    if (!centroid) {
      return null;
    }
    const box = computeBoundingBox(geometryInput);
    const size = box ? box.getSize(new THREE.Vector3()) : new THREE.Vector3(1, 1, 1);
    const deltaX = Math.max(size.x, EPSILON) || 1;
    const deltaY = Math.max(size.y, EPSILON) || 1;
    const deltaZ = Math.max(size.z, EPSILON) || 1;
    const basis = [
      centroid.clone(),
      centroid.clone().add(new THREE.Vector3(deltaX * 0.05, 0, 0)),
      centroid.clone().add(new THREE.Vector3(0, deltaY * 0.05, 0)),
      centroid.clone().add(new THREE.Vector3(0, 0, deltaZ * 0.05)),
    ];
    const mapped = basis.map((pt) => mapPoint(pt.clone()));
    return matrixFromPoints(basis, mapped);
  }

  function applyMorphToGeometry(geometryInput, mapPoint, { rigid = false } = {}) {
    if (!rigid) {
      return mapGeometryStructure(geometryInput, mapPoint);
    }
    const matrix = createRigidMorphMatrix(geometryInput, mapPoint);
    if (!matrix || matrixIsIdentity(matrix)) {
      return mapGeometryStructure(geometryInput, mapPoint);
    }
    return applyTransformToGeometry(geometryInput, matrix);
  }

  function ensureSurfaceEvaluator(surfaceInput) {
    if (!surfaceInput) {
      return null;
    }
    if (surfaceInput.evaluate && typeof surfaceInput.evaluate === 'function') {
      return surfaceInput;
    }
    if (surfaceInput.surface) {
      return ensureSurfaceEvaluator(surfaceInput.surface);
    }
    if (surfaceInput.geometry?.surface) {
      return ensureSurfaceEvaluator(surfaceInput.geometry.surface);
    }
    if (surfaceInput.plane) {
      const plane = ensurePlane(surfaceInput.plane);
      const domainU = extractDomain(surfaceInput.domainU ?? surfaceInput.uDomain ?? [0, 1], 0, 1);
      const domainV = extractDomain(surfaceInput.domainV ?? surfaceInput.vDomain ?? [0, 1], 0, 1);
      const evaluate = (u, v) => {
        const uValue = Number.isFinite(u) ? u : domainU.min;
        const vValue = Number.isFinite(v) ? v : domainV.min;
        return applyPlane(plane, uValue, vValue, 0);
      };
      return {
        type: 'surface',
        evaluate,
        domainU,
        domainV,
        plane,
        metadata: surfaceInput.metadata ?? {},
      };
    }
    if (Array.isArray(surfaceInput.points) && surfaceInput.points.length) {
      const rows = surfaceInput.points.length;
      const cols = Array.isArray(surfaceInput.points[0]) ? surfaceInput.points[0].length : 0;
      if (!cols) {
        return null;
      }
      const domainU = extractDomain(surfaceInput.domainU ?? surfaceInput.uDomain ?? [0, 1], 0, 1);
      const domainV = extractDomain(surfaceInput.domainV ?? surfaceInput.vDomain ?? [0, 1], 0, 1);
      const evaluate = (u, v) => {
        const normalizedU = domainU.max - domainU.min > EPSILON
          ? THREE.MathUtils.clamp((u - domainU.min) / (domainU.max - domainU.min), 0, 1)
          : 0;
        const normalizedV = domainV.max - domainV.min > EPSILON
          ? THREE.MathUtils.clamp((v - domainV.min) / (domainV.max - domainV.min), 0, 1)
          : 0;
        const uScaled = normalizedU * (cols - 1);
        const vScaled = normalizedV * (rows - 1);
        const i0 = Math.floor(uScaled);
        const i1 = Math.min(i0 + 1, cols - 1);
        const j0 = Math.floor(vScaled);
        const j1 = Math.min(j0 + 1, rows - 1);
        const fu = uScaled - i0;
        const fv = vScaled - j0;
        const p00 = ensurePoint(surfaceInput.points[j0][i0], new THREE.Vector3());
        const p01 = ensurePoint(surfaceInput.points[j0][i1], p00.clone());
        const p10 = ensurePoint(surfaceInput.points[j1][i0], p00.clone());
        const p11 = ensurePoint(surfaceInput.points[j1][i1], p00.clone());
        const a = p00.clone().lerp(p01, fu);
        const b = p10.clone().lerp(p11, fu);
        return a.lerp(b, fv);
      };
      return {
        type: 'surface',
        evaluate,
        domainU,
        domainV,
        metadata: surfaceInput.metadata ?? {},
      };
    }
    return null;
  }

  function evaluateSurfacePoint(surfaceInput, u, v) {
    const surface = ensureSurfaceEvaluator(surfaceInput);
    if (!surface) {
      return null;
    }
    const uValue = Number.isFinite(u) ? u : surface.domainU?.min ?? 0;
    const vValue = Number.isFinite(v) ? v : surface.domainV?.min ?? 0;
    const point = surface.evaluate(uValue, vValue);
    return ensurePoint(point, new THREE.Vector3());
  }

  function surfaceNormal(surfaceInput, u, v) {
    const surface = ensureSurfaceEvaluator(surfaceInput);
    if (!surface) {
      return null;
    }
    const domainU = surface.domainU ?? { min: 0, max: 1 };
    const domainV = surface.domainV ?? { min: 0, max: 1 };
    const spanU = Math.max(Math.abs(domainU.max - domainU.min), EPSILON);
    const spanV = Math.max(Math.abs(domainV.max - domainV.min), EPSILON);
    const deltaU = spanU * 1e-3;
    const deltaV = spanV * 1e-3;
    const base = evaluateSurfacePoint(surface, u, v);
    const pointU = evaluateSurfacePoint(surface, u + deltaU, v);
    const pointV = evaluateSurfacePoint(surface, u, v + deltaV);
    if (!base || !pointU || !pointV) {
      return null;
    }
    const tangentU = pointU.clone().sub(base);
    const tangentV = pointV.clone().sub(base);
    const normal = tangentU.clone().cross(tangentV);
    if (normal.lengthSq() < EPSILON) {
      return null;
    }
    return normal.normalize();
  }

  function surfaceClosestPoint(surfaceInput, pointInput, { extend = false, samplesU = 16, samplesV = 16 } = {}) {
    const surface = ensureSurfaceEvaluator(surfaceInput);
    if (!surface) {
      return null;
    }
    const target = ensurePoint(pointInput, new THREE.Vector3());
    const domainU = surface.domainU ?? { min: 0, max: 1 };
    const domainV = surface.domainV ?? { min: 0, max: 1 };
    let best = null;
    for (let i = 0; i <= samplesU; i += 1) {
      const u = THREE.MathUtils.lerp(domainU.min, domainU.max, i / samplesU);
      for (let j = 0; j <= samplesV; j += 1) {
        const v = THREE.MathUtils.lerp(domainV.min, domainV.max, j / samplesV);
        const sample = evaluateSurfacePoint(surface, u, v);
        if (!sample) continue;
        const distanceSq = sample.distanceToSquared(target);
        if (!best || distanceSq < best.distanceSq) {
          best = { u, v, point: sample, distanceSq };
        }
      }
    }
    if (!best) {
      return null;
    }
    let stepU = (domainU.max - domainU.min) / samplesU;
    let stepV = (domainV.max - domainV.min) / samplesV;
    for (let iteration = 0; iteration < 5; iteration += 1) {
      let improved = false;
      for (const du of [-stepU, 0, stepU]) {
        for (const dv of [-stepV, 0, stepV]) {
          if (du === 0 && dv === 0) continue;
          let candidateU = best.u + du;
          let candidateV = best.v + dv;
          if (!extend) {
            candidateU = THREE.MathUtils.clamp(candidateU, domainU.min, domainU.max);
            candidateV = THREE.MathUtils.clamp(candidateV, domainV.min, domainV.max);
          }
          const sample = evaluateSurfacePoint(surface, candidateU, candidateV);
          if (!sample) continue;
          const distanceSq = sample.distanceToSquared(target);
          if (distanceSq < best.distanceSq) {
            best = { u: candidateU, v: candidateV, point: sample, distanceSq };
            improved = true;
          }
        }
      }
      if (!improved) {
        stepU *= 0.5;
        stepV *= 0.5;
      }
    }
    const normal = surfaceNormal(surface, best.u, best.v) ?? new THREE.Vector3(0, 0, 1);
    const vector = target.clone().sub(best.point);
    const distance = vector.dot(normal);
    return {
      u: best.u,
      v: best.v,
      point: best.point,
      normal,
      distance,
      distanceSq: best.distanceSq,
    };
  }

  function ensureCurveSampler(curveInput, { segments = 128 } = {}) {
    if (!curveInput) {
      return null;
    }
    if (curveInput.curve) {
      return ensureCurveSampler(curveInput.curve, { segments });
    }
    if (curveInput.path) {
      return ensureCurveSampler(curveInput.path, { segments });
    }
    let getPointAt = null;
    let getTangentAt = null;
    let closed = Boolean(curveInput.closed);
    if (typeof curveInput.getPointAt === 'function') {
      getPointAt = (t) => ensurePoint(curveInput.getPointAt(THREE.MathUtils.clamp(t, 0, 1)), new THREE.Vector3());
      if (typeof curveInput.getTangentAt === 'function') {
        getTangentAt = (t) => ensureVector(curveInput.getTangentAt(THREE.MathUtils.clamp(t, 0, 1)), new THREE.Vector3(1, 0, 0)).normalize();
      }
    } else if (curveInput.path && typeof curveInput.path.getPointAt === 'function') {
      const path = curveInput.path;
      getPointAt = (t) => ensurePoint(path.getPointAt(THREE.MathUtils.clamp(t, 0, 1)), new THREE.Vector3());
      if (typeof path.getTangentAt === 'function') {
        getTangentAt = (t) => ensureVector(path.getTangentAt(THREE.MathUtils.clamp(t, 0, 1)), new THREE.Vector3(1, 0, 0)).normalize();
      }
      closed = Boolean(path.closed ?? curveInput.closed);
    } else if (Array.isArray(curveInput.points) && curveInput.points.length >= 2) {
      const points = curveInput.points.map((pt) => ensurePoint(pt, new THREE.Vector3()));
      const path = new THREE.CatmullRomCurve3(points, Boolean(curveInput.closed));
      getPointAt = (t) => ensurePoint(path.getPointAt(THREE.MathUtils.clamp(t, 0, 1)), new THREE.Vector3());
      getTangentAt = (t) => ensureVector(path.getTangentAt(THREE.MathUtils.clamp(t, 0, 1)), new THREE.Vector3(1, 0, 0)).normalize();
      closed = Boolean(curveInput.closed);
    } else if (Array.isArray(curveInput) && curveInput.length >= 2) {
      const points = curveInput.map((pt) => ensurePoint(pt, new THREE.Vector3()));
      const path = new THREE.CatmullRomCurve3(points, false);
      getPointAt = (t) => ensurePoint(path.getPointAt(THREE.MathUtils.clamp(t, 0, 1)), new THREE.Vector3());
      getTangentAt = (t) => ensureVector(path.getTangentAt(THREE.MathUtils.clamp(t, 0, 1)), new THREE.Vector3(1, 0, 0)).normalize();
      closed = false;
    }
    if (!getPointAt) {
      return null;
    }
    if (!getTangentAt) {
      getTangentAt = (t) => {
        const delta = 1e-3;
        const p0 = getPointAt(Math.max(0, t - delta));
        const p1 = getPointAt(Math.min(1, t + delta));
        const tangent = p1.clone().sub(p0);
        if (tangent.lengthSq() < EPSILON) {
          return new THREE.Vector3(1, 0, 0);
        }
        return tangent.normalize();
      };
    }
    const safeSegments = Math.max(segments, 64);
    const points = [];
    const cumulative = [0];
    let length = 0;
    let previous = getPointAt(0);
    points.push(previous.clone());
    for (let i = 1; i <= safeSegments; i += 1) {
      const t = i / safeSegments;
      const current = getPointAt(t);
      length += current.distanceTo(previous);
      cumulative.push(length);
      points.push(current.clone());
      previous = current;
    }
    const totalLength = length;
    const lengthAtParameter = (t) => {
      const clamped = THREE.MathUtils.clamp(t, 0, 1);
      const scaled = clamped * safeSegments;
      const index = Math.floor(scaled);
      const fraction = scaled - index;
      const length0 = cumulative[index];
      const length1 = cumulative[Math.min(index + 1, cumulative.length - 1)];
      return THREE.MathUtils.lerp(length0, length1, fraction);
    };
    const parameterAtLength = (targetLength) => {
      const clamped = THREE.MathUtils.clamp(targetLength, 0, totalLength);
      if (totalLength < EPSILON) {
        return 0;
      }
      let low = 0;
      let high = cumulative.length - 1;
      while (low + 1 < high) {
        const mid = Math.floor((low + high) / 2);
        if (cumulative[mid] > clamped) {
          high = mid;
        } else {
          low = mid;
        }
      }
      const span = cumulative[high] - cumulative[low];
      const factor = span < EPSILON ? 0 : (clamped - cumulative[low]) / span;
      return (low + factor) / safeSegments;
    };
    return {
      getPointAt,
      getTangentAt,
      points,
      cumulativeLengths: cumulative,
      length: totalLength,
      segments: safeSegments,
      closed,
      lengthAtParameter,
      parameterAtLength,
    };
  }

  function curveClosestPoint(curveInput, pointInput, { segments = 128, extend = true } = {}) {
    const sampler = ensureCurveSampler(curveInput, { segments });
    if (!sampler) {
      return null;
    }
    const target = ensurePoint(pointInput, new THREE.Vector3());
    let bestT = 0;
    let bestDistanceSq = Number.POSITIVE_INFINITY;
    for (let i = 0; i <= sampler.segments; i += 1) {
      const t = i / sampler.segments;
      const sample = sampler.getPointAt(t);
      const distanceSq = sample.distanceToSquared(target);
      if (distanceSq < bestDistanceSq) {
        bestDistanceSq = distanceSq;
        bestT = t;
      }
    }
    let step = 1 / sampler.segments;
    for (let iteration = 0; iteration < 5; iteration += 1) {
      let improved = false;
      for (const offset of [-step, 0, step]) {
        const candidate = bestT + offset;
        if (!extend && !sampler.closed) {
          if (candidate < 0 || candidate > 1) {
            continue;
          }
        }
        const t = sampler.closed ? ((candidate % 1) + 1) % 1 : THREE.MathUtils.clamp(candidate, 0, 1);
        const sample = sampler.getPointAt(t);
        const distanceSq = sample.distanceToSquared(target);
        if (distanceSq < bestDistanceSq) {
          bestDistanceSq = distanceSq;
          bestT = t;
          improved = true;
        }
      }
      if (!improved) {
        step *= 0.5;
      }
    }
    const closestPoint = sampler.getPointAt(bestT);
    const tangent = sampler.getTangentAt(bestT);
    const vector = target.clone().sub(closestPoint);
    const projection = tangent.clone().multiplyScalar(vector.dot(tangent));
    const perpendicular = vector.clone().sub(projection);
    const lengthAlongCurve = sampler.lengthAtParameter(bestT);
    return {
      parameter: bestT,
      point: closestPoint,
      tangent,
      perpendicular,
      distance: Math.sqrt(perpendicular.lengthSq()),
      length: lengthAlongCurve,
      distanceSq: bestDistanceSq,
    };
  }

  function createCurveFromPoints(pointsInput, { closed = false } = {}) {
    const points = collectPoints(pointsInput);
    if (!points.length) {
      return null;
    }
    if (points.length === 1) {
      return {
        type: 'curve-point',
        points: [points[0].clone()],
        length: 0,
        closed: false,
        domain: { min: 0, max: 1, start: 0, end: 1 },
        getPointAt: () => points[0].clone(),
        getTangentAt: () => new THREE.Vector3(1, 0, 0),
      };
    }
    const path = new THREE.CatmullRomCurve3(points.map((pt) => pt.clone()), closed, 'centripetal', 0.5);
    const segments = Math.max(points.length * 8, 64);
    const spaced = path.getSpacedPoints(segments).map((pt) => new THREE.Vector3(pt.x, pt.y, pt.z ?? 0));
    const curve = {
      type: 'curve',
      path,
      points: spaced,
      segments,
      length: path.getLength(),
      closed,
      domain: { min: 0, max: 1, start: 0, end: 1 },
    };
    curve.getPointAt = (t) => {
      const clamped = THREE.MathUtils.clamp(t, 0, 1);
      const pt = path.getPointAt(clamped);
      return new THREE.Vector3(pt.x, pt.y, pt.z ?? 0);
    };
    curve.getTangentAt = (t) => {
      const clamped = THREE.MathUtils.clamp(t, 0, 1);
      const tangent = path.getTangentAt(clamped);
      return new THREE.Vector3(tangent.x, tangent.y, tangent.z ?? 0).normalize();
    };
    return curve;
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
    createTwistedBox,
    isTwistedBox,
    ensureTwistedBox,
    evaluateTwistedBox,
    twistedBoxDerivatives,
    invertTwistedBox,
    mapGeometryStructure,
    createRigidMorphMatrix,
    applyMorphToGeometry,
    ensureSurfaceEvaluator,
    evaluateSurfacePoint,
    surfaceNormal,
    surfaceClosestPoint,
    ensureCurveSampler,
    curveClosestPoint,
    createCurveFromPoints,
    ensureMatrix4,
    collectTransforms,
    setTransformMetadata,
    getTransformMetadata,
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

export function registerTransformUtilComponents({ register, toNumber, toVector3 }) {
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
    ensureBoolean,
    applyTransformToGeometry,
    collectEntries,
    ensureMatrix4,
    collectTransforms,
    setTransformMetadata,
    getTransformMetadata,
  } = createTransformHelpers({ toNumber, toVector3 });

  const GROUP_MARKER = Symbol('ghx:group');

  function isGroup(value) {
    return Boolean(value && value[GROUP_MARKER]);
  }

  function createGroup(members = [], metadata = {}) {
    const normalizedMembers = [];
    for (const member of members) {
      if (member !== undefined) {
        normalizedMembers.push(member);
      }
    }
    const group = {
      type: 'group',
      members: normalizedMembers,
      size: normalizedMembers.length,
      metadata: { ...(metadata || {}) },
    };
    Object.defineProperty(group, GROUP_MARKER, {
      value: true,
      enumerable: false,
      configurable: false,
    });
    return group;
  }

  function collectGroupMembers(input) {
    if (input === undefined || input === null) {
      return [];
    }
    if (isGroup(input)) {
      return input.members.map((member) => member);
    }
    if (Array.isArray(input)) {
      const result = [];
      for (const entry of input) {
        result.push(...collectGroupMembers(entry));
      }
      return result;
    }
    if (typeof input === 'object') {
      if (Array.isArray(input.members)) {
        return input.members.map((member) => member);
      }
      if (Array.isArray(input.objects)) {
        return input.objects.map((member) => member);
      }
      if ('group' in input) {
        return collectGroupMembers(input.group);
      }
      if ('groups' in input) {
        return collectGroupMembers(input.groups);
      }
      if ('value' in input) {
        return collectGroupMembers(input.value);
      }
      if ('values' in input) {
        return collectGroupMembers(input.values);
      }
    }
    return collectEntries(input);
  }

  function normalizeGroup(input) {
    const members = collectGroupMembers(input);
    if (isGroup(input)) {
      return createGroup(members, { ...(input.metadata ?? {}) });
    }
    if (input && typeof input === 'object') {
      if (Array.isArray(input.members) || Array.isArray(input.objects)) {
        return createGroup(members, { ...(input.metadata ?? {}) });
      }
    }
    return createGroup(members);
  }

  function normalizeIndices(input, length, { wrap = false } = {}) {
    const entries = collectEntries(input);
    const indices = new Set();
    if (!Number.isFinite(length) || length <= 0) {
      return indices;
    }
    for (const entry of entries) {
      const numeric = toNumber(entry, Number.NaN);
      if (!Number.isFinite(numeric)) {
        continue;
      }
      let index = Math.trunc(numeric);
      if (wrap) {
        index = ((index % length) + length) % length;
      }
      if (index < 0 || index >= length) {
        continue;
      }
      indices.add(index);
    }
    return indices;
  }

  function combineMatrices(matrices, { recordFragments = false } = {}) {
    if (!Array.isArray(matrices) || matrices.length === 0) {
      const identity = identityMatrix();
      if (recordFragments) {
        setTransformMetadata(identity, { fragments: [] });
      }
      return { matrix: identity, fragments: [] };
    }
    const combined = identityMatrix();
    for (const matrix of matrices) {
      combined.premultiply(matrix);
    }
    const fragments = matrices.map((matrix) => matrix.clone());
    if (recordFragments) {
      setTransformMetadata(combined, { fragments: fragments.map((fragment) => fragment.clone()) });
    }
    return { matrix: combined, fragments };
  }

  function normalizeTransform(input, { fallbackIdentity = true } = {}) {
    const transforms = collectTransforms(input);
    if (!transforms.length) {
      if (!fallbackIdentity) {
        return { matrix: null, fragments: [] };
      }
      const matrix = identityMatrix();
      return { matrix, fragments: [matrix.clone()] };
    }
    if (transforms.length === 1) {
      const matrix = transforms[0];
      const metadata = getTransformMetadata(matrix);
      let fragments = [];
      if (metadata?.fragments?.length) {
        fragments = metadata.fragments.map((fragment) => ensureMatrix4(fragment));
      } else {
        fragments = [matrix.clone()];
        setTransformMetadata(matrix, { fragments: fragments.map((fragment) => fragment.clone()) });
      }
      return { matrix, fragments };
    }
    const { matrix, fragments } = combineMatrices(transforms, { recordFragments: true });
    return { matrix, fragments };
  }

  register(['{15204c6d-bba8-403d-9e8f-6660ab8e0df5}', 'merge group', 'gmerge'], {
    type: 'group',
    pinMap: {
      inputs: {
        A: 'groupA',
        'Group A': 'groupA',
        B: 'groupB',
        'Group B': 'groupB',
      },
      outputs: { G: 'group', Group: 'group' },
    },
    eval: ({ inputs }) => {
      const groupA = normalizeGroup(inputs.groupA);
      const groupB = normalizeGroup(inputs.groupB);
      const metadata = { ...(groupA.metadata ?? {}), ...(groupB.metadata ?? {}) };
      const members = [...groupA.members, ...groupB.members];
      const group = createGroup(members, metadata);
      return { group };
    },
  });

  register(['{874eebe7-835b-4f4f-9811-97e031c41597}', 'group'], {
    type: 'group',
    pinMap: {
      inputs: { O: 'objects', Objects: 'objects', objects: 'objects' },
      outputs: { G: 'group', Group: 'group' },
    },
    eval: ({ inputs }) => {
      const members = collectGroupMembers(inputs.objects);
      const group = createGroup(members);
      return { group };
    },
  });

  register(['{fd03419e-e1cc-4603-8a57-6dfa56ed5dec}', 'split group', 'gsplit'], {
    type: 'group',
    pinMap: {
      inputs: {
        G: 'group',
        Group: 'group',
        I: 'indices',
        Indices: 'indices',
        W: 'wrap',
        Wrap: 'wrap',
      },
      outputs: {
        A: 'groupA',
        'Group A': 'groupA',
        B: 'groupB',
        'Group B': 'groupB',
      },
    },
    eval: ({ inputs }) => {
      const baseGroup = normalizeGroup(inputs.group);
      const wrap = ensureBoolean(inputs.wrap, false);
      const indexSet = normalizeIndices(inputs.indices, baseGroup.members.length, { wrap });
      const included = [];
      const excluded = [];
      baseGroup.members.forEach((member, index) => {
        if (indexSet.has(index)) {
          included.push(member);
        } else {
          excluded.push(member);
        }
      });
      return {
        groupA: createGroup(included, { ...(baseGroup.metadata ?? {}) }),
        groupB: createGroup(excluded, { ...(baseGroup.metadata ?? {}) }),
      };
    },
  });

  register(['{a45f59c8-11c1-4ea7-9e10-847061b80d75}', 'ungroup'], {
    type: 'group',
    pinMap: {
      inputs: { G: 'group', Group: 'group' },
      outputs: { O: 'objects', Objects: 'objects' },
    },
    eval: ({ inputs }) => {
      const members = collectGroupMembers(inputs.group);
      return { objects: members };
    },
  });

  register(['{610e689b-5adc-47b3-af8f-e3a32b7ea341}', 'transform'], {
    type: 'geometry',
    pinMap: {
      inputs: { G: 'geometry', geometry: 'geometry', T: 'transform', Transform: 'transform' },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const transforms = collectTransforms(inputs.transform);
      if (!transforms.length) {
        const geometry = applyTransformToGeometry(inputs.geometry, identityMatrix());
        return { geometry };
      }
      const { matrix } = combineMatrices(transforms);
      const geometry = applyTransformToGeometry(inputs.geometry, matrix);
      return { geometry };
    },
  });

  register(['{51f61166-7202-45aa-9126-3d83055b269e}', 'inverse transform', 'inverse'], {
    type: 'transform',
    pinMap: {
      inputs: { T: 'transform', Transform: 'transform' },
      outputs: { T: 'transform', transform: 'transform' },
    },
    eval: ({ inputs }) => {
      const { matrix, fragments } = normalizeTransform(inputs.transform);
      if (!matrix) {
        const identity = identityMatrix();
        setTransformMetadata(identity, { fragments: [] });
        return { transform: identity };
      }
      const determinant = matrix.determinant();
      if (!Number.isFinite(determinant) || Math.abs(determinant) < EPSILON) {
        const identity = identityMatrix();
        setTransformMetadata(identity, { fragments: [] });
        return { transform: identity };
      }
      matrix.invert();
      if (fragments.length) {
        const invertedFragments = fragments
          .slice()
          .reverse()
          .map((fragment) => {
            const clone = fragment.clone();
            const det = clone.determinant();
            if (!Number.isFinite(det) || Math.abs(det) < EPSILON) {
              return identityMatrix();
            }
            clone.invert();
            return clone;
          });
        setTransformMetadata(matrix, { fragments: invertedFragments });
      }
      return { transform: matrix };
    },
  });

  register(['{915f8f93-f5d1-4a7b-aecb-c327bab88ffb}', 'split'], {
    type: 'transform',
    pinMap: {
      inputs: { T: 'transform', Transform: 'transform' },
      outputs: { F: 'fragments', Fragments: 'fragments' },
    },
    eval: ({ inputs }) => {
      const { matrix, fragments } = normalizeTransform(inputs.transform, { fallbackIdentity: false });
      if (fragments.length) {
        return { fragments: fragments.map((fragment) => fragment.clone()) };
      }
      if (matrix) {
        return { fragments: [matrix.clone()] };
      }
      return { fragments: [] };
    },
  });

  register(['{ca80054a-cde0-4f69-a132-10502b24866d}', 'compound', 'comp'], {
    type: 'transform',
    pinMap: {
      inputs: { T: 'transforms', Transform: 'transforms', transforms: 'transforms', Transforms: 'transforms' },
      outputs: { X: 'transform', transform: 'transform', Compound: 'transform' },
    },
    eval: ({ inputs }) => {
      const transforms = collectTransforms(inputs.transforms);
      if (!transforms.length) {
        const identity = identityMatrix();
        setTransformMetadata(identity, { fragments: [] });
        return { transform: identity };
      }
      const { matrix } = combineMatrices(transforms, { recordFragments: true });
      return { transform: matrix };
    },
  });
}

export function registerTransformMorphComponents({ register, toNumber, toVector3 }) {
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
    ensureBoolean,
    ensurePoint,
    ensureVector,
    ensurePlane,
    ensureLine,
    collectPoints,
    planeCoordinates,
    extractDomain,
    createTwistedBox,
    ensureTwistedBox,
    evaluateTwistedBox,
    invertTwistedBox,
    applyMorphToGeometry,
    ensureSurfaceEvaluator,
    evaluateSurfacePoint,
    surfaceNormal,
    surfaceClosestPoint,
    ensureCurveSampler,
    createCurveFromPoints,
  } = createTransformHelpers({ toNumber, toVector3 });

  const clamp01 = (value) => THREE.MathUtils.clamp(value, 0, 1);

  function lerp(a, b, t) {
    return a + (b - a) * t;
  }

  function parseDomainInput(domainInput, fallbackMin, fallbackMax) {
    const domain = extractDomain(domainInput, fallbackMin, fallbackMax);
    return {
      min: domain.min,
      max: domain.max,
      span: domain.max - domain.min,
    };
  }

  function parseSurfaceDomain(surface, domainInput) {
    const surfaceEval = ensureSurfaceEvaluator(surface);
    const fallbackU = surfaceEval?.domainU ?? { min: 0, max: 1 };
    const fallbackV = surfaceEval?.domainV ?? { min: 0, max: 1 };
    if (!domainInput) {
      return {
        u: parseDomainInput(fallbackU, fallbackU.min, fallbackU.max),
        v: parseDomainInput(fallbackV, fallbackV.min, fallbackV.max),
      };
    }
    const uInput = domainInput.U ?? domainInput.u ?? domainInput[0] ?? domainInput.domainU;
    const vInput = domainInput.V ?? domainInput.v ?? domainInput[1] ?? domainInput.domainV;
    return {
      u: parseDomainInput(uInput, fallbackU.min, fallbackU.max),
      v: parseDomainInput(vInput, fallbackV.min, fallbackV.max),
    };
  }

  function parseUVParameter(surface, parameterInput, fallback = null) {
    const surfaceEval = ensureSurfaceEvaluator(surface);
    const domainU = surfaceEval?.domainU ?? { min: 0, max: 1 };
    const domainV = surfaceEval?.domainV ?? { min: 0, max: 1 };
    let u = domainU.min;
    let v = domainV.min;
    if (fallback) {
      u = fallback.u;
      v = fallback.v;
    }
    if (Array.isArray(parameterInput)) {
      if (parameterInput.length >= 2) {
        u = toNumber(parameterInput[0], u);
        v = toNumber(parameterInput[1], v);
      }
    } else if (parameterInput && typeof parameterInput === 'object') {
      u = toNumber(parameterInput.u ?? parameterInput.U ?? parameterInput.x ?? parameterInput[0], u);
      v = toNumber(parameterInput.v ?? parameterInput.V ?? parameterInput.y ?? parameterInput[1], v);
    } else if (Number.isFinite(parameterInput)) {
      u = toNumber(parameterInput, u);
    }
    u = THREE.MathUtils.clamp(u, domainU.min, domainU.max);
    v = THREE.MathUtils.clamp(v, domainV.min, domainV.max);
    return { u, v };
  }

  function safeNormal(normal, fallback = new THREE.Vector3(0, 0, 1)) {
    if (!normal || normal.lengthSq() < EPSILON) {
      return fallback.clone();
    }
    return normal.clone().normalize();
  }

  function evaluateSurfaceWithOffset(surface, u, v, offset = 0) {
    const point = evaluateSurfacePoint(surface, u, v);
    if (!point) {
      return null;
    }
    if (Math.abs(offset) < EPSILON) {
      return point;
    }
    const normal = safeNormal(surfaceNormal(surface, u, v));
    return point.clone().add(normal.multiplyScalar(offset));
  }

  function computeSurfaceTangents(surface, u, v) {
    const delta = 1e-3;
    const base = evaluateSurfacePoint(surface, u, v);
    const pointU = evaluateSurfacePoint(surface, u + delta, v) ?? base.clone().add(new THREE.Vector3(1, 0, 0));
    const pointV = evaluateSurfacePoint(surface, u, v + delta) ?? base.clone().add(new THREE.Vector3(0, 1, 0));
    const tangentU = pointU.clone().sub(base);
    const tangentV = pointV.clone().sub(base);
    const normal = safeNormal(tangentU.clone().cross(tangentV));
    const xAxis = tangentU.lengthSq() < EPSILON ? new THREE.Vector3(1, 0, 0) : tangentU.clone().normalize();
    const yAxis = tangentV.lengthSq() < EPSILON ? normal.clone().cross(xAxis).normalize() : tangentV.clone().normalize();
    return { base, tangentU, tangentV, normal, xAxis, yAxis };
  }

  function buildPlaneFromSurface(surface, u, v) {
    const { base, xAxis, yAxis, normal } = computeSurfaceTangents(surface, u, v);
    return normalizePlaneAxes(base.clone(), xAxis, yAxis, normal);
  }

  function createCurveFrame(tangent) {
    const normalized = tangent.clone().normalize();
    let reference = new THREE.Vector3(0, 0, 1);
    if (Math.abs(normalized.dot(reference)) > 0.95) {
      reference = new THREE.Vector3(0, 1, 0);
    }
    const normal = normalized.clone().cross(reference).normalize();
    const binormal = normalized.clone().cross(normal).normalize();
    if (normal.lengthSq() < EPSILON || binormal.lengthSq() < EPSILON) {
      normal.set(1, 0, 0);
      binormal.copy(normalized.clone().cross(normal)).normalize();
    }
    return { tangent: normalized, normal, binormal };
  }

  function falloffFunction(falloffInput) {
    if (typeof falloffInput === 'function') {
      return (distance) => {
        const result = falloffInput(distance);
        const numeric = Number(result);
        if (!Number.isFinite(numeric)) {
          return 0;
        }
        return numeric;
      };
    }
    const numeric = toNumber(falloffInput, Number.NaN);
    if (Number.isFinite(numeric) && Math.abs(numeric) > EPSILON) {
      const scale = Math.abs(numeric);
      return (distance) => Math.exp(-(distance * distance) / (scale * scale));
    }
    if (typeof falloffInput === 'string') {
      try {
        const expression = new Function('x', `'use strict'; return (${falloffInput});`);
        return (distance) => {
          try {
            const value = expression(distance);
            const numericValue = Number(value);
            return Number.isFinite(numericValue) ? numericValue : 0;
          } catch (error) {
            return 0;
          }
        };
      } catch (error) {
        return (distance) => Math.exp(-distance);
      }
    }
    return (distance) => Math.exp(-distance);
  }

  function controlPointDeformation(pointsInput, motionsInput, falloff) {
    const controlPoints = collectPoints(pointsInput);
    const motions = collectPoints(motionsInput).map((vector) => ensureVector(vector, new THREE.Vector3()));
    const count = Math.min(controlPoints.length, motions.length);
    if (!count) {
      return () => new THREE.Vector3();
    }
    const falloffFn = typeof falloff === 'function' ? falloff : falloffFunction(falloff);
    return (point) => {
      const displacement = new THREE.Vector3();
      for (let i = 0; i < count; i += 1) {
        const anchor = controlPoints[i];
        const motion = motions[i];
        const distance = point.distanceTo(anchor);
        const weight = falloffFn(distance);
        displacement.add(motion.clone().multiplyScalar(weight));
      }
      return displacement;
    };
  }

  function axisBasis(direction) {
    const normalized = direction.clone().normalize();
    let reference = new THREE.Vector3(0, 0, 1);
    if (Math.abs(normalized.dot(reference)) > 0.95) {
      reference = new THREE.Vector3(0, 1, 0);
    }
    const u = normalized.clone().cross(reference).normalize();
    const v = normalized.clone().cross(u).normalize();
    return { direction: normalized, u, v };
  }

  function defaultGeometrySamples(geometryInput, sampleCount = 256) {
    return collectPoints(geometryInput, sampleCount);
  }

  function createBoxFromCorners(corners) {
    return createTwistedBox(corners);
  }

  function parameterizePointInBox(box, point) {
    const inversion = invertTwistedBox(box, point);
    return {
      u: inversion.u,
      v: inversion.v,
      w: inversion.w,
      success: inversion.success,
    };
  }

  function mapBoxToBox(referenceBox, targetBox, point) {
    const uvw = parameterizePointInBox(referenceBox, point);
    if (!uvw.success) {
      return point.clone();
    }
    return evaluateTwistedBox(targetBox, uvw.u, uvw.v, uvw.w);
  }

  function createPointMirror(point, planePoint, planeNormal) {
    const vector = point.clone().sub(planePoint);
    const distance = vector.dot(planeNormal);
    return {
      mirrored: point.clone().sub(planeNormal.clone().multiplyScalar(2 * distance)),
      distance,
    };
  }

  function mirrorPointAcrossCurve(point, curveInput, { extend = true, segments = 256 } = {}) {
    if (!curveInput) {
      return { point: point.clone(), distance: 0 };
    }
    let sampler = null;
    const isSampler =
      typeof curveInput?.getPointAt === 'function' &&
      typeof curveInput?.getTangentAt === 'function' &&
      Number.isFinite(curveInput?.segments) &&
      Array.isArray(curveInput?.points);
    if (isSampler) {
      sampler = curveInput;
    } else {
      sampler = ensureCurveSampler(curveInput, { segments });
    }
    if (!sampler) {
      return { point: point.clone(), distance: 0 };
    }
    const closest = closestPointOnSampler(sampler, point, extend);
    const mirrored = point.clone().sub(closest.perpendicular.clone().multiplyScalar(2));
    return { point: mirrored, distance: closest.perpendicular.length() };
  }

  function computeCurveLengthMapping(baseSampler, targetSampler, stretch, length) {
    if (!stretch) {
      return Math.min(length, targetSampler.length);
    }
    if (baseSampler.length < EPSILON) {
      return 0;
    }
    const normalized = THREE.MathUtils.clamp(length / baseSampler.length, 0, 1);
    return targetSampler.length * normalized;
  }

  function mapPointAlongAxis(point, axisStart, axisDirection, axisLength, { clampRange = true } = {}) {
    const relative = point.clone().sub(axisStart);
    const projection = relative.dot(axisDirection);
    const parameter = axisLength > EPSILON ? projection / axisLength : 0;
    const clampedParameter = clampRange ? THREE.MathUtils.clamp(parameter, 0, 1) : parameter;
    const basePoint = axisStart.clone().add(axisDirection.clone().multiplyScalar(axisLength * clampedParameter));
    const offset = relative.sub(axisDirection.clone().multiplyScalar(axisLength * clampedParameter));
    return { parameter, clampedParameter, basePoint, offset };
  }

  function closestPointOnSampler(sampler, point, extend = true) {
    const target = point.clone();
    let bestT = 0;
    let bestDistanceSq = Number.POSITIVE_INFINITY;
    for (let i = 0; i < sampler.points.length; i += 1) {
      const sample = sampler.points[i];
      const distanceSq = sample.distanceToSquared(target);
      if (distanceSq < bestDistanceSq) {
        bestDistanceSq = distanceSq;
        bestT = i / sampler.segments;
      }
    }
    let step = 1 / sampler.segments;
    for (let iteration = 0; iteration < 4; iteration += 1) {
      let improved = false;
      for (const offset of [-step, 0, step]) {
        let candidate = bestT + offset;
        if (!extend && !sampler.closed) {
          if (candidate < 0 || candidate > 1) {
            continue;
          }
        }
        candidate = sampler.closed ? ((candidate % 1) + 1) % 1 : THREE.MathUtils.clamp(candidate, 0, 1);
        const sample = sampler.getPointAt(candidate);
        const distanceSq = sample.distanceToSquared(target);
        if (distanceSq < bestDistanceSq) {
          bestDistanceSq = distanceSq;
          bestT = candidate;
          improved = true;
        }
      }
      if (!improved) {
        step *= 0.5;
      }
    }
    const pointOnCurve = sampler.getPointAt(bestT);
    const tangent = sampler.getTangentAt(bestT);
    const length = sampler.lengthAtParameter(bestT);
    const vector = target.clone().sub(pointOnCurve);
    const projection = tangent.clone().multiplyScalar(vector.dot(tangent));
    const perpendicular = vector.clone().sub(projection);
    return {
      parameter: bestT,
      point: pointOnCurve,
      tangent,
      length,
      perpendicular,
    };
  }

  function ensureArc(arcInput) {
    if (!arcInput) {
      return null;
    }
    if (arcInput.type === 'arc') {
      return arcInput;
    }
    if (arcInput.arc) {
      return ensureArc(arcInput.arc);
    }
    const plane = ensurePlane(arcInput.plane ?? arcInput);
    const radius = Math.max(toNumber(arcInput.radius ?? arcInput.R ?? arcInput.r ?? 1, 1), EPSILON);
    let startAngle = toNumber(arcInput.startAngle ?? arcInput.start ?? (Array.isArray(arcInput.angle) ? arcInput.angle[0] : 0), 0);
    let endAngle = toNumber(
      arcInput.endAngle ?? arcInput.end ?? (Array.isArray(arcInput.angle) ? arcInput.angle[1] : arcInput.angle ?? Math.PI / 2),
      startAngle + Math.PI / 2,
    );
    return {
      plane,
      radius,
      startAngle,
      endAngle,
    };
  }

  register(['{124de0f5-65f8-4ae0-8f61-8fb066e2ba02}', 'twisted box', 'tbox'], {
    type: 'twisted-box',
    pinMap: {
      inputs: {
        A: 'A',
        'Corner A': 'A',
        B: 'B',
        'Corner B': 'B',
        C: 'C',
        'Corner C': 'C',
        D: 'D',
        'Corner D': 'D',
        E: 'E',
        'Corner E': 'E',
        F: 'F',
        'Corner F': 'F',
        G: 'G',
        'Corner G': 'G',
        H: 'H',
        'Corner H': 'H',
      },
      outputs: { B: 'box', box: 'box', 'Twisted Box': 'box' },
    },
    eval: ({ inputs }) => {
      const corners = {
        A: inputs.A,
        B: inputs.B,
        C: inputs.C,
        D: inputs.D,
        E: inputs.E,
        F: inputs.F,
        G: inputs.G,
        H: inputs.H,
      };
      const box = createTwistedBox(corners);
      return { box };
    },
  });

  register(['{134a849b-0ff4-4f36-bdd5-95e3996bae8b}', 'maelstrom'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        P: 'plane',
        Plane: 'plane',
        R0: 'radius0',
        First: 'radius0',
        R1: 'radius1',
        Second: 'radius1',
        A: 'angle',
        Angle: 'angle',
        R: 'rigid',
        Rigid: 'rigid',
      },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const plane = ensurePlane(inputs.plane);
      const radius0 = Math.max(0, toNumber(inputs.radius0, 0));
      const radius1 = Math.max(radius0, toNumber(inputs.radius1, radius0 + 1));
      const radiusSpan = Math.max(Math.abs(radius1 - radius0), EPSILON);
      const angle = toNumber(inputs.angle, 0);
      const rigid = ensureBoolean(inputs.rigid, false);
      const zAxis = plane.zAxis.clone().normalize();

      const mapPoint = (point) => {
        const coords = planeCoordinates(point, plane);
        const radius = Math.sqrt((coords.x * coords.x) + (coords.y * coords.y));
        const normalized = clamp01((radius - radius0) / radiusSpan);
        const rotationAngle = angle * normalized;
        if (Math.abs(rotationAngle) < EPSILON) {
          return point.clone();
        }
        const inPlane = plane.xAxis.clone().multiplyScalar(coords.x).add(plane.yAxis.clone().multiplyScalar(coords.y));
        const rotation = new THREE.Matrix4().makeRotationAxis(zAxis, rotationAngle);
        const rotated = inPlane.clone().applyMatrix4(rotation);
        return plane.origin.clone()
          .add(rotated)
          .add(zAxis.clone().multiplyScalar(coords.z));
      };

      const geometry = applyMorphToGeometry(inputs.geometry, mapPoint, { rigid });
      return { geometry };
    },
  });

  register(['{331b74f1-1f1f-4f37-b253-24fcdada29e3}', 'spatial deform (custom)', 'deform'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        S: 'syntax',
        Syntax: 'syntax',
        F: 'forces',
        Forces: 'forces',
        f: 'falloff',
        Falloff: 'falloff',
      },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const displacement = controlPointDeformation(inputs.syntax, inputs.forces, inputs.falloff);
      const mapPoint = (point) => point.clone().add(displacement(point));
      const geometry = applyMorphToGeometry(inputs.geometry, mapPoint, { rigid: false });
      return { geometry };
    },
  });

  register(['{3431f5c6-7578-4d26-a2b6-dfc064a9c65e}', 'mirror surface', 'mirror'], {
    type: 'point',
    pinMap: {
      inputs: {
        P: 'point',
        Point: 'point',
        S: 'surface',
        Surface: 'surface',
        F: 'frame',
        Frame: 'frame',
      },
      outputs: { P: 'point', point: 'point', D: 'distance', distance: 'distance' },
    },
    eval: ({ inputs }) => {
      const point = ensurePoint(inputs.point, new THREE.Vector3());
      const extend = ensureBoolean(inputs.frame, false);
      const closest = surfaceClosestPoint(inputs.surface, point, { extend });
      if (!closest) {
        return { point, distance: 0 };
      }
      const normal = safeNormal(closest.normal);
      const mirrored = createPointMirror(point, closest.point, normal);
      return { point: mirrored.mirrored, distance: mirrored.distance };
    },
  });

  register(['{4dbd15c7-ebcb-4af6-b3bd-32e80502520c}', 'point deform', 'pdeform'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        P: 'points',
        Points: 'points',
        M: 'motion',
        Motion: 'motion',
      },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const displacement = controlPointDeformation(inputs.points, inputs.motion, 1);
      const mapPoint = (point) => point.clone().add(displacement(point));
      const geometry = applyMorphToGeometry(inputs.geometry, mapPoint, { rigid: false });
      return { geometry };
    },
  });

  register(['{4f65c681-9331-4818-9d54-6290cae686c3}', 'surface box', 'sbox'], {
    type: 'twisted-box',
    pinMap: {
      inputs: {
        S: 'surface',
        Surface: 'surface',
        D: 'domain',
        Domain: 'domain',
        H: 'height',
        Height: 'height',
      },
      outputs: { B: 'box', box: 'box', 'Twisted Box': 'box' },
    },
    eval: ({ inputs }) => {
      const surface = ensureSurfaceEvaluator(inputs.surface);
      if (!surface) {
        return { box: createTwistedBox() };
      }
      const domains = parseSurfaceDomain(surface, inputs.domain ?? {});
      const u0 = domains.u.min;
      const u1 = domains.u.max;
      const v0 = domains.v.min;
      const v1 = domains.v.max;
      const bottomCorners = [
        evaluateSurfacePoint(surface, u0, v0) ?? new THREE.Vector3(),
        evaluateSurfacePoint(surface, u1, v0) ?? new THREE.Vector3(),
        evaluateSurfacePoint(surface, u1, v1) ?? new THREE.Vector3(),
        evaluateSurfacePoint(surface, u0, v1) ?? new THREE.Vector3(),
      ];
      const height = toNumber(inputs.height, 0);
      const topCorners = bottomCorners.map((corner, index) => {
        const u = index === 0 || index === 3 ? u0 : u1;
        const v = index <= 1 ? v0 : v1;
        const normal = safeNormal(surfaceNormal(surface, u, v));
        return corner.clone().add(normal.multiplyScalar(height));
      });
      const corners = {
        A: bottomCorners[0],
        B: bottomCorners[1],
        C: bottomCorners[2],
        D: bottomCorners[3],
        E: topCorners[0],
        F: topCorners[1],
        G: topCorners[2],
        H: topCorners[3],
      };
      return { box: createBoxFromCorners(corners) };
    },
  });

  register(['{539f5564-4fc0-4fc1-a7d3-b802fa2ef072}', 'bend deform', 'bend'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        B: 'arc',
        'Bending Arc': 'arc',
      },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const arc = ensureArc(inputs.arc);
      if (!arc) {
        const geometry = applyMorphToGeometry(inputs.geometry, (point) => point.clone(), { rigid: false });
        return { geometry };
      }
      const plane = ensurePlane(arc.plane);
      const samples = defaultGeometrySamples(inputs.geometry, 256).map((pt) => planeCoordinates(pt, plane).x);
      let minX = Math.min(...samples);
      let maxX = Math.max(...samples);
      if (!Number.isFinite(minX) || !Number.isFinite(maxX)) {
        minX = 0;
        maxX = 1;
      }
      const axisLength = Math.max(maxX - minX, EPSILON);
      const angleSpan = arc.endAngle - arc.startAngle;
      const radius = arc.radius;
      const origin = plane.origin.clone();
      const xAxis = plane.xAxis.clone();
      const yAxis = plane.yAxis.clone();
      const zAxis = plane.zAxis.clone().normalize();

      const mapPoint = (point) => {
        const coords = planeCoordinates(point, plane);
        const normalized = clamp01((coords.x - minX) / axisLength);
        const theta = arc.startAngle + angleSpan * normalized;
        const radial = radius + coords.y;
        const cos = Math.cos(theta);
        const sin = Math.sin(theta);
        const base = origin.clone()
          .add(xAxis.clone().multiplyScalar(cos * radial))
          .add(yAxis.clone().multiplyScalar(sin * radial));
        return base.add(zAxis.clone().multiplyScalar(coords.z));
      };

      const geometry = applyMorphToGeometry(inputs.geometry, mapPoint, { rigid: false });
      return { geometry };
    },
  });

  register(['{2a27f87c-61c5-47c2-a0b7-7863f31a3594}', 'stretch'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        X: 'axis',
        Axis: 'axis',
        L: 'length',
        Length: 'length',
        R: 'rigid',
        Rigid: 'rigid',
      },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const line = ensureLine(inputs.axis);
      const targetLength = toNumber(inputs.length, line.direction.length());
      const rigid = ensureBoolean(inputs.rigid, false);
      const axisLength = line.direction.length();
      if (axisLength < EPSILON) {
        const geometry = applyMorphToGeometry(inputs.geometry, (point) => point.clone(), { rigid: false });
        return { geometry };
      }
      const direction = line.direction.clone().normalize();
      const start = line.start.clone();

      const mapPoint = (point) => {
        const relative = point.clone().sub(start);
        const projection = relative.dot(direction);
        const parameter = clamp01(projection / axisLength);
        const base = start.clone().add(direction.clone().multiplyScalar(targetLength * parameter));
        const offset = relative.sub(direction.clone().multiplyScalar(axisLength * parameter));
        return base.add(offset);
      };

      const geometry = applyMorphToGeometry(inputs.geometry, mapPoint, { rigid });
      return { geometry };
    },
  });

  register(['{5889b68f-fd88-4032-860f-869fb69654dd}', 'surface morph', 'srfmorph'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        R: 'reference',
        Reference: 'reference',
        S: 'surface',
        Surface: 'surface',
        U: 'uDomain',
        'U Domain': 'uDomain',
        V: 'vDomain',
        'V Domain': 'vDomain',
        W: 'wDomain',
        'W Domain': 'wDomain',
      },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const reference = ensureTwistedBox(inputs.reference);
      const surface = ensureSurfaceEvaluator(inputs.surface);
      if (!surface) {
        const geometry = applyMorphToGeometry(inputs.geometry, (point) => point.clone(), { rigid: false });
        return { geometry };
      }
      const domainU = parseDomainInput(inputs.uDomain, surface.domainU?.min ?? 0, surface.domainU?.max ?? 1);
      const domainV = parseDomainInput(inputs.vDomain, surface.domainV?.min ?? 0, surface.domainV?.max ?? 1);
      const domainW = parseDomainInput(inputs.wDomain ?? [0, 0], 0, 0);

      const mapPoint = (point) => {
        const uvw = parameterizePointInBox(reference, point);
        if (!uvw.success) {
          return point.clone();
        }
        const uValue = lerp(domainU.min, domainU.max, clamp01(uvw.u));
        const vValue = lerp(domainV.min, domainV.max, clamp01(uvw.v));
        const wValue = lerp(domainW.min, domainW.max, clamp01(uvw.w));
        const mapped = evaluateSurfaceWithOffset(surface, uValue, vValue, wValue);
        return mapped ?? point.clone();
      };

      const geometry = applyMorphToGeometry(inputs.geometry, mapPoint, { rigid: false });
      return { geometry };
    },
  });

  register(['{6283fb37-e273-4eb2-8d2a-e347881e3928}', 'blend box', 'blendbox'], {
    type: 'twisted-box',
    pinMap: {
      inputs: {
        Sa: 'surfaceA',
        'Surface A': 'surfaceA',
        Da: 'domainA',
        'Domain A': 'domainA',
        Sb: 'surfaceB',
        'Surface B': 'surfaceB',
        Db: 'domainB',
        'Domain B': 'domainB',
      },
      outputs: { B: 'box', box: 'box', 'Twisted Box': 'box' },
    },
    eval: ({ inputs }) => {
      const surfaceA = ensureSurfaceEvaluator(inputs.surfaceA);
      const surfaceB = ensureSurfaceEvaluator(inputs.surfaceB);
      if (!surfaceA || !surfaceB) {
        return { box: createTwistedBox() };
      }
      const domainA = parseSurfaceDomain(surfaceA, inputs.domainA ?? {});
      const domainB = parseSurfaceDomain(surfaceB, inputs.domainB ?? {});
      const bottom = [
        evaluateSurfacePoint(surfaceA, domainA.u.min, domainA.v.min) ?? new THREE.Vector3(),
        evaluateSurfacePoint(surfaceA, domainA.u.max, domainA.v.min) ?? new THREE.Vector3(),
        evaluateSurfacePoint(surfaceA, domainA.u.max, domainA.v.max) ?? new THREE.Vector3(),
        evaluateSurfacePoint(surfaceA, domainA.u.min, domainA.v.max) ?? new THREE.Vector3(),
      ];
      const top = [
        evaluateSurfacePoint(surfaceB, domainB.u.min, domainB.v.min) ?? new THREE.Vector3(),
        evaluateSurfacePoint(surfaceB, domainB.u.max, domainB.v.min) ?? new THREE.Vector3(),
        evaluateSurfacePoint(surfaceB, domainB.u.max, domainB.v.max) ?? new THREE.Vector3(),
        evaluateSurfacePoint(surfaceB, domainB.u.min, domainB.v.max) ?? new THREE.Vector3(),
      ];
      const corners = {
        A: bottom[0],
        B: bottom[1],
        C: bottom[2],
        D: bottom[3],
        E: top[0],
        F: top[1],
        G: top[2],
        H: top[3],
      };
      return { box: createBoxFromCorners(corners) };
    },
  });

  register(['{66e6596f-6c8f-4ac3-99e0-0c4b7a59a7f7}', 'spatial deform', 'deform'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        S: 'syntax',
        Syntax: 'syntax',
        F: 'forces',
        Forces: 'forces',
      },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const displacement = controlPointDeformation(inputs.syntax, inputs.forces, 1);
      const mapPoint = (point) => point.clone().add(displacement(point));
      const geometry = applyMorphToGeometry(inputs.geometry, mapPoint, { rigid: false });
      return { geometry };
    },
  });

  register(['{6ce1aa3c-626b-4db7-8b5b-bf74c78f8c5e}', 'mirror surface', 'mirror'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        S: 'surface',
        Surface: 'surface',
        F: 'frame',
        Frame: 'frame',
      },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const extend = ensureBoolean(inputs.frame, false);
      const mapPoint = (point) => {
        const closest = surfaceClosestPoint(inputs.surface, point, { extend });
        if (!closest) {
          return point.clone();
        }
        const normal = safeNormal(closest.normal);
        return createPointMirror(point, closest.point, normal).mirrored;
      };
      const geometry = applyMorphToGeometry(inputs.geometry, mapPoint, { rigid: false });
      return { geometry };
    },
  });

  register(['{7ee33ede-4ce1-482c-ab1a-eb7f9151fbc5}', 'camera obscura', 'co'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        P: 'point',
        Point: 'point',
        F: 'factor',
        Factor: 'factor',
      },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const center = ensurePoint(inputs.point, new THREE.Vector3());
      const factor = toNumber(inputs.factor, 1);
      const mapPoint = (point) => center.clone().add(point.clone().sub(center).multiplyScalar(-factor));
      const geometry = applyMorphToGeometry(inputs.geometry, mapPoint, { rigid: true });
      return { geometry };
    },
  });

  register(['{9509cb30-d24f-4f55-a5ac-bf0b12a06cfa}', 'twist'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        X: 'axis',
        Axis: 'axis',
        A: 'angle',
        Angle: 'angle',
        I: 'infinite',
        Infinite: 'infinite',
        R: 'rigid',
        Rigid: 'rigid',
      },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const line = ensureLine(inputs.axis);
      const angle = toNumber(inputs.angle, 0);
      const infinite = ensureBoolean(inputs.infinite, false);
      const rigid = ensureBoolean(inputs.rigid, false);
      const axisLength = line.direction.length();
      if (axisLength < EPSILON || Math.abs(angle) < EPSILON) {
        const geometry = applyMorphToGeometry(inputs.geometry, (point) => point.clone(), { rigid: false });
        return { geometry };
      }
      const direction = line.direction.clone().normalize();
      const start = line.start.clone();

      const mapPoint = (point) => {
        const { parameter, clampedParameter } = mapPointAlongAxis(point, start, direction, axisLength, { clampRange: !infinite });
        const t = infinite ? parameter : clampedParameter;
        const axisDistance = (infinite ? parameter : clampedParameter) * axisLength;
        const base = start.clone().add(direction.clone().multiplyScalar(axisDistance));
        const relative = point.clone().sub(base);
        if (relative.lengthSq() < EPSILON) {
          return point.clone();
        }
        const rotated = relative.clone().applyAxisAngle(direction, angle * t);
        return base.add(rotated);
      };

      const geometry = applyMorphToGeometry(inputs.geometry, mapPoint, { rigid });
      return { geometry };
    },
  });

  register(['{9c9f8219-ae88-4d29-ba1b-3433ed713639}', 'mirror curve', 'mirror'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        C: 'curve',
        Curve: 'curve',
        T: 'tangent',
        Tangent: 'tangent',
      },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const sampler = ensureCurveSampler(inputs.curve, { segments: 256 });
      if (!sampler) {
        const geometry = applyMorphToGeometry(inputs.geometry, (point) => point.clone(), { rigid: false });
        return { geometry };
      }
      const extend = ensureBoolean(inputs.tangent, false);
      const mapPoint = (point) => mirrorPointAcrossCurve(point, sampler, { extend }).point;
      const geometry = applyMorphToGeometry(inputs.geometry, mapPoint, { rigid: false });
      return { geometry };
    },
  });

  register(['{9cacad37-b09f-4b54-b2b1-1ccdc2e3ffea}', 'sporph'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        S0: 'baseSurface',
        Base: 'baseSurface',
        P0: 'baseParameter',
        Parameter: 'baseParameter',
        S1: 'targetSurface',
        Target: 'targetSurface',
        P1: 'targetParameter',
        'Target Parameter': 'targetParameter',
        R: 'rigid',
        Rigid: 'rigid',
      },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const baseSurface = ensureSurfaceEvaluator(inputs.baseSurface);
      const targetSurface = ensureSurfaceEvaluator(inputs.targetSurface);
      if (!baseSurface || !targetSurface) {
        const geometry = applyMorphToGeometry(inputs.geometry, (point) => point.clone(), { rigid: false });
        return { geometry };
      }
      const baseParam = parseUVParameter(baseSurface, inputs.baseParameter);
      const targetParam = parseUVParameter(targetSurface, inputs.targetParameter);
      const baseTangents = computeSurfaceTangents(baseSurface, baseParam.u, baseParam.v);
      const targetTangents = computeSurfaceTangents(targetSurface, targetParam.u, targetParam.v);
      const basePlane = normalizePlaneAxes(baseTangents.base.clone(), baseTangents.xAxis, baseTangents.yAxis, baseTangents.normal);
      const baseScaleU = baseTangents.tangentU.length() || 1;
      const baseScaleV = baseTangents.tangentV.length() || 1;
      const targetDomainU = targetSurface.domainU ?? { min: 0, max: 1 };
      const targetDomainV = targetSurface.domainV ?? { min: 0, max: 1 };
      const rigid = ensureBoolean(inputs.rigid, false);

      const mapPoint = (point) => {
        const coords = planeCoordinates(point, basePlane);
        const deltaU = baseScaleU > EPSILON ? coords.x / baseScaleU : 0;
        const deltaV = baseScaleV > EPSILON ? coords.y / baseScaleV : 0;
        const uTarget = THREE.MathUtils.clamp(targetParam.u + deltaU, targetDomainU.min, targetDomainU.max);
        const vTarget = THREE.MathUtils.clamp(targetParam.v + deltaV, targetDomainV.min, targetDomainV.max);
        const mapped = evaluateSurfacePoint(targetSurface, uTarget, vTarget);
        if (!mapped) {
          return point.clone();
        }
        const normal = safeNormal(surfaceNormal(targetSurface, uTarget, vTarget));
        return mapped.clone().add(normal.multiplyScalar(coords.z));
      };

      const geometry = applyMorphToGeometry(inputs.geometry, mapPoint, { rigid });
      return { geometry };
    },
  });

  register(['{ad0ee51e-c86f-4668-8de5-b55b850f6001}', 'taper'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        X: 'axis',
        Axis: 'axis',
        R0: 'radiusStart',
        Start: 'radiusStart',
        R1: 'radiusEnd',
        End: 'radiusEnd',
        F: 'flat',
        Flat: 'flat',
        I: 'infinite',
        Infinite: 'infinite',
        R: 'rigid',
        Rigid: 'rigid',
      },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const line = ensureLine(inputs.axis);
      const radiusStart = toNumber(inputs.radiusStart, 1);
      const radiusEnd = toNumber(inputs.radiusEnd, radiusStart);
      const flat = ensureBoolean(inputs.flat, false);
      const infinite = ensureBoolean(inputs.infinite, false);
      const rigid = ensureBoolean(inputs.rigid, false);
      const axisLength = line.direction.length();
      if (axisLength < EPSILON) {
        const geometry = applyMorphToGeometry(inputs.geometry, (point) => point.clone(), { rigid: false });
        return { geometry };
      }
      const basis = axisBasis(line.direction.clone());
      const start = line.start.clone();

      const baseRadius = Math.abs(radiusStart) > EPSILON ? radiusStart : (radiusEnd || 1);

      const mapPoint = (point) => {
        const { parameter, clampedParameter } = mapPointAlongAxis(point, start, basis.direction, axisLength, { clampRange: !infinite });
        const t = infinite ? parameter : clampedParameter;
        const axisDistance = (infinite ? parameter : clampedParameter) * axisLength;
        const axisPoint = start.clone().add(basis.direction.clone().multiplyScalar(axisDistance));
        const relative = point.clone().sub(axisPoint);
        const uComponent = relative.dot(basis.u);
        const vComponent = relative.dot(basis.v);
        const wComponent = relative.dot(basis.direction);
        const radius = lerp(radiusStart, radiusEnd, t);
        const factor = baseRadius !== 0 ? radius / baseRadius : radius;
        const scaledU = uComponent * factor;
        const scaledV = flat ? vComponent : vComponent * factor;
        return axisPoint.clone()
          .add(basis.u.clone().multiplyScalar(scaledU))
          .add(basis.v.clone().multiplyScalar(scaledV))
          .add(basis.direction.clone().multiplyScalar(wComponent));
      };

      const geometry = applyMorphToGeometry(inputs.geometry, mapPoint, { rigid });
      return { geometry };
    },
  });

  register(['{c3249da4-3f8e-4400-833e-e4e984d28657}', 'flow'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        C0: 'baseCurve',
        Base: 'baseCurve',
        C1: 'targetCurve',
        Target: 'targetCurve',
        R0: 'reverseBase',
        'Reverse Base': 'reverseBase',
        R1: 'reverseTarget',
        'Reverse Target': 'reverseTarget',
        S: 'stretch',
        Stretch: 'stretch',
        R: 'rigid',
        Rigid: 'rigid',
      },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const baseSampler = ensureCurveSampler(inputs.baseCurve, { segments: 256 });
      const targetSampler = ensureCurveSampler(inputs.targetCurve, { segments: 256 });
      if (!baseSampler || !targetSampler) {
        const geometry = applyMorphToGeometry(inputs.geometry, (point) => point.clone(), { rigid: false });
        return { geometry };
      }
      const reverseBase = ensureBoolean(inputs.reverseBase, false);
      const reverseTarget = ensureBoolean(inputs.reverseTarget, false);
      const stretch = ensureBoolean(inputs.stretch, false);
      const rigid = ensureBoolean(inputs.rigid, false);

      const mapPoint = (point) => {
        const closest = closestPointOnSampler(baseSampler, point, true);
        const tBaseOriginal = closest.parameter;
        const baseLength = baseSampler.lengthAtParameter(tBaseOriginal);
        const adjustedBaseLength = reverseBase ? baseSampler.length - baseLength : baseLength;
        const targetLength = computeCurveLengthMapping(baseSampler, targetSampler, stretch, adjustedBaseLength);
        let tTarget = targetSampler.parameterAtLength(targetLength);
        if (reverseTarget) {
          tTarget = 1 - tTarget;
        }
        const basePoint = closest.point;
        const baseTangent = closest.tangent.clone().normalize().multiplyScalar(reverseBase ? -1 : 1);
        const baseFrame = createCurveFrame(baseTangent);
        const diff = point.clone().sub(basePoint);
        const offsetNormal = diff.dot(baseFrame.normal);
        const offsetBinormal = diff.dot(baseFrame.binormal);
        const targetPoint = targetSampler.getPointAt(tTarget);
        const targetTangent = targetSampler.getTangentAt(tTarget).clone().normalize().multiplyScalar(reverseTarget ? -1 : 1);
        const targetFrame = createCurveFrame(targetTangent);
        return targetPoint.clone()
          .add(targetFrame.normal.clone().multiplyScalar(offsetNormal))
          .add(targetFrame.binormal.clone().multiplyScalar(offsetBinormal));
      };

      const geometry = applyMorphToGeometry(inputs.geometry, mapPoint, { rigid });
      return { geometry };
    },
  });

  register(['{d8940ff0-dd4a-4e74-9361-54df537b50db}', 'box morph', 'morph'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        R: 'reference',
        Reference: 'reference',
        T: 'target',
        Target: 'target',
      },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const reference = ensureTwistedBox(inputs.reference);
      const target = ensureTwistedBox(inputs.target);
      const mapPoint = (point) => mapBoxToBox(reference, target, point);
      const geometry = applyMorphToGeometry(inputs.geometry, mapPoint, { rigid: false });
      return { geometry };
    },
  });

  register(['{f8452dc8-aea6-4654-a72f-c0fd62626d36}', 'mirror curve', 'mirror'], {
    type: 'point',
    pinMap: {
      inputs: {
        P: 'point',
        Point: 'point',
        C: 'curve',
        Curve: 'curve',
        T: 'tangent',
        Tangent: 'tangent',
      },
      outputs: { P: 'point', point: 'point', D: 'distance', distance: 'distance' },
    },
    eval: ({ inputs }) => {
      const sampler = ensureCurveSampler(inputs.curve, { segments: 256 });
      if (!sampler) {
        const point = ensurePoint(inputs.point, new THREE.Vector3());
        return { point, distance: 0 };
      }
      const extend = ensureBoolean(inputs.tangent, false);
      const point = ensurePoint(inputs.point, new THREE.Vector3());
      const mirrored = mirrorPointAcrossCurve(point, sampler, { extend });
      return { point: mirrored.point, distance: mirrored.distance };
    },
  });

  register(['{fc5b7d12-7247-4de0-81bc-9b2c2f8f72f6}', 'map to surface', 'map srf'], {
    type: 'curve',
    pinMap: {
      inputs: {
        C: 'curve',
        Curve: 'curve',
        S: 'source',
        Source: 'source',
        T: 'target',
        Target: 'target',
      },
      outputs: { C: 'curve', curve: 'curve' },
    },
    eval: ({ inputs }) => {
      const sampler = ensureCurveSampler(inputs.curve, { segments: 128 });
      const sourceSurface = ensureSurfaceEvaluator(inputs.source);
      const targetSurface = ensureSurfaceEvaluator(inputs.target);
      if (!sampler || !sourceSurface || !targetSurface) {
        return { curve: inputs.curve };
      }
      const mappedPoints = [];
      for (let i = 0; i <= sampler.segments; i += 1) {
        const t = i / sampler.segments;
        const point = sampler.getPointAt(t);
        const closest = surfaceClosestPoint(sourceSurface, point, { extend: true });
        if (!closest) {
          mappedPoints.push(point.clone());
          continue;
        }
        const mapped = evaluateSurfaceWithOffset(targetSurface, closest.u, closest.v, closest.distance);
        mappedPoints.push(mapped ?? point.clone());
      }
      const curve = createCurveFromPoints(mappedPoints, { closed: sampler.closed }) ?? createCurveFromPoints(mappedPoints);
      return { curve };
    },
  });

  register(['{ff4e6ccd-47ba-4c8c-8287-2a1f2cb1fa5e}', 'splop'], {
    type: 'geometry',
    pinMap: {
      inputs: {
        G: 'geometry',
        geometry: 'geometry',
        P: 'plane',
        Plane: 'plane',
        S: 'surface',
        Surface: 'surface',
        uv: 'parameter',
        Parameter: 'parameter',
        A: 'angle',
        Angle: 'angle',
        R: 'rigid',
        Rigid: 'rigid',
      },
      outputs: { G: 'geometry', geometry: 'geometry' },
    },
    eval: ({ inputs }) => {
      const plane = ensurePlane(inputs.plane);
      const surface = ensureSurfaceEvaluator(inputs.surface);
      if (!surface) {
        const geometry = applyMorphToGeometry(inputs.geometry, (point) => point.clone(), { rigid: false });
        return { geometry };
      }
      const param = parseUVParameter(surface, inputs.parameter);
      const angle = toNumber(inputs.angle, 0);
      const rigid = ensureBoolean(inputs.rigid, false);
      const targetTangents = computeSurfaceTangents(surface, param.u, param.v);
      const scaleU = targetTangents.tangentU.length() || 1;
      const scaleV = targetTangents.tangentV.length() || 1;
      const cosA = Math.cos(angle);
      const sinA = Math.sin(angle);
      const domainU = surface.domainU ?? { min: 0, max: 1 };
      const domainV = surface.domainV ?? { min: 0, max: 1 };

      const mapPoint = (point) => {
        const coords = planeCoordinates(point, plane);
        const rotatedX = coords.x * cosA - coords.y * sinA;
        const rotatedY = coords.x * sinA + coords.y * cosA;
        const deltaU = scaleU > EPSILON ? rotatedX / scaleU : 0;
        const deltaV = scaleV > EPSILON ? rotatedY / scaleV : 0;
        const uTarget = THREE.MathUtils.clamp(param.u + deltaU, domainU.min, domainU.max);
        const vTarget = THREE.MathUtils.clamp(param.v + deltaV, domainV.min, domainV.max);
        const mapped = evaluateSurfacePoint(surface, uTarget, vTarget);
        if (!mapped) {
          return point.clone();
        }
        const normal = safeNormal(surfaceNormal(surface, uTarget, vTarget));
        return mapped.clone().add(normal.multiplyScalar(coords.z));
      };

      const geometry = applyMorphToGeometry(inputs.geometry, mapPoint, { rigid });
      return { geometry };
    },
  });
}

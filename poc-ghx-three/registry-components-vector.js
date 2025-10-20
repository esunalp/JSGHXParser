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

  function toStringValue(value) {
    if (value === undefined || value === null) {
      return '';
    }
    if (typeof value === 'string') {
      return value;
    }
    if (typeof value === 'number' || typeof value === 'boolean') {
      return String(value);
    }
    if (Array.isArray(value)) {
      return value.map((entry) => toStringValue(entry)).filter(Boolean).join(' ');
    }
    if (typeof value === 'object') {
      if ('text' in value) {
        return toStringValue(value.text);
      }
      if ('value' in value) {
        return toStringValue(value.value);
      }
    }
    return String(value);
  }

  function collectNumbers(input) {
    const numbers = [];

    function visit(value) {
      if (value === undefined || value === null) {
        return;
      }
      if (Array.isArray(value)) {
        for (const item of value) {
          visit(item);
        }
        return;
      }
      if (value?.isVector3) {
        numbers.push(value.x, value.y, value.z);
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
        if ('number' in value) {
          visit(value.number);
          return;
        }
        if ('x' in value || 'y' in value || 'z' in value) {
          const nx = toNumber(value.x, Number.NaN);
          const ny = toNumber(value.y, Number.NaN);
          const nz = toNumber(value.z, Number.NaN);
          if (Number.isFinite(nx)) numbers.push(nx);
          if (Number.isFinite(ny)) numbers.push(ny);
          if (Number.isFinite(nz)) numbers.push(nz);
          return;
        }
      }
      const numeric = toNumber(value, Number.NaN);
      if (Number.isFinite(numeric)) {
        numbers.push(numeric);
      }
    }

    visit(input);
    return numbers;
  }

  function collectPoints(input) {
    const points = [];

    function visit(value) {
      if (value === undefined || value === null) {
        return;
      }
      if (Array.isArray(value)) {
        for (const item of value) {
          visit(item);
        }
        return;
      }
      if (value?.isVector3) {
        points.push(value.clone());
        return;
      }
      if (typeof value === 'object') {
        if ('point' in value) {
          visit(value.point);
          return;
        }
        if ('value' in value) {
          visit(value.value);
          return;
        }
        if ('position' in value) {
          visit(value.position);
          return;
        }
        if ('points' in value) {
          visit(value.points);
          return;
        }
        if ('vertices' in value) {
          visit(value.vertices);
          return;
        }
        if ('x' in value || 'y' in value || 'z' in value) {
          points.push(toVector3(value, new THREE.Vector3()));
          return;
        }
      }
    }

    visit(input);
    return points;
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
        const xAxis = toVector3(input.xAxis ?? input.X ?? input.x ?? input.i ?? new THREE.Vector3(1, 0, 0), new THREE.Vector3(1, 0, 0));
        const yAxis = toVector3(input.yAxis ?? input.Y ?? input.y ?? input.j ?? new THREE.Vector3(0, 1, 0), new THREE.Vector3(0, 1, 0));
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


  function isPlaneLike(value) {
    if (!value) return false;
    if (value?.isPlane) return true;
    if (Array.isArray(value)) {
      if (value.length >= 3) {
        return true;
      }
      if (value.length === 1) {
        return isPlaneLike(value[0]);
      }
      return false;
    }
    if (typeof value === 'object') {
      if ('plane' in value) return true;
      if ('origin' in value || 'O' in value || 'o' in value) return true;
      if ('normal' in value) return true;
    }
    return false;
  }

  function applyPlane(plane, u, v, w = 0) {
    const result = plane.origin.clone();
    result.add(plane.xAxis.clone().multiplyScalar(u));
    result.add(plane.yAxis.clone().multiplyScalar(v));
    result.add(plane.zAxis.clone().multiplyScalar(w));
    return result;
  }
  function maskToComponents(maskInput) {
    if (maskInput === undefined || maskInput === null) {
      return ['x', 'y', 'z'];
    }
    if (Array.isArray(maskInput)) {
      if (maskInput.length === 0) {
        return ['x', 'y', 'z'];
      }
      if (maskInput.length === 1) {
        return maskToComponents(maskInput[0]);
      }
      return maskInput
        .map((entry) => maskToComponents(entry))
        .flat()
        .filter((component) => ['x', 'y', 'z'].includes(component));
    }
    if (typeof maskInput === 'number') {
      if (maskInput <= 1) return ['x'];
      if (maskInput === 2) return ['x', 'y'];
      return ['x', 'y', 'z'];
    }
    const normalized = String(maskInput)
      .toLowerCase()
      .split('')
      .filter((component) => ['x', 'y', 'z'].includes(component));
    return normalized.length ? normalized : ['x', 'y', 'z'];
  }

  function numbersToPoints(numbers, components) {
    if (!numbers.length) {
      return [];
    }
    const parts = components.length ? components : ['x', 'y', 'z'];
    const chunkSize = parts.length;
    if (!chunkSize) {
      return [];
    }
    const points = [];
    for (let index = 0; index < numbers.length; index += chunkSize) {
      let x = 0;
      let y = 0;
      let z = 0;
      for (let componentIndex = 0; componentIndex < parts.length; componentIndex += 1) {
        const value = numbers[index + componentIndex] ?? 0;
        const component = parts[componentIndex];
        if (component === 'x') x = value;
        if (component === 'y') y = value;
        if (component === 'z') z = value;
      }
      points.push(new THREE.Vector3(x, y, z));
    }
    return points;
  }

  function pointToNumbers(point, components) {
    const parts = components.length ? components : ['x', 'y', 'z'];
    const numeric = [];
    for (const component of parts) {
      if (component === 'x') {
        numeric.push(point.x);
      } else if (component === 'y') {
        numeric.push(point.y);
      } else if (component === 'z') {
        numeric.push(point.z);
      }
    }
    return numeric;
  }

  function extractGeometryPoints(geometry) {
    const points = [];

    function visit(value) {
      if (!value) return;
      if (Array.isArray(value)) {
        for (const item of value) {
          visit(item);
        }
        return;
      }
      if (value.isVector3) {
        points.push(value.clone());
        return;
      }
      if (value.isMesh && value.geometry) {
        visit(value.geometry);
        return;
      }
      if (value.isBufferGeometry) {
        const position = value.getAttribute?.('position');
        if (position?.isBufferAttribute) {
          for (let i = 0; i < position.count; i += 1) {
            points.push(new THREE.Vector3(position.getX(i), position.getY(i), position.getZ(i)));
          }
        }
        return;
      }
      if (value.isGeometry && Array.isArray(value.vertices)) {
        for (const vertex of value.vertices) {
          visit(vertex);
        }
        return;
      }
      if (typeof value === 'object') {
        if ('points' in value) {
          visit(value.points);
          return;
        }
        if ('vertices' in value) {
          visit(value.vertices);
          return;
        }
        if ('position' in value && value.position?.isBufferAttribute) {
          const position = value.position;
          for (let i = 0; i < position.count; i += 1) {
            points.push(new THREE.Vector3(position.getX(i), position.getY(i), position.getZ(i)));
          }
          return;
        }
        if ('point' in value) {
          visit(value.point);
          return;
        }
        if ('x' in value || 'y' in value || 'z' in value) {
          points.push(toVector3(value, new THREE.Vector3()));
          return;
        }
      }
    }

    visit(geometry);
    return points;
  }

  function computeClosestEntries(point, candidates) {
    const entries = [];
    candidates.forEach((candidate, index) => {
      entries.push({
        index,
        point: candidate,
        distance: point.distanceTo(candidate),
      });
    });
    entries.sort((a, b) => a.distance - b.distance);
    return entries;
  }

  function projectPointOntoPlane(point, direction, plane) {
    const normal = plane.zAxis.clone().normalize();
    const denominator = normal.dot(direction);
    if (Math.abs(denominator) < EPSILON) {
      const offset = point.clone().sub(plane.origin).dot(normal);
      return point.clone().sub(normal.multiplyScalar(offset));
    }
    const t = normal.dot(plane.origin.clone().sub(point)) / denominator;
    return point.clone().add(direction.clone().multiplyScalar(t));
  }

  function resolveCurvePoints(curveInput) {
    if (!curveInput) {
      return [];
    }
    if (Array.isArray(curveInput)) {
      const flattened = [];
      for (const entry of curveInput) {
        flattened.push(...resolveCurvePoints(entry));
      }
      return flattened;
    }
    if (curveInput.points && Array.isArray(curveInput.points)) {
      return collectPoints(curveInput.points);
    }
    if (curveInput.curve) {
      return resolveCurvePoints(curveInput.curve);
    }
    if (curveInput.shape?.getPoints) {
      const segments = curveInput.segments ?? 64;
      const pts2d = curveInput.shape.getPoints(Math.max(segments, 8));
      return pts2d.map((pt) => new THREE.Vector3(pt.x, pt.y, 0));
    }
    if (curveInput.isBufferGeometry || curveInput.isGeometry || curveInput.isMesh) {
      return extractGeometryPoints(curveInput);
    }
    if (curveInput.isVector3) {
      return [curveInput.clone()];
    }
    if (typeof curveInput === 'object') {
      if ('point' in curveInput) {
        return resolveCurvePoints(curveInput.point);
      }
      if ('points' in curveInput) {
        return resolveCurvePoints(curveInput.points);
      }
      if ('path' in curveInput && typeof curveInput.path?.getSpacedPoints === 'function') {
        return curveInput.path.getSpacedPoints(64).map((pt) => new THREE.Vector3(pt.x, pt.y, pt.z ?? 0));
      }
    }
    return [];
  }

  function parameterAlongPolyline(point, polyline) {
    if (polyline.length === 0) return 0;
    if (polyline.length === 1) {
      return point.distanceTo(polyline[0]);
    }
    let accumulated = 0;
    let bestDistance = Number.POSITIVE_INFINITY;
    let bestParameter = 0;
    for (let i = 0; i < polyline.length - 1; i += 1) {
      const a = polyline[i];
      const b = polyline[i + 1];
      const segment = b.clone().sub(a);
      const length = segment.length();
      if (length < EPSILON) {
        continue;
      }
      const ap = point.clone().sub(a);
      const t = THREE.MathUtils.clamp(ap.dot(segment) / (length * length), 0, 1);
      const closest = a.clone().add(segment.clone().multiplyScalar(t));
      const distance = point.distanceTo(closest);
      const parameter = accumulated + t * length;
      if (distance < bestDistance) {
        bestDistance = distance;
        bestParameter = parameter;
      }
      accumulated += length;
    }
    return bestParameter;
  }

  function unionFind(size) {
    const parent = Array.from({ length: size }, (_, index) => index);

    function find(index) {
      if (parent[index] !== index) {
        parent[index] = find(parent[index]);
      }
      return parent[index];
    }

    function union(a, b) {
      const rootA = find(a);
      const rootB = find(b);
      if (rootA === rootB) return;
      parent[rootB] = rootA;
    }

    return { find, union };
  }

  function createPlaneInstance(plane, origin) {
    const targetOrigin = origin ? origin.clone() : plane.origin.clone();
    return {
      origin: targetOrigin,
      xAxis: plane.xAxis.clone(),
      yAxis: plane.yAxis.clone(),
      zAxis: plane.zAxis.clone(),
    };
  }
  register(['{0ae07da9-951b-4b9b-98ca-d312c252374d}', 'numbers to points', 'num2pt'], {
    type: 'point',
    pinMap: {
      inputs: { N: 'numbers', Numbers: 'numbers', numbers: 'numbers', M: 'mask', Mask: 'mask', mask: 'mask' },
      outputs: { P: 'points', Pt: 'points', Points: 'points' },
    },
    eval: ({ inputs }) => {
      const numbers = collectNumbers(inputs.numbers);
      const mask = maskToComponents(inputs.mask);
      const points = numbersToPoints(numbers, mask);
      return { points };
    },
  });

  register([
    '{3581f42a-9592-4549-bd6b-1c0fc39d067b}',
    '{8a5aae11-8775-4ee5-b4fc-db3a1bd89c2f}',
    'construct point',
    'point xyz',
  ], {
    type: 'point',
    pinMap: {
      inputs: { X: 'x', x: 'x', Y: 'y', y: 'y', Z: 'z', z: 'z', S: 'system', System: 'system', system: 'system' },
      outputs: { Pt: 'point', P: 'point', point: 'point' },
    },
    eval: ({ inputs }) => {
      const x = toNumber(inputs.x, 0);
      const y = toNumber(inputs.y, 0);
      const z = toNumber(inputs.z, 0);
      if (inputs.system !== undefined) {
        const plane = ensurePlane(inputs.system);
        return { point: applyPlane(plane, x, y, z) };
      }
      return { point: new THREE.Vector3(x, y, z) };
    },
  });

  register(['{670fcdba-da07-4eb4-b1c1-bfa0729d767d}', 'deconstruct point', 'depoint'], {
    type: 'point',
    pinMap: {
      inputs: { P: 'point', point: 'point', Point: 'point', S: 'system', System: 'system', system: 'system' },
      outputs: { X: 'x', x: 'x', Y: 'y', y: 'y', Z: 'z', z: 'z' },
    },
    eval: ({ inputs }) => {
      const point = toVector3(inputs.point, new THREE.Vector3());
      if (inputs.system === undefined) {
        return { x: point.x, y: point.y, z: point.z };
      }
      const plane = ensurePlane(inputs.system);
      const relative = point.clone().sub(plane.origin);
      return {
        x: relative.dot(plane.xAxis),
        y: relative.dot(plane.yAxis),
        z: relative.dot(plane.zAxis),
      };
    },
  });

  register(['{9abae6b7-fa1d-448c-9209-4a8155345841}', 'deconstruct', 'pdecon'], {
    type: 'point',
    pinMap: {
      inputs: { P: 'point', point: 'point', Point: 'point' },
      outputs: { X: 'x', x: 'x', Y: 'y', y: 'y', Z: 'z', z: 'z' },
    },
    eval: ({ inputs }) => {
      const point = toVector3(inputs.point, new THREE.Vector3());
      return { x: point.x, y: point.y, z: point.z };
    },
  });

  register(['{61647ba2-31eb-4921-9632-df81e3286f7d}', 'to polar', 'polar'], {
    type: 'point',
    pinMap: {
      inputs: { P: 'point', point: 'point', Point: 'point', S: 'system', System: 'system', system: 'system' },
      outputs: { P: 'phi', Phi: 'phi', T: 'theta', Theta: 'theta', R: 'radius', Radius: 'radius' },
    },
    eval: ({ inputs }) => {
      const point = toVector3(inputs.point, new THREE.Vector3());
      const plane = inputs.system !== undefined ? ensurePlane(inputs.system) : defaultPlane();
      const relative = point.clone().sub(plane.origin);
      const x = relative.dot(plane.xAxis);
      const y = relative.dot(plane.yAxis);
      const z = relative.dot(plane.zAxis);
      const radius = Math.sqrt(x * x + y * y + z * z);
      const phi = Math.atan2(y, x);
      const theta = radius < EPSILON ? 0 : Math.asin(THREE.MathUtils.clamp(z / radius, -1, 1));
      return { phi, theta, radius };
    },
  });

  register(['{93b8e93d-f932-402c-b435-84be04d87666}', 'distance', 'dist'], {
    type: 'point',
    pinMap: {
      inputs: { A: 'a', a: 'a', B: 'b', b: 'b' },
      outputs: { D: 'distance', distance: 'distance' },
    },
    eval: ({ inputs }) => {
      const pointA = toVector3(inputs.a, new THREE.Vector3());
      const pointB = toVector3(inputs.b, new THREE.Vector3());
      return { distance: pointA.distanceTo(pointB) };
    },
  });
  register(['{446014c4-c11c-45a7-8839-c45dc60950d6}', 'closest points', 'closest pts'], {
    type: 'point',
    pinMap: {
      inputs: { P: 'point', point: 'point', Point: 'point', C: 'cloud', Cloud: 'cloud', cloud: 'cloud', N: 'count', Count: 'count', n: 'count' },
      outputs: { P: 'points', Points: 'points', i: 'indices', I: 'indices', D: 'distances', Distance: 'distances' },
    },
    eval: ({ inputs }) => {
      const point = toVector3(inputs.point, new THREE.Vector3());
      const cloud = collectPoints(inputs.cloud);
      if (!cloud.length) {
        return {};
      }
      const count = Math.max(1, Math.floor(toNumber(inputs.count, 1)) || 1);
      const entries = computeClosestEntries(point, cloud).slice(0, count);
      return {
        points: entries.map((entry) => entry.point.clone()),
        indices: entries.map((entry) => entry.index),
        distances: entries.map((entry) => entry.distance),
      };
    },
  });

  register(['{571ca323-6e55-425a-bf9e-ee103c7ba4b9}', 'closest point', 'cp'], {
    type: 'point',
    pinMap: {
      inputs: { P: 'point', point: 'point', Point: 'point', C: 'cloud', Cloud: 'cloud', cloud: 'cloud' },
      outputs: { P: 'point', Pt: 'point', i: 'index', I: 'index', D: 'distance', Distance: 'distance' },
    },
    eval: ({ inputs }) => {
      const point = toVector3(inputs.point, new THREE.Vector3());
      const cloud = collectPoints(inputs.cloud);
      if (!cloud.length) {
        return {};
      }
      const [closest] = computeClosestEntries(point, cloud);
      if (!closest) {
        return {};
      }
      return {
        point: closest.point.clone(),
        index: closest.index,
        distance: closest.distance,
      };
    },
  });

  register(['{5184b8cb-b71e-4def-a590-cd2c9bc58906}', 'project point', 'project'], {
    type: 'point',
    pinMap: {
      inputs: {
        P: 'point', point: 'point', Point: 'point',
        D: 'direction', direction: 'direction', Direction: 'direction',
        G: 'geometry', geometry: 'geometry', Geometry: 'geometry',
      },
      outputs: { P: 'point', Pt: 'point', I: 'index', index: 'index' },
    },
    eval: ({ inputs }) => {
      const point = toVector3(inputs.point, new THREE.Vector3());
      const direction = ensureUnitVector(inputs.direction, new THREE.Vector3(0, 0, -1));
      if (inputs.geometry) {
        if (isPlaneLike(inputs.geometry)) {
          const plane = ensurePlane(inputs.geometry);
          const projected = projectPointOntoPlane(point, direction, plane);
          return { point: projected, index: 0 };
        }
        const points = extractGeometryPoints(inputs.geometry);
        if (points.length) {
          const [closest] = computeClosestEntries(point, points);
          if (closest) {
            return { point: closest.point.clone(), index: closest.index };
          }
        }
      }
      const fallback = defaultPlane();
      return { point: projectPointOntoPlane(point, direction, fallback), index: 0 };
    },
  });

  register(['{902289da-28dc-454b-98d4-b8f8aa234516}', '{cf3a0865-4882-46bd-91a1-d512acf95be4}', 'pull point', 'pull'], {
    type: 'point',
    pinMap: {
      inputs: {
        P: 'point', point: 'point', Point: 'point',
        G: 'geometry', geometry: 'geometry', Geometry: 'geometry',
        C: 'closestOnly', Closest: 'closestOnly', closest: 'closestOnly',
      },
      outputs: { P: 'point', Pt: 'point', D: 'distance', Distance: 'distance' },
    },
    eval: ({ inputs }) => {
      const point = toVector3(inputs.point, new THREE.Vector3());
      const geometry = inputs.geometry;
      if (!geometry) {
        return { point: point.clone(), distance: 0 };
      }
      if (isPlaneLike(geometry)) {
        const plane = ensurePlane(geometry);
        const normal = plane.zAxis.clone().normalize();
        const offset = point.clone().sub(plane.origin).dot(normal);
        const projected = point.clone().sub(normal.multiplyScalar(offset));
        return { point: projected, distance: Math.abs(offset) };
      }
      const candidates = extractGeometryPoints(geometry);
      if (candidates.length) {
        const [closest] = computeClosestEntries(point, candidates);
        if (closest) {
          return { point: closest.point.clone(), distance: closest.distance };
        }
      }
      return { point: point.clone(), distance: 0 };
    },
  });

  register(['{4e86ba36-05e2-4cc0-a0f5-3ad57c91f04e}', 'sort points', 'sort pt'], {
    type: 'point',
    pinMap: {
      inputs: { P: 'points', Points: 'points', points: 'points' },
      outputs: { P: 'points', Points: 'points', I: 'indices', i: 'indices' },
    },
    eval: ({ inputs }) => {
      const list = collectPoints(inputs.points);
      const decorated = list.map((point, index) => ({ point, index }));
      decorated.sort((a, b) => a.point.x - b.point.x || a.point.y - b.point.y || a.point.z - b.point.z || a.index - b.index);
      return {
        points: decorated.map((entry) => entry.point.clone()),
        indices: decorated.map((entry) => entry.index),
      };
    },
  });

  register(['{59aaebf8-6654-46b7-8386-89223c773978}', 'sort along curve', 'alongcrv'], {
    type: 'point',
    pinMap: {
      inputs: { P: 'points', Points: 'points', points: 'points', C: 'curve', Curve: 'curve', curve: 'curve' },
      outputs: { P: 'points', Points: 'points', I: 'indices', i: 'indices' },
    },
    eval: ({ inputs }) => {
      const points = collectPoints(inputs.points);
      if (!points.length) {
        return { points: [], indices: [] };
      }
      const curvePoints = resolveCurvePoints(inputs.curve);
      if (curvePoints.length < 2) {
        const decorated = points.map((point, index) => ({ point, index, key: point.length() }));
        decorated.sort((a, b) => a.key - b.key || a.index - b.index);
        return {
          points: decorated.map((entry) => entry.point.clone()),
          indices: decorated.map((entry) => entry.index),
        };
      }
      const decorated = points.map((point, index) => ({
        point,
        index,
        key: parameterAlongPolyline(point, curvePoints),
      }));
      decorated.sort((a, b) => a.key - b.key || a.index - b.index);
      return {
        points: decorated.map((entry) => entry.point.clone()),
        indices: decorated.map((entry) => entry.index),
      };
    },
  });
  register(['{6eaffbb2-3392-441a-8556-2dc126aa8910}', 'cull duplicates', 'cullpt'], {
    type: 'point',
    pinMap: {
      inputs: { P: 'points', Points: 'points', points: 'points', T: 'tolerance', tol: 'tolerance', Tolerance: 'tolerance' },
      outputs: { P: 'points', Points: 'points', I: 'indices', indices: 'indices', V: 'valence', Valence: 'valence' },
    },
    eval: ({ inputs }) => {
      const tolerance = Math.max(0, toNumber(inputs.tolerance, 0));
      const points = collectPoints(inputs.points);
      const unique = [];
      const indices = [];
      const valence = [];
      points.forEach((point, index) => {
        let matchIndex = -1;
        for (let i = 0; i < unique.length; i += 1) {
          if (unique[i].distanceTo(point) <= tolerance) {
            matchIndex = i;
            break;
          }
        }
        if (matchIndex === -1) {
          unique.push(point.clone());
          indices.push(index);
          valence.push(1);
        } else {
          valence[matchIndex] = (valence[matchIndex] ?? 1) + 1;
        }
      });
      return { points: unique, indices, valence };
    },
  });

  register(['{81f6afc9-22d9-49f0-8579-1fd7e0df6fa6}', 'point groups', 'pgroups'], {
    type: 'point',
    pinMap: {
      inputs: { P: 'points', Points: 'points', points: 'points', D: 'distance', Distance: 'distance', distance: 'distance' },
      outputs: { G: 'groups', Groups: 'groups', I: 'indices', indices: 'indices' },
    },
    eval: ({ inputs }) => {
      const threshold = Math.max(0, toNumber(inputs.distance, 0));
      const points = collectPoints(inputs.points);
      const { find, union } = unionFind(points.length);
      for (let i = 0; i < points.length; i += 1) {
        for (let j = i + 1; j < points.length; j += 1) {
          if (points[i].distanceTo(points[j]) <= threshold) {
            union(i, j);
          }
        }
      }
      const groupsByRoot = new Map();
      for (let i = 0; i < points.length; i += 1) {
        const root = find(i);
        if (!groupsByRoot.has(root)) {
          groupsByRoot.set(root, []);
        }
        groupsByRoot.get(root).push(i);
      }
      const groups = [];
      const indices = [];
      for (const list of groupsByRoot.values()) {
        groups.push(list.map((index) => points[index].clone()));
        indices.push(list.slice());
      }
      return { groups, indices };
    },
  });

  register(['{9adffd61-f5d1-4e9e-9572-e8d9145730dc}', 'barycentric', 'bcentric'], {
    type: 'point',
    pinMap: {
      inputs: { A: 'a', a: 'a', B: 'b', b: 'b', C: 'c', c: 'c', U: 'u', u: 'u', V: 'v', v: 'v', W: 'w', w: 'w' },
      outputs: { P: 'point', Pt: 'point' },
    },
    eval: ({ inputs }) => {
      const pointA = toVector3(inputs.a, new THREE.Vector3());
      const pointB = toVector3(inputs.b, new THREE.Vector3());
      const pointC = toVector3(inputs.c, new THREE.Vector3());
      const u = toNumber(inputs.u, 0);
      const v = toNumber(inputs.v, 0);
      const w = toNumber(inputs.w, 0);
      const total = u + v + w;
      const result = new THREE.Vector3();
      result.add(pointA.clone().multiplyScalar(u));
      result.add(pointB.clone().multiplyScalar(v));
      result.add(pointC.clone().multiplyScalar(w));
      if (Math.abs(total) > EPSILON) {
        result.multiplyScalar(1 / total);
      }
      return { point: result };
    },
  });

  register(['{a435f5c8-28a2-43e8-a52a-0b6e73c2e300}', 'point polar'], {
    type: 'point',
    pinMap: {
      inputs: { P: 'plane', plane: 'plane', Plane: 'plane', xy: 'phi', XY: 'phi', z: 'theta', Z: 'theta', d: 'radius', D: 'radius' },
      outputs: { Pt: 'point', P: 'point' },
    },
    eval: ({ inputs }) => {
      const plane = ensurePlane(inputs.plane);
      const phi = toNumber(inputs.phi ?? inputs.xy, 0);
      const theta = toNumber(inputs.theta ?? inputs.z, 0);
      const radius = toNumber(inputs.radius ?? inputs.d, 0);
      const horizontal = radius * Math.cos(theta);
      const u = horizontal * Math.cos(phi);
      const v = horizontal * Math.sin(phi);
      const w = radius * Math.sin(theta);
      return { point: applyPlane(plane, u, v, w) };
    },
  });

  register(['{23603075-be64-4d86-9294-c3c125a12104}', 'point cylindrical'], {
    type: 'point',
    pinMap: {
      inputs: { P: 'plane', plane: 'plane', Plane: 'plane', A: 'angle', Angle: 'angle', R: 'radius', radius: 'radius', E: 'elevation', elevation: 'elevation' },
      outputs: { Pt: 'point', P: 'point' },
    },
    eval: ({ inputs }) => {
      const plane = ensurePlane(inputs.plane);
      const angle = toNumber(inputs.angle, 0);
      const radius = toNumber(inputs.radius, 0);
      const elevation = toNumber(inputs.elevation, 0);
      const u = radius * Math.cos(angle);
      const v = radius * Math.sin(angle);
      return { point: applyPlane(plane, u, v, elevation) };
    },
  });

  register(['{aa333235-5922-424c-9002-1e0b866a854b}', 'point oriented'], {
    type: 'point',
    pinMap: {
      inputs: { P: 'plane', plane: 'plane', Plane: 'plane', U: 'u', u: 'u', V: 'v', v: 'v', W: 'w', w: 'w' },
      outputs: { Pt: 'point', P: 'point' },
    },
    eval: ({ inputs }) => {
      const plane = ensurePlane(inputs.plane);
      const u = toNumber(inputs.u, 0);
      const v = toNumber(inputs.v, 0);
      const w = toNumber(inputs.w, 0);
      return { point: applyPlane(plane, u, v, w) };
    },
  });

  register(['{d24169cc-9922-4923-92bc-b9222efc413f}', 'points to numbers', 'pt2num'], {
    type: 'point',
    pinMap: {
      inputs: { P: 'points', Points: 'points', points: 'points', M: 'mask', Mask: 'mask', mask: 'mask' },
      outputs: { N: 'numbers', Numbers: 'numbers' },
    },
    eval: ({ inputs }) => {
      const points = collectPoints(inputs.points);
      const mask = maskToComponents(inputs.mask);
      const numbers = [];
      for (const point of points) {
        numbers.push(...pointToNumbers(point, mask));
      }
      return { numbers };
    },
  });
  register(['{99f1e47c-978d-468f-bb3d-a3df44552a8e}', 'grid rectangular obsolete', 'recgrid'], {
    type: 'point',
    pinMap: {
      inputs: { P: 'plane', plane: 'plane', Plane: 'plane', X: 'xCount', x: 'xCount', Y: 'yCount', y: 'yCount', S: 'spacing', s: 'spacing' },
      outputs: { G: 'grid', grid: 'grid', C: 'cells', cells: 'cells', M: 'centers', centers: 'centers' },
    },
    eval: ({ inputs }) => {
      const plane = ensurePlane(inputs.plane);
      const xCount = Math.max(0, Math.floor(toNumber(inputs.xCount, 0)));
      const yCount = Math.max(0, Math.floor(toNumber(inputs.yCount, 0)));
      const spacing = toNumber(inputs.spacing, 1) || 1;
      const grid = [];
      const cells = [];
      const centers = [];
      for (let y = 0; y <= yCount; y += 1) {
        for (let x = 0; x <= xCount; x += 1) {
          grid.push(applyPlane(plane, x * spacing, y * spacing, 0));
        }
      }
      for (let y = 0; y < yCount; y += 1) {
        for (let x = 0; x < xCount; x += 1) {
          const p0 = applyPlane(plane, x * spacing, y * spacing, 0);
          const p1 = applyPlane(plane, (x + 1) * spacing, y * spacing, 0);
          const p2 = applyPlane(plane, (x + 1) * spacing, (y + 1) * spacing, 0);
          const p3 = applyPlane(plane, x * spacing, (y + 1) * spacing, 0);
          cells.push([p0, p1, p2, p3]);
          const center = applyPlane(plane, (x + 0.5) * spacing, (y + 0.5) * spacing, 0);
          centers.push(createPlaneInstance(plane, center));
        }
      }
      return { grid, cells, centers };
    },
  });

  register(['{8ce6a747-6d36-4bd4-8af0-9a1081df417d}', 'grid hexagonal obsolete', 'hexgrid'], {
    type: 'point',
    pinMap: {
      inputs: { P: 'plane', plane: 'plane', Plane: 'plane', R: 'radius', radius: 'radius', S: 'spacing', s: 'spacing' },
      outputs: { G: 'grid', grid: 'grid', C: 'cells', cells: 'cells', M: 'centers', centers: 'centers' },
    },
    eval: ({ inputs }) => {
      const plane = ensurePlane(inputs.plane);
      const radius = Math.max(0, Math.floor(toNumber(inputs.radius, 0)));
      const spacing = toNumber(inputs.spacing, 1) || 1;
      const grid = [];
      const cells = [];
      const centers = [];
      const vertexRadius = spacing / Math.sqrt(3);
      for (let q = -radius; q <= radius; q += 1) {
        const rMin = Math.max(-radius, -q - radius);
        const rMax = Math.min(radius, -q + radius);
        for (let r = rMin; r <= rMax; r += 1) {
          const x = spacing * (Math.sqrt(3) * q + (Math.sqrt(3) / 2) * r);
          const y = spacing * (1.5 * r);
          const centerPoint = applyPlane(plane, x, y, 0);
          grid.push(centerPoint);
          centers.push(createPlaneInstance(plane, centerPoint));
          const cell = [];
          for (let i = 0; i < 6; i += 1) {
            const angle = Math.PI / 3 * i + Math.PI / 6;
            const vx = x + vertexRadius * Math.cos(angle);
            const vy = y + vertexRadius * Math.sin(angle);
            cell.push(applyPlane(plane, vx, vy, 0));
          }
          cells.push(cell);
        }
      }
      return { grid, cells, centers };
    },
  });
  register(['{4b3d38d3-0620-42e5-9ae8-0d4d9ad914cd}', 'text tag', 'tag'], {
    type: 'annotation',
    pinMap: {
      inputs: { L: 'location', location: 'location', Location: 'location', T: 'text', Text: 'text', text: 'text' },
    },
    eval: ({ inputs }) => {
      const location = toVector3(inputs.location, new THREE.Vector3());
      const text = toStringValue(inputs.text);
      const tag = { type: '2d', location, text };
      return { tags: [tag] };
    },
  });

  register([
    '{18564c36-5652-4c63-bb6f-f0e1273666dd}',
    '{ebf4d987-09b9-4825-a735-cac3d4770c19}',
    'text tag 3d',
  ], {
    type: 'annotation',
    pinMap: {
      inputs: {
        L: 'location', location: 'location', Location: 'location',
        T: 'text', Text: 'text', text: 'text',
        S: 'size', Size: 'size', size: 'size',
        C: 'colour', Colour: 'colour', color: 'colour', Color: 'colour',
      },
    },
    eval: ({ inputs }) => {
      const plane = ensurePlane(inputs.location);
      const text = toStringValue(inputs.text);
      const size = Math.max(0, toNumber(inputs.size, 1));
      let colour = null;
      if (inputs.colour !== undefined) {
        try {
          const color = new THREE.Color();
          color.set(inputs.colour);
          colour = `#${color.getHexString()}`;
        } catch (error) {
          colour = String(inputs.colour);
        }
      }
      const tag = { type: '3d', plane, text, size, colour };
      return { tags: [tag] };
    },
  });
}


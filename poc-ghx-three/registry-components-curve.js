import * as THREE from 'three';

export function registerCurvePrimitiveComponents({ register, toNumber, toVector3 }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register curve primitive components.');
  }
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register curve primitive components.');
  }
  if (typeof toVector3 !== 'function') {
    throw new Error('toVector3 function is required to register curve primitive components.');
  }

  const EPSILON = 1e-9;

  function cloneVector(vector) {
    if (vector?.isVector3) {
      return vector.clone();
    }
    return new THREE.Vector3();
  }

  function ensurePoint(value, fallback = new THREE.Vector3()) {
    return toVector3(value, fallback.clone());
  }

  function ensureArray(input) {
    if (input === undefined || input === null) {
      return [];
    }
    if (Array.isArray(input)) {
      return input;
    }
    return [input];
  }

  function collectPoints(input) {
    const result = [];

    function visit(value) {
      if (value === undefined || value === null) {
        return;
      }
      if (value?.isVector3) {
        result.push(value.clone());
        return;
      }
      if (Array.isArray(value)) {
        for (const entry of value) {
          visit(entry);
        }
        return;
      }
      if (typeof value === 'object') {
        if ('point' in value) {
          visit(value.point);
          return;
        }
        if ('points' in value) {
          visit(value.points);
          return;
        }
        if ('position' in value) {
          visit(value.position);
          return;
        }
        if ('x' in value || 'y' in value || 'z' in value) {
          const point = toVector3(value, null);
          if (point) {
            result.push(point);
          }
          return;
        }
      }
    }

    visit(input);
    return result;
  }

  function normalizeVector(vector, fallback = new THREE.Vector3(1, 0, 0)) {
    const result = vector.clone();
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
    const z = zAxis.clone().normalize();
    const x = xAxis.clone().normalize();
    let y = yAxis.clone();
    if (y.lengthSq() < EPSILON) {
      y = z.clone().cross(x);
    }
    y.normalize();
    const orthogonalX = y.clone().cross(z).normalize();
    const orthogonalY = z.clone().cross(orthogonalX).normalize();
    return {
      origin: origin.clone(),
      xAxis: orthogonalX,
      yAxis: orthogonalY,
      zAxis: z,
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

  function createPlane(origin, xAxis, yAxis, normal) {
    const zAxis = normal ? normal.clone() : xAxis.clone().cross(yAxis).normalize();
    return normalizePlaneAxes(origin, xAxis, yAxis, zAxis);
  }

  function ensurePlane(input) {
    if (input === undefined || input === null) {
      return defaultPlane();
    }
    if (Array.isArray(input)) {
      const points = collectPoints(input);
      if (points.length >= 3) {
        return planeFromPoints(points[0], points[1], points[2]);
      }
      if (points.length === 2) {
        const origin = points[0];
        const xAxis = points[1].clone().sub(points[0]).normalize();
        const normal = orthogonalVector(xAxis);
        return createPlane(origin, xAxis, normal.clone().cross(xAxis), normal);
      }
      if (points.length === 1) {
        return createPlane(points[0], new THREE.Vector3(1, 0, 0), new THREE.Vector3(0, 1, 0));
      }
    }
    if (typeof input === 'object') {
      if (input.origin && input.xAxis && input.yAxis && input.zAxis) {
        return normalizePlaneAxes(
          ensurePoint(input.origin, new THREE.Vector3()),
          normalizeVector(ensurePoint(input.xAxis, new THREE.Vector3(1, 0, 0)), new THREE.Vector3(1, 0, 0)),
          normalizeVector(ensurePoint(input.yAxis, new THREE.Vector3(0, 1, 0)), new THREE.Vector3(0, 1, 0)),
          normalizeVector(ensurePoint(input.zAxis, new THREE.Vector3(0, 0, 1)), new THREE.Vector3(0, 0, 1)),
        );
      }
      if (input.origin && input.normal) {
        const normal = normalizeVector(ensurePoint(input.normal, new THREE.Vector3(0, 0, 1)), new THREE.Vector3(0, 0, 1));
        const xAxis = orthogonalVector(normal);
        const yAxis = normal.clone().cross(xAxis).normalize();
        return createPlane(ensurePoint(input.origin, new THREE.Vector3()), xAxis, yAxis, normal);
      }
      if (input.point && input.normal) {
        const origin = ensurePoint(input.point, new THREE.Vector3());
        const normal = normalizeVector(ensurePoint(input.normal, new THREE.Vector3(0, 0, 1)), new THREE.Vector3(0, 0, 1));
        const xAxis = orthogonalVector(normal);
        const yAxis = normal.clone().cross(xAxis).normalize();
        return createPlane(origin, xAxis, yAxis, normal);
      }
      if (input.plane) {
        return ensurePlane(input.plane);
      }
    }
    return defaultPlane();
  }

  function planeFromPoints(a, b, c) {
    const origin = ensurePoint(a, new THREE.Vector3());
    const ab = ensurePoint(b, origin.clone()).sub(origin.clone());
    const ac = ensurePoint(c, origin.clone()).sub(origin.clone());
    const normal = ab.clone().cross(ac);
    if (normal.lengthSq() < EPSILON) {
      return defaultPlane();
    }
    const xAxis = ab.lengthSq() < EPSILON ? orthogonalVector(normal) : ab.clone().normalize();
    const yAxis = normal.clone().cross(xAxis).normalize();
    return createPlane(origin, xAxis, yAxis, normal);
  }

  function planeCoordinates(point, plane) {
    const relative = ensurePoint(point, plane.origin.clone()).clone().sub(plane.origin);
    return {
      x: relative.dot(plane.xAxis),
      y: relative.dot(plane.yAxis),
      z: relative.dot(plane.zAxis),
    };
  }

  function applyPlane(plane, x, y, z = 0) {
    const result = plane.origin.clone();
    result.add(plane.xAxis.clone().multiplyScalar(x));
    result.add(plane.yAxis.clone().multiplyScalar(y));
    result.add(plane.zAxis.clone().multiplyScalar(z));
    return result;
  }

  function createLine(start, end) {
    const a = ensurePoint(start, new THREE.Vector3());
    const b = ensurePoint(end, new THREE.Vector3(1, 0, 0));
    const direction = b.clone().sub(a);
    const length = direction.length();
    const safeDirection = length > EPSILON ? direction.clone().divideScalar(length) : new THREE.Vector3(1, 0, 0);
    return {
      type: 'line',
      start: a,
      end: b,
      length,
      direction: safeDirection,
    };
  }

  function createCircleData({ plane, center, radius, segments = 128 }) {
    const shape = new THREE.Shape();
    shape.absarc(0, 0, Math.max(radius, EPSILON), 0, Math.PI * 2, false);
    return {
      type: 'circle',
      plane,
      center,
      radius,
      shape,
      segments,
    };
  }

  function circleFromCenterNormalRadius(centerInput, normalInput, radiusInput) {
    const center = ensurePoint(centerInput, new THREE.Vector3());
    const radius = Math.max(toNumber(radiusInput, 1), EPSILON);
    const normal = normalizeVector(ensurePoint(normalInput, new THREE.Vector3(0, 0, 1)), new THREE.Vector3(0, 0, 1));
    const xAxis = orthogonalVector(normal);
    const yAxis = normal.clone().cross(xAxis).normalize();
    const plane = createPlane(center, xAxis, yAxis, normal);
    const shape = new THREE.Shape();
    shape.absarc(0, 0, radius, 0, Math.PI * 2, false);
    return {
      type: 'circle',
      plane,
      center,
      radius,
      shape,
      segments: 128,
    };
  }

  function circleFromThreePoints(aInput, bInput, cInput) {
    const a = ensurePoint(aInput, new THREE.Vector3());
    const b = ensurePoint(bInput, new THREE.Vector3(1, 0, 0));
    const c = ensurePoint(cInput, new THREE.Vector3(0, 1, 0));
    const plane = planeFromPoints(a, b, c);
    const abMid = a.clone().add(b).multiplyScalar(0.5);
    const acMid = a.clone().add(c).multiplyScalar(0.5);
    const abDir = b.clone().sub(a);
    const acDir = c.clone().sub(a);
    const normal = plane.zAxis.clone();
    const abPerp = normal.clone().cross(abDir).normalize();
    const acPerp = normal.clone().cross(acDir).normalize();
    const center = intersectLines(abMid, abMid.clone().add(abPerp), acMid, acMid.clone().add(acPerp));
    const radius = center ? center.distanceTo(a) : 0;
    if (!center || radius < EPSILON) {
      return null;
    }
    return createCircleData({ plane, center, radius });
  }

  function intersectLines(p1, p2, p3, p4) {
    const v1 = p2.clone().sub(p1);
    const v2 = p4.clone().sub(p3);
    const normal = v1.clone().cross(v2);
    if (normal.lengthSq() < EPSILON) {
      return null;
    }
    const plane = createPlane(p1, v1, normal.clone().cross(v1), normal);
    const lineDirection = v1.clone().normalize();
    const t = plane.zAxis.dot(p3.clone().sub(p1)) / plane.zAxis.dot(lineDirection);
    return p1.clone().add(lineDirection.multiplyScalar(t));
  }

  function createRectangleShape(width, height, radius = 0) {
    const w = Math.max(Math.abs(width), EPSILON);
    const h = Math.max(Math.abs(height), EPSILON);
    const fillet = Math.min(Math.max(radius, 0), Math.min(w, h) / 2);
    const halfW = w / 2;
    const halfH = h / 2;
    const shape = new THREE.Shape();

    if (fillet <= EPSILON) {
      shape.moveTo(-halfW, -halfH);
      shape.lineTo(halfW, -halfH);
      shape.lineTo(halfW, halfH);
      shape.lineTo(-halfW, halfH);
      shape.lineTo(-halfW, -halfH);
    } else {
      shape.moveTo(-halfW + fillet, -halfH);
      shape.lineTo(halfW - fillet, -halfH);
      shape.quadraticCurveTo(halfW, -halfH, halfW, -halfH + fillet);
      shape.lineTo(halfW, halfH - fillet);
      shape.quadraticCurveTo(halfW, halfH, halfW - fillet, halfH);
      shape.lineTo(-halfW + fillet, halfH);
      shape.quadraticCurveTo(-halfW, halfH, -halfW, halfH - fillet);
      shape.lineTo(-halfW, -halfH + fillet);
      shape.quadraticCurveTo(-halfW, -halfH, -halfW + fillet, -halfH);
    }
    return { shape, fillet };
  }

  function perimeterOfRectangle(width, height, radius = 0) {
    const w = Math.abs(width);
    const h = Math.abs(height);
    const fillet = Math.min(Math.max(radius, 0), Math.min(w, h) / 2);
    if (fillet <= EPSILON) {
      return 2 * (w + h);
    }
    const straight = 2 * ((w - 2 * fillet) + (h - 2 * fillet));
    const arc = 2 * Math.PI * fillet;
    return straight + arc;
  }

  function createRectangleFromDimensions(planeInput, widthInput, heightInput, radiusInput) {
    const plane = ensurePlane(planeInput);
    const width = Math.max(toNumber(widthInput, 1), EPSILON);
    const height = Math.max(toNumber(heightInput, 1), EPSILON);
    const radius = Math.max(toNumber(radiusInput, 0), 0);
    const { shape, fillet } = createRectangleShape(width, height, radius);
    const corners = [
      applyPlane(plane, -width / 2, -height / 2, 0),
      applyPlane(plane, width / 2, -height / 2, 0),
      applyPlane(plane, width / 2, height / 2, 0),
      applyPlane(plane, -width / 2, height / 2, 0),
    ];
    const length = perimeterOfRectangle(width, height, fillet);
    return {
      type: 'rectangle',
      plane,
      width,
      height,
      radius: fillet,
      corners,
      shape,
      segments: 4,
      length,
    };
  }

  function rectangleFromThreePoints(aInput, bInput, cInput) {
    const a = ensurePoint(aInput, new THREE.Vector3());
    const b = ensurePoint(bInput, new THREE.Vector3(1, 0, 0));
    const c = ensurePoint(cInput, new THREE.Vector3(0, 1, 0));
    const plane = planeFromPoints(a, b, c);
    const ab = b.clone().sub(a);
    const width = ab.length();
    const xAxis = width > EPSILON ? ab.clone().divideScalar(width) : plane.xAxis.clone();
    const ac = c.clone().sub(a);
    const heightVector = ac.clone().sub(xAxis.clone().multiplyScalar(ac.dot(xAxis)));
    const height = heightVector.length();
    const yAxis = height > EPSILON ? heightVector.clone().divideScalar(height) : plane.yAxis.clone();
    const normalizedPlane = createPlane(a.clone(), xAxis, yAxis, plane.zAxis.clone());
    const corners = [
      a.clone(),
      a.clone().add(xAxis.clone().multiplyScalar(width)),
      a.clone().add(xAxis.clone().multiplyScalar(width)).add(yAxis.clone().multiplyScalar(height)),
      a.clone().add(yAxis.clone().multiplyScalar(height)),
    ];
    const shape = new THREE.Shape();
    shape.moveTo(0, 0);
    shape.lineTo(width, 0);
    shape.lineTo(width, height);
    shape.lineTo(0, height);
    shape.lineTo(0, 0);
    const length = perimeterOfRectangle(width, height, 0);
    return {
      type: 'rectangle',
      plane: normalizedPlane,
      width,
      height,
      radius: 0,
      corners,
      shape,
      segments: 4,
      length,
    };
  }

  function rectangleFromTwoPoints(planeInput, aInput, bInput, radiusInput) {
    const plane = ensurePlane(planeInput);
    const a = ensurePoint(aInput, plane.origin.clone());
    const b = ensurePoint(bInput, plane.origin.clone().add(plane.xAxis));
    const coordA = planeCoordinates(a, plane);
    const coordB = planeCoordinates(b, plane);
    const minX = Math.min(coordA.x, coordB.x);
    const maxX = Math.max(coordA.x, coordB.x);
    const minY = Math.min(coordA.y, coordB.y);
    const maxY = Math.max(coordA.y, coordB.y);
    const width = Math.max(maxX - minX, EPSILON);
    const height = Math.max(maxY - minY, EPSILON);
    const radius = Math.max(toNumber(radiusInput, 0), 0);
    const { shape, fillet } = createRectangleShape(width, height, radius);
    const corners = [
      applyPlane(plane, minX, minY, 0),
      applyPlane(plane, maxX, minY, 0),
      applyPlane(plane, maxX, maxY, 0),
      applyPlane(plane, minX, maxY, 0),
    ];
    const length = perimeterOfRectangle(width, height, fillet);
    return {
      type: 'rectangle',
      plane,
      width,
      height,
      radius: fillet,
      corners,
      shape,
      segments: 4,
      length,
    };
  }

  function ellipseFromPlane(planeInput, radius1Input, radius2Input) {
    const plane = ensurePlane(planeInput);
    const radius1 = Math.max(toNumber(radius1Input, 1), EPSILON);
    const radius2 = Math.max(toNumber(radius2Input, radius1), EPSILON);
    const shape = new THREE.Shape();
    shape.absellipse(0, 0, radius1, radius2, 0, Math.PI * 2, false, 0);
    const focusDistance = Math.sqrt(Math.max(radius1 * radius1 - radius2 * radius2, 0));
    const f1 = applyPlane(plane, focusDistance, 0, 0);
    const f2 = applyPlane(plane, -focusDistance, 0, 0);
    return {
      type: 'ellipse',
      plane,
      radius1,
      radius2,
      shape,
      segments: 128,
      foci: [f1, f2],
    };
  }

  function createArcFromAngles(plane, radius, startAngle, endAngle) {
    const path = new THREE.Path();
    path.absarc(0, 0, Math.max(radius, EPSILON), startAngle, endAngle, endAngle < startAngle);
    const start = applyPlane(plane, Math.cos(startAngle) * radius, Math.sin(startAngle) * radius, 0);
    const end = applyPlane(plane, Math.cos(endAngle) * radius, Math.sin(endAngle) * radius, 0);
    const midAngle = (startAngle + endAngle) / 2;
    const mid = applyPlane(plane, Math.cos(midAngle) * radius, Math.sin(midAngle) * radius, 0);
    const length = Math.abs(endAngle - startAngle) * radius;
    return {
      type: 'arc',
      plane,
      radius,
      startAngle,
      endAngle,
      start,
      end,
      mid,
      length,
      path,
    };
  }

  function createArcFromPlaneRadiusAngles(planeInput, radiusInput, angleInput) {
    const plane = ensurePlane(planeInput);
    const radius = Math.max(toNumber(radiusInput, 1), EPSILON);
    const angleDomain = ensureArray(angleInput);
    let startAngle = 0;
    let endAngle = Math.PI / 2;
    if (angleDomain.length >= 2) {
      startAngle = toNumber(angleDomain[0], 0);
      endAngle = toNumber(angleDomain[1], Math.PI / 2);
    } else if (angleDomain.length === 1) {
      endAngle = toNumber(angleDomain[0], Math.PI / 2);
    }
    return createArcFromAngles(plane, radius, startAngle, endAngle);
  }

  function arcFromThreePoints(aInput, bInput, cInput) {
    const circle = circleFromThreePoints(aInput, bInput, cInput);
    if (!circle) {
      return null;
    }
    const plane = circle.plane;
    const center = circle.center;
    const radius = circle.radius;
    const a = ensurePoint(aInput, center.clone());
    const b = ensurePoint(bInput, center.clone());
    const c = ensurePoint(cInput, center.clone());
    const coordsA = planeCoordinates(a, plane);
    const coordsB = planeCoordinates(b, plane);
    const coordsC = planeCoordinates(c, plane);
    const startAngle = Math.atan2(coordsA.y, coordsA.x);
    const midAngle = Math.atan2(coordsB.y, coordsB.x);
    const endAngle = Math.atan2(coordsC.y, coordsC.x);
    let normalizedStart = startAngle;
    let normalizedEnd = endAngle;
    if (!isAngleBetween(midAngle, startAngle, endAngle)) {
      if (normalizedEnd < normalizedStart) {
        normalizedEnd += Math.PI * 2;
      } else {
        normalizedStart += Math.PI * 2;
      }
    }
    const arc = createArcFromAngles(plane, radius, normalizedStart, normalizedEnd);
    arc.center = center;
    return arc;
  }

  function isAngleBetween(angle, start, end) {
    const normalizedAngle = normalizeAngle(angle);
    let normalizedStart = normalizeAngle(start);
    let normalizedEnd = normalizeAngle(end);
    if (normalizedStart <= normalizedEnd) {
      return normalizedAngle >= normalizedStart && normalizedAngle <= normalizedEnd;
    }
    return normalizedAngle >= normalizedStart || normalizedAngle <= normalizedEnd;
  }

  function normalizeAngle(angle) {
    let result = angle;
    while (result < 0) {
      result += Math.PI * 2;
    }
    while (result >= Math.PI * 2) {
      result -= Math.PI * 2;
    }
    return result;
  }

  function arcFromSED(startInput, endInput, directionInput) {
    const start = ensurePoint(startInput, new THREE.Vector3());
    const end = ensurePoint(endInput, new THREE.Vector3(1, 0, 0));
    const direction = ensurePoint(directionInput, new THREE.Vector3(1, 0, 0));
    const chord = end.clone().sub(start);
    const chordLength = chord.length();
    if (chordLength < EPSILON) {
      return null;
    }
    const tangent = normalizeVector(direction.clone(), chord.clone().normalize());
    const normal = chord.clone().cross(tangent);
    if (normal.lengthSq() < EPSILON) {
      const fallbackNormal = orthogonalVector(chord.clone().normalize());
      normal.copy(fallbackNormal);
    }
    const plane = createPlane(start.clone(), tangent.clone(), normal.clone().cross(tangent), normal);
    const mid = start.clone().add(end).multiplyScalar(0.5);
    const midChord = mid.clone().sub(start).length();
    const tangentProjection = chord.clone().normalize().dot(tangent);
    const radiusDenominator = 2 * (1 - tangentProjection);
    if (Math.abs(radiusDenominator) < EPSILON) {
      return null;
    }
    const radius = chordLength / radiusDenominator;
    const center = start.clone().add(tangent.clone().multiplyScalar(radius));
    const coordsStart = planeCoordinates(start, plane);
    const coordsEnd = planeCoordinates(end, plane);
    const startAngle = Math.atan2(coordsStart.y, coordsStart.x);
    const endAngle = Math.atan2(coordsEnd.y, coordsEnd.x);
    const arc = createArcFromAngles(plane, Math.abs(radius), startAngle, endAngle);
    arc.center = center;
    return arc;
  }

  function fitCircleToPoints(pointsInput) {
    const points = collectPoints(pointsInput).filter((point) => point instanceof THREE.Vector3);
    if (points.length < 3) {
      return null;
    }
    const centroid = points.reduce((sum, point) => sum.add(point), new THREE.Vector3()).divideScalar(points.length);
    let uu = 0;
    let uv = 0;
    let vv = 0;
    let uuu = 0;
    let uvv = 0;
    let uuv = 0;
    let vvv = 0;
    const plane = fitPlaneToPoints(points);
    if (!plane) {
      return null;
    }
    for (const point of points) {
      const coords = planeCoordinates(point, plane);
      const u = coords.x;
      const v = coords.y;
      const uuLocal = u * u;
      const vvLocal = v * v;
      uu += uuLocal;
      uv += u * v;
      vv += vvLocal;
      uuu += uuLocal * u;
      uvv += u * vvLocal;
      uuv += uuLocal * v;
      vvv += vvLocal * v;
    }
    const denominator = 2 * (uu * vv - uv * uv);
    if (Math.abs(denominator) < EPSILON) {
      return null;
    }
    const uc = (vv * (uuu + uvv) - uv * (vvv + uuv)) / denominator;
    const vc = (uu * (vvv + uuv) - uv * (uuu + uvv)) / denominator;
    const center = applyPlane(plane, uc, vc, 0);
    const radius = Math.sqrt((uc * uc) + (vc * vc) + (uu + vv) / points.length);
    let maxDeviation = 0;
    for (const point of points) {
      const distance = center.distanceTo(point);
      maxDeviation = Math.max(maxDeviation, Math.abs(distance - radius));
    }
    const circle = createCircleData({ plane, center, radius });
    circle.deviation = maxDeviation;
    return circle;
  }

  function fitPlaneToPoints(points) {
    if (!points.length) {
      return null;
    }
    const centroid = points.reduce((sum, point) => sum.add(point), new THREE.Vector3()).divideScalar(points.length);
    let xx = 0;
    let xy = 0;
    let xz = 0;
    let yy = 0;
    let yz = 0;
    let zz = 0;
    for (const point of points) {
      const relative = point.clone().sub(centroid);
      xx += relative.x * relative.x;
      xy += relative.x * relative.y;
      xz += relative.x * relative.z;
      yy += relative.y * relative.y;
      yz += relative.y * relative.z;
      zz += relative.z * relative.z;
    }
    const covariance = new THREE.Matrix3();
    covariance.set(xx, xy, xz, xy, yy, yz, xz, yz, zz);
    const eigen = new THREE.Vector3();
    const eigenVectors = new THREE.Matrix3();
    covariance.clone().transpose();
    const normal = computeSmallestEigenVector(covariance);
    if (!normal) {
      return null;
    }
    const normalizedNormal = normalizeVector(normal, new THREE.Vector3(0, 0, 1));
    const xAxis = orthogonalVector(normalizedNormal);
    const yAxis = normalizedNormal.clone().cross(xAxis).normalize();
    return createPlane(centroid, xAxis, yAxis, normalizedNormal);
  }

  function computeSmallestEigenVector(matrix) {
    const m = matrix.elements;
    let vector = new THREE.Vector3(1, 0, 0);
    for (let iteration = 0; iteration < 32; iteration += 1) {
      const x = m[0] * vector.x + m[1] * vector.y + m[2] * vector.z;
      const y = m[3] * vector.x + m[4] * vector.y + m[5] * vector.z;
      const z = m[6] * vector.x + m[7] * vector.y + m[8] * vector.z;
      vector.set(x, y, z);
      const length = vector.length();
      if (length < EPSILON) {
        break;
      }
      vector.divideScalar(length);
    }
    return vector;
  }

  function registerRectangle() {
    const ids = [
      '{0ca0a214-396c-44ea-b22f-d3a1757c32d6}',
      '{d93100b6-d50b-40b2-831a-814659dc38e3}',
      'rectangle',
    ];
    register(ids, {
      type: 'curve',
      pinMap: {
        inputs: {
          P: 'plane', plane: 'plane', Plane: 'plane',
          X: 'xSize', x: 'xSize', 'X Size': 'xSize',
          Y: 'ySize', y: 'ySize', 'Y Size': 'ySize',
          R: 'radius', r: 'radius', Radius: 'radius',
        },
        outputs: { R: 'rectangle', rectangle: 'rectangle', L: 'length', length: 'length' },
      },
      eval: ({ inputs }) => {
        const rectangle = createRectangleFromDimensions(inputs.plane, inputs.xSize, inputs.ySize, inputs.radius);
        return { rectangle, length: rectangle.length };
      },
    });
  }

  registerRectangle();

  function fitLineToPoints(pointsInput) {
    const points = collectPoints(pointsInput).filter((point) => point instanceof THREE.Vector3);
    if (points.length === 0) {
      return null;
    }
    if (points.length === 1) {
      return createLine(points[0], points[0].clone().add(new THREE.Vector3(1, 0, 0)));
    }
    const centroid = points.reduce((sum, point) => sum.add(point), new THREE.Vector3()).divideScalar(points.length);
    let xx = 0;
    let xy = 0;
    let xz = 0;
    let yy = 0;
    let yz = 0;
    let zz = 0;
    for (const point of points) {
      const relative = point.clone().sub(centroid);
      xx += relative.x * relative.x;
      xy += relative.x * relative.y;
      xz += relative.x * relative.z;
      yy += relative.y * relative.y;
      yz += relative.y * relative.z;
      zz += relative.z * relative.z;
    }
    const covariance = new THREE.Matrix3();
    covariance.set(xx, xy, xz, xy, yy, yz, xz, yz, zz);
    const direction = computeLargestEigenVector(covariance);
    if (!direction || direction.lengthSq() < EPSILON) {
      const fallback = points[points.length - 1].clone().sub(points[0]);
      if (fallback.lengthSq() < EPSILON) {
        return createLine(points[0], points[0].clone().add(new THREE.Vector3(1, 0, 0)));
      }
      return createLine(points[0], points[0].clone().add(fallback));
    }
    const normalizedDirection = direction.clone().normalize();
    let minT = Number.POSITIVE_INFINITY;
    let maxT = Number.NEGATIVE_INFINITY;
    for (const point of points) {
      const relative = point.clone().sub(centroid);
      const projection = relative.dot(normalizedDirection);
      minT = Math.min(minT, projection);
      maxT = Math.max(maxT, projection);
    }
    if (!Number.isFinite(minT) || !Number.isFinite(maxT)) {
      return createLine(centroid.clone(), centroid.clone().add(normalizedDirection));
    }
    const start = centroid.clone().add(normalizedDirection.clone().multiplyScalar(minT));
    const end = centroid.clone().add(normalizedDirection.clone().multiplyScalar(maxT));
    return createLine(start, end);
  }

  function computeLargestEigenVector(matrix) {
    const m = matrix.elements;
    let vector = new THREE.Vector3(1, 0, 0);
    for (let iteration = 0; iteration < 32; iteration += 1) {
      const x = m[0] * vector.x + m[1] * vector.y + m[2] * vector.z;
      const y = m[3] * vector.x + m[4] * vector.y + m[5] * vector.z;
      const z = m[6] * vector.x + m[7] * vector.y + m[8] * vector.z;
      vector.set(x, y, z);
      const length = vector.length();
      if (length < EPSILON) {
        break;
      }
      vector.divideScalar(length);
    }
    return vector;
  }

  function lineFromStartDirectionLength(startInput, directionInput, lengthInput) {
    const start = ensurePoint(startInput, new THREE.Vector3());
    const direction = normalizeVector(ensurePoint(directionInput, new THREE.Vector3(1, 0, 0)), new THREE.Vector3(1, 0, 0));
    const length = Math.max(toNumber(lengthInput, 1), EPSILON);
    const end = start.clone().add(direction.clone().multiplyScalar(length));
    return createLine(start, end);
  }

  function lineFromGuideAndPlanes(lineInput, planeAInput, planeBInput) {
    const guide = ensureLine(lineInput);
    if (!guide) {
      return null;
    }
    const planeA = ensurePlane(planeAInput);
    const planeB = ensurePlane(planeBInput);
    const start = intersectLineWithPlane(guide, planeA);
    const end = intersectLineWithPlane(guide, planeB);
    if (!start || !end) {
      return null;
    }
    return createLine(start, end);
  }

  function ensureLine(input) {
    if (!input) {
      return null;
    }
    if (input.type === 'line' && input.start && input.end) {
      return createLine(input.start, input.end);
    }
    const points = collectPoints(input);
    if (points.length >= 2) {
      return createLine(points[0], points[points.length - 1]);
    }
    if (Array.isArray(input) && input.length === 2) {
      return createLine(ensurePoint(input[0], new THREE.Vector3()), ensurePoint(input[1], new THREE.Vector3(1, 0, 0)));
    }
    return null;
  }

  function intersectLineWithPlane(line, plane) {
    const direction = line.end.clone().sub(line.start);
    const denominator = plane.zAxis.dot(direction);
    if (Math.abs(denominator) < EPSILON) {
      return null;
    }
    const t = plane.zAxis.dot(plane.origin.clone().sub(line.start)) / denominator;
    return line.start.clone().add(direction.multiplyScalar(t));
  }

  function lineFromGuideAndPoints(lineInput, pointAInput, pointBInput) {
    const guide = ensureLine(lineInput);
    if (!guide) {
      return null;
    }
    const direction = guide.end.clone().sub(guide.start);
    if (direction.lengthSq() < EPSILON) {
      return null;
    }
    const normalized = direction.clone().normalize();
    const a = ensurePoint(pointAInput, guide.start.clone());
    const b = ensurePoint(pointBInput, guide.end.clone());
    const projA = projectPointOntoLineParameter(a, guide.start, normalized);
    const projB = projectPointOntoLineParameter(b, guide.start, normalized);
    const start = guide.start.clone().add(normalized.clone().multiplyScalar(projA));
    const end = guide.start.clone().add(normalized.clone().multiplyScalar(projB));
    return createLine(start, end);
  }

  function projectPointOntoLineParameter(point, origin, direction) {
    const relative = point.clone().sub(origin);
    return relative.dot(direction);
  }

  function triangleIncircle(aInput, bInput, cInput) {
    const a = ensurePoint(aInput, new THREE.Vector3());
    const b = ensurePoint(bInput, new THREE.Vector3(1, 0, 0));
    const c = ensurePoint(cInput, new THREE.Vector3(0, 1, 0));
    const plane = planeFromPoints(a, b, c);
    const sideA = b.clone().sub(c).length();
    const sideB = a.clone().sub(c).length();
    const sideC = a.clone().sub(b).length();
    const perimeter = sideA + sideB + sideC;
    if (perimeter < EPSILON) {
      return null;
    }
    const center = new THREE.Vector3();
    center.add(a.clone().multiplyScalar(sideA));
    center.add(b.clone().multiplyScalar(sideB));
    center.add(c.clone().multiplyScalar(sideC));
    center.divideScalar(perimeter);
    const semiPerimeter = perimeter / 2;
    const areaSquared = semiPerimeter * (semiPerimeter - sideA) * (semiPerimeter - sideB) * (semiPerimeter - sideC);
    const area = areaSquared > 0 ? Math.sqrt(areaSquared) : 0;
    if (area <= EPSILON) {
      return null;
    }
    const radius = area / semiPerimeter;
    return createCircleData({ plane, center, radius });
  }

  function registerRectangleThreePoint() {
    const ids = [
      '{34493ef6-3dfb-47c0-b149-691d02a93588}',
      '{9bc98a1d-2ecc-407e-948a-09a09ed3e69d}',
      'rectangle 3pt',
      'rect 3pt',
    ];
    register(ids, {
      type: 'curve',
      pinMap: {
        inputs: { A: 'pointA', B: 'pointB', C: 'pointC', a: 'pointA', b: 'pointB', c: 'pointC' },
        outputs: { R: 'rectangle', rectangle: 'rectangle', L: 'length', length: 'length' },
      },
      eval: ({ inputs }) => {
        const rectangle = rectangleFromThreePoints(inputs.pointA, inputs.pointB, inputs.pointC);
        if (!rectangle) {
          return {};
        }
        return { rectangle, length: rectangle.length };
      },
    });
  }

  function registerRectangleTwoPoint() {
    register('{575660b1-8c79-4b8d-9222-7ab4a6ddb359}', {
      type: 'curve',
      pinMap: {
        inputs: {
          P: 'plane', plane: 'plane', Plane: 'plane',
          A: 'pointA', a: 'pointA',
          B: 'pointB', b: 'pointB',
          R: 'radius', r: 'radius', Radius: 'radius',
        },
        outputs: { R: 'rectangle', rectangle: 'rectangle', L: 'length', length: 'length' },
      },
      eval: ({ inputs }) => {
        const rectangle = rectangleFromTwoPoints(inputs.plane, inputs.pointA, inputs.pointB, inputs.radius);
        if (!rectangle) {
          return {};
        }
        return { rectangle, length: rectangle.length };
      },
    });
  }

  function registerFitLine() {
    register('{1f798a28-9de6-47b5-8201-cac57256b777}', {
      type: 'curve',
      pinMap: {
        inputs: { P: 'points', Points: 'points', points: 'points' },
        outputs: { L: 'line', line: 'line' },
      },
      eval: ({ inputs }) => {
        const line = fitLineToPoints(inputs.points);
        if (!line) {
          return {};
        }
        return { line };
      },
    });
  }

  function registerInCircle() {
    register('{28b1c4d4-ab1c-4309-accd-1b7a954ed948}', {
      type: 'curve',
      pinMap: {
        inputs: { A: 'cornerA', B: 'cornerB', C: 'cornerC', a: 'cornerA', b: 'cornerB', c: 'cornerC' },
        outputs: { C: 'circle', circle: 'circle', P: 'plane', plane: 'plane', R: 'radius', radius: 'radius' },
      },
      eval: ({ inputs }) => {
        const circle = triangleIncircle(inputs.cornerA, inputs.cornerB, inputs.cornerC);
        if (!circle) {
          return {};
        }
        return { circle, plane: circle.plane, radius: circle.radius };
      },
    });
  }

  function registerCircleThreePoint() {
    const ids = [
      '{47886835-e3ff-4516-a3ed-1b419f055464}',
      'circle 3pt',
      'circle three point',
    ];
    register(ids, {
      type: 'curve',
      pinMap: {
        inputs: { A: 'pointA', B: 'pointB', C: 'pointC', a: 'pointA', b: 'pointB', c: 'pointC' },
        outputs: { C: 'circle', circle: 'circle', P: 'plane', plane: 'plane', R: 'radius', radius: 'radius' },
      },
      eval: ({ inputs }) => {
        const circle = circleFromThreePoints(inputs.pointA, inputs.pointB, inputs.pointC);
        if (!circle) {
          return {};
        }
        return { circle, plane: circle.plane, radius: circle.radius };
      },
    });
  }

  function registerLine() {
    register('{4c4e56eb-2f04-43f9-95a3-cc46a14f495a}', {
      type: 'curve',
      pinMap: {
        inputs: { A: 'start', B: 'end', a: 'start', b: 'end' },
        outputs: { L: 'line', line: 'line' },
      },
      eval: ({ inputs }) => {
        const line = createLine(inputs.start, inputs.end);
        return { line };
      },
    });
  }

  function registerLineSDL() {
    register('{4c619bc9-39fd-4717-82a6-1e07ea237bbe}', {
      type: 'curve',
      pinMap: {
        inputs: {
          S: 'start', s: 'start',
          D: 'direction', d: 'direction',
          L: 'length', l: 'length',
        },
        outputs: { L: 'line', line: 'line' },
      },
      eval: ({ inputs }) => {
        const line = lineFromStartDirectionLength(inputs.start, inputs.direction, inputs.length);
        return { line };
      },
    });
  }

  function registerLineBetweenPlanes() {
    register('{510c4a63-b9bf-42e7-9d07-9d71290264da}', {
      type: 'curve',
      pinMap: {
        inputs: {
          L: 'line', line: 'line',
          A: 'planeA', a: 'planeA',
          B: 'planeB', b: 'planeB',
        },
        outputs: { L: 'line', line: 'line' },
      },
      eval: ({ inputs }) => {
        const line = lineFromGuideAndPlanes(inputs.line, inputs.planeA, inputs.planeB);
        if (!line) {
          return {};
        }
        return { line };
      },
    });
  }

  function registerLineFourPoint() {
    register('{b9fde5fa-d654-4306-8ee1-6b69e6757604}', {
      type: 'curve',
      pinMap: {
        inputs: {
          L: 'line', line: 'line',
          A: 'pointA', a: 'pointA',
          B: 'pointB', b: 'pointB',
        },
        outputs: { L: 'line', line: 'line' },
      },
      eval: ({ inputs }) => {
        const line = lineFromGuideAndPoints(inputs.line, inputs.pointA, inputs.pointB);
        if (!line) {
          return {};
        }
        return { line };
      },
    });
  }

  function registerCirclePlaneRadius() {
    register('{807b86e3-be8d-4970-92b5-f8cdcb45b06b}', {
      type: 'curve',
      pinMap: {
        inputs: { P: 'plane', plane: 'plane', Plane: 'plane', R: 'radius', r: 'radius', Radius: 'radius' },
        outputs: { C: 'circle', circle: 'circle' },
      },
      eval: ({ inputs }) => {
        const plane = ensurePlane(inputs.plane);
        const radius = Math.max(toNumber(inputs.radius, 1), EPSILON);
        const center = plane.origin.clone();
        const circle = createCircleData({ plane, center, radius });
        return { circle };
      },
    });
  }

  function registerCircleCNR() {
    register('{d114323a-e6ee-4164-946b-e4ca0ce15efa}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'center', center: 'center', N: 'normal', normal: 'normal', R: 'radius', r: 'radius' },
        outputs: { C: 'circle', circle: 'circle' },
      },
      eval: ({ inputs }) => {
        const circle = circleFromCenterNormalRadius(inputs.center, inputs.normal, inputs.radius);
        return { circle };
      },
    });
  }

  function registerCircleFit() {
    register('{be52336f-a2e1-43b1-b5f5-178ba489508a}', {
      type: 'curve',
      pinMap: {
        inputs: { P: 'points', Points: 'points', points: 'points' },
        outputs: { C: 'circle', circle: 'circle', R: 'radius', radius: 'radius', D: 'deviation', deviation: 'deviation' },
      },
      eval: ({ inputs }) => {
        const circle = fitCircleToPoints(inputs.points);
        if (!circle) {
          return {};
        }
        return { circle, radius: circle.radius, deviation: circle.deviation ?? 0 };
      },
    });
  }

  function registerEllipse() {
    register('{46b5564d-d3eb-4bf1-ae16-15ed132cfd88}', {
      type: 'curve',
      pinMap: {
        inputs: { P: 'plane', plane: 'plane', Plane: 'plane', R1: 'radius1', R2: 'radius2', r1: 'radius1', r2: 'radius2' },
        outputs: { E: 'ellipse', ellipse: 'ellipse', F1: 'focus1', F2: 'focus2' },
      },
      eval: ({ inputs }) => {
        const ellipse = ellipseFromPlane(inputs.plane, inputs.radius1, inputs.radius2);
        return { ellipse, focus1: ellipse.foci[0], focus2: ellipse.foci[1] };
      },
    });
  }

  function registerArcPlaneRadiusAngle() {
    const ids = [
      '{bb59bffc-f54c-4682-9778-f6c3fe74fce3}',
      '{fd9fe288-a188-4e9b-a464-1148876d18ed}',
      'arc plane radius angle',
    ];
    register(ids, {
      type: 'curve',
      pinMap: {
        inputs: { P: 'plane', plane: 'plane', Plane: 'plane', R: 'radius', r: 'radius', A: 'angle', angle: 'angle' },
        outputs: { A: 'arc', arc: 'arc', L: 'length', length: 'length' },
      },
      eval: ({ inputs }) => {
        const arc = createArcFromPlaneRadiusAngles(inputs.plane, inputs.radius, inputs.angle);
        if (!arc) {
          return {};
        }
        return { arc, length: arc.length };
      },
    });
  }

  function registerArcThreePoint() {
    const ids = [
      '{9fa1b081-b1c7-4a12-a163-0aa8da9ff6c4}',
      '{32c57b97-b653-47dd-b78f-121e89fdd01c}',
      'arc 3pt',
    ];
    register(ids, {
      type: 'curve',
      pinMap: {
        inputs: { A: 'pointA', B: 'pointB', C: 'pointC', a: 'pointA', b: 'pointB', c: 'pointC' },
        outputs: { A: 'arc', arc: 'arc', P: 'plane', plane: 'plane', R: 'radius', radius: 'radius' },
      },
      eval: ({ inputs }) => {
        const arc = arcFromThreePoints(inputs.pointA, inputs.pointB, inputs.pointC);
        if (!arc) {
          return {};
        }
        return { arc, plane: arc.plane, radius: arc.radius };
      },
    });
  }

  function registerArcSED() {
    const ids = [
      '{9d2583dd-6cf5-497c-8c40-c9a290598396}',
      '{f17c37ae-b44a-481a-bd65-b4398be55ec8}',
      'arc sed',
    ];
    register(ids, {
      type: 'curve',
      pinMap: {
        inputs: { S: 'start', E: 'end', D: 'direction', s: 'start', e: 'end', d: 'direction' },
        outputs: { A: 'arc', arc: 'arc', P: 'plane', plane: 'plane', R: 'radius', radius: 'radius' },
      },
      eval: ({ inputs }) => {
        const arc = arcFromSED(inputs.start, inputs.end, inputs.direction);
        if (!arc) {
          return {};
        }
        return { arc, plane: arc.plane, radius: arc.radius };
      },
    });
  }

  function registerModifiedArc() {
    register('{9d8dec9c-3fd1-481c-9c3d-75ea5e15eb1a}', {
      type: 'curve',
      pinMap: {
        inputs: { Arc: 'arc', arc: 'arc', A: 'arc', R: 'radius', r: 'radius', Radius: 'radius', Angle: 'angleDomain', angle: 'angleDomain' },
        outputs: { A: 'arc', arc: 'arc' },
      },
      eval: ({ inputs }) => {
        const base = inputs.arc;
        if (!base) {
          return {};
        }
        const plane = base.plane ? ensurePlane(base.plane) : defaultPlane();
        const radius = inputs.radius !== undefined ? Math.max(toNumber(inputs.radius, base.radius ?? 1), EPSILON) : (base.radius ?? 1);
        let startAngle = base.startAngle ?? 0;
        let endAngle = base.endAngle ?? Math.PI / 2;
        if (inputs.angleDomain !== undefined) {
          const domain = ensureArray(inputs.angleDomain);
          if (domain.length >= 2) {
            startAngle = toNumber(domain[0], startAngle);
            endAngle = toNumber(domain[1], endAngle);
          } else if (domain.length === 1) {
            endAngle = toNumber(domain[0], endAngle);
          }
        }
        const arc = createArcFromAngles(plane, radius, startAngle, endAngle);
        return { arc };
      },
    });
  }

  registerRectangleThreePoint();
  registerRectangleTwoPoint();
  registerFitLine();
  registerInCircle();
  registerCircleThreePoint();
  registerLine();
  registerLineSDL();
  registerLineBetweenPlanes();
  registerLineFourPoint();
  registerCirclePlaneRadius();
  registerCircleCNR();
  registerCircleFit();
  registerEllipse();
  registerArcPlaneRadiusAngle();
  registerArcThreePoint();
  registerArcSED();
  registerModifiedArc();

  function regularPolygon(planeInput, radiusInput, segmentsInput, filletInput) {
    const plane = ensurePlane(planeInput);
    const segments = Math.max(3, Math.round(toNumber(segmentsInput, 6)));
    const radius = Math.max(toNumber(radiusInput, 1), EPSILON);
    const fillet = Math.max(toNumber(filletInput, 0), 0);
    const angleStep = (Math.PI * 2) / segments;
    const points = [];
    for (let i = 0; i < segments; i += 1) {
      const angle = angleStep * i;
      const x = Math.cos(angle) * radius;
      const y = Math.sin(angle) * radius;
      points.push(applyPlane(plane, x, y, 0));
    }
    const length = 2 * segments * radius * Math.sin(Math.PI / segments);
    const shape = new THREE.Shape();
    if (points.length) {
      const first = planeCoordinates(points[0], plane);
      shape.moveTo(first.x, first.y);
      for (let i = 1; i < points.length; i += 1) {
        const coord = planeCoordinates(points[i], plane);
        shape.lineTo(coord.x, coord.y);
      }
      shape.closePath();
    }
    return {
      type: 'polygon',
      plane,
      radius,
      segments,
      fillet,
      points,
      shape,
      length,
    };
  }

  function polygonFromEdge(edgeStartInput, edgeEndInput, planePointInput, segmentsInput) {
    const a = ensurePoint(edgeStartInput, new THREE.Vector3());
    const b = ensurePoint(edgeEndInput, new THREE.Vector3(1, 0, 0));
    const planePoint = ensurePoint(planePointInput, new THREE.Vector3(0, 1, 0));
    const segments = Math.max(3, Math.round(toNumber(segmentsInput, 3)));
    const edge = b.clone().sub(a);
    const edgeLength = edge.length();
    if (edgeLength < EPSILON) {
      return null;
    }
    const xAxis = edge.clone().divideScalar(edgeLength);
    let normal = edge.clone().cross(planePoint.clone().sub(a));
    if (normal.lengthSq() < EPSILON) {
      normal = orthogonalVector(edge.clone().normalize());
    }
    normal.normalize();
    let yAxis = normal.clone().cross(xAxis).normalize();
    const plane = createPlane(a.clone(), xAxis, yAxis, normal);
    const mid = a.clone().add(b).multiplyScalar(0.5);
    const planePointCoords = planeCoordinates(planePoint, plane);
    if (planePointCoords.y < 0) {
      yAxis = yAxis.multiplyScalar(-1);
    }
    const apothem = edgeLength / (2 * Math.tan(Math.PI / segments));
    const center = mid.clone().add(yAxis.clone().multiplyScalar(apothem));
    const circumradius = edgeLength / (2 * Math.sin(Math.PI / segments));
    const centerPlane = createPlane(center.clone(), xAxis, yAxis, normal);
    const coordA = planeCoordinates(a, centerPlane);
    const baseAngle = Math.atan2(coordA.y, coordA.x);
    const points = [];
    const shape = new THREE.Shape();
    for (let i = 0; i < segments; i += 1) {
      const angle = baseAngle + (i * 2 * Math.PI) / segments;
      const point = center.clone()
        .add(xAxis.clone().multiplyScalar(Math.cos(angle) * circumradius))
        .add(yAxis.clone().multiplyScalar(Math.sin(angle) * circumradius));
      points.push(point);
    }
    if (points.length) {
      const local = planeCoordinates(points[0], plane);
      shape.moveTo(local.x, local.y);
      for (let i = 1; i < points.length; i += 1) {
        const coord = planeCoordinates(points[i], plane);
        shape.lineTo(coord.x, coord.y);
      }
      shape.closePath();
    }
    const length = 2 * segments * circumradius * Math.sin(Math.PI / segments);
    return {
      type: 'polygon',
      plane,
      center,
      points,
      segments,
      radius: circumradius,
      edgeRadius: apothem,
      shape,
      length,
    };
  }

  function registerPolygon() {
    register('{845527a6-5cea-4ae9-a667-96ae1667a4e8}', {
      type: 'curve',
      pinMap: {
        inputs: {
          P: 'plane', plane: 'plane', Plane: 'plane',
          R: 'radius', r: 'radius', Radius: 'radius',
          S: 'segments', s: 'segments', Segments: 'segments',
          Rf: 'fillet', rf: 'fillet', Fillet: 'fillet',
        },
        outputs: { P: 'polygon', polygon: 'polygon', L: 'length', length: 'length' },
      },
      eval: ({ inputs }) => {
        const polygon = regularPolygon(inputs.plane, inputs.radius, inputs.segments, inputs.fillet);
        return { polygon, length: polygon.length };
      },
    });
  }

  function registerPolygonEdge() {
    register('{f4568ce6-aade-4511-8f32-f27d8a6bf9e9}', {
      type: 'curve',
      pinMap: {
        inputs: {
          E0: 'start', e0: 'start',
          E1: 'end', e1: 'end',
          P: 'planePoint', planePoint: 'planePoint',
          S: 'segments', s: 'segments', Segments: 'segments',
        },
        outputs: {
          P: 'polygon', polygon: 'polygon',
          C: 'center', center: 'center',
          Rc: 'cornerRadius', rc: 'cornerRadius',
          Re: 'edgeRadius', re: 'edgeRadius',
        },
      },
      eval: ({ inputs }) => {
        const polygon = polygonFromEdge(inputs.start, inputs.end, inputs.planePoint, inputs.segments);
        if (!polygon) {
          return {};
        }
        return {
          polygon,
          center: polygon.center,
          cornerRadius: polygon.radius,
          edgeRadius: polygon.edgeRadius,
        };
      },
    });
  }

  registerPolygon();
  registerPolygonEdge();

  function steinerInellipse(aInput, bInput, cInput) {
    const a = ensurePoint(aInput, new THREE.Vector3());
    const b = ensurePoint(bInput, new THREE.Vector3(1, 0, 0));
    const c = ensurePoint(cInput, new THREE.Vector3(0, 1, 0));
    const plane = planeFromPoints(a, b, c);
    const coordsA = planeCoordinates(a, plane);
    const coordsB = planeCoordinates(b, plane);
    const coordsC = planeCoordinates(c, plane);
    const f1x = coordsB.x - coordsA.x;
    const f1y = coordsB.y - coordsA.y;
    const f2x = coordsC.x - coordsA.x;
    const f2y = coordsC.y - coordsA.y;
    const invSqrt3 = 1 / Math.sqrt(3);
    const m11 = 0.5 * f1x;
    const m12 = (-0.5 * invSqrt3) * f1x + invSqrt3 * f2x;
    const m21 = 0.5 * f1y;
    const m22 = (-0.5 * invSqrt3) * f1y + invSqrt3 * f2y;
    const tx = coordsA.x + m11;
    const ty = coordsA.y + m21;
    const center2D = {
      x: m12 * (Math.sqrt(3) / 3) + tx,
      y: m22 * (Math.sqrt(3) / 3) + ty,
    };
    const ri = Math.sqrt(3) / 3;
    const center = applyPlane(plane, center2D.x, center2D.y, 0);
    const points = [];
    const segments = 128;
    const shape = new THREE.Shape();
    for (let i = 0; i <= segments; i += 1) {
      const theta = (i / segments) * Math.PI * 2;
      const cos = Math.cos(theta);
      const sin = Math.sin(theta);
      const x = center2D.x + m11 * (ri * cos) + m12 * (ri * sin);
      const y = center2D.y + m21 * (ri * cos) + m22 * (ri * sin);
      const point = applyPlane(plane, x, y, 0);
      points.push(point);
      if (i === 0) {
        shape.moveTo(x, y);
      } else {
        shape.lineTo(x, y);
      }
    }
    shape.closePath();
    const s11 = m11 * m11 + m12 * m12;
    const s12 = m11 * m21 + m12 * m22;
    const s22 = m21 * m21 + m22 * m22;
    const trace = s11 + s22;
    const determinant = s11 * s22 - s12 * s12;
    const discriminant = Math.max(trace * trace - 4 * determinant, 0);
    const lambda1 = (trace + Math.sqrt(discriminant)) / 2;
    const lambda2 = (trace - Math.sqrt(discriminant)) / 2;
    const major = Math.sqrt(Math.max(lambda1, lambda2));
    const minor = Math.sqrt(Math.max(Math.min(lambda1, lambda2), 0));
    const radius1 = ri * major;
    const radius2 = ri * minor;
    let eigenVector = new THREE.Vector2();
    if (Math.abs(s12) > EPSILON) {
      eigenVector.set(lambda1 - s22, s12);
    } else if (s11 >= s22) {
      eigenVector.set(1, 0);
    } else {
      eigenVector.set(0, 1);
    }
    eigenVector.normalize();
    const cDistance = Math.sqrt(Math.max(radius1 * radius1 - radius2 * radius2, 0));
    const focus1 = applyPlane(plane, center2D.x + eigenVector.x * cDistance, center2D.y + eigenVector.y * cDistance, 0);
    const focus2 = applyPlane(plane, center2D.x - eigenVector.x * cDistance, center2D.y - eigenVector.y * cDistance, 0);
    const perimeterEstimate = Math.PI * (3 * (radius1 + radius2) - Math.sqrt((3 * radius1 + radius2) * (radius1 + 3 * radius2)));
    return {
      type: 'ellipse',
      plane,
      center,
      radius1,
      radius2,
      points,
      shape,
      segments,
      foci: [focus1, focus2],
      perimeter: perimeterEstimate,
    };
  }

  function registerInEllipse() {
    register('{679a9c6a-ab97-4c20-b02c-680f9a9a1a44}', {
      type: 'curve',
      pinMap: {
        inputs: { A: 'cornerA', B: 'cornerB', C: 'cornerC', a: 'cornerA', b: 'cornerB', c: 'cornerC' },
        outputs: { E: 'ellipse', ellipse: 'ellipse', P: 'plane', plane: 'plane' },
      },
      eval: ({ inputs }) => {
        const ellipse = steinerInellipse(inputs.cornerA, inputs.cornerB, inputs.cornerC);
        if (!ellipse) {
          return {};
        }
        return { ellipse, plane: ellipse.plane };
      },
    });
  }

  registerInEllipse();

  function ensureCircle(input) {
    if (!input) {
      return null;
    }
    if (input.type === 'circle') {
      const center = ensurePoint(input.center ?? input.plane?.origin, new THREE.Vector3());
      const radius = Math.max(toNumber(input.radius, 0), 0);
      const plane = input.plane ? ensurePlane(input.plane) : circlePlaneFromCenterNormal(center, input.normal);
      return { center, radius, plane };
    }
    if (input.center && input.radius) {
      const center = ensurePoint(input.center, new THREE.Vector3());
      const radius = Math.max(toNumber(input.radius, 0), 0);
      const plane = input.plane ? ensurePlane(input.plane) : circlePlaneFromCenterNormal(center, input.normal);
      return { center, radius, plane };
    }
    if (input.origin && input.normal && input.radius) {
      const center = ensurePoint(input.origin, new THREE.Vector3());
      const radius = Math.max(toNumber(input.radius, 0), 0);
      const plane = ensurePlane({ origin: center, normal: input.normal });
      return { center, radius, plane };
    }
    return null;
  }

  function circlePlaneFromCenterNormal(center, normalInput) {
    const normal = normalInput ? normalizeVector(ensurePoint(normalInput, new THREE.Vector3(0, 0, 1)), new THREE.Vector3(0, 0, 1)) : new THREE.Vector3(0, 0, 1);
    const xAxis = orthogonalVector(normal);
    const yAxis = normal.clone().cross(xAxis).normalize();
    return createPlane(center.clone(), xAxis, yAxis, normal);
  }

  function tangentsFromPointToCircle(pointInput, circleInput) {
    const circle = ensureCircle(circleInput);
    if (!circle) {
      return [];
    }
    const plane = circle.plane;
    const point = ensurePoint(pointInput, plane.origin.clone());
    const pointCoords = planeCoordinates(point, plane);
    const centerCoords = planeCoordinates(circle.center, plane);
    const dx = pointCoords.x - centerCoords.x;
    const dy = pointCoords.y - centerCoords.y;
    const distance = Math.hypot(dx, dy);
    if (distance <= circle.radius + EPSILON) {
      return [];
    }
    const baseAngle = Math.atan2(dy, dx);
    const offset = Math.acos(circle.radius / distance);
    const lines = [];
    for (const sign of [1, -1]) {
      const theta = baseAngle + sign * offset;
      const tangentPoint = applyPlane(
        plane,
        centerCoords.x + Math.cos(theta) * circle.radius,
        centerCoords.y + Math.sin(theta) * circle.radius,
        0,
      );
      lines.push(createLine(point, tangentPoint));
    }
    return lines;
  }

  function tangentsBetweenCircles(circleAInput, circleBInput, { external = true } = {}) {
    const circleA = ensureCircle(circleAInput);
    const circleB = ensureCircle(circleBInput);
    if (!circleA || !circleB) {
      return [];
    }
    const plane = circleA.plane;
    const centerA = planeCoordinates(circleA.center, plane);
    const centerB = planeCoordinates(circleB.center, plane);
    const dx = centerB.x - centerA.x;
    const dy = centerB.y - centerA.y;
    const distSq = dx * dx + dy * dy;
    if (distSq < EPSILON) {
      return [];
    }
    const dist = Math.sqrt(distSq);
    const adjustedRadiusB = external ? circleB.radius : -circleB.radius;
    const difference = circleA.radius - adjustedRadiusB;
    if (Math.abs(difference) > dist) {
      return [];
    }
    const baseAngle = Math.atan2(dy, dx);
    const offset = Math.acos(difference / dist);
    const results = [];
    for (const sign of [1, -1]) {
      const theta = baseAngle + sign * offset;
      const tangentA = applyPlane(
        plane,
        centerA.x + Math.cos(theta) * circleA.radius,
        centerA.y + Math.sin(theta) * circleA.radius,
        0,
      );
      const tangentB = applyPlane(
        plane,
        centerB.x + Math.cos(theta) * adjustedRadiusB,
        centerB.y + Math.sin(theta) * adjustedRadiusB,
        0,
      );
      results.push(createLine(tangentA, tangentB));
    }
    return results;
  }

  function registerTangentLinesFromPoint() {
    register('{ea0f0996-af7a-481d-8099-09c041e6c2d5}', {
      type: 'curve',
      pinMap: {
        inputs: { P: 'point', point: 'point', C: 'circle', circle: 'circle' },
        outputs: { T1: 'tangent1', T2: 'tangent2' },
      },
      eval: ({ inputs }) => {
        const lines = tangentsFromPointToCircle(inputs.point, inputs.circle);
        if (!lines.length) {
          return {};
        }
        return { tangent1: lines[0], tangent2: lines[1] ?? lines[0] };
      },
    });
  }

  function registerTangentLinesExternal() {
    register('{d6d68c93-d00f-4cd5-ba89-903c7f6be64c}', {
      type: 'curve',
      pinMap: {
        inputs: { A: 'circleA', B: 'circleB', a: 'circleA', b: 'circleB' },
        outputs: { T1: 'tangent1', T2: 'tangent2' },
      },
      eval: ({ inputs }) => {
        const lines = tangentsBetweenCircles(inputs.circleA, inputs.circleB, { external: true });
        if (!lines.length) {
          return {};
        }
        return { tangent1: lines[0], tangent2: lines[1] ?? lines[0] };
      },
    });
  }

  function registerTangentLinesInternal() {
    register('{e0168047-c46a-48c6-8595-2fb3d8574f23}', {
      type: 'curve',
      pinMap: {
        inputs: { A: 'circleA', B: 'circleB', a: 'circleA', b: 'circleB' },
        outputs: { T1: 'tangent1', T2: 'tangent2' },
      },
      eval: ({ inputs }) => {
        const lines = tangentsBetweenCircles(inputs.circleA, inputs.circleB, { external: false });
        if (!lines.length) {
          return {};
        }
        return { tangent1: lines[0], tangent2: lines[1] ?? lines[0] };
      },
    });
  }

  registerTangentLinesFromPoint();
  registerTangentLinesExternal();
  registerTangentLinesInternal();

  function tangentAtArcPoint(arc, point) {
    const plane = arc.plane ? ensurePlane(arc.plane) : defaultPlane();
    const center = arc.center ?? plane.origin.clone();
    const radiusVector = point.clone().sub(center);
    const tangent = plane.zAxis.clone().cross(radiusVector).normalize();
    const direction = arc.end.clone().sub(arc.start);
    if (tangent.dot(direction) < 0) {
      tangent.multiplyScalar(-1);
    }
    return tangent;
  }

  function registerBiArc() {
    register('{75f4b0fd-9721-47b1-99e7-9c098b342e67}', {
      type: 'curve',
      pinMap: {
        inputs: {
          S: 'start', s: 'start',
          Ts: 'startTangent', ts: 'startTangent',
          E: 'end', e: 'end',
          Te: 'endTangent', te: 'endTangent',
          R: 'ratio', r: 'ratio',
        },
        outputs: { A1: 'arc1', A2: 'arc2', B: 'biArc' },
      },
      eval: ({ inputs }) => {
        const start = ensurePoint(inputs.start, new THREE.Vector3());
        const end = ensurePoint(inputs.end, new THREE.Vector3(1, 0, 0));
        const startTangent = normalizeVector(ensurePoint(inputs.startTangent, end.clone().sub(start)), end.clone().sub(start));
        const endTangent = normalizeVector(ensurePoint(inputs.endTangent, start.clone().sub(end)), start.clone().sub(end));
        const ratio = Math.min(Math.max(toNumber(inputs.ratio, 0.5), 0.05), 0.95);
        const chord = end.clone().sub(start);
        const mid = start.clone().add(chord.clone().multiplyScalar(ratio));
        let arc1 = arcFromSED(start, mid, startTangent);
        if (!arc1) {
          arc1 = { type: 'line', start, end: mid, length: start.distanceTo(mid) };
        }
        let midTangent = startTangent.clone();
        if (arc1?.type === 'arc') {
          midTangent = tangentAtArcPoint(arc1, arc1.end);
        }
        let arc2 = arcFromSED(mid, end, midTangent);
        if (!arc2) {
          arc2 = { type: 'line', start: mid, end, length: mid.distanceTo(end) };
        }
        const totalLength = (arc1.length ?? start.distanceTo(mid)) + (arc2.length ?? mid.distanceTo(end));
        const biArc = { type: 'biarc', arcs: [arc1, arc2], length: totalLength };
        return { arc1, arc2, biArc };
      },
    });
  }

  registerBiArc();

  function twoByFourJam(roomInput, widthInput, samplesInput) {
    const points = collectPoints(roomInput).filter((point) => point instanceof THREE.Vector3);
    if (points.length < 2) {
      return null;
    }
    const plane = fitPlaneToPoints(points) ?? defaultPlane();
    const coords = points.map((point) => planeCoordinates(point, plane));
    let minX = Number.POSITIVE_INFINITY;
    let maxX = Number.NEGATIVE_INFINITY;
    let minY = Number.POSITIVE_INFINITY;
    let maxY = Number.NEGATIVE_INFINITY;
    for (const coord of coords) {
      minX = Math.min(minX, coord.x);
      maxX = Math.max(maxX, coord.x);
      minY = Math.min(minY, coord.y);
      maxY = Math.max(maxY, coord.y);
    }
    const width = Math.max(toNumber(widthInput, 0.1), 0.01);
    const samples = Math.max(Math.round(toNumber(samplesInput, 10)), 1);
    const spanX = maxX - minX;
    const spanY = maxY - minY;
    let bestRectangle = null;
    let bestScore = Number.POSITIVE_INFINITY;
    for (let i = 0; i <= samples; i += 1) {
      const t = samples === 0 ? 0.5 : i / samples;
      const centerX = minX + spanX * t;
      const halfWidth = spanX / 2;
      const halfHeight = width / 2;
      const corners = [
        applyPlane(plane, centerX - halfWidth, minY + halfHeight, 0),
        applyPlane(plane, centerX + halfWidth, minY + halfHeight, 0),
        applyPlane(plane, centerX + halfWidth, minY + width - halfHeight, 0),
        applyPlane(plane, centerX - halfWidth, minY + width - halfHeight, 0),
      ];
      const score = corners.reduce((sum, corner) => sum + distanceToPolygon(corner, coords, plane), 0);
      if (score < bestScore) {
        bestScore = score;
        bestRectangle = {
          type: 'rectangle',
          plane,
          corners,
          width: halfWidth * 2,
          height: width,
          radius: 0,
          shape: (() => {
            const shape = new THREE.Shape();
            shape.moveTo(centerX - halfWidth, minY + halfHeight);
            shape.lineTo(centerX + halfWidth, minY + halfHeight);
            shape.lineTo(centerX + halfWidth, minY + width - halfHeight);
            shape.lineTo(centerX - halfWidth, minY + width - halfHeight);
            shape.closePath();
            return shape;
          })(),
          length: 2 * (halfWidth * 2 + width),
        };
      }
    }
    return bestRectangle;
  }

  function distanceToPolygon(point3D, coords, plane) {
    const coord = planeCoordinates(point3D, plane);
    let minDistance = Number.POSITIVE_INFINITY;
    for (let i = 0; i < coords.length; i += 1) {
      const a = coords[i];
      const b = coords[(i + 1) % coords.length];
      const distance = distancePointToSegment({ x: coord.x, y: coord.y }, a, b);
      minDistance = Math.min(minDistance, distance);
    }
    return minDistance;
  }

  function distancePointToSegment(point, a, b) {
    const abx = b.x - a.x;
    const aby = b.y - a.y;
    const apx = point.x - a.x;
    const apy = point.y - a.y;
    const abLenSq = abx * abx + aby * aby;
    if (abLenSq <= EPSILON) {
      return Math.hypot(apx, apy);
    }
    let t = (apx * abx + apy * aby) / abLenSq;
    t = Math.min(Math.max(t, 0), 1);
    const closestX = a.x + abx * t;
    const closestY = a.y + aby * t;
    return Math.hypot(point.x - closestX, point.y - closestY);
  }

  function registerTwoByFourJam() {
    register('{c21e7bd5-b1f2-4448-ac56-206f98f90aa7}', {
      type: 'curve',
      pinMap: {
        inputs: { R: 'room', room: 'room', W: 'width', w: 'width', S: 'samples', s: 'samples' },
        outputs: { R: 'rectangle', rectangle: 'rectangle' },
      },
      eval: ({ inputs }) => {
        const rectangle = twoByFourJam(inputs.room, inputs.width, inputs.samples);
        if (!rectangle) {
          return {};
        }
        return { rectangle };
      },
    });
  }

  registerTwoByFourJam();

  function circleTanTanCandidates(circleA, circleB, guidePoint) {
    const plane = circleA.plane;
    const centerA = planeCoordinates(circleA.center, plane);
    const centerB = planeCoordinates(circleB.center, plane);
    const guide = guidePoint ? planeCoordinates(ensurePoint(guidePoint, circleA.center.clone()), plane) : { x: (centerA.x + centerB.x) / 2, y: (centerA.y + centerB.y) / 2 };
    const distanceAB = Math.hypot(centerB.x - centerA.x, centerB.y - centerA.y);
    const minRadius = 0.01;
    const maxRadius = distanceAB + circleA.radius + circleB.radius;
    let best = null;
    let bestScore = Number.POSITIVE_INFINITY;
    const samples = 64;
    for (let i = 0; i <= samples; i += 1) {
      const r = minRadius + ((maxRadius - minRadius) * i) / samples;
      const candidateCenters = solveApollonius(centerA, circleA.radius + r, centerB, circleB.radius + r);
      for (const center of candidateCenters) {
        const score = Math.hypot(center.x - guide.x, center.y - guide.y);
        if (score < bestScore) {
          bestScore = score;
          best = { center, radius: r };
        }
      }
    }
    if (best) {
      const refineSamples = 16;
      const refineStep = (maxRadius - minRadius) / samples;
      const startR = Math.max(best.radius - refineStep, minRadius);
      const endR = Math.min(best.radius + refineStep, maxRadius);
      for (let i = 0; i <= refineSamples; i += 1) {
        const r = startR + ((endR - startR) * i) / refineSamples;
        const candidateCenters = solveApollonius(centerA, circleA.radius + r, centerB, circleB.radius + r);
        for (const center of candidateCenters) {
          const score = Math.hypot(center.x - guide.x, center.y - guide.y);
          if (score < bestScore) {
            bestScore = score;
            best = { center, radius: r };
          }
        }
      }
    }
    return best;
  }

  function solveApollonius(centerA, radiusA, centerB, radiusB) {
    const results = [];
    const dx = centerB.x - centerA.x;
    const dy = centerB.y - centerA.y;
    const distSq = dx * dx + dy * dy;
    const dist = Math.sqrt(distSq);
    if (distSq < EPSILON) {
      return results;
    }
    if (dist > radiusA + radiusB || dist < Math.abs(radiusA - radiusB)) {
      return results;
    }
    const a = (radiusA * radiusA - radiusB * radiusB + distSq) / (2 * dist);
    const hSq = radiusA * radiusA - a * a;
    if (hSq < -EPSILON) {
      return results;
    }
    const h = hSq > 0 ? Math.sqrt(Math.max(hSq, 0)) : 0;
    const ux = dx / dist;
    const uy = dy / dist;
    const baseX = centerA.x + a * ux;
    const baseY = centerA.y + a * uy;
    const offsetX = -uy * h;
    const offsetY = ux * h;
    results.push({ x: baseX + offsetX, y: baseY + offsetY });
    if (h > EPSILON) {
      results.push({ x: baseX - offsetX, y: baseY - offsetY });
    }
    return results;
  }

  function circleTanTan(circleAInput, circleBInput, guideInput) {
    const circleA = ensureCircle(circleAInput);
    const circleB = ensureCircle(circleBInput);
    if (!circleA || !circleB) {
      return null;
    }
    const plane = circleA.plane;
    const candidate = circleTanTanCandidates(circleA, circleB, guideInput);
    if (!candidate) {
      return null;
    }
    const center = applyPlane(plane, candidate.center.x, candidate.center.y, 0);
    return createCircleData({ plane, center, radius: candidate.radius });
  }

  function circleTanTanTan(circleAInput, circleBInput, circleCInput, guideInput) {
    const circleA = ensureCircle(circleAInput);
    const circleB = ensureCircle(circleBInput);
    const circleC = ensureCircle(circleCInput);
    if (!circleA || !circleB || !circleC) {
      return null;
    }
    const plane = circleA.plane;
    const candidate = circleTanTan(circleA, circleB, guideInput);
    if (!candidate) {
      return null;
    }
    const center = candidate.center;
    const centerCoords = planeCoordinates(center, plane);
    const centerC = planeCoordinates(circleC.center, plane);
    const desiredRadius = Math.max(Math.hypot(centerC.x - centerCoords.x, centerC.y - centerCoords.y) - circleC.radius, 0);
    const radius = (candidate.radius + desiredRadius) / 2;
    return createCircleData({ plane, center, radius });
  }

  function registerCircleTanTan() {
    register('{50b204ef-d3de-41bb-a006-02fba2d3f709}', {
      type: 'curve',
      pinMap: {
        inputs: { A: 'curveA', B: 'curveB', P: 'point', a: 'curveA', b: 'curveB', p: 'point' },
        outputs: { C: 'circle', circle: 'circle' },
      },
      eval: ({ inputs }) => {
        const circle = circleTanTan(inputs.curveA, inputs.curveB, inputs.point);
        if (!circle) {
          return {};
        }
        return { circle };
      },
    });
  }

  function registerCircleTanTanTan() {
    register('{dcaa922d-5491-4826-9a22-5adefa139f43}', {
      type: 'curve',
      pinMap: {
        inputs: { A: 'curveA', B: 'curveB', C: 'curveC', P: 'point', a: 'curveA', b: 'curveB', c: 'curveC', p: 'point' },
        outputs: { C: 'circle', circle: 'circle' },
      },
      eval: ({ inputs }) => {
        const circle = circleTanTanTan(inputs.curveA, inputs.curveB, inputs.curveC, inputs.point);
        if (!circle) {
          return {};
        }
        return { circle };
      },
    });
  }

  registerCircleTanTan();
  registerCircleTanTanTan();

  function registerTangentArcs() {
    register('{f1c0783b-60e9-42a7-8081-925bc755494c}', {
      type: 'curve',
      pinMap: {
        inputs: { A: 'circleA', B: 'circleB', R: 'radius', a: 'circleA', b: 'circleB', r: 'radius' },
        outputs: { A: 'arcA', B: 'arcB' },
      },
      eval: ({ inputs }) => {
        const circleA = ensureCircle(inputs.circleA);
        const circleB = ensureCircle(inputs.circleB);
        if (!circleA || !circleB) {
          return {};
        }
        const lines = tangentsBetweenCircles(circleA, circleB, { external: true });
        if (!lines.length) {
          return {};
        }
        const arcs = lines.map((line) => {
          const direction = line.end.clone().sub(line.start);
          if (direction.lengthSq() < EPSILON) {
            return null;
          }
          const tangent = direction.clone().normalize();
          return arcFromSED(line.start, line.end, tangent) ?? { type: 'line', start: line.start, end: line.end, length: direction.length() };
        }).filter(Boolean);
        return { arcA: arcs[0], arcB: arcs[1] ?? arcs[0] };
      },
    });
  }

  registerTangentArcs();
}

export function registerCurveSplineComponents({ register, toNumber, toVector3 }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register curve spline components.');
  }
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register curve spline components.');
  }
  if (typeof toVector3 !== 'function') {
    throw new Error('toVector3 function is required to register curve spline components.');
  }

  const EPSILON = 1e-9;

  function clamp(value, min, max) {
    return Math.min(Math.max(value, min), max);
  }

  function clamp01(value) {
    return clamp(value, 0, 1);
  }

  function ensureNumber(value, fallback = 0) {
    return toNumber(value, fallback);
  }

  function ensurePoint(value, fallback = new THREE.Vector3()) {
    return toVector3(value, fallback.clone());
  }

  function ensureBoolean(value, fallback = false) {
    if (value === undefined || value === null) {
      return fallback;
    }
    if (typeof value === 'boolean') {
      return value;
    }
    if (Array.isArray(value)) {
      if (!value.length) return fallback;
      return ensureBoolean(value[0], fallback);
    }
    if (typeof value === 'number') {
      if (!Number.isFinite(value)) return fallback;
      return value !== 0;
    }
    if (typeof value === 'string') {
      const normalized = value.trim().toLowerCase();
      if (!normalized) return fallback;
      if (['true', 'yes', 'on', '1'].includes(normalized)) return true;
      if (['false', 'no', 'off', '0'].includes(normalized)) return false;
      return fallback;
    }
    return Boolean(value);
  }

  function ensureArray(input) {
    if (input === undefined || input === null) {
      return [];
    }
    if (Array.isArray(input)) {
      return input;
    }
    return [input];
  }

  function collectPoints(input) {
    const result = [];

    function visit(value) {
      if (value === undefined || value === null) {
        return;
      }
      if (value.isVector3) {
        result.push(value.clone());
        return;
      }
      if (Array.isArray(value)) {
        for (const entry of value) {
          visit(entry);
        }
        return;
      }
      if (typeof value === 'object') {
        if ('point' in value) {
          visit(value.point);
          return;
        }
        if ('points' in value) {
          visit(value.points);
          return;
        }
        if ('position' in value) {
          visit(value.position);
          return;
        }
        if ('x' in value || 'y' in value || 'z' in value) {
          const point = toVector3(value, null);
          if (point) {
            result.push(point);
          }
          return;
        }
      }
    }

    visit(input);
    return result;
  }

  function computePolylineLength(points) {
    if (!points || points.length < 2) {
      return 0;
    }
    let length = 0;
    for (let i = 0; i < points.length - 1; i += 1) {
      length += points[i].distanceTo(points[i + 1]);
    }
    return length;
  }

  function createDomain(start = 0, end = 1) {
    const min = Math.min(start, end);
    const max = Math.max(start, end);
    const span = end - start;
    const length = max - min;
    const center = (start + end) / 2;
    return { start, end, min, max, span, length, center, dimension: 1 };
  }

  function convertVector(value) {
    if (!value) {
      return null;
    }
    if (value.isVector3) {
      return value.clone();
    }
    if (Array.isArray(value)) {
      const [x, y, z] = value;
      return new THREE.Vector3(ensureNumber(x, 0), ensureNumber(y, 0), ensureNumber(z, 0));
    }
    if (typeof value === 'object') {
      return toVector3(value, new THREE.Vector3());
    }
    if (typeof value === 'number') {
      return new THREE.Vector3(0, 0, ensureNumber(value, 0));
    }
    return null;
  }

  function createCurveFromPath(path, { segments = 64, closed = false, type = 'curve' } = {}) {
    if (!path) {
      return null;
    }
    const safeSegments = Math.max(segments, 8);
    const spaced = path.getSpacedPoints(safeSegments);
    const points = spaced.map((pt) => new THREE.Vector3(pt.x, pt.y, pt.z ?? 0));
    let length = 0;
    if (typeof path.getLength === 'function') {
      length = path.getLength();
    } else {
      length = computePolylineLength(points);
    }
    const curve = {
      type,
      path,
      points,
      segments: safeSegments,
      length,
      closed,
      domain: createDomain(0, 1),
    };
    curve.getPointAt = (t) => {
      const clamped = clamp01(t);
      if (typeof path.getPointAt === 'function') {
        const pt = path.getPointAt(clamped);
        return new THREE.Vector3(pt.x, pt.y, pt.z ?? 0);
      }
      const pt = path.getPoint(clamped);
      return new THREE.Vector3(pt.x, pt.y, pt.z ?? 0);
    };
    curve.getTangentAt = (t) => {
      if (typeof path.getTangentAt === 'function') {
        const tangent = path.getTangentAt(clamp01(t));
        return new THREE.Vector3(tangent.x, tangent.y, tangent.z ?? 0).normalize();
      }
      const delta = 1e-3;
      const p0 = curve.getPointAt(clamp01(t - delta));
      const p1 = curve.getPointAt(clamp01(t + delta));
      return p1.clone().sub(p0).normalize();
    };
    return curve;
  }

  function createCurveFromPoints(pointsInput, { closed = false, curveType = 'centripetal', tension = 0.5, samples } = {}) {
    const points = collectPoints(pointsInput);
    if (points.length === 0) {
      return null;
    }
    if (points.length === 1) {
      const point = points[0];
      const path = new THREE.LineCurve3(point.clone(), point.clone());
      return createCurveFromPath(path, { segments: 1, closed, type: 'curve-point' });
    }
    if (points.length === 2) {
      const path = new THREE.LineCurve3(points[0].clone(), points[1].clone());
      return createCurveFromPath(path, { segments: samples ?? 8, closed, type: 'polyline' });
    }
    const path = new THREE.CatmullRomCurve3(points.map((pt) => pt.clone()), closed, curveType, tension);
    return createCurveFromPath(path, { segments: samples ?? (points.length * 8), closed, type: 'curve' });
  }

  function sampleCurvePoints(curve, segments = 32) {
    if (!curve) {
      return [];
    }
    if (curve.path?.getSpacedPoints) {
      return curve.path.getSpacedPoints(Math.max(segments, 8)).map((pt) => new THREE.Vector3(pt.x, pt.y, pt.z ?? 0));
    }
    if (curve.points && Array.isArray(curve.points)) {
      if (curve.points.length === segments + 1) {
        return curve.points.map((pt) => pt.clone());
      }
      const result = [];
      for (let i = 0; i <= segments; i += 1) {
        const t = i / segments;
        result.push(curvePointAt(curve, t));
      }
      return result;
    }
    return [];
  }

  function curvePointAt(curve, t) {
    if (!curve) {
      return null;
    }
    if (typeof curve.getPointAt === 'function') {
      return curve.getPointAt(t);
    }
    if (curve.path?.getPointAt) {
      const pt = curve.path.getPointAt(clamp01(t));
      return new THREE.Vector3(pt.x, pt.y, pt.z ?? 0);
    }
    if (curve.points?.length) {
      const points = curve.points;
      if (points.length === 1) {
        return points[0].clone();
      }
      const scaled = clamp01(t) * (points.length - 1);
      const index = Math.floor(scaled);
      const alpha = scaled - index;
      const current = points[index];
      const next = points[Math.min(index + 1, points.length - 1)];
      return current.clone().lerp(next, alpha);
    }
    return null;
  }

  function curveTangentAt(curve, t) {
    if (!curve) {
      return new THREE.Vector3(1, 0, 0);
    }
    if (typeof curve.getTangentAt === 'function') {
      return curve.getTangentAt(t);
    }
    if (curve.path?.getTangentAt) {
      const tangent = curve.path.getTangentAt(clamp01(t));
      return new THREE.Vector3(tangent.x, tangent.y, tangent.z ?? 0).normalize();
    }
    const delta = 1e-3;
    const p0 = curvePointAt(curve, clamp01(t - delta));
    const p1 = curvePointAt(curve, clamp01(t + delta));
    if (!p0 || !p1) {
      return new THREE.Vector3(1, 0, 0);
    }
    const tangent = p1.clone().sub(p0);
    if (tangent.lengthSq() < EPSILON) {
      return new THREE.Vector3(1, 0, 0);
    }
    return tangent.normalize();
  }

  function defaultPlane() {
    return {
      origin: new THREE.Vector3(0, 0, 0),
      xAxis: new THREE.Vector3(1, 0, 0),
      yAxis: new THREE.Vector3(0, 1, 0),
      zAxis: new THREE.Vector3(0, 0, 1),
    };
  }

  function normalizeVector(vector, fallback = new THREE.Vector3(1, 0, 0)) {
    const candidate = vector.clone();
    if (candidate.lengthSq() < EPSILON) {
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

  function normalizePlaneAxes(origin, xAxis, yAxis, zAxis) {
    const z = normalizeVector(zAxis, new THREE.Vector3(0, 0, 1));
    const x = normalizeVector(xAxis, new THREE.Vector3(1, 0, 0));
    let y = yAxis.clone();
    if (y.lengthSq() < EPSILON) {
      y = z.clone().cross(x);
    }
    y.normalize();
    const orthogonalX = y.clone().cross(z).normalize();
    const orthogonalY = z.clone().cross(orthogonalX).normalize();
    return {
      origin: origin.clone(),
      xAxis: orthogonalX,
      yAxis: orthogonalY,
      zAxis: z,
    };
  }

  function ensurePlane(input) {
    if (!input) {
      return defaultPlane();
    }
    if (Array.isArray(input)) {
      const points = collectPoints(input);
      if (points.length >= 3) {
        const origin = points[0];
        const xAxis = normalizeVector(points[1].clone().sub(points[0]), new THREE.Vector3(1, 0, 0));
        const normal = normalizeVector(points[1].clone().sub(points[0]).cross(points[2].clone().sub(points[0])), new THREE.Vector3(0, 0, 1));
        const yAxis = normal.clone().cross(xAxis).normalize();
        return normalizePlaneAxes(origin, xAxis, yAxis, normal);
      }
    }
    if (typeof input === 'object') {
      if (input.origin && input.xAxis && input.yAxis && input.zAxis) {
        return normalizePlaneAxes(ensurePoint(input.origin, new THREE.Vector3()), ensurePoint(input.xAxis, new THREE.Vector3(1, 0, 0)), ensurePoint(input.yAxis, new THREE.Vector3(0, 1, 0)), ensurePoint(input.zAxis, new THREE.Vector3(0, 0, 1)));
      }
      if (input.origin && input.normal) {
        const origin = ensurePoint(input.origin, new THREE.Vector3());
        const normal = normalizeVector(ensurePoint(input.normal, new THREE.Vector3(0, 0, 1)), new THREE.Vector3(0, 0, 1));
        const xAxis = orthogonalVector(normal);
        const yAxis = normal.clone().cross(xAxis).normalize();
        return normalizePlaneAxes(origin, xAxis, yAxis, normal);
      }
    }
    return defaultPlane();
  }

  function planeCoordinates(point, plane) {
    const relative = ensurePoint(point, plane.origin.clone()).clone().sub(plane.origin);
    return {
      x: relative.dot(plane.xAxis),
      y: relative.dot(plane.yAxis),
      z: relative.dot(plane.zAxis),
    };
  }

  function applyPlane(plane, x, y, z = 0) {
    const result = plane.origin.clone();
    result.add(plane.xAxis.clone().multiplyScalar(x));
    result.add(plane.yAxis.clone().multiplyScalar(y));
    result.add(plane.zAxis.clone().multiplyScalar(z));
    return result;
  }

  function ensureCircle(input) {
    if (!input) {
      return null;
    }
    if (Array.isArray(input)) {
      return ensureCircle(input[0]);
    }
    if (input.circle) {
      return ensureCircle(input.circle);
    }
    const plane = ensurePlane(input.plane);
    const center = ensurePoint(input.center ?? input.origin ?? input.point ?? input.position ?? plane.origin, plane.origin.clone());
    let radius = ensureNumber(input.radius ?? (input.diameter ? ensureNumber(input.diameter, 0) / 2 : undefined), Number.NaN);
    if (!Number.isFinite(radius) || radius <= 0) {
      if (input.points) {
        const points = collectPoints(input.points);
        if (points.length) {
          radius = points.reduce((max, pt) => Math.max(max, pt.distanceTo(center)), 0);
        }
      }
    }
    if (!Number.isFinite(radius) || radius <= EPSILON) {
      radius = 1;
    }
    return { type: 'circle', plane, center, radius, shape: input.shape, segments: input.segments ?? 64 };
  }

  function intersectCircleCenters2D(centerA, radiusA, centerB, radiusB) {
    const dx = centerB.x - centerA.x;
    const dy = centerB.y - centerA.y;
    const distSq = dx * dx + dy * dy;
    const dist = Math.sqrt(distSq);
    if (distSq < EPSILON) {
      return [];
    }
    if (dist > radiusA + radiusB || dist < Math.abs(radiusA - radiusB)) {
      return [];
    }
    const a = (radiusA * radiusA - radiusB * radiusB + distSq) / (2 * dist);
    const hSq = Math.max(radiusA * radiusA - a * a, 0);
    const h = Math.sqrt(hSq);
    const ux = dx / dist;
    const uy = dy / dist;
    const baseX = centerA.x + a * ux;
    const baseY = centerA.y + a * uy;
    const offsetX = -uy * h;
    const offsetY = ux * h;
    const result = [{ x: baseX + offsetX, y: baseY + offsetY }];
    if (h > EPSILON) {
      result.push({ x: baseX - offsetX, y: baseY - offsetY });
    }
    return result;
  }

  function tangentCirclesWithRadius(circleAInput, circleBInput, radiusInput) {
    const circleA = ensureCircle(circleAInput);
    const circleB = ensureCircle(circleBInput);
    if (!circleA || !circleB) {
      return [];
    }
    const radius = Math.max(ensureNumber(radiusInput, (circleA.radius + circleB.radius) / 2 || 1), EPSILON);
    const plane = circleA.plane ?? circleB.plane ?? defaultPlane();
    const centerA = planeCoordinates(circleA.center, plane);
    const centerB = planeCoordinates(circleB.center, plane);
    const candidates = intersectCircleCenters2D(centerA, circleA.radius + radius, centerB, circleB.radius + radius);
    return candidates.map((candidate) => {
      const center = applyPlane(plane, candidate.x, candidate.y, 0);
      const shape = new THREE.Shape();
      shape.absarc(0, 0, radius, 0, Math.PI * 2, false);
      return {
        type: 'circle',
        plane,
        center,
        radius,
        shape,
        segments: 128,
      };
    });
  }

  function ensureCurve(input) {
    if (!input) {
      return null;
    }
    if (Array.isArray(input)) {
      if (input.length === 0) {
        return null;
      }
      if (input.length === 1) {
        return ensureCurve(input[0]);
      }
      const points = collectPoints(input);
      if (points.length >= 2) {
        return createCurveFromPoints(points);
      }
    }
    if (input.curve) {
      return ensureCurve(input.curve);
    }
    if (input.path && typeof input.path.getPointAt === 'function') {
      return createCurveFromPath(input.path, { segments: input.segments ?? 64, closed: Boolean(input.closed), type: input.type ?? 'curve' });
    }
    if (input.points && Array.isArray(input.points)) {
      return createCurveFromPoints(input.points, { closed: Boolean(input.closed) });
    }
    if (input.shape?.getPoints) {
      const segments = input.segments ?? 64;
      const pts2d = input.shape.getPoints(Math.max(segments, 8));
      const points = pts2d.map((pt) => new THREE.Vector3(pt.x, pt.y, 0));
      return createCurveFromPoints(points, { closed: true, samples: segments });
    }
    if (input.center && input.radius) {
      const circle = ensureCircle(input);
      const shape = circle.shape ?? (() => {
        const s = new THREE.Shape();
        s.absarc(0, 0, circle.radius, 0, Math.PI * 2, false);
        return s;
      })();
      const path = new THREE.CurvePath();
      path.add(new THREE.EllipseCurve(0, 0, circle.radius, circle.radius, 0, Math.PI * 2, false, 0));
      const curve = createCurveFromPath(path, { segments: circle.segments ?? 128, closed: true, type: 'circle' });
      curve.center = circle.center.clone();
      curve.radius = circle.radius;
      curve.plane = circle.plane;
      curve.shape = shape;
      return curve;
    }
    if (input.start && input.end) {
      const start = ensurePoint(input.start, new THREE.Vector3());
      const end = ensurePoint(input.end, new THREE.Vector3(1, 0, 0));
      const path = new THREE.LineCurve3(start, end);
      return createCurveFromPath(path, { segments: input.segments ?? 8, closed: false, type: 'polyline' });
    }
    if (input.isBufferGeometry || input.isGeometry || input.isMesh) {
      const geometry = input.isMesh ? input.geometry : input;
      if (!geometry) {
        return null;
      }
      const position = geometry.getAttribute?.('position');
      if (!position) {
        return null;
      }
      const points = [];
      for (let i = 0; i < position.count; i += 1) {
        points.push(new THREE.Vector3(position.getX(i), position.getY(i), position.getZ(i)));
      }
      if (!points.length) {
        return null;
      }
      return createCurveFromPoints(points);
    }
    if (input.type && input.points && input.points.length) {
      return createCurveFromPoints(input.points);
    }
    return null;
  }

  function ensureDomain(input) {
    if (!input) {
      return createDomain(0, 1);
    }
    if (Array.isArray(input)) {
      if (input.length >= 2) {
        const start = ensureNumber(input[0], 0);
        const end = ensureNumber(input[1], 1);
        return createDomain(start, end);
      }
      if (input.length === 1) {
        return ensureDomain(input[0]);
      }
      return createDomain(0, 1);
    }
    if (typeof input === 'object') {
      if (typeof input.start !== 'undefined' && typeof input.end !== 'undefined') {
        return createDomain(ensureNumber(input.start, 0), ensureNumber(input.end, 1));
      }
      if (typeof input.min !== 'undefined' && typeof input.max !== 'undefined') {
        return createDomain(ensureNumber(input.min, 0), ensureNumber(input.max, 1));
      }
      if (typeof input.t0 !== 'undefined' && typeof input.t1 !== 'undefined') {
        return createDomain(ensureNumber(input.t0, 0), ensureNumber(input.t1, 1));
      }
    }
    const numeric = ensureNumber(input, Number.NaN);
    if (Number.isFinite(numeric)) {
      return createDomain(numeric, numeric);
    }
    return createDomain(0, 1);
  }

  function ensureUV(input) {
    if (!input) {
      return { u: 0, v: 0 };
    }
    if (Array.isArray(input)) {
      const [u, v] = input;
      return { u: ensureNumber(u, 0), v: ensureNumber(v, 0) };
    }
    if (typeof input === 'object') {
      return {
        u: ensureNumber(input.u ?? input.x ?? input.U ?? input[0], 0),
        v: ensureNumber(input.v ?? input.y ?? input.V ?? input[1], 0),
      };
    }
    const numeric = ensureNumber(input, 0);
    return { u: numeric, v: numeric };
  }

  function ensureSurfaceEvaluator(surfaceInput) {
    if (!surfaceInput) {
      return null;
    }
    if (surfaceInput.surface) {
      return ensureSurfaceEvaluator(surfaceInput.surface);
    }
    if (typeof surfaceInput.getPoint === 'function') {
      return {
        domainU: surfaceInput.domainU ?? surfaceInput.domain?.u ?? createDomain(0, 1),
        domainV: surfaceInput.domainV ?? surfaceInput.domain?.v ?? createDomain(0, 1),
        evaluate: (u, v) => {
          const target = new THREE.Vector3();
          surfaceInput.getPoint(u, v, target);
          return target;
        },
      };
    }
    if (typeof surfaceInput.evaluate === 'function') {
      return {
        domainU: surfaceInput.domainU ?? surfaceInput.domain?.u ?? createDomain(0, 1),
        domainV: surfaceInput.domainV ?? surfaceInput.domain?.v ?? createDomain(0, 1),
        evaluate: (u, v) => {
          const result = surfaceInput.evaluate(u, v);
          return ensurePoint(result, new THREE.Vector3());
        },
      };
    }
    if (surfaceInput.points && Array.isArray(surfaceInput.points) && surfaceInput.points.length) {
      const rows = surfaceInput.points.length;
      const cols = Array.isArray(surfaceInput.points[0]) ? surfaceInput.points[0].length : 0;
      if (rows && cols) {
        const grid = surfaceInput.points.map((row) => row.map((pt) => ensurePoint(pt, new THREE.Vector3())));
        return {
          domainU: createDomain(0, 1),
          domainV: createDomain(0, 1),
          evaluate: (u, v) => {
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
            const p00 = grid[j0][i0];
            const p01 = grid[j0][i1];
            const p10 = grid[j1][i0];
            const p11 = grid[j1][i1];
            const a = p00.clone().lerp(p01, fu);
            const b = p10.clone().lerp(p11, fu);
            return a.lerp(b, fv);
          },
        };
      }
    }
    return null;
  }

  function sampleIsoCurve(surfaceEvaluator, direction, coordinate, segments = 32) {
    if (!surfaceEvaluator) {
      return null;
    }
    const { domainU, domainV } = surfaceEvaluator;
    const points = [];
    for (let i = 0; i <= segments; i += 1) {
      const t = i / segments;
      let u = domainU.start + (domainU.end - domainU.start) * t;
      let v = domainV.start + (domainV.end - domainV.start) * t;
      if (direction === 'u') {
        v = clamp(domainV.start + (domainV.end - domainV.start) * clamp01(coordinate), domainV.min, domainV.max);
      } else {
        u = clamp(domainU.start + (domainU.end - domainU.start) * clamp01(coordinate), domainU.min, domainU.max);
      }
      const point = surfaceEvaluator.evaluate(u, v);
      points.push(point);
    }
    return createCurveFromPoints(points, { closed: false });
  }

  function createHermitePath(points, tangents, { closed = false } = {}) {
    if (points.length < 2) {
      return null;
    }
    const path = new THREE.CurvePath();
    for (let i = 0; i < points.length - 1; i += 1) {
      const p0 = points[i];
      const p1 = points[i + 1];
      const t0 = tangents[i] ?? p1.clone().sub(p0);
      const t1 = tangents[i + 1] ?? t0.clone();
      const control0 = p0.clone().add(t0.clone().multiplyScalar(1 / 3));
      const control1 = p1.clone().sub(t1.clone().multiplyScalar(1 / 3));
      const segment = new THREE.CubicBezierCurve3(p0.clone(), control0, control1, p1.clone());
      path.add(segment);
    }
    if (closed) {
      const p0 = points[points.length - 1];
      const p1 = points[0];
      const t0 = tangents[points.length - 1] ?? p1.clone().sub(p0);
      const t1 = tangents[0] ?? tangents[points.length - 1] ?? t0.clone();
      const control0 = p0.clone().add(t0.clone().multiplyScalar(1 / 3));
      const control1 = p1.clone().sub(t1.clone().multiplyScalar(1 / 3));
      path.add(new THREE.CubicBezierCurve3(p0.clone(), control0, control1, p1.clone()));
    }
    return path;
  }

  function createInterpolatedCurve({
    points: pointsInput,
    degree = 3,
    periodic = false,
    knotStyle = 2,
    startTangent,
    endTangent,
    tangents,
    closed = false,
  }) {
    const points = collectPoints(pointsInput);
    if (points.length < 2) {
      return null;
    }
    const catmullConfig = (() => {
      const style = Math.round(ensureNumber(knotStyle, 2));
      if (style === 0) return { type: 'catmullrom', tension: 0 };
      if (style === 1) return { type: 'chordal', tension: 0 };
      return { type: 'centripetal', tension: 0 };
    })();
    if (tangents && tangents.length) {
      const tangentVectors = [];
      for (let i = 0; i < points.length; i += 1) {
        const tangent = tangents[i] ?? tangents[tangents.length - 1];
        if (tangent) {
          tangentVectors.push(convertVector(tangent) ?? new THREE.Vector3());
        } else {
          const prev = points[Math.max(i - 1, 0)];
          const next = points[Math.min(i + 1, points.length - 1)];
          tangentVectors.push(next.clone().sub(prev).multiplyScalar(0.5));
        }
      }
      const path = createHermitePath(points, tangentVectors, { closed: periodic || closed });
      return createCurveFromPath(path, { segments: points.length * 16, closed: periodic || closed });
    }

    const augmentedPoints = points.map((pt) => pt.clone());
    if (startTangent) {
      const tangent = convertVector(startTangent);
      if (tangent) {
        const scale = points[0].distanceTo(points[1] ?? points[0]) || 1;
        augmentedPoints.unshift(points[0].clone().sub(tangent.clone().normalize().multiplyScalar(scale * 0.3)));
      }
    }
    if (endTangent) {
      const tangent = convertVector(endTangent);
      if (tangent) {
        const last = points[points.length - 1];
        const prev = points[points.length - 2] ?? last;
        const scale = last.distanceTo(prev) || 1;
        augmentedPoints.push(last.clone().add(tangent.clone().normalize().multiplyScalar(scale * 0.3)));
      }
    }
    const path = new THREE.CatmullRomCurve3(augmentedPoints, periodic || closed, catmullConfig.type, catmullConfig.tension || (degree <= 2 ? 1 : 0.5));
    return createCurveFromPath(path, { segments: augmentedPoints.length * 8, closed: periodic || closed });
  }

  function curveSummary(curve) {
    if (!curve) {
      return {};
    }
    return { curve, length: curve.length, domain: curve.domain };
  }

  function registerCircleFitObsolete() {
    register('{0a80e903-e15b-4992-9675-19b2c488e853}', {
      type: 'curve',
      pinMap: {
        inputs: { A: 'circleA', B: 'circleB', R: 'radius' },
        outputs: { A: 'fitA', B: 'fitB' },
      },
      eval: ({ inputs }) => {
        const solutions = tangentCirclesWithRadius(inputs.circleA, inputs.circleB, inputs.radius);
        if (!solutions.length) {
          return {};
        }
        const [first, second] = solutions;
        return { fitA: first, fitB: second ?? first };
      },
    });
  }

  function registerTweenCurve() {
    register('{139619d2-8b18-47b6-b3b9-bf4fec0d6eb1}', {
      type: 'curve',
      pinMap: {
        inputs: { A: 'curveA', B: 'curveB', F: 'factor' },
        outputs: { T: 'tween' },
      },
      eval: ({ inputs }) => {
        const curveA = ensureCurve(inputs.curveA);
        const curveB = ensureCurve(inputs.curveB);
        if (!curveA || !curveB) {
          return {};
        }
        const factor = clamp01(ensureNumber(inputs.factor, 0.5));
        const segments = Math.max(curveA.points?.length ?? 0, curveB.points?.length ?? 0, 32) - 1;
        const count = Math.max(segments, 16);
        const pointsA = sampleCurvePoints(curveA, count);
        const pointsB = sampleCurvePoints(curveB, count);
        const points = [];
        const limit = Math.min(pointsA.length, pointsB.length);
        for (let i = 0; i < limit; i += 1) {
          points.push(pointsA[i].clone().lerp(pointsB[i], factor));
        }
        const tween = createCurveFromPoints(points, { closed: curveA.closed && curveB.closed });
        return { tween };
      },
    });
  }

  function registerBlendCurvePoint() {
    register('{14cf43b6-5eb9-460f-899c-bdece732213a}', {
      type: 'curve',
      pinMap: {
        inputs: { A: 'curveA', B: 'curveB', P: 'point', C: 'continuity' },
        outputs: { B: 'blend' },
      },
      eval: ({ inputs }) => {
        const curveA = ensureCurve(inputs.curveA);
        const curveB = ensureCurve(inputs.curveB);
        if (!curveA || !curveB) {
          return {};
        }
        const endA = curvePointAt(curveA, 1) ?? ensurePoint(inputs.point, new THREE.Vector3());
        const startB = curvePointAt(curveB, 0) ?? ensurePoint(inputs.point, new THREE.Vector3());
        const anchor = ensurePoint(inputs.point, endA.clone().lerp(startB, 0.5));
        const continuity = Math.max(1, Math.round(ensureNumber(inputs.continuity, 1)));
        const tangentA = curveTangentAt(curveA, 1);
        const tangentB = curveTangentAt(curveB, 0).multiplyScalar(-1);
        const distanceA = Math.max(endA.distanceTo(anchor), EPSILON);
        const distanceB = Math.max(startB.distanceTo(anchor), EPSILON);
        const scaleA = distanceA / (continuity + 1);
        const scaleB = distanceB / (continuity + 1);
        const controlA1 = endA.clone().add(tangentA.clone().multiplyScalar(scaleA));
        const controlA2 = anchor.clone().sub(tangentA.clone().multiplyScalar(scaleA * 0.5));
        const controlB1 = anchor.clone().add(tangentB.clone().multiplyScalar(scaleB * 0.5));
        const controlB2 = startB.clone().sub(tangentB.clone().multiplyScalar(scaleB));
        const path = new THREE.CurvePath();
        path.add(new THREE.CubicBezierCurve3(endA.clone(), controlA1, controlA2, anchor.clone()));
        path.add(new THREE.CubicBezierCurve3(anchor.clone(), controlB1, controlB2, startB.clone()));
        const blend = createCurveFromPath(path, { segments: 64, closed: false });
        return { blend };
      },
    });
  }

  function registerNurbsCurvePWK() {
    register('{1f8e1ff7-8278-4421-b39d-350e71d85d37}', {
      type: 'curve',
      pinMap: {
        inputs: { P: 'points', W: 'weights', K: 'knots' },
        outputs: { C: 'curve', L: 'length', D: 'domain' },
      },
      eval: ({ inputs }) => {
        const points = collectPoints(inputs.points);
        if (points.length < 2) {
          return {};
        }
        const weights = ensureArray(inputs.weights).map((value) => ensureNumber(value, 1));
        const curve = createInterpolatedCurve({ points, degree: 3, periodic: false, knotStyle: ensureNumber(inputs.knots?.style ?? inputs.knotStyle, 2) });
        if (curve) {
          curve.weights = weights;
          curve.knots = ensureArray(inputs.knots).map((value) => ensureNumber(value, 0));
        }
        return curveSummary(curve);
      },
    });
  }

  function registerIsoCurve(guid) {
    register(guid, {
      type: 'curve',
      pinMap: {
        inputs: { S: 'surface', uv: 'uv' },
        outputs: { U: 'uCurve', V: 'vCurve' },
      },
      eval: ({ inputs }) => {
        const evaluator = ensureSurfaceEvaluator(inputs.surface);
        if (!evaluator) {
          return {};
        }
        const { u, v } = ensureUV(inputs.uv);
        const uCurve = sampleIsoCurve(evaluator, 'u', v, 64);
        const vCurve = sampleIsoCurve(evaluator, 'v', u, 64);
        return { uCurve, vCurve };
      },
    });
  }

  function createCatenaryPoints(start, end, length, gravity) {
    const direction = end.clone().sub(start);
    const span = direction.length();
    if (span < EPSILON) {
      return [start.clone(), end.clone()];
    }
    const gravityDir = gravity.lengthSq() < EPSILON ? new THREE.Vector3(0, -1, 0) : gravity.clone().normalize();
    const horizontal = direction.clone().normalize();
    const binormal = horizontal.clone().cross(gravityDir).normalize();
    const vertical = binormal.clone().cross(horizontal).normalize();
    const extraLength = Math.max(length - span, 0);
    const sag = extraLength / 2;
    const midPoint = start.clone().add(direction.clone().multiplyScalar(0.5)).add(vertical.clone().multiplyScalar(-sag));
    return [start.clone(), midPoint, end.clone()];
  }

  function registerCatenary() {
    register('{275671d4-3e87-40bd-8aff-8e6a5fdbb892}', {
      type: 'curve',
      pinMap: {
        inputs: { A: 'pointA', B: 'pointB', L: 'length', G: 'gravity' },
        outputs: { C: 'catenary' },
      },
      eval: ({ inputs }) => {
        const pointA = ensurePoint(inputs.pointA, new THREE.Vector3());
        const pointB = ensurePoint(inputs.pointB, new THREE.Vector3(1, 0, 0));
        const minLength = pointA.distanceTo(pointB);
        const length = Math.max(ensureNumber(inputs.length, minLength), minLength);
        const gravity = convertVector(inputs.gravity) ?? new THREE.Vector3(0, -1, 0);
        const points = createCatenaryPoints(pointA, pointB, length, gravity);
        const curve = createCurveFromPoints(points, { samples: 64 });
        return { catenary: curve };
      },
    });
  }

  function registerMatchCurve() {
    register('{282bf4eb-668a-4a2c-81af-2432ac863ddd}', {
      type: 'curve',
      pinMap: {
        inputs: { A: 'curveA', B: 'curveB', C: 'continuity' },
        outputs: { M: 'match' },
      },
      eval: ({ inputs }) => {
        const curveA = ensureCurve(inputs.curveA);
        const curveB = ensureCurve(inputs.curveB);
        if (!curveA || !curveB) {
          return {};
        }
        const continuity = Math.max(1, Math.round(ensureNumber(inputs.continuity, 1)));
        const start = curvePointAt(curveA, 0);
        const end = curvePointAt(curveB, 1);
        const tangentStart = curveTangentAt(curveA, 0);
        const tangentEnd = curveTangentAt(curveB, 1);
        const controlA = start.clone().add(tangentStart.clone().multiplyScalar(continuity));
        const controlB = end.clone().sub(tangentEnd.clone().multiplyScalar(continuity));
        const path = new THREE.CubicBezierCurve3(start.clone(), controlA, controlB, end.clone());
        const match = createCurveFromPath(path, { segments: 64, closed: false });
        return { match };
      },
    });
  }

  function registerInterpolate() {
    register('{2b2a4145-3dff-41d4-a8de-1ea9d29eef33}', {
      type: 'curve',
      pinMap: {
        inputs: { V: 'points', D: 'degree', P: 'periodic', K: 'knotStyle' },
        outputs: { C: 'curve', L: 'length', D: 'domain' },
      },
      eval: ({ inputs }) => {
        const points = collectPoints(inputs.points);
        if (points.length < 2) {
          return {};
        }
        const periodic = ensureBoolean(inputs.periodic, false);
        const degree = Math.max(1, Math.round(ensureNumber(inputs.degree, 3)));
        const knotStyle = ensureNumber(inputs.knotStyle, 2);
        const curve = createInterpolatedCurve({ points, degree, periodic, knotStyle, closed: periodic });
        return curveSummary(curve);
      },
    });
  }

  function registerBezierSpan() {
    register('{30ce59ce-22a1-49ee-9e21-e6d16b3684a8}', {
      type: 'curve',
      pinMap: {
        inputs: { A: 'start', At: 'startTangent', B: 'end', Bt: 'endTangent' },
        outputs: { C: 'curve', L: 'length', D: 'domain' },
      },
      eval: ({ inputs }) => {
        const start = ensurePoint(inputs.start, new THREE.Vector3());
        const end = ensurePoint(inputs.end, new THREE.Vector3(1, 0, 0));
        const tangentStart = convertVector(inputs.startTangent) ?? end.clone().sub(start);
        const tangentEnd = convertVector(inputs.endTangent) ?? end.clone().sub(start);
        const control1 = start.clone().add(tangentStart.clone().multiplyScalar(1 / 3));
        const control2 = end.clone().sub(tangentEnd.clone().multiplyScalar(1 / 3));
        const path = new THREE.CubicBezierCurve3(start.clone(), control1, control2, end.clone());
        const curve = createCurveFromPath(path, { segments: 64, closed: false });
        return curveSummary(curve);
      },
    });
  }

  function registerSwingArc() {
    register('{3edc4fbd-24c6-43de-aaa8-5bdf0704373d}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'centers', P: 'plane', R: 'radius' },
        outputs: { A: 'curveA', B: 'curveB', C: 'circles' },
      },
      eval: ({ inputs }) => {
        const centers = collectPoints(inputs.centers);
        if (centers.length < 2) {
          return {};
        }
        const radius = Math.max(ensureNumber(inputs.radius, centers[0].distanceTo(centers[1]) || 1), EPSILON);
        const plane = ensurePlane(inputs.plane ?? { origin: centers[0], normal: new THREE.Vector3(0, 0, 1) });
        const circles = centers.map((center) => ({
          type: 'circle',
          plane,
          center: center.clone(),
          radius,
          segments: 128,
          shape: (() => {
            const shape = new THREE.Shape();
            shape.absarc(0, 0, radius, 0, Math.PI * 2, false);
            return shape;
          })(),
        }));
        const curveA = createCurveFromPoints(centers, { curveType: 'chordal', samples: centers.length * 8 });
        const mirrored = centers.map((pt) => {
          const coords = planeCoordinates(pt, plane);
          return applyPlane(plane, coords.x, coords.y, -coords.z);
        });
        const curveB = createCurveFromPoints(mirrored.reverse(), { curveType: 'catmullrom', samples: centers.length * 8 });
        return { curveA, curveB, circles };
      },
    });
  }

  function registerSubCurve() {
    register('{429cbba9-55ee-4e84-98ea-876c44db879a}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', D: 'domain' },
        outputs: { C: 'subCurve' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const domain = ensureDomain(inputs.domain);
        const start = clamp01(domain.start ?? domain.min ?? 0);
        const end = clamp01(domain.end ?? domain.max ?? 1);
        const segments = 64;
        const points = [];
        for (let i = 0; i <= segments; i += 1) {
          const t = start + (end - start) * (i / segments);
          const point = curvePointAt(curve, t);
          if (point) {
            points.push(point);
          }
        }
        const subCurve = createCurveFromPoints(points, { samples: segments });
        return { subCurve };
      },
    });
  }

  function registerInterpolateWithTangents(guid, { includeDegree }) {
    register(guid, {
      type: 'curve',
      pinMap: {
        inputs: includeDegree
          ? { V: 'points', D: 'degree', Ts: 'startTangent', Te: 'endTangent', K: 'knotStyle' }
          : { V: 'points', Ts: 'startTangent', Te: 'endTangent', K: 'knotStyle' },
        outputs: { C: 'curve', L: 'length', D: 'domain' },
      },
      eval: ({ inputs }) => {
        const points = collectPoints(inputs.points);
        if (points.length < 2) {
          return {};
        }
        const degree = includeDegree ? Math.max(1, Math.round(ensureNumber(inputs.degree, 3))) : 3;
        const knotStyle = ensureNumber(inputs.knotStyle, 2);
        const curve = createInterpolatedCurve({
          points,
          degree,
          knotStyle,
          startTangent: inputs.startTangent,
          endTangent: inputs.endTangent,
        });
        return curveSummary(curve);
      },
    });
  }

  function registerBlendCurve() {
    register('{5909dbcb-4950-4ce4-9433-7cf9e62ee011}', {
      type: 'curve',
      pinMap: {
        inputs: { A: 'curveA', B: 'curveB', Fa: 'factorA', Fb: 'factorB', C: 'continuity' },
        outputs: { B: 'blend' },
      },
      eval: ({ inputs }) => {
        const curveA = ensureCurve(inputs.curveA);
        const curveB = ensureCurve(inputs.curveB);
        if (!curveA || !curveB) {
          return {};
        }
        const factorA = clamp01(ensureNumber(inputs.factorA, 0.5));
        const factorB = clamp01(ensureNumber(inputs.factorB, 0.5));
        const segments = 64;
        const points = [];
        for (let i = 0; i <= segments; i += 1) {
          const t = i / segments;
          const pointA = curvePointAt(curveA, t) ?? curvePointAt(curveA, clamp01(t * factorA));
          const pointB = curvePointAt(curveB, t) ?? curvePointAt(curveB, clamp01(t * factorB));
          if (pointA && pointB) {
            const weight = clamp01((1 - t) * factorA + t * factorB);
            points.push(pointA.clone().lerp(pointB, weight));
          }
        }
        const blend = createCurveFromPoints(points, { samples: segments });
        return { blend };
      },
    });
  }

  function registerKinkyCurve() {
    register('{6f0993e8-5f2f-4fc0-bd73-b84bc240e78e}', {
      type: 'curve',
      pinMap: {
        inputs: { V: 'points', D: 'degree', A: 'angle' },
        outputs: { C: 'curve', L: 'length', D: 'domain' },
      },
      eval: ({ inputs }) => {
        const points = collectPoints(inputs.points);
        if (points.length < 2) {
          return {};
        }
        const angleThreshold = ensureNumber(inputs.angle, Math.PI / 6);
        const filtered = [points[0].clone()];
        for (let i = 1; i < points.length - 1; i += 1) {
          const prev = points[i - 1];
          const current = points[i];
          const next = points[i + 1];
          const v1 = current.clone().sub(prev).normalize();
          const v2 = next.clone().sub(current).normalize();
          const angle = Math.acos(clamp(v1.dot(v2), -1, 1));
          filtered.push(current.clone());
          if (angle > angleThreshold) {
            filtered.push(current.clone());
          }
        }
        filtered.push(points[points.length - 1].clone());
        const curve = createCurveFromPoints(filtered, { curveType: 'chordal', samples: filtered.length * 8 });
        return curveSummary(curve);
      },
    });
  }

  function registerPolyArcDetailed() {
    register('{7159ef59-e4ef-44b8-8cb2-91231e278292}', {
      type: 'curve',
      pinMap: {
        inputs: { V: 'points', T: 'tangent', C: 'closed' },
        outputs: { Crv: 'polyArc' },
      },
      eval: ({ inputs }) => {
        const points = collectPoints(inputs.points);
        if (points.length < 2) {
          return {};
        }
        const closed = ensureBoolean(inputs.closed, false);
        const tangent = convertVector(inputs.tangent);
        const augmented = points.map((pt) => pt.clone());
        if (tangent) {
          const offset = tangent.clone().normalize().multiplyScalar(points[0].distanceTo(points[1] ?? points[0]) * 0.5);
          augmented.unshift(points[0].clone().sub(offset));
        }
        const curve = createCurveFromPoints(augmented, { curveType: 'catmullrom', samples: augmented.length * 8, closed });
        return { polyArc: curve };
      },
    });
  }

  function registerPolyLine() {
    register('{71b5b089-500a-4ea6-81c5-2f960441a0e8}', {
      type: 'curve',
      pinMap: {
        inputs: { V: 'points', C: 'closed' },
        outputs: { Pl: 'polyline' },
      },
      eval: ({ inputs }) => {
        const points = collectPoints(inputs.points);
        if (points.length < 2) {
          return {};
        }
        const closed = ensureBoolean(inputs.closed, false);
        if (closed && points.length >= 2) {
          points.push(points[0].clone());
        }
        const polyline = createCurveFromPoints(points, { curveType: 'chordal', samples: points.length - 1 });
        return { polyline };
      },
    });
  }

  function registerCatenaryEx() {
    register('{769f9064-17f5-4c4a-921f-c3a0ee05ba3a}', {
      type: 'curve',
      pinMap: {
        inputs: { A: 'pointA', B: 'pointB', L: 'length', W: 'weights', G: 'gravity' },
        outputs: { C: 'catenary', S: 'segments' },
      },
      eval: ({ inputs }) => {
        const pointA = ensurePoint(inputs.pointA, new THREE.Vector3());
        const pointB = ensurePoint(inputs.pointB, new THREE.Vector3(1, 0, 0));
        const minLength = pointA.distanceTo(pointB);
        const length = Math.max(ensureNumber(inputs.length, minLength), minLength);
        const weights = ensureArray(inputs.weights).map((value) => Math.max(ensureNumber(value, 1), 0));
        const gravity = convertVector(inputs.gravity) ?? new THREE.Vector3(0, -1, 0);
        const weightFactor = weights.length ? weights.reduce((sum, value) => sum + value, 0) / weights.length : 1;
        const adjustedLength = length + weightFactor * 0.25;
        const points = createCatenaryPoints(pointA, pointB, adjustedLength, gravity);
        const curve = createCurveFromPoints(points, { samples: 64 });
        const segments = sampleCurvePoints(curve, 32);
        return { catenary: curve, segments };
      },
    });
  }

  function registerKnotVector() {
    register('{846470bd-4918-4d00-9388-7e022b2cba73}', {
      type: 'curve',
      pinMap: {
        inputs: { N: 'count', D: 'degree', P: 'periodic' },
        outputs: { K: 'knots' },
      },
      eval: ({ inputs }) => {
        const count = Math.max(2, Math.round(ensureNumber(inputs.count, 4)));
        const degree = Math.max(1, Math.round(ensureNumber(inputs.degree, 3)));
        const periodic = ensureBoolean(inputs.periodic, false);
        const knotCount = count + degree + (periodic ? 1 : 1);
        const knots = [];
        if (periodic) {
          for (let i = 0; i < knotCount; i += 1) {
            knots.push(i / (knotCount - 1));
          }
        } else {
          for (let i = 0; i < knotCount; i += 1) {
            if (i <= degree) {
              knots.push(0);
            } else if (i >= count) {
              knots.push(1);
            } else {
              knots.push((i - degree) / (count - degree));
            }
          }
        }
        return { knots };
      },
    });
  }

  function registerPolyArcSimple() {
    register('{a5e4f966-417e-465d-afa9-f6607afea056}', {
      type: 'curve',
      pinMap: {
        inputs: { V: 'points' },
        outputs: { Crv: 'polyArc' },
      },
      eval: ({ inputs }) => {
        const points = collectPoints(inputs.points);
        if (points.length < 2) {
          return {};
        }
        const curve = createCurveFromPoints(points, { curveType: 'centripetal', samples: points.length * 8 });
        return { polyArc: curve };
      },
    });
  }

  function registerGeodesic() {
    register('{ce5963b4-1cea-4f71-acd2-a3c28ab85662}', {
      type: 'curve',
      pinMap: {
        inputs: { S: 'surface', Surface: 'surface', Start: 'start', s: 'start', E: 'end', end: 'end' },
        outputs: { G: 'geodesic' },
      },
      eval: ({ inputs }) => {
        const evaluator = ensureSurfaceEvaluator(inputs.surface);
        const start = ensurePoint(inputs.start, new THREE.Vector3());
        const end = ensurePoint(inputs.end, new THREE.Vector3(1, 0, 0));
        if (!evaluator) {
          const path = new THREE.LineCurve3(start, end);
          return { geodesic: createCurveFromPath(path, { segments: 32 }) };
        }
        const segments = 64;
        const points = [];
        for (let i = 0; i <= segments; i += 1) {
          const t = i / segments;
          const u = evaluator.domainU.start + (evaluator.domainU.end - evaluator.domainU.start) * t;
          const v = evaluator.domainV.start + (evaluator.domainV.end - evaluator.domainV.start) * t;
          const blendPoint = start.clone().lerp(end, t);
          const surfacePoint = evaluator.evaluate(u, v);
          points.push(blendPoint.clone().lerp(surfacePoint, 0.5));
        }
        const curve = createCurveFromPoints(points, { samples: segments });
        return { geodesic: curve };
      },
    });
  }

  function registerConnectCurves() {
    register('{d0a1b843-873d-4d1d-965c-b5423b35f327}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curves', curves: 'curves', G: 'continuity', L: 'close', B: 'bulge' },
        outputs: { C: 'curve' },
      },
      eval: ({ inputs }) => {
        const curves = ensureArray(inputs.curves).map((entry) => ensureCurve(entry)).filter(Boolean);
        if (!curves.length) {
          return {};
        }
        const continuity = Math.max(0, Math.round(ensureNumber(inputs.continuity, 0)));
        const closed = ensureBoolean(inputs.close, false);
        const bulge = ensureNumber(inputs.bulge, 0.5);
        const path = new THREE.CurvePath();
        const addCurvePoints = (curve) => {
          if (curve.path?.curves) {
            for (const segment of curve.path.curves) {
              path.add(segment.clone());
            }
          } else {
            const points = sampleCurvePoints(curve, 32);
            for (let i = 0; i < points.length - 1; i += 1) {
              path.add(new THREE.LineCurve3(points[i].clone(), points[i + 1].clone()));
            }
          }
        };
        addCurvePoints(curves[0]);
        for (let i = 1; i < curves.length; i += 1) {
          const prev = curves[i - 1];
          const next = curves[i];
          const start = curvePointAt(prev, 1);
          const end = curvePointAt(next, 0);
          if (start && end && start.distanceToSquared(end) > EPSILON) {
            const tangentPrev = curveTangentAt(prev, 1);
            const tangentNext = curveTangentAt(next, 0).multiplyScalar(-1);
            if (continuity === 0) {
              path.add(new THREE.LineCurve3(start.clone(), end.clone()));
            } else {
              const controlA = start.clone().add(tangentPrev.clone().multiplyScalar(bulge));
              const controlB = end.clone().add(tangentNext.clone().multiplyScalar(bulge));
              path.add(new THREE.CubicBezierCurve3(start.clone(), controlA, controlB, end.clone()));
            }
          }
          addCurvePoints(next);
        }
        if (closed) {
          const start = curvePointAt(curves[0], 0);
          const end = curvePointAt(curves[curves.length - 1], 1);
          if (start && end && start.distanceToSquared(end) > EPSILON) {
            path.add(new THREE.LineCurve3(end.clone(), start.clone()));
          }
        }
        const curve = createCurveFromPath(path, { segments: curves.length * 32, closed });
        return { curve };
      },
    });
  }

  function registerNurbsCurveSimple() {
    register('{dde71aef-d6ed-40a6-af98-6b0673983c82}', {
      type: 'curve',
      pinMap: {
        inputs: { V: 'points', D: 'degree', P: 'periodic' },
        outputs: { C: 'curve', L: 'length', D: 'domain' },
      },
      eval: ({ inputs }) => {
        const points = collectPoints(inputs.points);
        if (points.length < 2) {
          return {};
        }
        const degree = Math.max(1, Math.round(ensureNumber(inputs.degree, 3)));
        const periodic = ensureBoolean(inputs.periodic, false);
        const curve = createInterpolatedCurve({ points, degree, periodic, knotStyle: 2, closed: periodic });
        return curveSummary(curve);
      },
    });
  }

  function registerInterpolateDuplicate() {
    register('{f5ea9d41-f062-487e-8dbf-7666ca53fbcd}', {
      type: 'curve',
      pinMap: {
        inputs: { V: 'points', D: 'degree', P: 'periodic' },
        outputs: { C: 'curve', L: 'length', D: 'domain' },
      },
      eval: ({ inputs }) => {
        const points = collectPoints(inputs.points);
        if (points.length < 2) {
          return {};
        }
        const degree = Math.max(1, Math.round(ensureNumber(inputs.degree, 3)));
        const periodic = ensureBoolean(inputs.periodic, false);
        const curve = createInterpolatedCurve({ points, degree, periodic, knotStyle: 2, closed: periodic });
        return curveSummary(curve);
      },
    });
  }

  function registerTangentCurve() {
    register('{f73498c5-178b-4e09-ad61-73d172fa6e56}', {
      type: 'curve',
      pinMap: {
        inputs: { V: 'points', T: 'tangents', B: 'blend', D: 'degree' },
        outputs: { C: 'curve', L: 'length', D: 'domain' },
      },
      eval: ({ inputs }) => {
        const points = collectPoints(inputs.points);
        if (points.length < 2) {
          return {};
        }
        const blend = clamp01(ensureNumber(inputs.blend, 0.5));
        const tangents = ensureArray(inputs.tangents).map((entry, index) => {
          const vector = convertVector(entry) ?? new THREE.Vector3();
          return vector.multiplyScalar(blend || 1);
        });
        const curve = createInterpolatedCurve({ points, tangents, degree: Math.max(1, Math.round(ensureNumber(inputs.degree, 3))) });
        return curveSummary(curve);
      },
    });
  }

  function registerCurveOnSurface() {
    register('{ffe2dbed-9b5d-4f91-8fe3-10c8961ac2f8}', {
      type: 'curve',
      pinMap: {
        inputs: { S: 'surface', uv: 'uvs', C: 'closed' },
        outputs: { C: 'curve', L: 'length', D: 'domain' },
      },
      eval: ({ inputs }) => {
        const evaluator = ensureSurfaceEvaluator(inputs.surface);
        const uvEntries = ensureArray(inputs.uvs).map((entry) => ensureUV(entry));
        if (!evaluator || !uvEntries.length) {
          return {};
        }
        const closed = ensureBoolean(inputs.closed, false);
        const points = uvEntries.map(({ u, v }) => {
          const uu = evaluator.domainU.start + (evaluator.domainU.end - evaluator.domainU.start) * clamp01(u);
          const vv = evaluator.domainV.start + (evaluator.domainV.end - evaluator.domainV.start) * clamp01(v);
          return evaluator.evaluate(uu, vv);
        });
        if (closed && points.length) {
          points.push(points[0].clone());
        }
        const curve = createCurveFromPoints(points, { samples: points.length * 8, closed });
        return curveSummary(curve);
      },
    });
  }

  registerCircleFitObsolete();
  registerTweenCurve();
  registerBlendCurvePoint();
  registerNurbsCurvePWK();
  registerIsoCurve('{21ca41ee-bc18-4ac8-ba20-713e7edf541e}');
  registerIsoCurve('{d1d57181-d594-41e8-8efb-041e29f8a5ca}');
  registerCatenary();
  registerMatchCurve();
  registerInterpolate();
  registerBezierSpan();
  registerSwingArc();
  registerSubCurve();
  registerInterpolateWithTangents('{50870118-be51-4872-ab3c-410d79f2356e}', { includeDegree: true });
  registerInterpolateWithTangents('{75eb156d-d023-42f9-a85e-2f2456b8bcce}', { includeDegree: false });
  registerInterpolateWithTangents('{e8e00fbb-9710-4cfa-a60f-2aae50b79d06}', { includeDegree: true });
  registerBlendCurve();
  registerKinkyCurve();
  registerPolyArcDetailed();
  registerPolyLine();
  registerCatenaryEx();
  registerKnotVector();
  registerPolyArcSimple();
  registerGeodesic();
  registerConnectCurves();
  registerNurbsCurveSimple();
  registerInterpolateDuplicate();
  registerTangentCurve();
  registerCurveOnSurface();
}

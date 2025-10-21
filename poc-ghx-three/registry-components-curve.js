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

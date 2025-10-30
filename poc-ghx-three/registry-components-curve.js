import { loadThreeCore } from './three-loader.js';

const THREE = await loadThreeCore();

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

  function clamp(value, min, max) {
    return Math.min(Math.max(value, min), max);
  }

  function clamp01(value) {
    return clamp(value, 0, 1);
  }

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

  function ensureNumber(value, fallback = 0) {
    return toNumber(value, fallback);
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

  function createCurveFromPath(path, { segments = 64, closed = false, type = 'curve' } = {}) {
    if (!path) {
      return null;
    }
    const safeSegments = Math.max(segments, 8);
    const spaced = path.getSpacedPoints(safeSegments) ?? [];
    const points = spaced.map((pt, index) => {
      if (pt && (Number.isFinite(pt.x) || Number.isFinite(pt.y) || Number.isFinite(pt.z))) {
        return new THREE.Vector3(pt.x ?? 0, pt.y ?? 0, pt.z ?? 0);
      }
      const fallback = spaced[index - 1] ?? spaced[index + 1] ?? { x: 0, y: 0, z: 0 };
      return new THREE.Vector3(fallback.x ?? 0, fallback.y ?? 0, fallback.z ?? 0);
    });
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
    curve.isNativeCurve = true;
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
    const safeRadius = Math.max(radius, EPSILON);
    const centerPoint = center.clone();
    const normalizedPlane = normalizePlaneAxes(
      centerPoint.clone(),
      plane.xAxis.clone(),
      plane.yAxis.clone(),
      plane.zAxis.clone(),
    );
    const shape = new THREE.Shape();
    shape.absarc(0, 0, safeRadius, 0, Math.PI * 2, false);
    return {
      type: 'circle',
      plane: normalizedPlane,
      center: centerPoint,
      radius: safeRadius,
      shape,
      segments,
      curveDefinition: {
        type: 'circle',
        plane: normalizedPlane,
        center: centerPoint.clone(),
        radius: safeRadius,
        startAngle: 0,
        endAngle: Math.PI * 2,
        segments,
      },
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
    if (v1.lengthSq() < EPSILON || v2.lengthSq() < EPSILON) {
      return null;
    }
    const normal = v1.clone().cross(v2);
    if (normal.lengthSq() < EPSILON) {
      return null;
    }
    const plane = createPlane(p1, v1, normal.clone().cross(v1), normal);
    const origin1 = planeCoordinates(p1, plane);
    const dir1 = planeCoordinates(p2, plane);
    const origin2 = planeCoordinates(p3, plane);
    const dir2 = planeCoordinates(p4, plane);
    const d1 = new THREE.Vector2(dir1.x - origin1.x, dir1.y - origin1.y);
    const d2 = new THREE.Vector2(dir2.x - origin2.x, dir2.y - origin2.y);
    const determinant = d1.x * d2.y - d1.y * d2.x;
    if (Math.abs(determinant) < EPSILON) {
      return null;
    }
    const offset = new THREE.Vector2(origin2.x - origin1.x, origin2.y - origin1.y);
    const t = (offset.x * d2.y - offset.y * d2.x) / determinant;
    const intersection2d = new THREE.Vector2(origin1.x + d1.x * t, origin1.y + d1.y * t);
    return applyPlane(plane, intersection2d.x, intersection2d.y, 0);
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

  function createArcFromAngles(planeInput, radiusInput, startAngleInput, endAngleInput, centerInput) {
    const basePlane = planeInput ? ensurePlane(planeInput) : defaultPlane();
    const safeRadius = Math.max(Math.abs(radiusInput ?? 0), EPSILON);
    const center = centerInput
      ? ensurePoint(centerInput, basePlane.origin.clone())
      : basePlane.origin.clone();
    const normalizedPlane = normalizePlaneAxes(
      center.clone(),
      basePlane.xAxis.clone(),
      basePlane.yAxis.clone(),
      basePlane.zAxis.clone(),
    );
    const startAngle = startAngleInput;
    const endAngle = endAngleInput;
    const segments = 128;
    const curve = createCircularCurve({
      plane: normalizedPlane,
      center,
      radius: safeRadius,
      startAngle,
      endAngle,
      segments,
    });
    return {
      type: 'arc',
      plane: normalizedPlane,
      center: curve.center.clone(),
      radius: safeRadius,
      startAngle,
      endAngle,
      start: curve.start.clone(),
      end: curve.end.clone(),
      mid: curve.mid.clone(),
      length: curve.length,
      segments: curve.segments,
      curveDefinition: {
        type: 'arc',
        plane: normalizedPlane,
        center: curve.center.clone(),
        radius: safeRadius,
        startAngle,
        endAngle,
        segments: curve.segments,
      },
      path: curve.path,
      curve,
    };
  }

  function createCircularCurve({ plane, center, radius, startAngle, endAngle, segments = 128 }) {
    const safeRadius = Math.max(Math.abs(radius), EPSILON);
    const normalizedPlane = normalizePlaneAxes(
      center.clone(),
      plane.xAxis.clone(),
      plane.yAxis.clone(),
      plane.zAxis.clone(),
    );
    const deltaAngle = endAngle - startAngle;
    const totalAngle = Math.abs(deltaAngle);
    const fullCircle = Math.abs(totalAngle - Math.PI * 2) <= 1e-6;
    const segmentValue = Number(segments ?? 128);
    const safeSegments = Math.max(Math.round(Number.isFinite(segmentValue) ? segmentValue : 128), 8);
    const path = new THREE.Path();
    path.absarc(0, 0, safeRadius, startAngle, endAngle, deltaAngle < 0);
    const curve = createCurveFromPath(path, {
      segments: safeSegments,
      closed: fullCircle,
      type: fullCircle ? 'circle' : 'arc',
    });
    const computePoint = (t) => {
      const clamped = clamp01(t);
      const angle = startAngle + deltaAngle * clamped;
      return applyPlane(normalizedPlane, Math.cos(angle) * safeRadius, Math.sin(angle) * safeRadius, 0);
    };
    curve.getPointAt = (t) => computePoint(t);
    curve.getTangentAt = (t) => {
      const clamped = clamp01(t);
      const angle = startAngle + deltaAngle * clamped;
      const derivative = new THREE.Vector3(-Math.sin(angle), Math.cos(angle), 0).multiplyScalar(deltaAngle);
      const tangent = normalizedPlane.xAxis.clone().multiplyScalar(derivative.x)
        .add(normalizedPlane.yAxis.clone().multiplyScalar(derivative.y));
      if (tangent.lengthSq() < EPSILON) {
        return normalizedPlane.xAxis.clone();
      }
      return tangent.normalize();
    };
    const points = [];
    for (let i = 0; i <= safeSegments; i += 1) {
      const t = safeSegments === 0 ? 0 : i / safeSegments;
      points.push(curve.getPointAt(t));
    }
    curve.points = points;
    curve.plane = normalizedPlane;
    curve.center = normalizedPlane.origin.clone();
    curve.radius = safeRadius;
    curve.startAngle = startAngle;
    curve.endAngle = endAngle;
    curve.start = curve.getPointAt(0);
    curve.end = curve.getPointAt(1);
    curve.mid = curve.getPointAt(0.5);
    curve.length = totalAngle * safeRadius;
    return curve;
  }

  function extractArcParameters(input) {
    if (!input) {
      return null;
    }
    const definition = input.curveDefinition ?? input;
    const basePlane = definition.plane ? ensurePlane(definition.plane) : defaultPlane();
    const centerSource = definition.center ?? input.center;
    const center = centerSource
      ? ensurePoint(centerSource, basePlane.origin.clone())
      : basePlane.origin.clone();
    const normalizedPlane = normalizePlaneAxes(
      center.clone(),
      basePlane.xAxis.clone(),
      basePlane.yAxis.clone(),
      basePlane.zAxis.clone(),
    );
    const radiusValue = definition.radius ?? input.radius ?? 0;
    const radius = Math.max(Math.abs(ensureNumber(radiusValue, 0)), EPSILON);
    let startAngle = definition.startAngle;
    if (startAngle !== undefined) {
      const numericStart = ensureNumber(startAngle, Number.NaN);
      startAngle = Number.isFinite(numericStart) ? numericStart : undefined;
    }
    let endAngle = definition.endAngle;
    if (endAngle !== undefined) {
      const numericEnd = ensureNumber(endAngle, Number.NaN);
      endAngle = Number.isFinite(numericEnd) ? numericEnd : undefined;
    }
    const startPoint = definition.start ?? input.start;
    const endPoint = definition.end ?? input.end;
    const midPoint = definition.mid ?? input.mid;
    if (startAngle === undefined && startPoint) {
      const coords = planeCoordinates(startPoint, normalizedPlane);
      startAngle = Math.atan2(coords.y, coords.x);
    }
    if (endAngle === undefined && endPoint) {
      const coords = planeCoordinates(endPoint, normalizedPlane);
      endAngle = Math.atan2(coords.y, coords.x);
    }
    if (startAngle === undefined) {
      startAngle = 0;
    }
    if (endAngle === undefined) {
      endAngle = startAngle + Math.PI * 2;
    }
    if (Math.abs(endAngle - startAngle) < EPSILON && midPoint) {
      const coordsMid = planeCoordinates(midPoint, normalizedPlane);
      const midAngle = Math.atan2(coordsMid.y, coordsMid.x);
      endAngle = startAngle + Math.PI * 2;
      if (!isAngleBetween(midAngle, startAngle, endAngle)) {
        startAngle -= Math.PI * 2;
      }
    } else if (midPoint) {
      const coordsMid = planeCoordinates(midPoint, normalizedPlane);
      const midAngle = Math.atan2(coordsMid.y, coordsMid.x);
      if (!isAngleBetween(midAngle, startAngle, endAngle)) {
        if (endAngle < startAngle) {
          endAngle += Math.PI * 2;
        } else {
          startAngle -= Math.PI * 2;
        }
      }
    }
    if (Math.abs(endAngle - startAngle) < EPSILON) {
      endAngle = startAngle + Math.PI * 2;
    }
    const segments = ensureNumber(definition.segments ?? input.segments ?? 128, 128);
    return {
      plane: normalizedPlane,
      center,
      radius,
      startAngle,
      endAngle,
      segments,
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
    return createArcFromAngles(plane, radius, normalizedStart, normalizedEnd, center);
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
    return createArcFromAngles(plane, Math.abs(radius), startAngle, endAngle, center);
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
    if (Array.isArray(input)) {
      for (const entry of input) {
        if (entry?.start?.isVector3 && entry?.end?.isVector3) {
          return createLine(entry.start, entry.end);
        }
        if (entry && typeof entry === 'object' && 'line' in entry) {
          const nested = ensureLine(entry.line);
          if (nested) {
            return nested;
          }
        }
      }
      if (input.length === 2) {
        return createLine(ensurePoint(input[0], new THREE.Vector3()), ensurePoint(input[1], new THREE.Vector3(1, 0, 0)));
      }
      if (input.length === 1) {
        return ensureLine(input[0]);
      }
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
        const startPoints = collectPoints(inputs.start);
        const endPoints = collectPoints(inputs.end);

        const fallbackStart = ensurePoint(inputs.start, new THREE.Vector3());
        const fallbackEnd = ensurePoint(
          inputs.end,
          fallbackStart.clone().add(new THREE.Vector3(1, 0, 0)),
        );

        const startList = startPoints.length ? startPoints : [fallbackStart];
        const endList = endPoints.length ? endPoints : [fallbackEnd];
        const count = Math.max(startList.length, endList.length);

        if (count > 1) {
          const lines = [];
          for (let index = 0; index < count; index += 1) {
            const start = startList[index % startList.length];
            const end = endList[index % endList.length];
            lines.push(createLine(start, end));
          }
          return { line: lines };
        }

        const line = createLine(startList[0], endList[0]);
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

export function registerCurveDivisionComponents({ register, toNumber, toVector3 }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register curve division components.');
  }
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register curve division components.');
  }
  if (typeof toVector3 !== 'function') {
    throw new Error('toVector3 function is required to register curve division components.');
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
    if (typeof value === 'object') {
      if ('value' in value) {
        return ensureBoolean(value.value, fallback);
      }
      if ('flag' in value) {
        return ensureBoolean(value.flag, fallback);
      }
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

  function ensurePoint(value, fallback = new THREE.Vector3()) {
    return toVector3(value, fallback.clone());
  }

  function collectNumbers(input) {
    const numbers = [];

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
        if ('number' in value) {
          visit(value.number);
          return;
        }
        if ('numbers' in value) {
          visit(value.numbers);
          return;
        }
      }
      const numeric = ensureNumber(value, Number.NaN);
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
        for (const entry of value) {
          visit(entry);
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
        if ('points' in value) {
          visit(value.points);
          return;
        }
        if ('position' in value) {
          visit(value.position);
          return;
        }
        if ('value' in value) {
          visit(value.value);
          return;
        }
        if ('vertices' in value) {
          visit(value.vertices);
          return;
        }
        if ('x' in value || 'y' in value || 'z' in value) {
          const point = toVector3(value, null);
          if (point) {
            points.push(point);
          }
          return;
        }
      }
    }

    visit(input);
    return points;
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

  function ensurePlane(input) {
    if (input === undefined || input === null) {
      return defaultPlane();
    }
    if (Array.isArray(input)) {
      const points = collectPoints(input);
      if (points.length >= 3) {
        const origin = points[0];
        const xAxis = points[1].clone().sub(points[0]);
        const yAxis = points[2].clone().sub(points[0]);
        const zAxis = xAxis.clone().cross(yAxis);
        if (zAxis.lengthSq() < EPSILON) {
          return defaultPlane();
        }
        return normalizePlaneAxes(origin, xAxis, yAxis, zAxis);
      }
      if (points.length === 2) {
        const origin = points[0];
        const direction = points[1].clone().sub(points[0]);
        const normal = orthogonalVector(direction);
        const yAxis = normal.clone().cross(direction);
        return normalizePlaneAxes(origin, direction, yAxis, normal);
      }
      if (points.length === 1) {
        return normalizePlaneAxes(points[0], new THREE.Vector3(1, 0, 0), new THREE.Vector3(0, 1, 0), new THREE.Vector3(0, 0, 1));
      }
    }
    if (typeof input === 'object') {
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
        const normal = ensurePoint(input.normal, new THREE.Vector3(0, 0, 1)).normalize();
        const xAxis = orthogonalVector(normal);
        const yAxis = normal.clone().cross(xAxis).normalize();
        return normalizePlaneAxes(origin, xAxis, yAxis, normal);
      }
      if (input.point && input.normal) {
        const origin = ensurePoint(input.point, new THREE.Vector3());
        const normal = ensurePoint(input.normal, new THREE.Vector3(0, 0, 1)).normalize();
        const xAxis = orthogonalVector(normal);
        const yAxis = normal.clone().cross(xAxis).normalize();
        return normalizePlaneAxes(origin, xAxis, yAxis, normal);
      }
      if (input.plane) {
        return ensurePlane(input.plane);
      }
    }
    return defaultPlane();
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
    curve.isNativeCurve = true;
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
    if (curve.points && Array.isArray(curve.points)) {
      if (curve.points.length === segments + 1) {
        return curve.points.map((pt) => pt.clone());
      }
      const result = [];
      for (let i = 0; i <= segments; i += 1) {
        const t = i / segments;
        const point = curvePointAt(curve, t);
        if (point) {
          result.push(point);
        }
      }
      return result;
    }
    if (curve.path?.getSpacedPoints) {
      return curve.path.getSpacedPoints(Math.max(segments, 8)).map((pt) => new THREE.Vector3(pt.x, pt.y, pt.z ?? 0));
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

  function ensureCircle(input) {
    if (!input) {
      return null;
    }
    if (input.circle) {
      return ensureCircle(input.circle);
    }
    if (input.type === 'circle') {
      const plane = input.plane
        ? normalizePlaneAxes(input.plane.origin, input.plane.xAxis, input.plane.yAxis, input.plane.zAxis)
        : defaultPlane();
      const center = ensurePoint(input.center, plane.origin.clone());
      const radius = Math.max(ensureNumber(input.radius, 1), EPSILON);
      const segments = input.segments ?? 128;
      return {
        type: 'circle',
        plane,
        center,
        radius,
        shape: input.shape,
        segments,
        curveDefinition: {
          type: 'circle',
          plane,
          center: center.clone(),
          radius,
          startAngle: 0,
          endAngle: Math.PI * 2,
          segments,
        },
      };
    }
    if (input.center && input.radius) {
      const plane = input.plane ? ensurePlane(input.plane) : defaultPlane();
      const center = ensurePoint(input.center, plane.origin.clone());
      const radius = Math.max(ensureNumber(input.radius, 1), EPSILON);
      const segments = input.segments ?? 128;
      return {
        type: 'circle',
        plane,
        center,
        radius,
        shape: input.shape,
        segments,
        curveDefinition: {
          type: 'circle',
          plane,
          center: center.clone(),
          radius,
          startAngle: 0,
          endAngle: Math.PI * 2,
          segments,
        },
      };
    }
    return null;
  }

  function ensureCurve(input) {
    if (!input) {
      return null;
    }
    if (input.isNativeCurve) {
      return input;
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
      if (!circle) {
        return null;
      }
      const path = new THREE.CurvePath();
      path.add(new THREE.EllipseCurve(0, 0, circle.radius, circle.radius, 0, Math.PI * 2, false, 0));
      const curve = createCurveFromPath(path, { segments: circle.segments ?? 128, closed: true, type: 'circle' });
      curve.center = circle.center.clone();
      curve.radius = circle.radius;
      curve.plane = circle.plane;
      curve.shape = circle.shape;
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

  function mapNormalizedParameter(curve, normalized) {
    const domain = curve?.domain;
    if (!domain) {
      return clamp01(normalized);
    }
    return domain.start + (domain.end - domain.start) * clamp01(normalized);
  }

  function buildCurveLengthData(curve, divisions = 256) {
    const segments = Math.max(divisions, 32);
    const samples = [];
    let totalLength = 0;
    let previousPoint = curvePointAt(curve, 0) ?? new THREE.Vector3();
    samples.push({ t: 0, length: 0, point: previousPoint.clone() });
    for (let i = 1; i <= segments; i += 1) {
      const t = i / segments;
      const point = curvePointAt(curve, t) ?? previousPoint.clone();
      totalLength += point.distanceTo(previousPoint);
      samples.push({ t, length: totalLength, point: point.clone() });
      previousPoint = point;
    }
    return { samples, totalLength };
  }

  function parameterAtLength(targetLength, lengthData) {
    const { samples, totalLength } = lengthData;
    if (totalLength <= EPSILON) {
      return 0;
    }
    const clampedLength = clamp(targetLength, 0, totalLength);
    for (let i = 0; i < samples.length - 1; i += 1) {
      const current = samples[i];
      const next = samples[i + 1];
      if (clampedLength >= current.length && clampedLength <= next.length) {
        const span = next.length - current.length;
        const alpha = span > EPSILON ? (clampedLength - current.length) / span : 0;
        return current.t + (next.t - current.t) * alpha;
      }
    }
    return 1;
  }

  function parametersBySegmentCount(curve, segmentCount, lengthData) {
    const data = lengthData ?? buildCurveLengthData(curve, segmentCount * 32);
    const parameters = [];
    const total = data.totalLength;
    if (segmentCount <= 0 || total <= EPSILON) {
      return { parameters: [0], data };
    }
    for (let i = 0; i <= segmentCount; i += 1) {
      const target = (total * i) / segmentCount;
      parameters.push(parameterAtLength(target, data));
    }
    return { parameters, data };
  }

  function createSubCurve(curve, startT, endT, { samples = 64 } = {}) {
    const start = clamp01(Math.min(startT, endT));
    const end = clamp01(Math.max(startT, endT));
    if (end - start <= EPSILON) {
      return null;
    }
    const sampleCount = Math.max(4, Math.round(samples * Math.max(1, (end - start))));
    const points = [];
    for (let i = 0; i <= sampleCount; i += 1) {
      const t = start + (end - start) * (i / sampleCount);
      const point = curvePointAt(curve, t);
      if (point) {
        points.push(point);
      }
    }
    if (points.length < 2) {
      return null;
    }
    return createCurveFromPoints(points, { samples: sampleCount, closed: false });
  }

  function computeSegmentDeviation(curve, t0, t1, samples = 8) {
    const startPoint = curvePointAt(curve, t0);
    const endPoint = curvePointAt(curve, t1);
    if (!startPoint || !endPoint) {
      return 0;
    }
    const chord = endPoint.clone().sub(startPoint);
    const lengthSq = chord.lengthSq();
    if (lengthSq < EPSILON) {
      return 0;
    }
    let maxDeviation = 0;
    for (let i = 1; i < samples; i += 1) {
      const t = t0 + ((t1 - t0) * i) / samples;
      const point = curvePointAt(curve, t);
      if (!point) continue;
      const toPoint = point.clone().sub(startPoint);
      const projection = chord.clone().multiplyScalar(toPoint.dot(chord) / lengthSq);
      const deviation = toPoint.clone().sub(projection).length();
      if (deviation > maxDeviation) {
        maxDeviation = deviation;
      }
    }
    return maxDeviation;
  }

  function createFramePlane(origin, xAxis, yAxis, zAxis) {
    return {
      origin: origin.clone(),
      xAxis: xAxis.clone().normalize(),
      yAxis: yAxis.clone().normalize(),
      zAxis: zAxis.clone().normalize(),
    };
  }

  function computeFrenetFrame(curve, t) {
    const tangent = curveTangentAt(curve, t).normalize();
    const delta = 1e-3;
    const tangentBefore = curveTangentAt(curve, clamp01(t - delta));
    const tangentAfter = curveTangentAt(curve, clamp01(t + delta));
    const derivative = tangentAfter.clone().sub(tangentBefore);
    let normal;
    if (derivative.lengthSq() > EPSILON) {
      normal = derivative.normalize();
    } else {
      normal = orthogonalVector(tangent);
    }
    const binormal = tangent.clone().cross(normal).normalize();
    if (binormal.lengthSq() < EPSILON) {
      const fallbackNormal = orthogonalVector(tangent);
      const fallbackBinormal = tangent.clone().cross(fallbackNormal).normalize();
      return {
        tangent,
        normal: fallbackNormal,
        binormal: fallbackBinormal,
      };
    }
    normal = binormal.clone().cross(tangent).normalize();
    return { tangent, normal, binormal };
  }

  function projectVectorToPlane(vector, normal) {
    const projection = normal.clone().multiplyScalar(vector.dot(normal));
    return vector.clone().sub(projection);
  }

  function computeParallelFrame(curve, t, previousFrame) {
    const tangent = curveTangentAt(curve, t).normalize();
    let normal;
    if (previousFrame) {
      const projected = projectVectorToPlane(previousFrame.normal.clone(), tangent);
      if (projected.lengthSq() > EPSILON) {
        normal = projected.normalize();
      } else {
        normal = orthogonalVector(tangent);
      }
    } else {
      normal = orthogonalVector(tangent);
    }
    const binormal = tangent.clone().cross(normal).normalize();
    if (binormal.lengthSq() < EPSILON) {
      const fallbackNormal = orthogonalVector(tangent);
      const fallbackBinormal = tangent.clone().cross(fallbackNormal).normalize();
      return {
        tangent,
        normal: fallbackNormal,
        binormal: fallbackBinormal,
      };
    }
    normal = binormal.clone().cross(tangent).normalize();
    return { tangent, normal, binormal };
  }

  function computeHorizontalFrame(curve, t, upVector = new THREE.Vector3(0, 0, 1)) {
    const tangent = curveTangentAt(curve, t).normalize();
    const vertical = upVector.clone().normalize();
    let xAxis = tangent.clone().sub(vertical.clone().multiplyScalar(tangent.dot(vertical)));
    if (xAxis.lengthSq() < EPSILON) {
      xAxis = orthogonalVector(vertical);
    }
    xAxis.normalize();
    let yAxis = vertical.clone().cross(xAxis).normalize();
    if (yAxis.lengthSq() < EPSILON) {
      yAxis = orthogonalVector(vertical);
    }
    const zAxis = vertical.clone();
    xAxis = yAxis.clone().cross(zAxis).normalize();
    return { tangent: xAxis.clone(), normal: yAxis.clone(), binormal: zAxis.clone() };
  }

  function refinePlaneIntersection(curve, t0, t1, planeOrigin, planeNormal, iterations = 6) {
    let a = t0;
    let b = t1;
    let fa = planeNormal.dot(curvePointAt(curve, a).clone().sub(planeOrigin));
    let fb = planeNormal.dot(curvePointAt(curve, b).clone().sub(planeOrigin));
    for (let i = 0; i < iterations; i += 1) {
      const mid = 0.5 * (a + b);
      const point = curvePointAt(curve, mid);
      const fm = planeNormal.dot(point.clone().sub(planeOrigin));
      if (Math.abs(fm) < 1e-6) {
        return mid;
      }
      if (fa * fm < 0) {
        b = mid;
        fb = fm;
      } else {
        a = mid;
        fa = fm;
      }
    }
    return 0.5 * (a + b);
  }

  function intersectCurveWithPlane(curve, planeOrigin, planeNormal, segments = 256) {
    const intersections = [];
    let previousT = 0;
    let previousPoint = curvePointAt(curve, 0);
    if (!previousPoint) {
      return intersections;
    }
    let previousValue = planeNormal.dot(previousPoint.clone().sub(planeOrigin));
    for (let i = 1; i <= segments; i += 1) {
      const t = i / segments;
      const point = curvePointAt(curve, t);
      if (!point) continue;
      const value = planeNormal.dot(point.clone().sub(planeOrigin));
      if (Math.abs(value) < 1e-6) {
        const refined = refinePlaneIntersection(curve, previousT, t, planeOrigin, planeNormal);
        if (intersections.length === 0 || Math.abs(refined - intersections[intersections.length - 1].t) > 1e-4) {
          intersections.push({ t: refined, point: curvePointAt(curve, refined) });
        }
      } else if (previousValue * value < 0) {
        const refined = refinePlaneIntersection(curve, previousT, t, planeOrigin, planeNormal);
        if (intersections.length === 0 || Math.abs(refined - intersections[intersections.length - 1].t) > 1e-4) {
          intersections.push({ t: refined, point: curvePointAt(curve, refined) });
        }
      }
      previousT = t;
      previousPoint = point;
      previousValue = value;
    }
    return intersections;
  }

  function registerDivideCurve() {
    register('{2162e72e-72fc-4bf8-9459-d4d82fa8aa14}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', curve: 'curve', Curve: 'curve', N: 'count', Count: 'count', K: 'kinks', kinks: 'kinks' },
        outputs: { P: 'points', Points: 'points', T: 'tangents', t: 'parameters' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const count = Math.max(1, Math.round(ensureNumber(inputs.count, 10)));
        const { parameters } = parametersBySegmentCount(curve, count, null);
        const points = parameters.map((t) => curvePointAt(curve, t));
        const tangents = parameters.map((t) => curveTangentAt(curve, t));
        const mappedParameters = parameters.map((t) => mapNormalizedParameter(curve, t));
        return { points, tangents, parameters: mappedParameters };
      },
    });
  }

  function registerDivideDistance() {
    register('{1e531c08-9c80-46d6-8850-1b50d1dae69f}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', curve: 'curve', Curve: 'curve', D: 'distance', distance: 'distance' },
        outputs: { P: 'points', Points: 'points', T: 'tangents', t: 'parameters' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const distance = Math.max(ensureNumber(inputs.distance, curve.length ? curve.length / 10 : 1), EPSILON);
        const lengthData = buildCurveLengthData(curve, 512);
        const totalLength = lengthData.totalLength;
        if (totalLength <= EPSILON) {
          const point = curvePointAt(curve, 0) ?? new THREE.Vector3();
          const tangent = curveTangentAt(curve, 0);
          return { points: [point], tangents: [tangent], parameters: [mapNormalizedParameter(curve, 0)] };
        }
        const parameters = [0];
        let current = distance;
        while (current < totalLength - EPSILON) {
          parameters.push(parameterAtLength(current, lengthData));
          current += distance;
          if (parameters.length > 1024) {
            break;
          }
        }
        if (parameters[parameters.length - 1] < 1 - 1e-5) {
          parameters.push(1);
        }
        const points = parameters.map((t) => curvePointAt(curve, t));
        const tangents = parameters.map((t) => curveTangentAt(curve, t));
        const mappedParameters = parameters.map((t) => mapNormalizedParameter(curve, t));
        return { points, tangents, parameters: mappedParameters };
      },
    });
  }

  function registerDivideLength() {
    register('{fdc466a9-d3b8-4056-852a-09dba0f74aca}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', curve: 'curve', Curve: 'curve', L: 'length', length: 'length' },
        outputs: { P: 'points', Points: 'points', T: 'tangents', t: 'parameters' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const targetLength = Math.max(ensureNumber(inputs.length, curve.length ? curve.length / 10 : 1), EPSILON);
        const lengthData = buildCurveLengthData(curve, 512);
        const totalLength = lengthData.totalLength;
        if (totalLength <= EPSILON) {
          const point = curvePointAt(curve, 0) ?? new THREE.Vector3();
          const tangent = curveTangentAt(curve, 0);
          return { points: [point], tangents: [tangent], parameters: [mapNormalizedParameter(curve, 0)] };
        }
        const segmentCount = Math.max(1, Math.round(totalLength / targetLength));
        const { parameters } = parametersBySegmentCount(curve, segmentCount, lengthData);
        const points = parameters.map((t) => curvePointAt(curve, t));
        const tangents = parameters.map((t) => curveTangentAt(curve, t));
        const mappedParameters = parameters.map((t) => mapNormalizedParameter(curve, t));
        return { points, tangents, parameters: mappedParameters };
      },
    });
  }

  function registerDivideByDeviation() {
    register('{6e9c0577-ae4a-4b21-8880-0ec3daf3eb4d}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', curve: 'curve', Curve: 'curve', N: 'count', Count: 'count' },
        outputs: { P: 'points', Points: 'points', T: 'tangents', t: 'parameters', d: 'deviation', Deviation: 'deviation' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const count = Math.max(1, Math.round(ensureNumber(inputs.count, 10)));
        const { parameters } = parametersBySegmentCount(curve, count, null);
        const points = parameters.map((t) => curvePointAt(curve, t));
        const tangents = parameters.map((t) => curveTangentAt(curve, t));
        const mappedParameters = parameters.map((t) => mapNormalizedParameter(curve, t));
        const deviation = [];
        for (let i = 0; i < parameters.length - 1; i += 1) {
          deviation.push(computeSegmentDeviation(curve, parameters[i], parameters[i + 1]));
        }
        return { points, tangents, parameters: mappedParameters, deviation };
      },
    });
  }

  function registerCurveFrames() {
    register('{0e94542a-2e46-4793-9f98-2200b06b28f4}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', curve: 'curve', Curve: 'curve', N: 'count', Count: 'count' },
        outputs: { F: 'frames', Frames: 'frames', t: 'parameters', Parameters: 'parameters' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const count = Math.max(1, Math.round(ensureNumber(inputs.count, 10)));
        const { parameters } = parametersBySegmentCount(curve, count, null);
        const frames = [];
        const mappedParameters = [];
        for (const t of parameters) {
          const frame = computeFrenetFrame(curve, t);
          const point = curvePointAt(curve, t) ?? new THREE.Vector3();
          frames.push(createFramePlane(point, frame.tangent, frame.normal, frame.binormal));
          mappedParameters.push(mapNormalizedParameter(curve, t));
        }
        return { frames, parameters: mappedParameters };
      },
    });
  }

  function registerHorizontalFrames() {
    register('{8d058945-ce47-4e7c-82af-3269295d7890}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', curve: 'curve', Curve: 'curve', N: 'count', Count: 'count' },
        outputs: { F: 'frames', Frames: 'frames', t: 'parameters', Parameters: 'parameters' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const count = Math.max(1, Math.round(ensureNumber(inputs.count, 10)));
        const { parameters } = parametersBySegmentCount(curve, count, null);
        const frames = [];
        const mappedParameters = [];
        for (const t of parameters) {
          const frame = computeHorizontalFrame(curve, t, new THREE.Vector3(0, 0, 1));
          const point = curvePointAt(curve, t) ?? new THREE.Vector3();
          frames.push(createFramePlane(point, frame.tangent, frame.normal, frame.binormal));
          mappedParameters.push(mapNormalizedParameter(curve, t));
        }
        return { frames, parameters: mappedParameters };
      },
    });
  }

  function registerPerpFrames() {
    register('{983c7600-980c-44da-bc53-c804067f667f}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', curve: 'curve', Curve: 'curve', N: 'count', Count: 'count', A: 'align', Align: 'align' },
        outputs: { F: 'frames', Frames: 'frames', t: 'parameters', Parameters: 'parameters' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const count = Math.max(1, Math.round(ensureNumber(inputs.count, 10)));
        const align = ensureBoolean(inputs.align, false);
        const { parameters } = parametersBySegmentCount(curve, count, null);
        const frames = [];
        const mappedParameters = [];
        let previousFrame = null;
        for (const t of parameters) {
          let frame;
          if (!previousFrame && align) {
            frame = computeHorizontalFrame(curve, t, new THREE.Vector3(0, 0, 1));
          } else {
            frame = computeParallelFrame(curve, t, previousFrame);
          }
          const point = curvePointAt(curve, t) ?? new THREE.Vector3();
          const plane = createFramePlane(point, frame.tangent, frame.normal, frame.binormal);
          frames.push(plane);
          mappedParameters.push(mapNormalizedParameter(curve, t));
          previousFrame = frame;
        }
        return { frames, parameters: mappedParameters };
      },
    });
  }

  function registerShatter() {
    register('{2ad2a4d4-3de1-42f6-a4b8-f71835f35710}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', curve: 'curve', Curve: 'curve', t: 'parameters', Parameters: 'parameters' },
        outputs: { S: 'segments', Segments: 'segments' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const domain = curve.domain ?? createDomain(0, 1);
        const values = collectNumbers(inputs.parameters).map((value) => {
          const normalized = (value - domain.start) / (domain.end - domain.start || 1);
          return clamp01(normalized);
        });
        const unique = Array.from(new Set([0, ...values, 1])).sort((a, b) => a - b);
        const segments = [];
        for (let i = 0; i < unique.length - 1; i += 1) {
          const start = unique[i];
          const end = unique[i + 1];
          if (end - start <= EPSILON) continue;
          const segment = createSubCurve(curve, start, end, { samples: 64 });
          if (segment) {
            segments.push(segment);
          }
        }
        return { segments };
      },
    });
  }

  function registerDashPattern() {
    register('{95866bbe-648e-4e2b-a97c-7d04679e94e0}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', curve: 'curve', Curve: 'curve', Pt: 'pattern', pattern: 'pattern' },
        outputs: { D: 'dashes', G: 'gaps' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const rawPattern = collectNumbers(inputs.pattern).map((value) => Math.abs(value)).filter((value) => value > EPSILON);
        const pattern = rawPattern.length ? rawPattern : [curve.length || 1];
        const lengthData = buildCurveLengthData(curve, 512);
        const totalLength = lengthData.totalLength;
        if (totalLength <= EPSILON) {
          return { dashes: [], gaps: [] };
        }
        const dashes = [];
        const gaps = [];
        let position = 0;
        let index = 0;
        let dashPhase = true;
        while (position < totalLength - EPSILON && index < 4096) {
          const segmentLength = pattern[index % pattern.length];
          if (segmentLength <= EPSILON) {
            index += 1;
            dashPhase = !dashPhase;
            continue;
          }
          const nextPosition = Math.min(totalLength, position + segmentLength);
          const t0 = parameterAtLength(position, lengthData);
          const t1 = parameterAtLength(nextPosition, lengthData);
          const segment = createSubCurve(curve, t0, t1, { samples: 64 });
          if (segment) {
            if (dashPhase) {
              dashes.push(segment);
            } else {
              gaps.push(segment);
            }
          }
          position = nextPosition;
          index += 1;
          dashPhase = !dashPhase;
        }
        return { dashes, gaps };
      },
    });
  }

  function registerContour() {
    register('{88cff285-7f5e-41b3-96d5-9588ff9a52b1}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', curve: 'curve', Curve: 'curve', P: 'point', Point: 'point', N: 'direction', direction: 'direction', D: 'distance', distance: 'distance' },
        outputs: { C: 'contours', contours: 'contours', t: 'parameters', Parameters: 'parameters' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const basePoint = ensurePoint(inputs.point, curvePointAt(curve, 0) ?? new THREE.Vector3());
        const direction = ensurePoint(inputs.direction, new THREE.Vector3(0, 0, 1));
        const distance = Math.max(ensureNumber(inputs.distance, 1), EPSILON);
        const normal = direction.clone();
        if (normal.lengthSq() < EPSILON) {
          normal.set(0, 0, 1);
        }
        normal.normalize();
        const points = sampleCurvePoints(curve, 256);
        if (!points.length) {
          return {};
        }
        let minOffset = 0;
        let maxOffset = 0;
        for (const point of points) {
          const offset = normal.dot(point.clone().sub(basePoint));
          minOffset = Math.min(minOffset, offset);
          maxOffset = Math.max(maxOffset, offset);
        }
        const offsets = [];
        let current = 0;
        while (current >= minOffset - distance && offsets.length < 256) {
          offsets.unshift(current);
          current -= distance;
        }
        current = distance;
        while (current <= maxOffset + distance && offsets.length < 512) {
          offsets.push(current);
          current += distance;
        }
        const contourPoints = offsets.map(() => []);
        const contourParameters = offsets.map(() => []);
        offsets.forEach((offset, index) => {
          const planeOrigin = basePoint.clone().add(normal.clone().multiplyScalar(offset));
          const intersections = intersectCurveWithPlane(curve, planeOrigin, normal, 512);
          for (const { t, point } of intersections) {
            contourPoints[index].push(point);
            contourParameters[index].push(mapNormalizedParameter(curve, t));
          }
        });
        return { contours: contourPoints, parameters: contourParameters };
      },
    });
  }

  function registerContourEx() {
    register('{3e7e4827-6edd-4e10-93ac-cc234414d2b9}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', curve: 'curve', Curve: 'curve', P: 'plane', plane: 'plane', Plane: 'plane', O: 'offsets', offsets: 'offsets', D: 'distances', distances: 'distances' },
        outputs: { C: 'contours', contours: 'contours', t: 'parameters', Parameters: 'parameters' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const plane = ensurePlane(inputs.plane);
        const offsets = collectNumbers(inputs.offsets);
        const distances = collectNumbers(inputs.distances);
        const offsetValues = (() => {
          if (offsets.length) {
            const unique = Array.from(new Set(offsets.map((value) => ensureNumber(value, 0))));
            unique.sort((a, b) => a - b);
            return unique;
          }
          if (distances.length) {
            const values = [0];
            let cumulative = 0;
            for (const distance of distances) {
              const numeric = Math.abs(ensureNumber(distance, 0));
              if (numeric <= EPSILON) continue;
              cumulative += numeric;
              values.push(cumulative);
              values.push(-cumulative);
            }
            const unique = Array.from(new Set(values));
            unique.sort((a, b) => a - b);
            return unique;
          }
          return [0];
        })();
        const contourPoints = offsetValues.map(() => []);
        const contourParameters = offsetValues.map(() => []);
        offsetValues.forEach((offset, index) => {
          const planeOrigin = plane.origin.clone().add(plane.zAxis.clone().normalize().multiplyScalar(offset));
          const intersections = intersectCurveWithPlane(curve, planeOrigin, plane.zAxis.clone().normalize(), 512);
          for (const { t, point } of intersections) {
            contourPoints[index].push(point);
            contourParameters[index].push(mapNormalizedParameter(curve, t));
          }
        });
        return { contours: contourPoints, parameters: contourParameters };
      },
    });
  }

  registerDivideCurve();
  registerDivideDistance();
  registerDivideLength();
  registerDivideByDeviation();
  registerCurveFrames();
  registerHorizontalFrames();
  registerPerpFrames();
  registerShatter();
  registerDashPattern();
  registerContour();
  registerContourEx();
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
    curve.isNativeCurve = true;
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
    if (curve.path?.getSpacedPoints) {
      return curve.path.getSpacedPoints(Math.max(segments, 8)).map((pt) => new THREE.Vector3(pt.x, pt.y, pt.z ?? 0));
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
    if (input.isNativeCurve) {
      return input;
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

export function registerCurveAnalysisComponents({ register, toNumber, toVector3 }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register curve analysis components.');
  }
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register curve analysis components.');
  }
  if (typeof toVector3 !== 'function') {
    throw new Error('toVector3 function is required to register curve analysis components.');
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

  function ensureArray(input) {
    if (input === undefined || input === null) {
      return [];
    }
    return Array.isArray(input) ? input : [input];
  }

  function ensurePoint(value, fallback = new THREE.Vector3()) {
    return toVector3(value, fallback.clone ? fallback.clone() : fallback);
  }

  function ensureBoolean(value, fallback = false) {
    if (value === undefined || value === null) {
      return fallback;
    }
    if (typeof value === 'boolean') {
      return value;
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
      const numeric = Number(normalized);
      if (Number.isFinite(numeric)) {
        return numeric !== 0;
      }
      return fallback;
    }
    if (Array.isArray(value)) {
      if (!value.length) return fallback;
      return ensureBoolean(value[value.length - 1], fallback);
    }
    if (typeof value === 'object') {
      if ('value' in value) {
        return ensureBoolean(value.value, fallback);
      }
      if ('flag' in value) {
        return ensureBoolean(value.flag, fallback);
      }
    }
    return Boolean(value);
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
        if ('vertices' in value) {
          visit(value.vertices);
          return;
        }
        if ('value' in value) {
          visit(value.value);
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

  function orthogonalVector(vector) {
    const absX = Math.abs(vector.x);
    const absY = Math.abs(vector.y);
    const absZ = Math.abs(vector.z);
    if (absX <= absY && absX <= absZ) {
      const result = new THREE.Vector3(0, -vector.z, vector.y);
      return result.lengthSq() < EPSILON ? new THREE.Vector3(0, 1, 0) : result.normalize();
    }
    if (absY <= absX && absY <= absZ) {
      const result = new THREE.Vector3(-vector.z, 0, vector.x);
      return result.lengthSq() < EPSILON ? new THREE.Vector3(1, 0, 0) : result.normalize();
    }
    const result = new THREE.Vector3(-vector.y, vector.x, 0);
    return result.lengthSq() < EPSILON ? new THREE.Vector3(1, 0, 0) : result.normalize();
  }

  function normalizeVector(vector, fallback = new THREE.Vector3(1, 0, 0)) {
    const result = vector.clone();
    if (result.lengthSq() < EPSILON) {
      return fallback.clone();
    }
    return result.normalize();
  }

  function defaultPlane() {
    return {
      origin: new THREE.Vector3(0, 0, 0),
      xAxis: new THREE.Vector3(1, 0, 0),
      yAxis: new THREE.Vector3(0, 1, 0),
      zAxis: new THREE.Vector3(0, 0, 1),
    };
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

  function createPlane(origin, xAxis, yAxis, normal) {
    const zAxis = normal ? normalizeVector(normal, new THREE.Vector3(0, 0, 1)) : xAxis.clone().cross(yAxis).normalize();
    return normalizePlaneAxes(origin, xAxis, yAxis, zAxis);
  }

  function planeFromPoints(a, b, c) {
    const ab = b.clone().sub(a);
    const ac = c.clone().sub(a);
    const normal = ab.clone().cross(ac);
    if (normal.lengthSq() < EPSILON) {
      return null;
    }
    const xAxis = ab.lengthSq() < EPSILON ? orthogonalVector(normal) : ab.clone().normalize();
    const yAxis = normal.clone().cross(xAxis).normalize();
    return createPlane(a.clone(), xAxis, yAxis, normal);
  }

  function ensurePlane(input) {
    if (!input) {
      return defaultPlane();
    }
    if (Array.isArray(input)) {
      const points = collectPoints(input);
      if (points.length >= 3) {
        const plane = planeFromPoints(points[0], points[1], points[2]);
        return plane ?? defaultPlane();
      }
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
      return createPlane(origin, xAxis, yAxis, normal);
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
    return defaultPlane();
  }

  function planeCoordinates(point, plane) {
    const relative = point.clone().sub(plane.origin);
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
    const domain = createDomain(0, 1);
    const curve = {
      type,
      path,
      points,
      segments: safeSegments,
      length,
      closed,
      domain,
    };
    curve.isNativeCurve = true;
    curve.getPointAt = (t) => {
      const clamped = clamp01(t);
      if (typeof path.getPointAt === 'function') {
        const pt = path.getPointAt(clamped);
        return new THREE.Vector3(pt.x, pt.y, pt.z ?? 0);
      }
      if (typeof path.getPoint === 'function') {
        const pt = path.getPoint(clamped);
        return new THREE.Vector3(pt.x, pt.y, pt.z ?? 0);
      }
      const index = Math.floor(clamped * (points.length - 1));
      const alpha = clamped * (points.length - 1) - index;
      const current = points[index];
      const next = points[Math.min(index + 1, points.length - 1)];
      return current.clone().lerp(next, alpha);
    };
    curve.getTangentAt = (t) => {
      if (typeof path.getTangentAt === 'function') {
        const tangent = path.getTangentAt(clamp01(t));
        return new THREE.Vector3(tangent.x, tangent.y, tangent.z ?? 0).normalize();
      }
      const delta = 1e-3;
      const p0 = curve.getPointAt(clamp01(t - delta));
      const p1 = curve.getPointAt(clamp01(t + delta));
      const tangent = p1.clone().sub(p0);
      if (tangent.lengthSq() < EPSILON) {
        return new THREE.Vector3(1, 0, 0);
      }
      return tangent.normalize();
    };
    return curve;
  }

  function createCurveFromPoints(pointsInput, { closed = false, curveType = 'centripetal', tension = 0.5, samples } = {}) {
    const points = collectPoints(pointsInput);
    if (!points.length) {
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

  function sampleCurvePoints(curve, segments = 64) {
    if (!curve) {
      return [];
    }
    if (curve.points && Array.isArray(curve.points)) {
      return curve.points.map((pt) => pt.clone());
    }
    if (curve.path?.getSpacedPoints) {
      return curve.path.getSpacedPoints(Math.max(segments, 8)).map((pt) => new THREE.Vector3(pt.x, pt.y, pt.z ?? 0));
    }
    const points = [];
    for (let i = 0; i <= segments; i += 1) {
      const t = i / segments;
      const point = curvePointAt(curve, t);
      if (point) {
        points.push(point);
      }
    }
    return points;
  }

  function ensureCurve(input) {
    if (!input) {
      return null;
    }
    if (input.isNativeCurve) {
      return input;
    }
    if (Array.isArray(input)) {
      if (!input.length) {
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
      const center = ensurePoint(input.center, new THREE.Vector3());
      const radius = Math.max(ensureNumber(input.radius, 1), EPSILON);
      const plane = input.plane ? ensurePlane(input.plane) : defaultPlane();
      const startAngle = input.startAngle ?? 0;
      const endAngle = input.endAngle ?? Math.PI * 2;
      const path = new THREE.Path();
      path.absarc(0, 0, radius, startAngle, endAngle, endAngle < startAngle);
      const curve = createCurveFromPath(path, { segments: input.segments ?? 128, closed: Math.abs(endAngle - startAngle) >= Math.PI * 2 - 1e-6, type: 'arc' });
      curve.center = center.clone();
      curve.radius = radius;
      curve.plane = plane;
      curve.startAngle = startAngle;
      curve.endAngle = endAngle;
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

  function curvePointAt(curve, t) {
    if (!curve) {
      return null;
    }
    if (typeof curve.getPointAt === 'function') {
      return curve.getPointAt(clamp01(t));
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
      return curve.getTangentAt(clamp01(t)).clone().normalize();
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

  function resolveCurveDomain(curve) {
    if (curve?.domain) {
      return curve.domain;
    }
    return createDomain(0, 1);
  }

  function parameterToNormalized(curve, parameter) {
    const domain = resolveCurveDomain(curve);
    if (!domain) {
      return clamp01(ensureNumber(parameter, 0));
    }
    const start = domain.start ?? 0;
    const end = domain.end ?? 1;
    const span = end - start;
    if (Math.abs(span) < EPSILON) {
      return 0;
    }
    const value = ensureNumber(parameter, start);
    return clamp01((value - start) / span);
  }

  function normalizedToParameter(curve, normalized) {
    const domain = resolveCurveDomain(curve);
    if (!domain) {
      return clamp01(normalized);
    }
    return domain.start + (domain.end - domain.start) * clamp01(normalized);
  }

  function resolveParameter(curve, input) {
    if (input === undefined || input === null) {
      return 0;
    }
    if (typeof input === 'object' && !Array.isArray(input)) {
      if ('t' in input) {
        return parameterToNormalized(curve, input.t);
      }
      if ('parameter' in input) {
        return parameterToNormalized(curve, input.parameter);
      }
      if ('value' in input) {
        return parameterToNormalized(curve, input.value);
      }
    }
    return parameterToNormalized(curve, input);
  }

  function resolveDomainInput(curve, domainInput) {
    const domain = resolveCurveDomain(curve);
    if (!domainInput) {
      return {
        normalized: { start: 0, end: 1 },
        domain,
      };
    }
    let startValue = domain.start;
    let endValue = domain.end;
    if (Array.isArray(domainInput)) {
      if (domainInput.length >= 2) {
        startValue = ensureNumber(domainInput[0], domain.start);
        endValue = ensureNumber(domainInput[1], domain.end);
      } else if (domainInput.length === 1) {
        endValue = ensureNumber(domainInput[0], domain.end);
      }
    } else if (typeof domainInput === 'object') {
      if ('start' in domainInput || 'end' in domainInput) {
        startValue = ensureNumber(domainInput.start, domain.start);
        endValue = ensureNumber(domainInput.end, domain.end);
      } else if ('min' in domainInput || 'max' in domainInput) {
        startValue = ensureNumber(domainInput.min, domain.start);
        endValue = ensureNumber(domainInput.max, domain.end);
      } else if ('t0' in domainInput || 't1' in domainInput) {
        startValue = ensureNumber(domainInput.t0, domain.start);
        endValue = ensureNumber(domainInput.t1, domain.end);
      } else if ('a' in domainInput || 'b' in domainInput) {
        startValue = ensureNumber(domainInput.a, domain.start);
        endValue = ensureNumber(domainInput.b, domain.end);
      } else if ('domain' in domainInput) {
        return resolveDomainInput(curve, domainInput.domain);
      }
    } else if (typeof domainInput === 'number') {
      endValue = ensureNumber(domainInput, domain.end);
    }
    const normalizedStart = parameterToNormalized(curve, startValue);
    const normalizedEnd = parameterToNormalized(curve, endValue);
    const resolvedDomain = createDomain(startValue, endValue);
    return {
      normalized: {
        start: normalizedStart,
        end: normalizedEnd,
      },
      domain: resolvedDomain,
    };
  }

  function buildCurveLengthData(curve, divisions = 256) {
    const segments = Math.max(divisions, 32);
    const samples = [];
    let totalLength = 0;
    let previousPoint = curvePointAt(curve, 0) ?? new THREE.Vector3();
    samples.push({ t: 0, length: 0, point: previousPoint.clone() });
    for (let i = 1; i <= segments; i += 1) {
      const t = i / segments;
      const point = curvePointAt(curve, t) ?? previousPoint.clone();
      totalLength += point.distanceTo(previousPoint);
      samples.push({ t, length: totalLength, point: point.clone() });
      previousPoint = point;
    }
    return { samples, totalLength };
  }

  function lengthAtNormalized(data, normalized) {
    const clamped = clamp01(normalized);
    if (!data || !data.samples.length) {
      return clamped * (data?.totalLength ?? 0);
    }
    const samples = data.samples;
    for (let i = 0; i < samples.length - 1; i += 1) {
      const current = samples[i];
      const next = samples[i + 1];
      if (clamped >= current.t && clamped <= next.t) {
        const span = next.t - current.t;
        const alpha = span > EPSILON ? (clamped - current.t) / span : 0;
        return current.length + (next.length - current.length) * alpha;
      }
    }
    return data.totalLength;
  }

  function lengthBetweenNormalized(curve, start, end, lengthData) {
    const data = lengthData ?? buildCurveLengthData(curve, 256);
    const startLength = lengthAtNormalized(data, start);
    const endLength = lengthAtNormalized(data, end);
    return Math.abs(endLength - startLength);
  }

  function parameterAtLength(targetLength, data) {
    const { samples, totalLength } = data;
    if (totalLength <= EPSILON) {
      return 0;
    }
    const clampedLength = clamp(targetLength, 0, totalLength);
    for (let i = 0; i < samples.length - 1; i += 1) {
      const current = samples[i];
      const next = samples[i + 1];
      if (clampedLength >= current.length && clampedLength <= next.length) {
        const span = next.length - current.length;
        const alpha = span > EPSILON ? (clampedLength - current.length) / span : 0;
        return current.t + (next.t - current.t) * alpha;
      }
    }
    return 1;
  }

  function computeCurvePlane(curve) {
    if (!curve) {
      return defaultPlane();
    }
    if (curve.plane) {
      return ensurePlane(curve.plane);
    }
    const samples = sampleCurvePoints(curve, 32);
    if (samples.length < 3) {
      if (samples.length === 2) {
        const origin = samples[0];
        const direction = samples[1].clone().sub(samples[0]);
        const normal = orthogonalVector(direction);
        const yAxis = normal.clone().cross(direction).normalize();
        return createPlane(origin.clone(), direction.clone().normalize(), yAxis, normal);
      }
      return defaultPlane();
    }
    for (let i = 0; i < samples.length - 2; i += 1) {
      const plane = planeFromPoints(samples[i], samples[i + 1], samples[i + 2]);
      if (plane) {
        return plane;
      }
    }
    const origin = samples.reduce((sum, point) => sum.add(point), new THREE.Vector3()).multiplyScalar(1 / samples.length);
    let covarianceXX = 0;
    let covarianceXY = 0;
    let covarianceXZ = 0;
    let covarianceYY = 0;
    let covarianceYZ = 0;
    let covarianceZZ = 0;
    for (const point of samples) {
      const relative = point.clone().sub(origin);
      covarianceXX += relative.x * relative.x;
      covarianceXY += relative.x * relative.y;
      covarianceXZ += relative.x * relative.z;
      covarianceYY += relative.y * relative.y;
      covarianceYZ += relative.y * relative.z;
      covarianceZZ += relative.z * relative.z;
    }
    const covarianceMatrix = new THREE.Matrix3();
    covarianceMatrix.set(
      covarianceXX, covarianceXY, covarianceXZ,
      covarianceXY, covarianceYY, covarianceYZ,
      covarianceXZ, covarianceYZ, covarianceZZ,
    );
    const candidates = [
      new THREE.Vector3(1, 0, 0),
      new THREE.Vector3(0, 1, 0),
      new THREE.Vector3(0, 0, 1),
      new THREE.Vector3(1, 1, 0),
      new THREE.Vector3(1, 0, 1),
      new THREE.Vector3(0, 1, 1),
    ];
    let bestVector = candidates[0].clone();
    let bestLength = Infinity;
    for (const candidate of candidates) {
      const transformed = candidate.clone().applyMatrix3(covarianceMatrix);
      const length = transformed.lengthSq();
      if (length < bestLength) {
        bestLength = length;
        bestVector = candidate.clone();
      }
    }
    const normalVector = bestVector.clone().normalize();
    const xAxis = orthogonalVector(normalVector);
    const yAxis = normalVector.clone().cross(xAxis).normalize();
    return createPlane(origin.clone(), xAxis, yAxis, normalVector);
  }

  function projectPointToPlane(point, plane) {
    const coordinates = planeCoordinates(point, plane);
    return applyPlane(plane, coordinates.x, coordinates.y, 0);
  }

  function pointOnCurveEdge(point2D, polygon) {
    for (let i = 0; i < polygon.length - 1; i += 1) {
      const a = polygon[i];
      const b = polygon[i + 1];
      const edge = new THREE.Vector2(b.x - a.x, b.y - a.y);
      const toPoint = new THREE.Vector2(point2D.x - a.x, point2D.y - a.y);
      const edgeLengthSq = edge.lengthSq();
      if (edgeLengthSq < EPSILON) {
        continue;
      }
      const projection = (toPoint.x * edge.x + toPoint.y * edge.y) / edgeLengthSq;
      if (projection >= -1e-6 && projection <= 1 + 1e-6) {
        const closestX = a.x + edge.x * projection;
        const closestY = a.y + edge.y * projection;
        const distanceSq = (closestX - point2D.x) ** 2 + (closestY - point2D.y) ** 2;
        if (distanceSq <= 1e-8) {
          return true;
        }
      }
    }
    return false;
  }

  function pointInPolygon2D(point2D, polygon) {
    if (polygon.length < 3) {
      return 0;
    }
    let inside = false;
    for (let i = 0, j = polygon.length - 1; i < polygon.length; j = i, i += 1) {
      const pi = polygon[i];
      const pj = polygon[j];
      const intersects = ((pi.y > point2D.y) !== (pj.y > point2D.y)) &&
        (point2D.x < ((pj.x - pi.x) * (point2D.y - pi.y)) / ((pj.y - pi.y) || EPSILON) + pi.x);
      if (intersects) {
        inside = !inside;
      }
    }
    return inside ? 2 : 0;
  }

  function evaluateContainment(pointInput, curvesInput) {
    const point = ensurePoint(pointInput, new THREE.Vector3());
    const curveInputs = ensureArray(curvesInput);
    const curves = curveInputs.map((entry) => ensureCurve(entry)).filter(Boolean);
    let relationship = 0;
    let index = -1;
    let projectedPoint = point.clone();
    for (let i = 0; i < curves.length; i += 1) {
      const curve = curves[i];
      const plane = computeCurvePlane(curve);
      const projection = projectPointToPlane(point, plane);
      const polygonPoints = sampleCurvePoints(curve, 128).map((pt) => planeCoordinates(pt, plane));
      if (polygonPoints.length && (polygonPoints[0].x !== polygonPoints[polygonPoints.length - 1].x || polygonPoints[0].y !== polygonPoints[polygonPoints.length - 1].y)) {
        polygonPoints.push(polygonPoints[0]);
      }
      const coordinates = planeCoordinates(projection, plane);
      if (pointOnCurveEdge(coordinates, polygonPoints)) {
        relationship = 1;
        index = i;
        projectedPoint = projection.clone();
        break;
      }
      const inside = pointInPolygon2D(coordinates, polygonPoints);
      if (inside === 2) {
        relationship = 2;
        index = i;
        projectedPoint = projection.clone();
        break;
      }
    }
    return { relationship, index, projectedPoint };
  }

  function closestPointOnCurve(point, curve, { segments = 256, refinementSteps = 5 } = {}) {
    const target = ensurePoint(point, new THREE.Vector3());
    let bestT = 0;
    let bestPoint = curvePointAt(curve, 0) ?? new THREE.Vector3();
    let bestDistanceSq = bestPoint.distanceToSquared(target);
    for (let i = 1; i <= segments; i += 1) {
      const t = i / segments;
      const sample = curvePointAt(curve, t);
      if (!sample) continue;
      const distanceSq = sample.distanceToSquared(target);
      if (distanceSq < bestDistanceSq) {
        bestDistanceSq = distanceSq;
        bestT = t;
        bestPoint = sample;
      }
    }
    for (let step = 0; step < refinementSteps; step += 1) {
      const delta = 1 / (segments * Math.pow(2, step + 1));
      const candidates = [bestT - delta, bestT + delta];
      for (const candidate of candidates) {
        const clamped = clamp01(candidate);
        const sample = curvePointAt(curve, clamped);
        if (!sample) continue;
        const distanceSq = sample.distanceToSquared(target);
        if (distanceSq < bestDistanceSq - 1e-12) {
          bestDistanceSq = distanceSq;
          bestT = clamped;
          bestPoint = sample;
        }
      }
    }
    return {
      parameter: bestT,
      point: bestPoint.clone(),
      distance: Math.sqrt(Math.max(bestDistanceSq, 0)),
      distanceSq: bestDistanceSq,
    };
  }

  function computeParallelFrame(curve, t, previousFrame) {
    const tangent = curveTangentAt(curve, t).normalize();
    let normal;
    if (previousFrame) {
      const projected = previousFrame.normal.clone().sub(tangent.clone().multiplyScalar(previousFrame.normal.clone().dot(tangent)));
      if (projected.lengthSq() > EPSILON) {
        normal = projected.normalize();
      } else {
        normal = orthogonalVector(tangent);
      }
    } else {
      normal = orthogonalVector(tangent);
    }
    const binormal = tangent.clone().cross(normal).normalize();
    if (binormal.lengthSq() < EPSILON) {
      const fallbackNormal = orthogonalVector(tangent);
      const fallbackBinormal = tangent.clone().cross(fallbackNormal).normalize();
      return {
        tangent,
        normal: fallbackNormal,
        binormal: fallbackBinormal,
      };
    }
    normal = binormal.clone().cross(tangent).normalize();
    return { tangent, normal, binormal };
  }

  function computeFrenetFrame(curve, t) {
    const tangent = curveTangentAt(curve, t).normalize();
    const delta = 1e-3;
    const tangentBefore = curveTangentAt(curve, clamp01(t - delta));
    const tangentAfter = curveTangentAt(curve, clamp01(t + delta));
    const derivative = tangentAfter.clone().sub(tangentBefore);
    let normal;
    if (derivative.lengthSq() > EPSILON) {
      normal = derivative.normalize();
    } else {
      normal = orthogonalVector(tangent);
    }
    const binormal = tangent.clone().cross(normal).normalize();
    if (binormal.lengthSq() < EPSILON) {
      const fallbackNormal = orthogonalVector(tangent);
      const fallbackBinormal = tangent.clone().cross(fallbackNormal).normalize();
      return {
        tangent,
        normal: fallbackNormal,
        binormal: fallbackBinormal,
      };
    }
    normal = binormal.clone().cross(tangent).normalize();
    return { tangent, normal, binormal };
  }

  function computeHorizontalFrame(curve, t, upVector = new THREE.Vector3(0, 0, 1)) {
    const tangent = curveTangentAt(curve, t).normalize();
    const vertical = upVector.clone().normalize();
    let xAxis = tangent.clone().sub(vertical.clone().multiplyScalar(tangent.dot(vertical)));
    if (xAxis.lengthSq() < EPSILON) {
      xAxis = orthogonalVector(vertical);
    }
    xAxis.normalize();
    let yAxis = vertical.clone().cross(xAxis).normalize();
    if (yAxis.lengthSq() < EPSILON) {
      yAxis = orthogonalVector(vertical);
    }
    const zAxis = vertical.clone();
    xAxis = yAxis.clone().cross(zAxis).normalize();
    return { tangent: xAxis.clone(), normal: yAxis.clone(), binormal: zAxis.clone() };
  }

  function computeCurveDerivatives(curve, t, order = 1) {
    const derivatives = [];
    const clamped = clamp01(t);
    const safeDelta = Math.min(1e-3, Math.max(1e-4, Math.min(clamped, 1 - clamped, 0.1)) || 1e-3);
    const p0 = curvePointAt(curve, clamped) ?? new THREE.Vector3();
    const pPrev = curvePointAt(curve, clamp01(clamped - safeDelta)) ?? p0.clone();
    const pNext = curvePointAt(curve, clamp01(clamped + safeDelta)) ?? p0.clone();
    const first = pNext.clone().sub(pPrev).multiplyScalar(1 / (2 * safeDelta));
    derivatives.push(first.clone());
    if (order >= 2) {
      const second = pNext.clone().add(pPrev).add(p0.clone().multiplyScalar(-2)).multiplyScalar(1 / (safeDelta * safeDelta));
      derivatives.push(second.clone());
    }
    if (order >= 3) {
      const pPrev2 = curvePointAt(curve, clamp01(clamped - 2 * safeDelta)) ?? pPrev.clone();
      const pNext2 = curvePointAt(curve, clamp01(clamped + 2 * safeDelta)) ?? pNext.clone();
      const third = pPrev2.clone().sub(pPrev.clone().multiplyScalar(2)).add(pNext.clone().multiplyScalar(2)).sub(pNext2).multiplyScalar(1 / (2 * Math.pow(safeDelta, 3)));
      derivatives.push(third.clone());
    }
    return derivatives;
  }

  function computeCurvature(curve, t) {
    const clamped = clamp01(t);
    const delta = Math.min(1e-3, Math.max(1e-4, Math.min(clamped, 1 - clamped, 0.1)) || 1e-3);
    const tangent = curveTangentAt(curve, clamped).normalize();
    const tangentBefore = curveTangentAt(curve, clamp01(clamped - delta)).normalize();
    const tangentAfter = curveTangentAt(curve, clamp01(clamped + delta)).normalize();
    const dT = tangentAfter.clone().sub(tangentBefore).multiplyScalar(1 / (2 * delta));
    const derivatives = computeCurveDerivatives(curve, clamped, 2);
    const velocity = derivatives[0];
    const speed = velocity.length();
    if (speed < EPSILON) {
      return { curvature: 0, radius: Infinity, normal: orthogonalVector(tangent), center: null, point: curvePointAt(curve, clamped) };
    }
    const curvatureVector = dT.clone().divideScalar(speed);
    const curvature = curvatureVector.length();
    const normal = curvature > EPSILON ? curvatureVector.clone().normalize() : orthogonalVector(tangent);
    const radius = curvature > EPSILON ? 1 / curvature : Infinity;
    const point = curvePointAt(curve, clamped) ?? new THREE.Vector3();
    const center = curvature > EPSILON ? point.clone().add(normal.clone().multiplyScalar(radius)) : null;
    return { curvature, radius, normal, center, point };
  }

  function computeTorsion(curve, t) {
    const derivatives = computeCurveDerivatives(curve, t, 3);
    if (derivatives.length < 3) {
      return 0;
    }
    const d1 = derivatives[0];
    const d2 = derivatives[1];
    const d3 = derivatives[2];
    const cross = d1.clone().cross(d2);
    const denominator = cross.lengthSq();
    if (denominator < EPSILON) {
      return 0;
    }
    const numerator = cross.dot(d3);
    return numerator / denominator;
  }

  function extractCurvePoints(curve) {
    if (!curve) {
      return [];
    }
    if (Array.isArray(curve.controlPoints)) {
      return curve.controlPoints.map((pt) => ensurePoint(pt, new THREE.Vector3()));
    }
    if (curve.points?.length) {
      return curve.points.map((pt) => pt.clone());
    }
    return sampleCurvePoints(curve, 64);
  }

  function createPolyline(points, { closed = false } = {}) {
    if (!points.length) {
      return null;
    }
    const path = new THREE.CurvePath();
    for (let i = 0; i < points.length - 1; i += 1) {
      path.add(new THREE.LineCurve3(points[i].clone(), points[i + 1].clone()));
    }
    if (closed && points.length > 2) {
      path.add(new THREE.LineCurve3(points[points.length - 1].clone(), points[0].clone()));
    }
    const curve = createCurveFromPath(path, { segments: Math.max(points.length * 2, 16), closed });
    if (curve) {
      curve.type = 'polyline';
      curve.points = points.map((pt) => pt.clone());
      curve.closed = closed;
    }
    return curve;
  }

  function polygonVertexAverage(points) {
    if (!points.length) {
      return new THREE.Vector3();
    }
    const total = points.reduce((sum, pt) => sum.add(pt), new THREE.Vector3());
    return total.multiplyScalar(1 / points.length);
  }

  function polygonEdgeAverage(points) {
    if (points.length < 2) {
      return polygonVertexAverage(points);
    }
    let totalLength = 0;
    const accumulator = new THREE.Vector3();
    const count = points.length;
    for (let i = 0; i < count; i += 1) {
      const a = points[i];
      const b = points[(i + 1) % count];
      const length = a.distanceTo(b);
      if (length <= EPSILON) {
        continue;
      }
      totalLength += length;
      const midpoint = a.clone().add(b).multiplyScalar(0.5);
      accumulator.add(midpoint.multiplyScalar(length));
    }
    if (totalLength < EPSILON) {
      return polygonVertexAverage(points);
    }
    return accumulator.multiplyScalar(1 / totalLength);
  }

  function polygonAreaCentroid(points) {
    if (points.length < 3) {
      return polygonVertexAverage(points);
    }
    const plane = computeCurvePlane({ points, closed: true });
    const coords = points.map((pt) => planeCoordinates(pt, plane));
    if (coords.length && (coords[0].x !== coords[coords.length - 1].x || coords[0].y !== coords[coords.length - 1].y)) {
      coords.push(coords[0]);
    }
    let area = 0;
    let centroidX = 0;
    let centroidY = 0;
    for (let i = 0; i < coords.length - 1; i += 1) {
      const a = coords[i];
      const b = coords[i + 1];
      const cross = a.x * b.y - b.x * a.y;
      area += cross;
      centroidX += (a.x + b.x) * cross;
      centroidY += (a.y + b.y) * cross;
    }
    area *= 0.5;
    if (Math.abs(area) < EPSILON) {
      return polygonVertexAverage(points);
    }
    centroidX /= (6 * area);
    centroidY /= (6 * area);
    return applyPlane(plane, centroidX, centroidY, 0);
  }

  function curveCurveProximity(curveA, curveB) {
    if (!curveA || !curveB) {
      return null;
    }
    let best = null;
    const segments = 128;
    for (let i = 0; i <= segments; i += 1) {
      const tA = i / segments;
      const pointA = curvePointAt(curveA, tA);
      if (!pointA) continue;
      const closestB = closestPointOnCurve(pointA, curveB, { segments: 128, refinementSteps: 4 });
      if (!closestB) continue;
      if (!best || closestB.distanceSq < best.distanceSq) {
        best = {
          pointA: pointA.clone(),
          parameterA: tA,
          pointB: closestB.point.clone(),
          parameterB: closestB.parameter,
          distanceSq: closestB.distanceSq,
        };
      }
    }
    return best;
  }

  function collectGeometries(input) {
    if (!input) {
      return [];
    }
    if (Array.isArray(input)) {
      return input.flatMap((entry) => collectGeometries(entry));
    }
    if (typeof input === 'object' && 'value' in input) {
      return collectGeometries(input.value);
    }
    return [input];
  }

  function registerPointInCurves() {
    register('{0b04e8b9-00d7-47a7-95c3-0d51e654fe88}', {
      type: 'curve',
      pinMap: {
        inputs: { P: 'point', point: 'point', C: 'curves', Curves: 'curves' },
        outputs: { R: 'relationship', I: 'index', "P'": 'projectedPoint' },
      },
      eval: ({ inputs }) => {
        const { relationship, index, projectedPoint } = evaluateContainment(inputs.point, inputs.curves);
        return { relationship, index, projectedPoint };
      },
    });
  }

  function registerEndPoints() {
    register('{11bbd48b-bb0a-4f1b-8167-fa297590390d}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', Curve: 'curve' },
        outputs: { S: 'start', E: 'end' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const start = curvePointAt(curve, 0);
        const end = curvePointAt(curve, 1);
        return { start, end };
      },
    });
  }

  function registerCurveDomainObsolete() {
    register('{15ac45a8-b190-420a-bd66-e78ed6bcfaa4}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve' },
        outputs: { D: 'domain' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        return { domain: resolveCurveDomain(curve) };
      },
    });
  }

  function registerEvaluateCurveSimple() {
    register('{164d0429-e5f5-4292-aa80-3f88d43cdac2}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', curve: 'curve', t: 'parameter', Parameter: 'parameter' },
        outputs: { P: 'point', T: 'tangent' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const parameter = resolveParameter(curve, inputs.parameter ?? inputs.t);
        const point = curvePointAt(curve, parameter);
        const tangent = curveTangentAt(curve, parameter);
        return { point, tangent };
      },
    });
  }

  function registerLengthDomain() {
    register('{188edd02-14a9-4828-a521-34995b0d1e4a}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', curve: 'curve', D: 'domain', Domain: 'domain' },
        outputs: { L: 'length' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const { normalized } = resolveDomainInput(curve, inputs.domain ?? inputs.D);
        const data = buildCurveLengthData(curve, 256);
        const length = lengthBetweenNormalized(curve, normalized.start, normalized.end, data);
        return { length };
      },
    });
  }

  function registerDeconstructArc() {
    register('{23862862-049a-40be-b558-2418aacbd916}', {
      type: 'curve',
      pinMap: {
        inputs: { A: 'arc', arc: 'arc' },
        outputs: { B: 'basePlane', R: 'radius', A: 'angleDomain' },
      },
      eval: ({ inputs }) => {
        const arcInput = inputs.arc ?? inputs.A;
        if (!arcInput) {
          return {};
        }
        const arc = ensureCurve(arcInput);
        const basePlane = arc?.plane ? ensurePlane(arc.plane) : defaultPlane();
        const radius = arc?.radius ?? ensureNumber(arcInput.radius, 0);
        const startAngle = arc?.startAngle ?? 0;
        const endAngle = arc?.endAngle ?? Math.PI * 2;
        return { basePlane, radius, angleDomain: [startAngle, endAngle] };
      },
    });
  }

  function registerDiscontinuity() {
    register('{269eaa85-9997-4d77-a9ba-4c58cb45c9d3}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', L: 'level', level: 'level' },
        outputs: { P: 'points', t: 'parameters' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const level = Math.max(0, Math.min(2, Math.round(ensureNumber(inputs.level, 1))));
        const segments = 256;
        const points = [];
        const parameters = [];
        let previousTangent = curveTangentAt(curve, 0);
        let previousSecond = computeCurveDerivatives(curve, 0, 2)[1];
        for (let i = 1; i <= segments; i += 1) {
          const t = i / segments;
          const tangent = curveTangentAt(curve, t);
          const derivatives = computeCurveDerivatives(curve, t, 2);
          const angle = previousTangent.angleTo(tangent);
          const second = derivatives[1];
          const secondChange = previousSecond ? previousSecond.clone().sub(second).length() : 0;
          let isDiscontinuity = false;
          if (level === 0) {
            isDiscontinuity = angle > Math.PI / 18;
          } else if (level === 1) {
            isDiscontinuity = angle > Math.PI / 36;
          } else {
            isDiscontinuity = angle > Math.PI / 36 || secondChange > 10;
          }
          if (isDiscontinuity) {
            const point = curvePointAt(curve, t);
            if (point) {
              points.push(point);
              parameters.push(normalizedToParameter(curve, t));
            }
            previousTangent = tangent;
            previousSecond = second;
          }
        }
        return { points, parameters };
      },
    });
  }

  function registerCurveClosestPoint() {
    register('{2dc44b22-b1dd-460a-a704-6462d6e91096}', {
      type: 'curve',
      pinMap: {
        inputs: { P: 'point', point: 'point', C: 'curve', curve: 'curve' },
        outputs: { P: 'point', t: 'parameter', D: 'distance' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        const point = ensurePoint(inputs.point, new THREE.Vector3());
        if (!curve) {
          return {};
        }
        const closest = closestPointOnCurve(point, curve);
        return {
          point: closest.point,
          parameter: normalizedToParameter(curve, closest.parameter),
          distance: closest.distance,
        };
      },
    });
  }

  function registerClosed() {
    register('{323f3245-af49-4489-8677-7a2c73664077}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve' },
        outputs: { C: 'closed', P: 'periodic' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const start = curvePointAt(curve, 0);
        const end = curvePointAt(curve, 1);
        const closed = start && end ? start.distanceTo(end) <= 1e-5 : Boolean(curve.closed);
        const periodic = Boolean(curve.closed);
        return { closed, periodic };
      },
    });
  }

  function registerControlPointsDetailed() {
    register('{424eb433-2b3a-4859-beaf-804d8af0afd7}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve' },
        outputs: { P: 'points', W: 'weights', K: 'knots' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const points = extractCurvePoints(curve);
        const weights = Array.isArray(curve.weights) ? curve.weights.map((value) => ensureNumber(value, 1)) : new Array(points.length).fill(1);
        const knots = Array.isArray(curve.knots) ? curve.knots.map((value) => ensureNumber(value, 0)) : [];
        return { points, weights, knots };
      },
    });
  }

  function registerPlanar() {
    register('{5816ec9c-f170-4c59-ac44-364401ff84cd}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve' },
        outputs: { p: 'planar', P: 'plane', D: 'deviation' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const plane = computeCurvePlane(curve);
        const samples = sampleCurvePoints(curve, 128);
        let deviation = 0;
        for (const point of samples) {
          const coords = planeCoordinates(point, plane);
          deviation = Math.max(deviation, Math.abs(coords.z));
        }
        return { planar: deviation < 1e-5, plane, deviation };
      },
    });
  }

  function registerPolygonCenterFull() {
    register('{59e94548-cefd-4774-b3de-48142fc783fb}', {
      type: 'curve',
      pinMap: {
        inputs: { P: 'polyline', Polyline: 'polyline' },
        outputs: { Cv: 'centerVertices', Ce: 'centerEdges', Ca: 'centerArea' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.polyline);
        const points = curve ? extractCurvePoints(curve) : collectPoints(inputs.polyline);
        if (!points.length) {
          return {};
        }
        const centerVertices = polygonVertexAverage(points);
        const centerEdges = polygonEdgeAverage(points);
        const centerArea = polygonAreaCentroid(points);
        return { centerVertices, centerEdges, centerArea };
      },
    });
  }

  function registerControlPolygon() {
    register('{66d2a68e-2f1d-43d2-a53b-c6a4d17e627b}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve' },
        outputs: { C: 'polygon', P: 'points' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const points = extractCurvePoints(curve);
        const polygon = createPolyline(points, { closed: Boolean(curve.closed) });
        return { polygon, points };
      },
    });
  }

  function registerPerpFrame() {
    register('{69f3e5ee-4770-44b3-8851-ae10ae555398}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', t: 'parameter', Parameter: 'parameter' },
        outputs: { F: 'frame' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const parameter = resolveParameter(curve, inputs.parameter ?? inputs.t);
        const frame = computeParallelFrame(curve, parameter);
        const point = curvePointAt(curve, parameter);
        return {
          frame: {
            origin: point,
            xAxis: frame.tangent,
            yAxis: frame.normal,
            zAxis: frame.binormal,
          },
        };
      },
    });
  }

  function registerEvaluateLength() {
    register('{6b021f56-b194-4210-b9a1-6cef3b7d0848}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', curve: 'curve', L: 'length', length: 'length', N: 'normalized', normalized: 'normalized' },
        outputs: { P: 'point', T: 'tangent', t: 'parameter' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const normalizedFlag = ensureBoolean(inputs.normalized ?? inputs.N, false);
        const data = buildCurveLengthData(curve, 512);
        const totalLength = data.totalLength;
        const lengthValue = ensureNumber(inputs.length ?? inputs.L, normalizedFlag ? 0.5 : totalLength / 2);
        const targetLength = normalizedFlag ? clamp01(lengthValue) * totalLength : clamp(lengthValue, 0, totalLength);
        const parameter = parameterAtLength(targetLength, data);
        const point = curvePointAt(curve, parameter);
        const tangent = curveTangentAt(curve, parameter);
        return { point, tangent, parameter: normalizedToParameter(curve, parameter) };
      },
    });
  }

  function registerCurveFrame() {
    register('{6b2a5853-07aa-4329-ba84-0a5d46b51dbd}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', t: 'parameter', Parameter: 'parameter' },
        outputs: { F: 'frame' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const parameter = resolveParameter(curve, inputs.parameter ?? inputs.t);
        const frame = computeFrenetFrame(curve, parameter);
        const point = curvePointAt(curve, parameter);
        return {
          frame: {
            origin: point,
            xAxis: frame.tangent,
            yAxis: frame.normal,
            zAxis: frame.binormal,
          },
        };
      },
    });
  }

  function registerCurveProximity() {
    register('{6b7ba278-5c9d-42f1-a61d-6209cbd44907}', {
      type: 'curve',
      pinMap: {
        inputs: { A: 'curveA', B: 'curveB' },
        outputs: { A: 'pointA', B: 'pointB', D: 'distance' },
      },
      eval: ({ inputs }) => {
        const curveA = ensureCurve(inputs.curveA ?? inputs.A);
        const curveB = ensureCurve(inputs.curveB ?? inputs.B);
        if (!curveA || !curveB) {
          return {};
        }
        const result = curveCurveProximity(curveA, curveB);
        if (!result) {
          return {};
        }
        return {
          pointA: result.pointA,
          pointB: result.pointB,
          distance: Math.sqrt(Math.max(result.distanceSq, 0)),
        };
      },
    });
  }

  function registerCurvatureGraph() {
    register('{7376fe41-74ec-497e-b367-1ffe5072608b}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', D: 'distance', S: 'scale' },
        outputs: {},
      },
      eval: () => ({}),
    });
  }

  function registerCurveNearestObject() {
    register('{748f214a-bc64-4556-9da5-4fa59a30c5c7}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', curve: 'curve', G: 'geometry', geometry: 'geometry' },
        outputs: { A: 'pointA', B: 'pointB', I: 'index' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        const geometries = collectGeometries(inputs.geometry ?? inputs.G);
        if (!curve || !geometries.length) {
          return {};
        }
        let bestDistanceSq = Infinity;
        let bestPointA = null;
        let bestPointB = null;
        let bestIndex = -1;
        geometries.forEach((entry, index) => {
          if (entry?.isVector3 || Array.isArray(entry) || typeof entry === 'object') {
            const points = collectPoints(entry);
            if (points.length === 1) {
              const result = closestPointOnCurve(points[0], curve);
              if (result.distanceSq < bestDistanceSq) {
                bestDistanceSq = result.distanceSq;
                bestPointA = result.point;
                bestPointB = points[0].clone();
                bestIndex = index;
              }
            } else if (points.length > 1) {
              const curveB = createCurveFromPoints(points);
              if (curveB) {
                const result = curveCurveProximity(curve, curveB);
                if (result && result.distanceSq < bestDistanceSq) {
                  bestDistanceSq = result.distanceSq;
                  bestPointA = result.pointA;
                  bestPointB = result.pointB;
                  bestIndex = index;
                }
              }
            }
          }
          const otherCurve = ensureCurve(entry);
          if (otherCurve) {
            const result = curveCurveProximity(curve, otherCurve);
            if (result && result.distanceSq < bestDistanceSq) {
              bestDistanceSq = result.distanceSq;
              bestPointA = result.pointA;
              bestPointB = result.pointB;
              bestIndex = index;
            }
          }
        });
        if (bestIndex < 0) {
          return {};
        }
        return {
          pointA: bestPointA,
          pointB: bestPointB,
          index: bestIndex,
        };
      },
    });
  }

  function registerPolygonCenterSimple() {
    register('{7bd7b551-ca79-4f01-b95a-7e9ab876f24d}', {
      type: 'curve',
      pinMap: {
        inputs: { P: 'polyline' },
        outputs: { C: 'center' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.polyline);
        const points = curve ? extractCurvePoints(curve) : collectPoints(inputs.polyline);
        if (!points.length) {
          return {};
        }
        return { center: polygonVertexAverage(points) };
      },
    });
  }

  function registerPolygonCenterPartial() {
    register('{87e7f480-14dc-4478-b1e6-2b8b035d9edc}', {
      type: 'curve',
      pinMap: {
        inputs: { P: 'polyline' },
        outputs: { Cv: 'centerVertices', Ce: 'centerEdges' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.polyline);
        const points = curve ? extractCurvePoints(curve) : collectPoints(inputs.polyline);
        if (!points.length) {
          return {};
        }
        return {
          centerVertices: polygonVertexAverage(points),
          centerEdges: polygonEdgeAverage(points),
        };
      },
    });
  }

  function registerLengthParameter() {
    register('{a1c16251-74f0-400f-9e7c-5e379d739963}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', P: 'parameter' },
        outputs: { 'L-': 'lengthBefore', 'L+': 'lengthAfter' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const parameter = resolveParameter(curve, inputs.parameter ?? inputs.P);
        const data = buildCurveLengthData(curve, 512);
        const total = data.totalLength;
        const lengthBefore = lengthAtNormalized(data, parameter);
        return {
          'lengthBefore': lengthBefore,
          'lengthAfter': Math.max(total - lengthBefore, 0),
        };
      },
    });
  }

  function registerCurveDepth() {
    register('{a583f722-240a-4fc9-aa1d-021720a4516a}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', Min: 'min', Max: 'max' },
        outputs: { tMin: 'parameterMin', dMin: 'depthMin', tMax: 'parameterMax', dMax: 'depthMax' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const plane = computeCurvePlane(curve);
        const samples = sampleCurvePoints(curve, 256);
        let minDepth = Infinity;
        let maxDepth = -Infinity;
        let minParameter = 0;
        let maxParameter = 0;
        samples.forEach((point, index) => {
          const coords = planeCoordinates(point, plane);
          if (coords.z < minDepth) {
            minDepth = coords.z;
            minParameter = index / Math.max(samples.length - 1, 1);
          }
          if (coords.z > maxDepth) {
            maxDepth = coords.z;
            maxParameter = index / Math.max(samples.length - 1, 1);
          }
        });
        const minLimit = Number.isFinite(ensureNumber(inputs.min, Number.NEGATIVE_INFINITY)) ? ensureNumber(inputs.min, minDepth) : Number.NEGATIVE_INFINITY;
        const maxLimit = Number.isFinite(ensureNumber(inputs.max, Number.POSITIVE_INFINITY)) ? ensureNumber(inputs.max, maxDepth) : Number.POSITIVE_INFINITY;
        const clampedMin = clamp(minDepth, minLimit, maxLimit);
        const clampedMax = clamp(maxDepth, minLimit, maxLimit);
        return {
          parameterMin: normalizedToParameter(curve, minParameter),
          depthMin: clampedMin,
          parameterMax: normalizedToParameter(curve, maxParameter),
          depthMax: clampedMax,
        };
      },
    });
  }

  function registerPointInCurve() {
    register('{a72b0bd3-c7a7-458e-875d-09ae1624638c}', {
      type: 'curve',
      pinMap: {
        inputs: { P: 'point', C: 'curve' },
        outputs: { R: 'relationship', "P'": 'projectedPoint' },
      },
      eval: ({ inputs }) => {
        const { relationship, index, projectedPoint } = evaluateContainment(inputs.point, inputs.curve);
        return { relationship, projectedPoint, index };
      },
    });
  }

  function registerCurvature() {
    register('{aaa665bd-fd6e-4ccb-8d2c-c5b33072125d}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', t: 'parameter', Parameter: 'parameter' },
        outputs: { P: 'point', K: 'curvature', C: 'center' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const parameter = resolveParameter(curve, inputs.parameter ?? inputs.t);
        const data = computeCurvature(curve, parameter);
        return { point: data.point, curvature: data.curvature, center: data.center };
      },
    });
  }

  function registerDerivativesFirst() {
    register('{ab14760f-87a6-462e-b481-4a2c26a9a0d7}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', t: 'parameter', Parameter: 'parameter' },
        outputs: { P: 'point', '1': 'firstDerivative' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const parameter = resolveParameter(curve, inputs.parameter ?? inputs.t);
        const derivatives = computeCurveDerivatives(curve, parameter, 1);
        const point = curvePointAt(curve, parameter);
        return { point, firstDerivative: derivatives[0] };
      },
    });
  }

  function registerArcCenter() {
    register('{afff17ed-5975-460b-9883-525ae0677088}', {
      type: 'curve',
      pinMap: {
        inputs: { A: 'arc' },
        outputs: { C: 'center', R: 'radius' },
      },
      eval: ({ inputs }) => {
        const arc = ensureCurve(inputs.arc ?? inputs.A);
        if (!arc) {
          return {};
        }
        const center = arc.center ?? computeCurvePlane(arc).origin;
        const radius = arc.radius ?? 0;
        return { center, radius };
      },
    });
  }

  function registerCurveSide() {
    register('{bb2e13da-09ca-43fd-bef8-8d71f3653af9}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', P: 'point', Pl: 'plane' },
        outputs: { S: 'side', L: 'isLeft', R: 'isRight' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        const point = ensurePoint(inputs.point, new THREE.Vector3());
        if (!curve) {
          return {};
        }
        const plane = inputs.Pl ? ensurePlane(inputs.Pl) : computeCurvePlane(curve);
        const closest = closestPointOnCurve(point, curve);
        const curvePoint = closest.point;
        const tangent = curveTangentAt(curve, closest.parameter);
        const toPoint = point.clone().sub(curvePoint);
        const sign = Math.sign(tangent.clone().cross(toPoint).dot(plane.zAxis));
        const side = sign > 0 ? 1 : sign < 0 ? -1 : 0;
        return { side, isLeft: side > 0, isRight: side < 0 };
      },
    });
  }

  function registerHorizontalFrame() {
    register('{c048ad76-ffcd-43b1-a007-4dd1b2373326}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', t: 'parameter', Parameter: 'parameter' },
        outputs: { F: 'frame' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const parameter = resolveParameter(curve, inputs.parameter ?? inputs.t);
        const frame = computeHorizontalFrame(curve, parameter);
        const point = curvePointAt(curve, parameter);
        return {
          frame: {
            origin: point,
            xAxis: frame.tangent,
            yAxis: frame.normal,
            zAxis: frame.binormal,
          },
        };
      },
    });
  }

  function registerContainment() {
    register('{c076845a-1a09-4a95-bdcb-cb31c0936c99}', {
      type: 'curve',
      pinMap: {
        inputs: { P: 'point', C: 'curve' },
        outputs: { R: 'relationship', "P'": 'projectedPoint' },
      },
      eval: ({ inputs }) => {
        const { relationship, projectedPoint } = evaluateContainment(inputs.point, inputs.curve);
        return { relationship, projectedPoint };
      },
    });
  }

  function registerDerivativesList() {
    register('{c2e16ca3-9508-4fa4-aeb3-0b1f0ebb72e3}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', t: 'parameter', Parameter: 'parameter', N: 'count' },
        outputs: { P: 'point', d: 'derivatives' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const parameter = resolveParameter(curve, inputs.parameter ?? inputs.t);
        const count = Math.max(1, Math.min(3, Math.round(ensureNumber(inputs.count ?? inputs.N, 1))));
        const derivatives = computeCurveDerivatives(curve, parameter, count);
        const point = curvePointAt(curve, parameter);
        return { point, derivatives };
      },
    });
  }

  function registerCurveLength() {
    register('{c75b62fa-0a33-4da7-a5bd-03fd0068fd93}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve' },
        outputs: { L: 'length' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const data = buildCurveLengthData(curve, 512);
        return { length: data.totalLength };
      },
    });
  }

  function registerCurveMiddle() {
    register('{ccc7b468-e743-4049-891f-299432545898}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve' },
        outputs: { M: 'midpoint' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const data = buildCurveLengthData(curve, 512);
        const half = data.totalLength / 2;
        const parameter = parameterAtLength(half, data);
        return { midpoint: curvePointAt(curve, parameter) };
      },
    });
  }

  function registerCurveDomainAdjust() {
    register('{ccfd6ba8-ecb1-44df-a47e-08126a653c51}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', D: 'domain' },
        outputs: { C: 'curve', D: 'domain' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const { domain } = resolveDomainInput(curve, inputs.domain ?? inputs.D);
        const adjusted = { ...curve, domain };
        return { curve: adjusted, domain };
      },
    });
  }

  function registerControlPointsSimple() {
    register('{d7df7658-e02d-4a48-a345-2195a68db4ef}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve' },
        outputs: { P: 'points', W: 'weights' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const points = extractCurvePoints(curve);
        const weights = Array.isArray(curve.weights) ? curve.weights.map((value) => ensureNumber(value, 1)) : new Array(points.length).fill(1);
        return { points, weights };
      },
    });
  }

  function registerTorsion() {
    register('{dbe9fce4-b6b3-465f-9615-34833c4763bd}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', t: 'parameter', Parameter: 'parameter' },
        outputs: { P: 'point', T: 'torsion' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const parameter = resolveParameter(curve, inputs.parameter ?? inputs.t);
        const point = curvePointAt(curve, parameter);
        const torsion = computeTorsion(curve, parameter);
        return { point, torsion };
      },
    });
  }

  function registerDeconstructRectangle() {
    register('{e5c33a79-53d5-4f2b-9a97-d3d45c780edc}', {
      type: 'curve',
      pinMap: {
        inputs: { R: 'rectangle', rectangle: 'rectangle' },
        outputs: { B: 'basePlane', X: 'xInterval', Y: 'yInterval' },
      },
      eval: ({ inputs }) => {
        const rectangle = inputs.rectangle ?? inputs.R;
        if (!rectangle) {
          return {};
        }
        const plane = rectangle.plane ? ensurePlane(rectangle.plane) : defaultPlane();
        const width = ensureNumber(rectangle.width ?? rectangle.xSize, 1);
        const height = ensureNumber(rectangle.height ?? rectangle.ySize, 1);
        return {
          basePlane: plane,
          xInterval: createDomain(-width / 2, width / 2),
          yInterval: createDomain(-height / 2, height / 2),
        };
      },
    });
  }

  function registerExtremes() {
    register('{ebd6c758-19ae-4d74-aed7-b8a0392ff743}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', P: 'plane' },
        outputs: { H: 'highest', L: 'lowest' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const plane = inputs.plane ? ensurePlane(inputs.plane) : computeCurvePlane(curve);
        const samples = sampleCurvePoints(curve, 256);
        let max = -Infinity;
        let min = Infinity;
        const highest = [];
        const lowest = [];
        samples.forEach((point) => {
          const value = planeCoordinates(point, plane).z;
          if (value > max - 1e-6) {
            if (value > max + 1e-6) highest.length = 0;
            max = value;
            highest.push(point.clone());
          }
          if (value < min + 1e-6) {
            if (value < min - 1e-6) lowest.length = 0;
            min = value;
            lowest.push(point.clone());
          }
        });
        return { highest, lowest };
      },
    });
  }

  function registerClosedObsolete() {
    register('{f2030fa9-db3f-437e-9b50-5607db6daf87}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve' },
        outputs: { C: 'closed' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const start = curvePointAt(curve, 0);
        const end = curvePointAt(curve, 1);
        return { closed: start && end ? start.distanceTo(end) <= 1e-5 : Boolean(curve.closed) };
      },
    });
  }

  function registerSegmentLengths() {
    register('{f88a6cd9-1035-4361-b896-4f2dfe79272d}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve' },
        outputs: { Sl: 'shortestLength', Sd: 'shortestDomain', Ll: 'longestLength', Ld: 'longestDomain' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const data = buildCurveLengthData(curve, 512);
        const samples = data.samples;
        let minLength = Infinity;
        let maxLength = 0;
        let minDomain = createDomain(0, 0);
        let maxDomain = createDomain(0, 0);
        for (let i = 0; i < samples.length - 1; i += 1) {
          const current = samples[i];
          const next = samples[i + 1];
          const length = next.length - current.length;
          if (length < minLength) {
            minLength = length;
            minDomain = createDomain(normalizedToParameter(curve, current.t), normalizedToParameter(curve, next.t));
          }
          if (length > maxLength) {
            maxLength = length;
            maxDomain = createDomain(normalizedToParameter(curve, current.t), normalizedToParameter(curve, next.t));
          }
        }
        return { shortestLength: minLength, shortestDomain: minDomain, longestLength: maxLength, longestDomain: maxDomain };
      },
    });
  }

  function registerEvaluateCurveAngle() {
    register('{fc6979e4-7e91-4508-8e05-37c680779751}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', t: 'parameter', Parameter: 'parameter' },
        outputs: { P: 'point', T: 'tangent', A: 'angle' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const parameter = resolveParameter(curve, inputs.parameter ?? inputs.t);
        const point = curvePointAt(curve, parameter);
        const tangent = curveTangentAt(curve, parameter);
        const delta = 1e-3;
        const tangentBefore = curveTangentAt(curve, clamp01(parameter - delta));
        const tangentAfter = curveTangentAt(curve, clamp01(parameter + delta));
        const angle = tangentBefore.angleTo(tangentAfter);
        return { point, tangent, angle };
      },
    });
  }

  function registerEvaluateCurveLength() {
    register('{fdf09135-fae5-4e5f-b427-b1f384ca3009}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', t: 'parameter', Parameter: 'parameter' },
        outputs: { P: 'point', T: 'tangent', L: 'length' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const parameter = resolveParameter(curve, inputs.parameter ?? inputs.t);
        const point = curvePointAt(curve, parameter);
        const tangent = curveTangentAt(curve, parameter);
        const data = buildCurveLengthData(curve, 512);
        const length = lengthAtNormalized(data, parameter);
        return { point, tangent, length };
      },
    });
  }

  registerPointInCurves();
  registerEndPoints();
  registerCurveDomainObsolete();
  registerEvaluateCurveSimple();
  registerLengthDomain();
  registerDeconstructArc();
  registerDiscontinuity();
  registerCurveClosestPoint();
  registerClosed();
  registerControlPointsDetailed();
  registerPlanar();
  registerPolygonCenterFull();
  registerControlPolygon();
  registerPerpFrame();
  registerEvaluateLength();
  registerCurveFrame();
  registerCurveProximity();
  registerCurvatureGraph();
  registerCurveNearestObject();
  registerPolygonCenterSimple();
  registerPolygonCenterPartial();
  registerLengthParameter();
  registerCurveDepth();
  registerPointInCurve();
  registerCurvature();
  registerDerivativesFirst();
  registerArcCenter();
  registerCurveSide();
  registerHorizontalFrame();
  registerContainment();
  registerDerivativesList();
  registerCurveLength();
  registerCurveMiddle();
  registerCurveDomainAdjust();
  registerControlPointsSimple();
  registerTorsion();
  registerDeconstructRectangle();
  registerExtremes();
  registerClosedObsolete();
  registerSegmentLengths();
  registerEvaluateCurveAngle();
  registerEvaluateCurveLength();
}

export function registerCurveUtilComponents({ register, toNumber, toVector3 }) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register curve util components.');
  }
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register curve util components.');
  }
  if (typeof toVector3 !== 'function') {
    throw new Error('toVector3 function is required to register curve util components.');
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

  function ensureBoolean(value, fallback = false) {
    if (value === undefined || value === null) {
      return fallback;
    }
    if (typeof value === 'boolean') {
      return value;
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
      const numeric = Number(normalized);
      if (Number.isFinite(numeric)) {
        return numeric !== 0;
      }
      return fallback;
    }
    if (Array.isArray(value)) {
      if (!value.length) return fallback;
      return ensureBoolean(value[value.length - 1], fallback);
    }
    if (typeof value === 'object') {
      if ('value' in value) {
        return ensureBoolean(value.value, fallback);
      }
      if ('flag' in value) {
        return ensureBoolean(value.flag, fallback);
      }
    }
    return Boolean(value);
  }

  function ensureArray(input) {
    if (input === undefined || input === null) {
      return [];
    }
    return Array.isArray(input) ? input : [input];
  }

  function ensurePoint(value, fallback = new THREE.Vector3()) {
    return toVector3(value, fallback.clone ? fallback.clone() : fallback);
  }

  function convertVector(value, fallback = new THREE.Vector3()) {
    if (value?.isVector3) {
      return value.clone();
    }
    if (Array.isArray(value)) {
      const [x, y, z] = value;
      return new THREE.Vector3(ensureNumber(x, 0), ensureNumber(y, 0), ensureNumber(z, 0));
    }
    if (typeof value === 'number') {
      return new THREE.Vector3(0, 0, ensureNumber(value, 0));
    }
    if (typeof value === 'object' && value) {
      const vector = toVector3(value, null);
      if (vector) {
        return vector;
      }
    }
    return fallback.clone();
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
        if ('vertices' in value) {
          visit(value.vertices);
          return;
        }
        if ('value' in value) {
          visit(value.value);
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
    curve.isNativeCurve = true;
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
    if (curve.points && Array.isArray(curve.points)) {
      if (curve.points.length === segments + 1) {
        return curve.points.map((pt) => pt.clone());
      }
      const result = [];
      for (let i = 0; i <= segments; i += 1) {
        const t = i / segments;
        const point = curvePointAt(curve, t);
        if (point) {
          result.push(point);
        }
      }
      return result;
    }
    if (curve.path?.getSpacedPoints) {
      return curve.path.getSpacedPoints(Math.max(segments, 8)).map((pt) => new THREE.Vector3(pt.x, pt.y, pt.z ?? 0));
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
      const candidate = new THREE.Vector3(0, -vector.z, vector.y);
      return candidate.lengthSq() < EPSILON ? new THREE.Vector3(0, 1, 0) : candidate.normalize();
    }
    if (absY <= absX && absY <= absZ) {
      const candidate = new THREE.Vector3(-vector.z, 0, vector.x);
      return candidate.lengthSq() < EPSILON ? new THREE.Vector3(1, 0, 0) : candidate.normalize();
    }
    const candidate = new THREE.Vector3(-vector.y, vector.x, 0);
    return candidate.lengthSq() < EPSILON ? new THREE.Vector3(1, 0, 0) : candidate.normalize();
  }

  function defaultPlane() {
    return {
      origin: new THREE.Vector3(0, 0, 0),
      xAxis: new THREE.Vector3(1, 0, 0),
      yAxis: new THREE.Vector3(0, 1, 0),
      zAxis: new THREE.Vector3(0, 0, 1),
    };
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
    const normal = computeSmallestEigenVector(covariance);
    if (!normal) {
      return null;
    }
    const normalizedNormal = normalizeVector(normal, new THREE.Vector3(0, 0, 1));
    const xAxis = orthogonalVector(normalizedNormal);
    const yAxis = normalizedNormal.clone().cross(xAxis).normalize();
    return normalizePlaneAxes(centroid, xAxis, yAxis, normalizedNormal);
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

  function ensureCurve(input) {
    if (!input) {
      return null;
    }
    if (input.isNativeCurve) {
      return input;
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
      const center = ensurePoint(input.center, new THREE.Vector3());
      const radius = Math.max(ensureNumber(input.radius, 0), EPSILON);
      const normal = input.plane?.zAxis ? ensurePoint(input.plane.zAxis, new THREE.Vector3(0, 0, 1)) : new THREE.Vector3(0, 0, 1);
      const xAxis = orthogonalVector(normal);
      const yAxis = normal.clone().cross(xAxis).normalize();
      const plane = normalizePlaneAxes(center.clone(), xAxis, yAxis, normal);
      const shape = new THREE.Shape();
      shape.absarc(0, 0, radius, 0, Math.PI * 2, false);
      const curve = createCurveFromPath(shape, { segments: input.segments ?? 128, closed: true, type: 'circle' });
      curve.center = center.clone();
      curve.radius = radius;
      curve.plane = plane;
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

  function extractCurvePoints(curve, segments = 64) {
    if (!curve) {
      return [];
    }
    if (Array.isArray(curve.controlPoints)) {
      return curve.controlPoints.map((pt) => ensurePoint(pt, new THREE.Vector3()));
    }
    if (curve.points?.length) {
      return curve.points.map((pt) => pt.clone());
    }
    return sampleCurvePoints(curve, segments);
  }
  function isClosedPolyline(points) {
    if (!points || points.length < 3) {
      return false;
    }
    return points[0].distanceTo(points[points.length - 1]) < 1e-6;
  }

  function normalizePolylinePoints(pointsInput, { closed = false } = {}) {
    const points = pointsInput.map((pt) => pt.clone());
    if (!points.length) {
      return points;
    }
    if (closed) {
      const first = points[0];
      const last = points[points.length - 1];
      if (first.distanceTo(last) < 1e-6) {
        points.pop();
      }
    }
    const result = [points[0]];
    for (let i = 1; i < points.length; i += 1) {
      if (points[i].distanceTo(result[result.length - 1]) > EPSILON) {
        result.push(points[i]);
      }
    }
    if (closed && result.length >= 3) {
      result.push(result[0].clone());
    }
    return result;
  }

  function computePolylineNormals2D(coords, { closed = false } = {}) {
    const count = coords.length;
    const normals = new Array(count).fill(null).map(() => ({ x: 0, y: 0 }));
    const segmentCount = closed ? count : count - 1;

    const addNormal = (index, normal) => {
      normals[index].x += normal.x;
      normals[index].y += normal.y;
    };

    for (let i = 0; i < segmentCount; i += 1) {
      const a = coords[i];
      const b = coords[(i + 1) % count];
      const dx = b.x - a.x;
      const dy = b.y - a.y;
      const length = Math.hypot(dx, dy);
      if (length < EPSILON) {
        continue;
      }
      const nx = -dy / length;
      const ny = dx / length;
      addNormal(i, { x: nx, y: ny });
      addNormal((i + 1) % count, { x: nx, y: ny });
    }

    for (let i = 0; i < count; i += 1) {
      const normal = normals[i];
      const length = Math.hypot(normal.x, normal.y);
      if (length < EPSILON) {
        let prevIndex = i - 1;
        let nextIndex = i + 1;
        if (closed) {
          prevIndex = (i - 1 + count) % count;
          nextIndex = (i + 1) % count;
        }
        const prev = coords[prevIndex] ?? coords[i];
        const next = coords[nextIndex] ?? coords[i];
        const dx = next.x - prev.x;
        const dy = next.y - prev.y;
        const fallbackLength = Math.hypot(dx, dy);
        if (fallbackLength > EPSILON) {
          normal.x = -dy / fallbackLength;
          normal.y = dx / fallbackLength;
        } else {
          normal.x = 0;
          normal.y = 0;
        }
      } else {
        normal.x /= length;
        normal.y /= length;
      }
    }
    return normals;
  }

  function offsetPolylinePoints(pointsInput, plane, distance, { closed = false } = {}) {
    const coords = pointsInput.map((pt) => planeCoordinates(pt, plane));
    const normals = computePolylineNormals2D(coords, { closed });
    const offsetCoords = coords.map((coord, index) => ({
      x: coord.x + normals[index].x * distance,
      y: coord.y + normals[index].y * distance,
    }));
    return offsetCoords.map((coord) => applyPlane(plane, coord.x, coord.y, 0));
  }

  function offsetPolylineLoose(pointsInput, plane, distance, { closed = false } = {}) {
    if (Math.abs(distance) < EPSILON) {
      return pointsInput.map((pt) => pt.clone());
    }
    const coords = pointsInput.map((pt) => planeCoordinates(pt, plane));
    const result = [];
    for (let i = 0; i < coords.length; i += 1) {
      const prev = coords[(i - 1 + coords.length) % coords.length];
      const current = coords[i];
      const next = coords[(i + 1) % coords.length];
      const dx = next.x - prev.x;
      const dy = next.y - prev.y;
      const length = Math.hypot(dx, dy);
      let nx = 0;
      let ny = 0;
      if (length > EPSILON) {
        nx = -dy / length;
        ny = dx / length;
      }
      if (!closed && (i === 0 || i === coords.length - 1)) {
        const neighbor = i === 0 ? next : prev;
        const dxEdge = current.x - neighbor.x;
        const dyEdge = current.y - neighbor.y;
        const edgeLength = Math.hypot(dxEdge, dyEdge);
        if (edgeLength > EPSILON) {
          nx = -dyEdge / edgeLength;
          ny = dxEdge / edgeLength;
        }
      }
      result.push(applyPlane(plane, current.x + nx * distance, current.y + ny * distance, 0));
    }
    return result;
  }

  function approximateCurveWithTolerance(curve, {
    distanceTolerance = 0.01,
    angleTolerance = 0,
    minEdge = 0,
    maxEdge = Infinity,
    maxDepth = 10,
    closed = false,
  } = {}) {
    if (!curve) {
      return [];
    }
    const startPoint = curvePointAt(curve, 0) ?? new THREE.Vector3();
    const endPoint = curvePointAt(curve, 1) ?? startPoint.clone();

    function segmentLength(a, b) {
      return a.distanceTo(b);
    }

    function maxDeviation(t0, p0, t1, p1) {
      const tm = (t0 + t1) / 2;
      const pm = curvePointAt(curve, tm);
      if (!pm) {
        return { deviation: 0, midT: tm, midPoint: p0.clone().lerp(p1, 0.5) };
      }
      const chordMid = p0.clone().lerp(p1, 0.5);
      const deviation = pm.distanceTo(chordMid);
      return { deviation, midT: tm, midPoint: pm };
    }

    function angleBetween(t0, t1) {
      const tangent0 = curveTangentAt(curve, t0);
      const tangent1 = curveTangentAt(curve, t1);
      const dot = clamp(tangent0.dot(tangent1), -1, 1);
      return Math.acos(dot);
    }

    const segments = [];
    const stack = [{ t0: 0, p0: startPoint, t1: 1, p1: endPoint, depth: 0 }];

    while (stack.length) {
      const { t0, p0, t1, p1, depth } = stack.pop();
      const length = segmentLength(p0, p1);
      const { deviation, midT, midPoint } = maxDeviation(t0, p0, t1, p1);
      const shouldSplitByDistance = deviation > distanceTolerance;
      const shouldSplitByEdge = length > maxEdge;
      const angle = angleBetween(t0, t1);
      const angleDeviation = Math.PI - angle;
      const shouldSplitByAngle = angleTolerance > 0 && angleDeviation > angleTolerance;
      const shouldSplitByMinEdge = length > EPSILON && length < minEdge;
      if (
        depth < maxDepth &&
        (shouldSplitByDistance || shouldSplitByAngle || shouldSplitByEdge || shouldSplitByMinEdge)
      ) {
        stack.push({ t0: midT, p0: midPoint, t1, p1, depth: depth + 1 });
        stack.push({ t0, p0, t1: midT, p1: midPoint, depth: depth + 1 });
      } else {
        segments.push({ t0, p0, t1, p1 });
      }
    }

    segments.sort((a, b) => a.t0 - b.t0);
    if (segments.length === 0) {
      return [startPoint.clone(), endPoint.clone()];
    }
    const points = [];
    points.push(segments[0].p0.clone());
    for (const segment of segments) {
      points.push(segment.p1.clone());
    }
    if (closed) {
      const first = points[0];
      const last = points[points.length - 1];
      if (first.distanceTo(last) > EPSILON) {
        points.push(first.clone());
      }
    }
    return points;
  }

  function rdpSimplify(pointsInput, tolerance, angleTolerance = 0) {
    if (!pointsInput || pointsInput.length <= 2) {
      return pointsInput.map((pt) => pt.clone());
    }
    const points = pointsInput.map((pt) => pt.clone());
    const stack = [[0, points.length - 1]];
    const keep = new Array(points.length).fill(false);
    keep[0] = true;
    keep[points.length - 1] = true;

    function perpendicularDistance(point, start, end) {
      const seg = end.clone().sub(start);
      const lengthSq = seg.lengthSq();
      if (lengthSq < EPSILON) {
        return point.distanceTo(start);
      }
      const toPoint = point.clone().sub(start);
      const cross = seg.clone().cross(toPoint);
      return cross.length() / Math.sqrt(lengthSq);
    }

    function cornerAngle(index) {
      if (index <= 0 || index >= points.length - 1) {
        return Math.PI;
      }
      const prev = points[index - 1];
      const current = points[index];
      const next = points[index + 1];
      const v1 = current.clone().sub(prev);
      const v2 = next.clone().sub(current);
      if (v1.lengthSq() < EPSILON || v2.lengthSq() < EPSILON) {
        return Math.PI;
      }
      v1.normalize();
      v2.normalize();
      const dot = clamp(v1.dot(v2), -1, 1);
      return Math.acos(dot);
    }

    while (stack.length) {
      const [startIndex, endIndex] = stack.pop();
      if (endIndex <= startIndex + 1) {
        continue;
      }
      const start = points[startIndex];
      const end = points[endIndex];
      let maxDistance = -1;
      let index = -1;
      for (let i = startIndex + 1; i < endIndex; i += 1) {
        const distance = perpendicularDistance(points[i], start, end);
        if (distance > maxDistance) {
          maxDistance = distance;
          index = i;
        }
      }
      const angle = angleTolerance > 0 ? Math.PI - cornerAngle(index) : 0;
      if (maxDistance > tolerance || angle > angleTolerance) {
        keep[index] = true;
        stack.push([startIndex, index]);
        stack.push([index, endIndex]);
      }
    }

    const result = [];
    for (let i = 0; i < points.length; i += 1) {
      if (keep[i]) {
        result.push(points[i].clone());
      }
    }
    if (result.length === 1 && points.length > 1) {
      result.push(points[points.length - 1].clone());
    }
    return result;
  }

  function smoothPolylinePoints(pointsInput, strength = 0.5, iterations = 1, { closed = false } = {}) {
    const clampedStrength = clamp(strength, 0, 1);
    if (clampedStrength <= EPSILON || iterations <= 0) {
      return pointsInput.map((pt) => pt.clone());
    }
    let points = pointsInput.map((pt) => pt.clone());
    const count = points.length;
    if (count <= 2) {
      return points;
    }
    for (let iteration = 0; iteration < iterations; iteration += 1) {
      const nextPoints = points.map((pt) => pt.clone());
      for (let i = 0; i < count; i += 1) {
        const isEndpoint = !closed && (i === 0 || i === count - 1);
        if (isEndpoint) {
          continue;
        }
        const prev = points[(i - 1 + count) % count];
        const current = points[i];
        const next = points[(i + 1) % count];
        const target = prev.clone().add(next).multiplyScalar(0.5);
        nextPoints[i] = current.clone().lerp(target, clampedStrength);
      }
      points = nextPoints;
    }
    return points;
  }

  function addPointIfDistinct(points, point) {
    if (!point) {
      return;
    }
    if (!points.length) {
      points.push(point.clone());
      return;
    }
    const last = points[points.length - 1];
    if (last.distanceToSquared(point) > EPSILON * EPSILON) {
      points.push(point.clone());
    }
  }

  function shortenPolylineStart(pointsInput, distance) {
    if (!pointsInput.length) {
      return [];
    }
    if (pointsInput.length === 1 || distance <= EPSILON) {
      return pointsInput.map((pt) => pt.clone());
    }
    const points = pointsInput.map((pt) => pt.clone());
    let remaining = distance;
    let index = 0;
    let newStart = points[0].clone();
    while (index < points.length - 1 && remaining > EPSILON) {
      const current = points[index];
      const next = points[index + 1];
      const segment = next.clone().sub(current);
      const segmentLength = segment.length();
      if (segmentLength <= EPSILON) {
        index += 1;
        newStart = next.clone();
        continue;
      }
      if (remaining >= segmentLength - EPSILON) {
        remaining -= segmentLength;
        index += 1;
        newStart = next.clone();
        continue;
      }
      const ratio = remaining / segmentLength;
      newStart = current.clone().add(segment.multiplyScalar(ratio));
      remaining = 0;
      index += 1;
      break;
    }
    const result = [];
    addPointIfDistinct(result, newStart);
    for (let i = index; i < points.length; i += 1) {
      addPointIfDistinct(result, points[i]);
    }
    if (result.length === 1) {
      result.push(result[0].clone());
    }
    return result;
  }

  function shortenPolylineEnd(pointsInput, distance) {
    if (!pointsInput.length) {
      return [];
    }
    if (pointsInput.length === 1 || distance <= EPSILON) {
      return pointsInput.map((pt) => pt.clone());
    }
    const points = pointsInput.map((pt) => pt.clone());
    let remaining = distance;
    let index = points.length - 1;
    let newEnd = points[index].clone();
    while (index > 0 && remaining > EPSILON) {
      const current = points[index];
      const prev = points[index - 1];
      const segment = current.clone().sub(prev);
      const segmentLength = segment.length();
      if (segmentLength <= EPSILON) {
        index -= 1;
        newEnd = prev.clone();
        continue;
      }
      if (remaining >= segmentLength - EPSILON) {
        remaining -= segmentLength;
        index -= 1;
        newEnd = prev.clone();
        continue;
      }
      const ratio = remaining / segmentLength;
      newEnd = prev.clone().lerp(current, 1 - ratio);
      remaining = 0;
      index -= 1;
      break;
    }
    const result = [];
    for (let i = 0; i <= index; i += 1) {
      addPointIfDistinct(result, points[i]);
    }
    addPointIfDistinct(result, newEnd);
    if (result.length === 1) {
      result.unshift(result[0].clone());
    }
    return result;
  }

  function extendCurvePoints(curve, startLength, endLength) {
    const basePoints = extractCurvePoints(curve, 64);
    if (basePoints.length < 2) {
      return basePoints.map((pt) => pt.clone());
    }
    const shortenStart = startLength < -EPSILON ? -startLength : 0;
    const shortenEnd = endLength < -EPSILON ? -endLength : 0;
    const extendStart = startLength > EPSILON ? startLength : 0;
    const extendEnd = endLength > EPSILON ? endLength : 0;

    let adjusted = basePoints.map((pt) => pt.clone());
    if (shortenStart > EPSILON) {
      adjusted = shortenPolylineStart(adjusted, shortenStart);
    }
    if (shortenEnd > EPSILON) {
      adjusted = shortenPolylineEnd(adjusted, shortenEnd);
    }
    if (adjusted.length < 2) {
      return adjusted;
    }

    const startPoint = adjusted[0].clone();
    const endPoint = adjusted[adjusted.length - 1].clone();

    const fallbackStart = normalizeVector(curveTangentAt(curve, 0), new THREE.Vector3(1, 0, 0));
    const fallbackEnd = normalizeVector(curveTangentAt(curve, 1), new THREE.Vector3(1, 0, 0));

    let startTangent = fallbackStart.clone();
    if (adjusted.length >= 2) {
      const startSegment = adjusted[1].clone().sub(adjusted[0]);
      if (startSegment.lengthSq() > EPSILON) {
        startTangent = startSegment.normalize();
      }
    }

    let endTangent = fallbackEnd.clone();
    if (adjusted.length >= 2) {
      const endSegment = adjusted[adjusted.length - 1].clone().sub(adjusted[adjusted.length - 2]);
      if (endSegment.lengthSq() > EPSILON) {
        endTangent = endSegment.normalize();
      }
    }

    const extended = adjusted.map((pt) => pt.clone());
    if (extendStart > EPSILON) {
      const extension = startTangent.clone().multiplyScalar(-extendStart);
      extended.unshift(startPoint.clone().add(extension));
    }
    if (extendEnd > EPSILON) {
      const extension = endTangent.clone().multiplyScalar(extendEnd);
      extended.push(endPoint.clone().add(extension));
    }
    return extended;
  }

  function tryMergePolylines(base, candidate, { preserveDirection, tolerance = 1e-6 }) {
    if (!base || !candidate) {
      return null;
    }
    if (base.closed || candidate.closed) {
      return null;
    }
    const baseStart = base.points[0];
    const baseEnd = base.points[base.points.length - 1];
    const candidateStart = candidate.points[0];
    const candidateEnd = candidate.points[candidate.points.length - 1];

    const reversedCandidate = {
      points: candidate.points.map((pt) => pt.clone()).reverse(),
      closed: false,
    };

    const canReverse = !preserveDirection;

    if (baseEnd.distanceTo(candidateStart) <= tolerance) {
      const merged = base.points.slice(0, -1).concat(candidate.points);
      return { points: merged, closed: false };
    }

    if (canReverse && baseEnd.distanceTo(candidateEnd) <= tolerance) {
      const merged = base.points.slice(0, -1).concat(reversedCandidate.points);
      return { points: merged, closed: false };
    }

    if (baseStart.distanceTo(candidateEnd) <= tolerance) {
      const merged = candidate.points.slice(0, -1).concat(base.points);
      return { points: merged, closed: false };
    }

    if (canReverse && baseStart.distanceTo(candidateStart) <= tolerance) {
      const merged = reversedCandidate.points.slice(0, -1).concat(base.points);
      return { points: merged, closed: false };
    }

    if (base.points.length > 2 && candidate.points.length > 2) {
      const closedCandidate = baseEnd.distanceTo(candidateEnd) <= tolerance && baseStart.distanceTo(candidateStart) <= tolerance;
      if (closedCandidate && !preserveDirection) {
        const merged = base.points.slice(0, -1).concat(candidate.points);
        return { points: merged, closed: true };
      }
    }
    return null;
  }

  function joinPolylines(inputs, { preserveDirection, tolerance = 1e-6 } = {}) {
    const queue = inputs.filter((polyline) => polyline?.points?.length >= 2).map((polyline) => ({
      points: polyline.points.map((pt) => pt.clone()),
      closed: Boolean(polyline.closed),
    }));
    const result = [];
    while (queue.length) {
      let current = queue.shift();
      let changed = true;
      while (changed) {
        changed = false;
        for (let i = 0; i < queue.length; i += 1) {
          const candidate = queue[i];
          const merged = tryMergePolylines(current, candidate, { preserveDirection, tolerance });
          if (merged) {
            current = merged;
            queue.splice(i, 1);
            changed = true;
            break;
          }
        }
      }
      result.push(current);
    }
    return result.map((polyline) => {
      const normalized = normalizePolylinePoints(polyline.points, { closed: polyline.closed });
      const curve = createCurveFromPoints(normalized, { closed: polyline.closed, samples: normalized.length * 2 });
      if (polyline.closed) {
        curve.closed = true;
      }
      return curve;
    });
  }

  function createLineSegment(start, end) {
    const direction = end.clone().sub(start);
    const length = direction.length();
    return {
      type: 'line',
      start: start.clone(),
      end: end.clone(),
      length,
      direction: length > EPSILON ? direction.clone().divideScalar(length) : new THREE.Vector3(1, 0, 0),
    };
  }

  function ensureSurfacePlane(surfaceInput, fallback = defaultPlane()) {
    if (!surfaceInput) {
      return fallback;
    }
    if (surfaceInput.plane) {
      return ensurePlane(surfaceInput.plane);
    }
    if (surfaceInput.origin && surfaceInput.normal) {
      return ensurePlane({ origin: surfaceInput.origin, normal: surfaceInput.normal });
    }
    if (surfaceInput.point && surfaceInput.normal) {
      return ensurePlane({ origin: surfaceInput.point, normal: surfaceInput.normal });
    }
    if (surfaceInput.points) {
      const points = collectPoints(surfaceInput.points);
      const plane = fitPlaneToPoints(points);
      if (plane) {
        return plane;
      }
    }
    if (surfaceInput.position && surfaceInput.normal) {
      return ensurePlane({ origin: surfaceInput.position, normal: surfaceInput.normal });
    }
    return fallback;
  }

  function projectPointToPlaneAlongDirection(pointInput, plane, directionInput) {
    const point = ensurePoint(pointInput, plane.origin.clone());
    const direction = convertVector(directionInput, plane.zAxis.clone());
    if (direction.lengthSq() < EPSILON) {
      direction.copy(plane.zAxis);
    }
    const normalizedDirection = direction.clone().normalize();
    const relative = point.clone().sub(plane.origin);
    const denominator = normalizedDirection.dot(plane.zAxis);
    if (Math.abs(denominator) < EPSILON) {
      return point.clone();
    }
    const distance = -relative.dot(plane.zAxis) / denominator;
    return point.clone().add(normalizedDirection.multiplyScalar(distance));
  }

  function offsetCurve(curveInput, distanceInput, planeInput, { loose = false, project = null, closedOverride = null } = {}) {
    const curve = ensureCurve(curveInput);
    if (!curve) {
      return [];
    }
    const distance = ensureNumber(distanceInput, 0);
    if (!Number.isFinite(distance) || Math.abs(distance) < EPSILON) {
      return [curve];
    }
    const basePoints = extractCurvePoints(curve, 64);
    if (basePoints.length < 2) {
      return [];
    }
    const closedCurve = closedOverride !== null ? closedOverride : isClosedPolyline(basePoints);
    const plane = planeInput ? ensurePlane(planeInput) : (curve.plane ? ensurePlane(curve.plane) : fitPlaneToPoints(basePoints) ?? defaultPlane());
    const normalized = normalizePolylinePoints(basePoints, { closed: closedCurve });
    const pointsToOffset = closedCurve ? normalized.slice(0, -1) : normalized;
    const offsetPoints = loose
      ? offsetPolylineLoose(pointsToOffset, plane, distance, { closed: closedCurve })
      : offsetPolylinePoints(pointsToOffset, plane, distance, { closed: closedCurve });
    const finalPoints = closedCurve ? [...offsetPoints, offsetPoints[0].clone()] : offsetPoints;
    let projectedPoints = finalPoints;
    if (project) {
      projectedPoints = finalPoints.map((pt) => project(pt));
    }
    const resultCurve = createCurveFromPoints(projectedPoints, { closed: closedCurve, samples: projectedPoints.length * 2 });
    resultCurve.plane = plane;
    return [resultCurve];
  }

  function intersectLines2D(pointA, dirA, pointB, dirB) {
    const det = dirA.x * dirB.y - dirA.y * dirB.x;
    if (Math.abs(det) < EPSILON) {
      return null;
    }
    const dx = pointB.x - pointA.x;
    const dy = pointB.y - pointA.y;
    const t = (dx * dirB.y - dy * dirB.x) / det;
    return {
      x: pointA.x + dirA.x * t,
      y: pointA.y + dirA.y * t,
    };
  }

  function filletPolyline(pointsInput, {
    radius = 0,
    distance = null,
    plane,
    closed = false,
    targetIndices = null,
  } = {}) {
    if (pointsInput.length < 3) {
      return pointsInput.map((pt) => pt.clone());
    }
    const working = normalizePolylinePoints(pointsInput, { closed });
    const planeToUse = plane ?? fitPlaneToPoints(working) ?? defaultPlane();
    const coords = working.map((pt) => planeCoordinates(pt, planeToUse));
    const effectiveIndices = targetIndices instanceof Set ? targetIndices : null;
    const result = [];

    const isEndpoint = (index) => !closed && (index === 0 || index === coords.length - 1);

    for (let i = 0; i < coords.length; i += 1) {
      if (isEndpoint(i)) {
        result.push(coords[i]);
        continue;
      }
      if (effectiveIndices && !effectiveIndices.has(i)) {
        result.push(coords[i]);
        continue;
      }
      const prev = coords[(i - 1 + coords.length) % coords.length];
      const current = coords[i];
      const next = coords[(i + 1) % coords.length];
      const v1 = { x: current.x - prev.x, y: current.y - prev.y };
      const v2 = { x: next.x - current.x, y: next.y - current.y };
      const len1 = Math.hypot(v1.x, v1.y);
      const len2 = Math.hypot(v2.x, v2.y);
      if (len1 < EPSILON || len2 < EPSILON) {
        result.push(current);
        continue;
      }
      const dir1 = { x: v1.x / len1, y: v1.y / len1 };
      const dir2 = { x: v2.x / len2, y: v2.y / len2 };
      const dot = clamp(-(dir1.x * dir2.x + dir1.y * dir2.y), -1, 1);
      const angle = Math.acos(dot);
      if (!Number.isFinite(angle) || angle < 1e-3) {
        result.push(current);
        continue;
      }
      const cross = dir1.x * dir2.y - dir1.y * dir2.x;
      const interiorSign = cross >= 0 ? 1 : -1;
      const trimDistance = distance !== null
        ? Math.min(distance, len1 - EPSILON, len2 - EPSILON)
        : Math.min(len1 - EPSILON, len2 - EPSILON, Math.abs(radius) / Math.tan(angle / 2));
      const effectiveRadius = distance !== null
        ? Math.max(trimDistance * Math.tan(angle / 2), EPSILON)
        : Math.max(Math.abs(radius), EPSILON);
      if (!Number.isFinite(trimDistance) || trimDistance <= EPSILON || !Number.isFinite(effectiveRadius)) {
        result.push(current);
        continue;
      }
      const start = {
        x: current.x - dir1.x * trimDistance,
        y: current.y - dir1.y * trimDistance,
      };
      const end = {
        x: current.x + dir2.x * trimDistance,
        y: current.y + dir2.y * trimDistance,
      };
      const normal1 = interiorSign >= 0 ? { x: -dir1.y, y: dir1.x } : { x: dir1.y, y: -dir1.x };
      const normal2 = interiorSign >= 0 ? { x: -dir2.y, y: dir2.x } : { x: dir2.y, y: -dir2.x };
      const center = intersectLines2D(start, normal1, end, normal2);
      if (!center) {
        result.push(start, end);
        continue;
      }
      const radiusVectorStart = { x: start.x - center.x, y: start.y - center.y };
      const radiusVectorEnd = { x: end.x - center.x, y: end.y - center.y };
      let startAngle = Math.atan2(radiusVectorStart.y, radiusVectorStart.x);
      let endAngle = Math.atan2(radiusVectorEnd.y, radiusVectorEnd.x);
      let delta = endAngle - startAngle;
      if (interiorSign >= 0 && delta < 0) {
        delta += Math.PI * 2;
      } else if (interiorSign < 0 && delta > 0) {
        delta -= Math.PI * 2;
      }
      const segments = Math.max(3, Math.ceil(Math.abs(delta) / (Math.PI / 18)));
      if (!result.length) {
        result.push(start);
      } else {
        const prevPoint = result[result.length - 1];
        if (Math.hypot(prevPoint.x - start.x, prevPoint.y - start.y) > EPSILON) {
          result.push(start);
        }
      }
      for (let step = 1; step <= segments; step += 1) {
        const t = step / segments;
        const angleValue = startAngle + delta * t;
        result.push({
          x: center.x + Math.cos(angleValue) * effectiveRadius,
          y: center.y + Math.sin(angleValue) * effectiveRadius,
        });
      }
    }

    if (!closed) {
      const first = coords[0];
      if (result.length === 0 || Math.hypot(result[0].x - first.x, result[0].y - first.y) > EPSILON) {
        result.unshift(first);
      }
      const last = coords[coords.length - 1];
      if (Math.hypot(result[result.length - 1].x - last.x, result[result.length - 1].y - last.y) > EPSILON) {
        result.push(last);
      }
    } else if (result.length) {
      const first = result[0];
      const last = result[result.length - 1];
      if (Math.hypot(first.x - last.x, first.y - last.y) > EPSILON) {
        result.push(first);
      }
    }

    return result.map((coord) => applyPlane(planeToUse, coord.x, coord.y, 0));
  }
  function collapsePolylineShortSegments(pointsInput, tolerance, { closed = false } = {}) {
    const points = normalizePolylinePoints(pointsInput, { closed });
    if (points.length <= 1) {
      return { points: points.map((pt) => pt.clone()), collapsed: 0 };
    }
    const result = [points[0].clone()];
    let collapsed = 0;
    for (let i = 1; i < points.length; i += 1) {
      const current = points[i];
      const previous = result[result.length - 1];
      if (previous.distanceTo(current) < tolerance) {
        collapsed += 1;
        continue;
      }
      result.push(current.clone());
    }
    if (closed && result.length >= 2) {
      const first = result[0];
      const last = result[result.length - 1];
      if (first.distanceTo(last) < tolerance) {
        result[result.length - 1] = first.clone();
      } else {
        result.push(first.clone());
      }
    }
    return { points: result, collapsed };
  }

  function explodeCurveSegments(curve, { recursive = false, segments = 32 } = {}) {
    const sampleCount = recursive ? Math.max(segments * 2, 64) : Math.max(segments, 16);
    const points = sampleCurvePoints(curve, sampleCount);
    const segmentsList = [];
    for (let i = 0; i < points.length - 1; i += 1) {
      segmentsList.push(createLineSegment(points[i], points[i + 1]));
    }
    const vertices = points.map((pt) => pt.clone());
    return { segments: segmentsList, vertices };
  }

  function computeCurveLength(curve) {
    if (!curve) {
      return 0;
    }
    if (Number.isFinite(curve.length)) {
      return curve.length;
    }
    const points = extractCurvePoints(curve, 128);
    return computePolylineLength(points);
  }

  function sampleNormalizedPoints(curve, count) {
    const samples = [];
    const safeCount = Math.max(1, count);
    for (let i = 0; i <= safeCount; i += 1) {
      const t = safeCount === 0 ? 0 : i / safeCount;
      const point = curvePointAt(curve, t);
      if (point) {
        samples.push(point.clone());
      }
    }
    return samples;
  }

  function computeCurveFrames(curve, count, { alignNormal } = {}) {
    const frames = [];
    const parameters = [];
    if (!curve) {
      return { frames, parameters };
    }
    const samples = Math.max(1, count);
    const points = sampleCurvePoints(curve, samples * 2);
    const plane = fitPlaneToPoints(points) ?? defaultPlane();
    let referenceNormal = alignNormal ? plane.zAxis.clone() : null;
    for (let i = 0; i <= samples; i += 1) {
      const t = samples === 0 ? 0 : i / samples;
      const point = curvePointAt(curve, t) ?? points[i] ?? plane.origin.clone();
      let tangent = curveTangentAt(curve, t);
      if (tangent.lengthSq() < EPSILON) {
        const prev = curvePointAt(curve, clamp01(t - 1e-3)) ?? point.clone();
        tangent = point.clone().sub(prev);
      }
      tangent = normalizeVector(tangent, new THREE.Vector3(1, 0, 0));
      let normal = referenceNormal ? referenceNormal.clone() : plane.zAxis.clone();
      let yAxis = normal.clone().cross(tangent);
      if (yAxis.lengthSq() < EPSILON) {
        normal = orthogonalVector(tangent);
        yAxis = normal.clone().cross(tangent);
      }
      yAxis.normalize();
      const zAxis = tangent.clone().cross(yAxis).normalize();
      if (referenceNormal) {
        referenceNormal = zAxis.clone();
      }
      const xAxis = tangent.clone();
      frames.push({
        origin: point.clone(),
        xAxis,
        yAxis,
        zAxis,
      });
      parameters.push(t);
    }
    return { frames, parameters };
  }

  function rebuildCurve(curve, { count, degree, preserveTangents } = {}) {
    const safeCount = Math.max(2, Math.round(count ?? 10));
    const points = [];
    const tangents = [];
    for (let i = 0; i < safeCount; i += 1) {
      const t = safeCount === 1 ? 0 : i / (safeCount - 1);
      const point = curvePointAt(curve, t) ?? new THREE.Vector3();
      points.push(point);
      tangents.push(curveTangentAt(curve, t));
    }
    if (preserveTangents && points.length >= 2) {
      const startOffset = tangents[0].clone().multiplyScalar(points[1].distanceTo(points[0]) * 0.25);
      points[0] = points[0].clone().sub(startOffset);
      const endIndex = points.length - 1;
      const endOffset = tangents[endIndex].clone().multiplyScalar(points[endIndex].distanceTo(points[endIndex - 1]) * 0.25);
      points[endIndex] = points[endIndex].clone().add(endOffset);
    }
    const closed = isClosedPolyline(points);
    const rebuilt = createCurveFromPoints(points, {
      closed,
      samples: safeCount * Math.max(4, degree ?? 3),
    });
    return rebuilt;
  }

  function fitCurveWithTolerance(curve, tolerance, degree = 3) {
    const samples = extractCurvePoints(curve, degree * 8);
    const simplified = rdpSimplify(samples, tolerance, 0);
    return createCurveFromPoints(simplified, { closed: isClosedPolyline(samples), samples: simplified.length * 4 });
  }

  function reverseCurve(curve) {
    const points = extractCurvePoints(curve, 64);
    const reversed = points.map((pt) => pt.clone()).reverse();
    const newCurve = createCurveFromPoints(reversed, { closed: isClosedPolyline(points), samples: points.length * 2 });
    return newCurve;
  }

  function adjustCurveSeam(curve, parameter) {
    const normalized = clamp01(parameter);
    const points = extractCurvePoints(curve, 128);
    if (!isClosedPolyline(points) || points.length < 3) {
      return curve;
    }
    const rotationIndex = Math.floor(normalized * (points.length - 1));
    const rotated = [];
    for (let i = 0; i < points.length - 1; i += 1) {
      rotated.push(points[(rotationIndex + i) % (points.length - 1)].clone());
    }
    rotated.push(rotated[0].clone());
    return createCurveFromPoints(rotated, { closed: true, samples: rotated.length * 2 });
  }

  function curveSummary(curve) {
    if (!curve) {
      return {};
    }
    const length = computeCurveLength(curve);
    const domain = curve.domain ?? createDomain(0, 1);
    return { curve, length, domain };
  }
  function registerOffsetCurve() {
    register('{1a38d325-98de-455c-93f1-bca431bc1243}', {
      type: 'curve',
      pinMap: {
        inputs: {
          C: 'curve', Curve: 'curve', curve: 'curve',
          D: 'distance', Distance: 'distance', distance: 'distance',
          P: 'plane', Plane: 'plane', plane: 'plane',
          Corners: 'cornerType', corners: 'cornerType',
        },
        outputs: { C: 'curves', curve: 'curves', Curves: 'curves' },
      },
      eval: ({ inputs }) => {
        const curves = offsetCurve(inputs.curve, inputs.distance, inputs.plane);
        if (!curves.length) {
          return {};
        }
        return { curves };
      },
    });
  }

  function registerFlipCurve() {
    register('{22990b1f-9be6-477c-ad89-f775cd347105}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', Curve: 'curve', curve: 'curve', G: 'guide', Guide: 'guide', guide: 'guide' },
        outputs: { C: 'curve', curve: 'curve', F: 'flipped', Flag: 'flipped', flag: 'flipped' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        let shouldFlip = true;
        const points = extractCurvePoints(curve, 64);
        const closed = isClosedPolyline(points);
        if (closed) {
          shouldFlip = false;
        } else {
          const start = points[0];
          const end = points[points.length - 1];
          const guide = ensureCurve(inputs.guide);
          if (guide) {
            const guidePoints = extractCurvePoints(guide, 64);
            if (guidePoints.length >= 2) {
              const guideStart = guidePoints[0];
              const guideEnd = guidePoints[guidePoints.length - 1];
              const startToStart = start.distanceTo(guideStart);
              const startToEnd = start.distanceTo(guideEnd);
              shouldFlip = startToEnd < startToStart;
            }
          }
        }
        const resultCurve = shouldFlip ? reverseCurve(curve) : curve;
        return { curve: resultCurve, flipped: shouldFlip };
      },
    });
  }

  function registerCurveToPolyline() {
    register('{2956d989-3599-476f-bc92-1d847aff98b6}', {
      type: 'curve',
      pinMap: {
        inputs: {
          C: 'curve', Curve: 'curve', curve: 'curve',
          Td: 'distanceTolerance', 'Tolerance (distance)': 'distanceTolerance',
          Ta: 'angleTolerance', 'Tolerance (angle)': 'angleTolerance',
          'E-': 'minEdge', MinEdge: 'minEdge',
          'E+': 'maxEdge', MaxEdge: 'maxEdge',
        },
        outputs: { P: 'polyline', polyline: 'polyline', Polyline: 'polyline', S: 'segments', Segments: 'segments' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const distanceTolerance = Math.max(ensureNumber(inputs.distanceTolerance, 0.01), 0.0001);
        const angleTolerance = Math.max(ensureNumber(inputs.angleTolerance, 0), 0);
        const minEdge = Math.max(ensureNumber(inputs.minEdge, 0), 0);
        const maxEdge = Math.max(ensureNumber(inputs.maxEdge, Infinity), minEdge || EPSILON);
        const closed = Boolean(curve.closed) || isClosedPolyline(extractCurvePoints(curve, 32));
        const points = approximateCurveWithTolerance(curve, {
          distanceTolerance,
          angleTolerance,
          minEdge,
          maxEdge,
          closed,
        });
        if (points.length < 2) {
          return {};
        }
        const polyline = createCurveFromPoints(points, { closed, samples: points.length - 1 });
        return { polyline, segments: Math.max(points.length - 1, 1) };
      },
    });
  }

  function registerCurveFillet() {
    register('{2f407944-81c3-4062-a485-276454ec4b8c}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', Curve: 'curve', curve: 'curve', R: 'radius', Radius: 'radius', radius: 'radius' },
        outputs: { C: 'curve', curve: 'curve' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const radius = Math.max(ensureNumber(inputs.radius, 0), 0);
        if (radius <= EPSILON) {
          return { curve };
        }
        const points = extractCurvePoints(curve, 128);
        if (points.length < 3) {
          return { curve };
        }
        const closed = isClosedPolyline(points);
        const plane = curve.plane ? ensurePlane(curve.plane) : fitPlaneToPoints(points) ?? defaultPlane();
        const filleted = filletPolyline(points, { radius, plane, closed });
        if (!filleted.length) {
          return { curve };
        }
        const resultCurve = createCurveFromPoints(filleted, { closed, samples: filleted.length * 2 });
        resultCurve.plane = plane;
        return { curve: resultCurve };
      },
    });
  }

  function registerSeam() {
    register('{42ad8dc1-b0c0-40df-91f5-2c46e589e6c2}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', Curve: 'curve', curve: 'curve', Seam: 'seam', seam: 'seam', t: 'seam' },
        outputs: { C: 'curve', curve: 'curve' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const seam = ensureNumber(inputs.seam, 0);
        const adjusted = adjustCurveSeam(curve, seam);
        return { curve: adjusted };
      },
    });
  }

  function registerSmoothPolyline() {
    register('{5c5fbc42-3e1d-4081-9cf1-148d0b1d9610}', {
      type: 'curve',
      pinMap: {
        inputs: { P: 'polyline', Polyline: 'polyline', polyline: 'polyline', S: 'strength', Strength: 'strength', T: 'iterations', Times: 'iterations' },
        outputs: { P: 'polyline', polyline: 'polyline', Polyline: 'polyline' },
      },
      eval: ({ inputs }) => {
        const polylineInput = ensureCurve(inputs.polyline);
        const points = polylineInput ? extractCurvePoints(polylineInput, 128) : collectPoints(inputs.polyline);
        if (points.length < 2) {
          return {};
        }
        const strength = clamp(ensureNumber(inputs.strength, 0.5), 0, 1);
        const iterations = Math.max(Math.round(ensureNumber(inputs.iterations, 1)), 0);
        const closed = isClosedPolyline(points);
        const smoothed = smoothPolylinePoints(points, strength, iterations, { closed });
        const result = createCurveFromPoints(smoothed, { closed, samples: smoothed.length * 2 });
        return { polyline: result };
      },
    });
  }

  function registerExtendCurve() {
    register('{62cc9684-6a39-422e-aefa-ed44643557b9}', {
      type: 'curve',
      pinMap: {
        inputs: {
          C: 'curve', Curve: 'curve', curve: 'curve',
          T: 'extensionType', Type: 'extensionType',
          L0: 'startLength', Start: 'startLength',
          L1: 'endLength', End: 'endLength',
        },
        outputs: { C: 'curve', curve: 'curve' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const rawStartLength = ensureNumber(inputs.startLength, 0);
        const rawEndLength = ensureNumber(inputs.endLength, 0);
        const startLength = Number.isFinite(rawStartLength) ? rawStartLength : 0;
        const endLength = Number.isFinite(rawEndLength) ? rawEndLength : 0;
        if (Math.abs(startLength) <= EPSILON && Math.abs(endLength) <= EPSILON) {
          return { curve };
        }
        const extendedPoints = extendCurvePoints(curve, startLength, endLength);
        const resultCurve = createCurveFromPoints(extendedPoints, { closed: isClosedPolyline(extendedPoints), samples: extendedPoints.length * 2 });
        return { curve: resultCurve };
      },
    });
  }

  function registerPullCurve() {
    register('{6b5812f5-bb36-4d74-97fc-5a1f2f77452d}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', Curve: 'curve', curve: 'curve', S: 'surface', Surface: 'surface', surface: 'surface' },
        outputs: { C: 'curve', curve: 'curve' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const points = extractCurvePoints(curve, 128);
        if (!points.length) {
          return {};
        }
        const fallbackPlane = fitPlaneToPoints(points) ?? defaultPlane();
        const plane = ensureSurfacePlane(inputs.surface, fallbackPlane);
        const projected = points.map((pt) => projectPointToPlaneAlongDirection(pt, plane, plane.zAxis));
        const pulled = createCurveFromPoints(projected, { closed: isClosedPolyline(points), samples: projected.length * 2 });
        pulled.plane = plane;
        return { curve: pulled };
      },
    });
  }

  function registerPerpFramesObsolete() {
    register('{6da4b70c-ce98-4d52-a2bb-2fadccf39da0}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', Curve: 'curve', curve: 'curve', N: 'count', Number: 'count', count: 'count' },
        outputs: { F: 'frames', frames: 'frames', t: 'parameters', Parameters: 'parameters' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const count = Math.max(1, Math.round(ensureNumber(inputs.count, 10)));
        const { frames, parameters } = computeCurveFrames(curve, count, { alignNormal: true });
        return { frames, parameters };
      },
    });
  }

  function registerFilletDistance() {
    register('{6fb21315-a032-400e-a80f-248687f5507f}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', Curve: 'curve', curve: 'curve', D: 'distance', Distance: 'distance', distance: 'distance' },
        outputs: { C: 'curve', curve: 'curve' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const distance = Math.max(ensureNumber(inputs.distance, 0), 0);
        if (distance <= EPSILON) {
          return { curve };
        }
        const points = extractCurvePoints(curve, 128);
        if (points.length < 3) {
          return { curve };
        }
        const closed = isClosedPolyline(points);
        const plane = curve.plane ? ensurePlane(curve.plane) : fitPlaneToPoints(points) ?? defaultPlane();
        const filleted = filletPolyline(points, { distance, plane, closed });
        if (!filleted.length) {
          return { curve };
        }
        const resultCurve = createCurveFromPoints(filleted, { closed, samples: filleted.length * 2 });
        resultCurve.plane = plane;
        return { curve: resultCurve };
      },
    });
  }

  function registerJoinCurves() {
    register('{8073a420-6bec-49e3-9b18-367f6fd76ac3}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curves', curves: 'curves', Curves: 'curves', P: 'preserve', Preserve: 'preserve', preserve: 'preserve' },
        outputs: { C: 'curves', curves: 'curves', Curves: 'curves' },
      },
      eval: ({ inputs }) => {
        const curveInputs = ensureArray(inputs.curves).map((entry) => ensureCurve(entry)).filter(Boolean);
        if (!curveInputs.length) {
          return {};
        }
        const polylines = curveInputs.map((curve) => ({
          points: extractCurvePoints(curve, 256),
          closed: Boolean(curve.closed) || isClosedPolyline(extractCurvePoints(curve, 32)),
        }));
        const preserveDirection = ensureBoolean(inputs.preserve, false);
        const joined = joinPolylines(polylines, { preserveDirection });
        if (!joined.length) {
          return {};
        }
        return { curves: joined };
      },
    });
  }

  function registerOffsetCurveLoose() {
    register('{80e55fc2-933b-4bfb-a353-12358786dba8}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', Curve: 'curve', curve: 'curve', D: 'distance', Distance: 'distance', distance: 'distance', P: 'plane', Plane: 'plane', plane: 'plane' },
        outputs: { C: 'curve', curve: 'curve' },
      },
      eval: ({ inputs }) => {
        const curves = offsetCurve(inputs.curve, inputs.distance, inputs.plane, { loose: true });
        if (!curves.length) {
          return {};
        }
        return { curve: curves[0] };
      },
    });
  }

  function registerReducePolyline() {
    register('{884646c3-0e70-4ad1-90c5-42601ee26450}', {
      type: 'curve',
      pinMap: {
        inputs: { P: 'polyline', Polyline: 'polyline', polyline: 'polyline', T: 'tolerance', Tolerance: 'tolerance' },
        outputs: { P: 'polyline', polyline: 'polyline', R: 'reduction', Reduction: 'reduction' },
      },
      eval: ({ inputs }) => {
        const polyline = ensureCurve(inputs.polyline);
        const points = polyline ? extractCurvePoints(polyline, 256) : collectPoints(inputs.polyline);
        if (points.length < 2) {
          return {};
        }
        const tolerance = Math.max(ensureNumber(inputs.tolerance, 0.01), 0);
        const simplified = rdpSimplify(points, tolerance, 0);
        const closed = isClosedPolyline(points);
        const result = createCurveFromPoints(simplified, { closed, samples: simplified.length * 2 });
        const reduction = Math.max(points.length - simplified.length, 0);
        return { polyline: result, reduction };
      },
    });
  }

  function registerSimplifyCurve() {
    register('{922dc7e5-0f0e-4c21-ae4b-f6a8654e63f6}', {
      type: 'curve',
      pinMap: {
        inputs: {
          C: 'curve', Curve: 'curve', curve: 'curve',
          t: 'tolerance', tolerance: 'tolerance', Tolerance: 'tolerance',
          a: 'angleTolerance', 'Angle Tolerance': 'angleTolerance',
        },
        outputs: { C: 'curve', curve: 'curve', S: 'simplified', Simplified: 'simplified' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const tolerance = Math.max(ensureNumber(inputs.tolerance, 0.01), 0);
        const angleTolerance = Math.max(ensureNumber(inputs.angleTolerance, 0), 0);
        const points = extractCurvePoints(curve, 256);
        const simplifiedPoints = rdpSimplify(points, tolerance, angleTolerance);
        const changed = simplifiedPoints.length !== points.length;
        const simplifiedCurve = createCurveFromPoints(simplifiedPoints, {
          closed: isClosedPolyline(points),
          samples: simplifiedPoints.length * 2,
        });
        return { curve: simplifiedCurve, simplified: changed };
      },
    });
  }

  function registerRebuildCurve() {
    register('{9333c5b3-11f9-423c-bbb5-7e5156430219}', {
      type: 'curve',
      pinMap: {
        inputs: {
          C: 'curve', Curve: 'curve', curve: 'curve',
          D: 'degree', Degree: 'degree',
          N: 'count', Count: 'count',
          T: 'tangents', Tangents: 'tangents',
        },
        outputs: { C: 'curve', curve: 'curve' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const count = Math.max(2, Math.round(ensureNumber(inputs.count, 10)));
        const degree = Math.max(1, Math.round(ensureNumber(inputs.degree, 3)));
        const preserveTangents = ensureBoolean(inputs.tangents, false);
        const rebuilt = rebuildCurve(curve, { count, degree, preserveTangents });
        return { curve: rebuilt };
      },
    });
  }

  function registerDivideCurveObsolete() {
    register('{93b1066f-060e-440d-a638-aae8cbe7acb7}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', Curve: 'curve', curve: 'curve', N: 'count', Number: 'count' },
        outputs: { P: 'points', points: 'points', T: 'tangents', tangents: 'tangents', t: 'parameters', Parameters: 'parameters' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const count = Math.max(1, Math.round(ensureNumber(inputs.count, 10)));
        const points = [];
        const tangents = [];
        const parameters = [];
        for (let i = 0; i <= count; i += 1) {
          const t = count === 0 ? 0 : i / count;
          const point = curvePointAt(curve, t);
          const tangent = curveTangentAt(curve, t);
          if (point) {
            points.push(point.clone());
            tangents.push(tangent.clone());
            parameters.push(t);
          }
        }
        return { points, tangents, parameters };
      },
    });
  }

  function registerFitCurve() {
    register('{a3f9f19e-3e6c-4ac7-97c3-946de32c3e8e}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', Curve: 'curve', curve: 'curve', D: 'degree', Degree: 'degree', Ft: 'tolerance', Tolerance: 'tolerance' },
        outputs: { C: 'curve', curve: 'curve' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const tolerance = Math.max(ensureNumber(inputs.tolerance, 0.01), 0.0001);
        const degree = Math.max(1, Math.round(ensureNumber(inputs.degree, 3)));
        const fitted = fitCurveWithTolerance(curve, tolerance, degree);
        return { curve: fitted };
      },
    });
  }

  function registerExplodeCurve() {
    register('{afb96615-c59a-45c9-9cac-e27acb1c7ca0}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', Curve: 'curve', curve: 'curve', R: 'recursive', Recursive: 'recursive' },
        outputs: { S: 'segments', segments: 'segments', V: 'vertices', vertices: 'vertices' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const recursive = ensureBoolean(inputs.recursive, false);
        const { segments, vertices } = explodeCurveSegments(curve, { recursive });
        return { segments, vertices };
      },
    });
  }

  function registerOffsetOnSurface() {
    register('{b6f5cb51-f260-4c74-bf73-deb47de1bf91}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', Curve: 'curve', curve: 'curve', D: 'distance', Distance: 'distance', distance: 'distance', S: 'surface', Surface: 'surface', surface: 'surface' },
        outputs: { C: 'curves', curves: 'curves', Curve: 'curves' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const basePoints = extractCurvePoints(curve, 128);
        if (!basePoints.length) {
          return {};
        }
        const basePlane = fitPlaneToPoints(basePoints) ?? defaultPlane();
        const surfacePlane = ensureSurfacePlane(inputs.surface, basePlane);
        const projector = (pt) => projectPointToPlaneAlongDirection(pt, surfacePlane, surfacePlane.zAxis);
        const curves = offsetCurve(curve, inputs.distance, surfacePlane, { project: projector });
        if (!curves.length) {
          return {};
        }
        return { curves };
      },
    });
  }

  function registerPolylineCollapse() {
    register('{be298882-28c9-45b1-980d-7192a531c9a9}', {
      type: 'curve',
      pinMap: {
        inputs: { P: 'polyline', Polyline: 'polyline', polyline: 'polyline', t: 'tolerance', Tolerance: 'tolerance' },
        outputs: { Pl: 'polyline', polyline: 'polyline', P: 'polyline', N: 'collapsed', Count: 'collapsed' },
      },
      eval: ({ inputs }) => {
        const polyline = ensureCurve(inputs.polyline);
        const points = polyline ? extractCurvePoints(polyline, 256) : collectPoints(inputs.polyline);
        if (points.length < 2) {
          return {};
        }
        const tolerance = Math.max(ensureNumber(inputs.tolerance, 0.01), 0);
        const closed = isClosedPolyline(points);
        const { points: collapsedPoints, collapsed } = collapsePolylineShortSegments(points, tolerance, { closed });
        const collapsedCurve = createCurveFromPoints(collapsedPoints, { closed, samples: collapsedPoints.length * 2 });
        return { polyline: collapsedCurve, collapsed };
      },
    });
  }

  function registerOffsetLoose3D() {
    register('{c6fe61e7-25e2-4333-9172-f4e2a123fcfe}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', Curve: 'curve', curve: 'curve', D: 'distance', Distance: 'distance', distance: 'distance' },
        outputs: { C: 'curve', curve: 'curve' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const distance = ensureNumber(inputs.distance, 0);
        const points = extractCurvePoints(curve, 128);
        if (!points.length) {
          return {};
        }
        const plane = fitPlaneToPoints(points) ?? defaultPlane();
        const offsetPoints = points.map((pt) => pt.clone().add(plane.zAxis.clone().normalize().multiplyScalar(distance)));
        const offsetCurve = createCurveFromPoints(offsetPoints, { closed: isClosedPolyline(points), samples: offsetPoints.length * 2 });
        return { curve: offsetCurve };
      },
    });
  }

  function registerFilletParameter() {
    register('{c92cdfc8-3df8-4c4e-abc1-ede092a0aa8a}', {
      type: 'curve',
      pinMap: {
        inputs: { C: 'curve', Curve: 'curve', curve: 'curve', t: 'parameter', Parameter: 'parameter', R: 'radius', Radius: 'radius', radius: 'radius' },
        outputs: { C: 'curve', curve: 'curve', t: 'parameter', Parameter: 'parameter' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const radius = Math.max(ensureNumber(inputs.radius, 0), 0);
        const normalizedParam = clamp01(ensureNumber(inputs.parameter, 0));
        if (radius <= EPSILON) {
          return { curve, parameter: normalizedParam };
        }
        const points = extractCurvePoints(curve, 128);
        if (points.length < 3) {
          return { curve, parameter: normalizedParam };
        }
        const closed = isClosedPolyline(points);
        const plane = curve.plane ? ensurePlane(curve.plane) : fitPlaneToPoints(points) ?? defaultPlane();
        let index = Math.round(normalizedParam * (points.length - 1));
        if (!closed) {
          index = Math.min(Math.max(index, 1), points.length - 2);
        } else {
          index = (index + (points.length - 1)) % (points.length - 1);
        }
        const targetIndices = new Set([index]);
        const filleted = filletPolyline(points, { radius, plane, closed, targetIndices });
        const resultCurve = createCurveFromPoints(filleted, { closed, samples: filleted.length * 2 });
        resultCurve.plane = plane;
        return { curve: resultCurve, parameter: normalizedParam };
      },
    });
  }

  function registerProjectCurve() {
    register('{d7ee52ff-89b8-4d1a-8662-3e0dd391d0af}', {
      type: 'curve',
      pinMap: {
        inputs: {
          C: 'curve', Curve: 'curve', curve: 'curve',
          B: 'brep', Brep: 'brep', brep: 'brep',
          D: 'direction', Direction: 'direction', direction: 'direction',
        },
        outputs: { C: 'curves', curves: 'curves', Curve: 'curves' },
      },
      eval: ({ inputs }) => {
        const curve = ensureCurve(inputs.curve);
        if (!curve) {
          return {};
        }
        const points = extractCurvePoints(curve, 128);
        if (!points.length) {
          return {};
        }
        const plane = ensureSurfacePlane(inputs.brep, fitPlaneToPoints(points) ?? defaultPlane());
        const direction = convertVector(inputs.direction, plane.zAxis.clone());
        if (direction.lengthSq() < EPSILON) {
          direction.copy(plane.zAxis);
        }
        const projected = points.map((pt) => projectPointToPlaneAlongDirection(pt, plane, direction));
        const projectedCurve = createCurveFromPoints(projected, { closed: isClosedPolyline(points), samples: projected.length * 2 });
        return { curves: [projectedCurve] };
      },
    });
  }

  function registerOffsetPolyline() {
    register('{e2c6cab3-91ea-4c01-900c-646642d3e436}', {
      type: 'curve',
      pinMap: {
        inputs: { P: 'polyline', Polyline: 'polyline', polyline: 'polyline', D: 'distance', Distance: 'distance', distance: 'distance' },
        outputs: { O: 'offsets', Offset: 'offsets', offsets: 'offsets', V: 'valid', Valid: 'valid', valid: 'valid' },
      },
      eval: ({ inputs }) => {
        const polyline = ensureCurve(inputs.polyline);
        if (!polyline) {
          return {};
        }
        const points = extractCurvePoints(polyline, 128);
        if (points.length < 2) {
          return {};
        }
        const closed = isClosedPolyline(points);
        const plane = polyline.plane ? ensurePlane(polyline.plane) : fitPlaneToPoints(points) ?? defaultPlane();
        const curves = offsetCurve(polyline, inputs.distance, plane, { closedOverride: closed });
        if (!curves.length) {
          return {};
        }
        return { offsets: curves, valid: curves.map(() => true) };
      },
    });
  }

  registerOffsetCurve();
  registerFlipCurve();
  registerCurveToPolyline();
  registerCurveFillet();
  registerSeam();
  registerSmoothPolyline();
  registerExtendCurve();
  registerPullCurve();
  registerPerpFramesObsolete();
  registerFilletDistance();
  registerJoinCurves();
  registerOffsetCurveLoose();
  registerReducePolyline();
  registerSimplifyCurve();
  registerRebuildCurve();
  registerDivideCurveObsolete();
  registerFitCurve();
  registerExplodeCurve();
  registerOffsetOnSurface();
  registerPolylineCollapse();
  registerOffsetLoose3D();
  registerFilletParameter();
  registerProjectCurve();
  registerOffsetPolyline();
}

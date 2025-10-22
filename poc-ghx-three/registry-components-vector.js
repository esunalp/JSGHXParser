import * as THREE from 'three';

export function registerVectorPointComponents({ register, toNumber, toVector3 }, options = {}) {
  if (typeof register !== 'function') {
    throw new Error('register function is required to register vector point components.');
  }
  if (typeof toNumber !== 'function') {
    throw new Error('toNumber function is required to register vector point components.');
  }
  if (typeof toVector3 !== 'function') {
    throw new Error('toVector3 function is required to register vector point components.');
  }

  const {
    includeFieldComponents = true,
    includePointComponents = true,
    includePlaneComponents = false,
    includeVectorComponents = true,
  } = options;

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

  function toBooleanFlag(value, fallback = false) {
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
      return true;
    }
    if (Array.isArray(value)) {
      if (!value.length) {
        return fallback;
      }
      return toBooleanFlag(value[value.length - 1], fallback);
    }
    if (typeof value === 'object') {
      if ('value' in value) {
        return toBooleanFlag(value.value, fallback);
      }
      if ('values' in value) {
        return toBooleanFlag(value.values, fallback);
      }
    }
    return Boolean(value);
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

  function collectVectors(input) {
    const vectors = [];

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
        vectors.push(value.clone());
        return;
      }
      if (typeof value === 'object') {
        if ('vector' in value) {
          visit(value.vector);
          return;
        }
        if ('value' in value) {
          visit(value.value);
          return;
        }
        if ('direction' in value) {
          visit(value.direction);
          return;
        }
        if ('normal' in value) {
          visit(value.normal);
          return;
        }
        if ('vectors' in value) {
          visit(value.vectors);
          return;
        }
        if ('values' in value) {
          visit(value.values);
          return;
        }
        if ('x' in value || 'y' in value || 'z' in value) {
          vectors.push(toVector3(value, new THREE.Vector3()));
          return;
        }
      }
    }

    visit(input);
    return vectors;
  }

  function ensureColor(value, fallback = new THREE.Color(1, 1, 1)) {
    const fallbackColor = fallback?.isColor ? fallback : new THREE.Color().set(fallback ?? 0xffffff);
    const color = new THREE.Color();

    if (value === undefined || value === null) {
      color.copy(fallbackColor);
      return color;
    }

    if (value?.isColor) {
      return value.clone();
    }

    if (Array.isArray(value)) {
      if (value.length >= 3) {
        const r = THREE.MathUtils.clamp(toNumber(value[0], fallbackColor.r), 0, 1);
        const g = THREE.MathUtils.clamp(toNumber(value[1], fallbackColor.g), 0, 1);
        const b = THREE.MathUtils.clamp(toNumber(value[2], fallbackColor.b), 0, 1);
        color.setRGB(r, g, b);
        return color;
      }
      if (value.length === 1) {
        return ensureColor(value[0], fallbackColor);
      }
    }

    if (typeof value === 'object') {
      if ('color' in value) {
        return ensureColor(value.color, fallbackColor);
      }
      if ('value' in value) {
        return ensureColor(value.value, fallbackColor);
      }
      if ('r' in value || 'g' in value || 'b' in value) {
        const r = THREE.MathUtils.clamp(toNumber(value.r ?? value.red, fallbackColor.r), 0, 1);
        const g = THREE.MathUtils.clamp(toNumber(value.g ?? value.green, fallbackColor.g), 0, 1);
        const b = THREE.MathUtils.clamp(toNumber(value.b ?? value.blue, fallbackColor.b), 0, 1);
        color.setRGB(r, g, b);
        return color;
      }
      if ('hex' in value) {
        try {
          color.set(value.hex);
          return color;
        } catch (error) {
          // ignore and fall back
        }
      }
    }

    try {
      color.set(value);
      return color;
    } catch (error) {
      color.copy(fallbackColor);
      return color;
    }
  }

  function normalizeBoundsVectors(minVec, maxVec) {
    const min = new THREE.Vector3(
      Math.min(minVec.x, maxVec.x),
      Math.min(minVec.y, maxVec.y),
      Math.min(minVec.z, maxVec.z),
    );
    const max = new THREE.Vector3(
      Math.max(minVec.x, maxVec.x),
      Math.max(minVec.y, maxVec.y),
      Math.max(minVec.z, maxVec.z),
    );
    return { min, max };
  }

  function ensureBounds(input) {
    if (input === undefined || input === null) {
      return null;
    }
    if (input?.isBox3) {
      return normalizeBoundsVectors(
        input.min ?? new THREE.Vector3(),
        input.max ?? new THREE.Vector3(1, 1, 1),
      );
    }
    if (Array.isArray(input)) {
      if (input.length >= 2) {
        const min = toVector3(input[0], new THREE.Vector3());
        const max = toVector3(input[1], min.clone());
        return normalizeBoundsVectors(min, max);
      }
      if (input.length === 1) {
        return ensureBounds(input[0]);
      }
    }
    if (typeof input === 'object') {
      if ('bounds' in input) {
        return ensureBounds(input.bounds);
      }
      if ('min' in input || 'max' in input || 'minimum' in input || 'maximum' in input) {
        const min = toVector3(input.min ?? input.minimum ?? input.lower ?? new THREE.Vector3(), new THREE.Vector3());
        const max = toVector3(input.max ?? input.maximum ?? input.upper ?? min.clone(), min.clone());
        return normalizeBoundsVectors(min, max);
      }
      if ('center' in input || 'size' in input || 'extent' in input || 'extents' in input) {
        const center = toVector3(input.center ?? input.origin ?? new THREE.Vector3(), new THREE.Vector3());
        let sizeValue = input.size ?? input.extents ?? input.extent ?? 0;
        let sx = toNumber(sizeValue?.x ?? sizeValue?.width ?? sizeValue?.w ?? sizeValue?.[0], Number.NaN);
        let sy = toNumber(sizeValue?.y ?? sizeValue?.height ?? sizeValue?.h ?? sizeValue?.[1], Number.NaN);
        let sz = toNumber(sizeValue?.z ?? sizeValue?.depth ?? sizeValue?.d ?? sizeValue?.[2], Number.NaN);
        if (!Number.isFinite(sx)) sx = toNumber(sizeValue, 1);
        if (!Number.isFinite(sy)) sy = sx;
        if (!Number.isFinite(sz)) sz = sx;
        const half = new THREE.Vector3(Math.abs(sx) / 2, Math.abs(sy) / 2, Math.abs(sz) / 2);
        return normalizeBoundsVectors(center.clone().sub(half), center.clone().add(half));
      }
    }
    if (typeof input === 'number') {
      const half = Math.abs(input) / 2;
      return normalizeBoundsVectors(
        new THREE.Vector3(-half, -half, -half),
        new THREE.Vector3(half, half, half),
      );
    }
    return null;
  }

  function mergeBounds(boundsA, boundsB) {
    if (!boundsA && !boundsB) {
      return null;
    }
    if (!boundsA) {
      return {
        min: boundsB.min.clone(),
        max: boundsB.max.clone(),
      };
    }
    if (!boundsB) {
      return {
        min: boundsA.min.clone(),
        max: boundsA.max.clone(),
      };
    }
    return {
      min: new THREE.Vector3(
        Math.min(boundsA.min.x, boundsB.min.x),
        Math.min(boundsA.min.y, boundsB.min.y),
        Math.min(boundsA.min.z, boundsB.min.z),
      ),
      max: new THREE.Vector3(
        Math.max(boundsA.max.x, boundsB.max.x),
        Math.max(boundsA.max.y, boundsB.max.y),
        Math.max(boundsA.max.z, boundsB.max.z),
      ),
    };
  }

  function pointInBounds(point, bounds) {
    if (!bounds) {
      return true;
    }
    if (!point) {
      return false;
    }
    return (
      point.x >= bounds.min.x - EPSILON &&
      point.x <= bounds.max.x + EPSILON &&
      point.y >= bounds.min.y - EPSILON &&
      point.y <= bounds.max.y + EPSILON &&
      point.z >= bounds.min.z - EPSILON &&
      point.z <= bounds.max.z + EPSILON
    );
  }

  function sumVectors(vectors) {
    const total = new THREE.Vector3();
    for (const vector of vectors) {
      total.add(vector);
    }
    return total;
  }

  function normalizeVector(vector) {
    const length = vector.length();
    if (length < EPSILON) {
      return new THREE.Vector3(0, 0, 0);
    }
    return vector.clone().divideScalar(length);
  }

  function computeAngleInfo(vectorA, vectorB, orientationNormal = null) {
    const a = vectorA.clone();
    const b = vectorB.clone();
    const lengthA = a.length();
    const lengthB = b.length();
    if (lengthA < EPSILON || lengthB < EPSILON) {
      return { angle: 0, reflex: 0 };
    }
    const normA = a.divideScalar(lengthA);
    const normB = b.divideScalar(lengthB);
    const dot = THREE.MathUtils.clamp(normA.dot(normB), -1, 1);
    const cross = normA.clone().cross(normB);
    let sinValue = cross.length();
    if (orientationNormal) {
      const orientation = cross.dot(orientationNormal);
      if (Math.abs(orientation) < EPSILON) {
        sinValue = 0;
      } else if (orientation < 0) {
        sinValue = -sinValue;
      }
    }
    let orientedAngle = Math.atan2(sinValue, dot);
    if (orientedAngle < 0) {
      orientedAngle += Math.PI * 2;
    }
    const angle = orientedAngle > Math.PI ? (Math.PI * 2 - orientedAngle) : orientedAngle;
    return { angle, reflex: orientedAngle };
  }

  function ensureDate(value, fallback = new Date()) {
    if (value instanceof Date) {
      return Number.isNaN(value.getTime()) ? new Date(fallback.getTime()) : new Date(value.getTime());
    }
    if (typeof value === 'number') {
      const date = new Date(value);
      return Number.isNaN(date.getTime()) ? new Date(fallback.getTime()) : date;
    }
    if (typeof value === 'string') {
      const date = new Date(value);
      return Number.isNaN(date.getTime()) ? new Date(fallback.getTime()) : date;
    }
    if (Array.isArray(value)) {
      if (!value.length) {
        return new Date(fallback.getTime());
      }
      return ensureDate(value[0], fallback);
    }
    if (value && typeof value === 'object') {
      if ('date' in value) {
        return ensureDate(value.date, fallback);
      }
      if ('time' in value) {
        return ensureDate(value.time, fallback);
      }
      const year = toNumber(value.year, Number.NaN);
      const month = toNumber(value.month ?? value.mon, Number.NaN);
      const day = toNumber(value.day ?? value.date, Number.NaN);
      const hour = toNumber(value.hour ?? value.hours ?? value.h, Number.NaN);
      const minute = toNumber(value.minute ?? value.minutes ?? value.min, Number.NaN);
      const second = toNumber(value.second ?? value.seconds ?? value.sec, Number.NaN);
      if (Number.isFinite(year) && Number.isFinite(month) && Number.isFinite(day)) {
        const constructed = new Date(Date.UTC(year, Number(month) - 1, day));
        if (Number.isFinite(hour)) constructed.setUTCHours(hour);
        if (Number.isFinite(minute)) constructed.setUTCMinutes(minute);
        if (Number.isFinite(second)) constructed.setUTCSeconds(second);
        return constructed;
      }
    }
    return new Date(fallback.getTime());
  }

  function ensureGeoLocation(value) {
    const fallback = { latitude: 0, longitude: 0, timezoneHours: 0 };
    if (!value) {
      return fallback;
    }
    if (Array.isArray(value)) {
      if (value.length >= 2) {
        const latitude = toNumber(value[0], Number.NaN);
        const longitude = toNumber(value[1], Number.NaN);
        return {
          latitude: Number.isFinite(latitude) ? latitude : fallback.latitude,
          longitude: Number.isFinite(longitude) ? longitude : fallback.longitude,
          timezoneHours: fallback.timezoneHours,
        };
      }
      if (value.length === 1) {
        return ensureGeoLocation(value[0]);
      }
      return fallback;
    }
    if (typeof value === 'string') {
      const parts = value.split(/[,;\s]+/).map((part) => part.trim()).filter(Boolean);
      if (parts.length >= 2) {
        const latitude = toNumber(parts[0], Number.NaN);
        const longitude = toNumber(parts[1], Number.NaN);
        return {
          latitude: Number.isFinite(latitude) ? latitude : fallback.latitude,
          longitude: Number.isFinite(longitude) ? longitude : fallback.longitude,
          timezoneHours: fallback.timezoneHours,
        };
      }
      return fallback;
    }
    if (typeof value === 'object') {
      if ('location' in value) {
        return ensureGeoLocation(value.location);
      }
      const latitude = toNumber(value.latitude ?? value.lat ?? value.y ?? value.north, Number.NaN);
      const longitude = toNumber(value.longitude ?? value.lon ?? value.lng ?? value.x ?? value.east, Number.NaN);
      let timezoneHours = toNumber(value.timezone ?? value.offset ?? value.utcOffset ?? value.gmt, Number.NaN);
      if (!Number.isFinite(timezoneHours) && Number.isFinite(value.timezoneHours)) {
        timezoneHours = toNumber(value.timezoneHours, Number.NaN);
      }
      return {
        latitude: Number.isFinite(latitude) ? latitude : fallback.latitude,
        longitude: Number.isFinite(longitude) ? longitude : fallback.longitude,
        timezoneHours: Number.isFinite(timezoneHours) ? timezoneHours : fallback.timezoneHours,
      };
    }
    return fallback;
  }

  function computeSolarPosition(date, latitude, longitude, timezoneHours = 0) {
    const rad = Math.PI / 180;
    const year = date.getUTCFullYear();
    const startOfYear = Date.UTC(year, 0, 1);
    const dayOfYear = Math.floor((date.getTime() - startOfYear) / 86400000) + 1;
    const fractionalHour = date.getUTCHours() + date.getUTCMinutes() / 60 + date.getUTCSeconds() / 3600;
    const gamma = (2 * Math.PI / 365) * (dayOfYear - 1 + (fractionalHour - 12) / 24);
    const eqtime = 229.18 * (
      0.000075 +
      0.001868 * Math.cos(gamma) -
      0.032077 * Math.sin(gamma) -
      0.014615 * Math.cos(2 * gamma) -
      0.040849 * Math.sin(2 * gamma)
    );
    const decl =
      0.006918 -
      0.399912 * Math.cos(gamma) +
      0.070257 * Math.sin(gamma) -
      0.006758 * Math.cos(2 * gamma) +
      0.000907 * Math.sin(2 * gamma) -
      0.002697 * Math.cos(3 * gamma) +
      0.00148 * Math.sin(3 * gamma);
    const timeOffset = eqtime + 4 * longitude - 60 * timezoneHours;
    const trueSolarMinutes = fractionalHour * 60 + timeOffset;
    const hourAngle = trueSolarMinutes / 4 - 180;
    const hourAngleRad = hourAngle * rad;
    const latRad = latitude * rad;
    const cosZenith = THREE.MathUtils.clamp(
      Math.sin(latRad) * Math.sin(decl) + Math.cos(latRad) * Math.cos(decl) * Math.cos(hourAngleRad),
      -1,
      1,
    );
    const zenith = Math.acos(cosZenith);
    const elevation = Math.PI / 2 - zenith;
    let azimuth = Math.atan2(
      Math.sin(hourAngleRad),
      Math.cos(hourAngleRad) * Math.sin(latRad) - Math.tan(decl) * Math.cos(latRad),
    );
    azimuth += Math.PI;
    azimuth = (azimuth % (Math.PI * 2) + Math.PI * 2) % (Math.PI * 2);
    return { elevation, azimuth };
  }

  function solarColourFromElevation(elevation) {
    const clamped = THREE.MathUtils.clamp((elevation + Math.PI / 2) / Math.PI, 0, 1);
    const color = new THREE.Color();
    const hue = THREE.MathUtils.lerp(0.07, 0.14, clamped);
    const saturation = THREE.MathUtils.lerp(0.9, 0.4, clamped);
    const lightness = THREE.MathUtils.lerp(0.3, 0.7, clamped);
    color.setHSL(hue, saturation, lightness);
    return `#${color.getHexString()}`;
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

  function planeCoordinates(point, plane) {
    const relative = point.clone().sub(plane.origin);
    return {
      u: relative.dot(plane.xAxis),
      v: relative.dot(plane.yAxis),
      w: relative.dot(plane.zAxis),
    };
  }

  function ensureSection(sectionInput) {
    if (sectionInput === undefined || sectionInput === null) {
      const plane = defaultPlane();
      return {
        plane,
        center: plane.origin.clone(),
        width: 10,
        height: 10,
      };
    }

    if (Array.isArray(sectionInput)) {
      if (sectionInput.length >= 3 && isPlaneLike(sectionInput[0])) {
        const plane = ensurePlane(sectionInput[0]);
        const width = Math.max(EPSILON, Math.abs(toNumber(sectionInput[1], 10)));
        const height = Math.max(EPSILON, Math.abs(toNumber(sectionInput[2], width)));
        return {
          plane,
          center: plane.origin.clone(),
          width,
          height,
        };
      }
      if (sectionInput.length >= 2) {
        const planeCandidate = sectionInput[2];
        const plane = isPlaneLike(planeCandidate) ? ensurePlane(planeCandidate) : defaultPlane();
        const width = Math.max(EPSILON, Math.abs(toNumber(sectionInput[0], 10)));
        const height = Math.max(EPSILON, Math.abs(toNumber(sectionInput[1], width)));
        return {
          plane,
          center: plane.origin.clone(),
          width,
          height,
        };
      }
      if (sectionInput.length === 1) {
        return ensureSection(sectionInput[0]);
      }
    }

    const plane = ensurePlane(sectionInput?.plane ?? sectionInput);
    const center = toVector3(sectionInput?.center ?? sectionInput?.origin ?? plane.origin, plane.origin.clone());
    let width = toNumber(
      sectionInput?.width ??
        sectionInput?.w ??
        sectionInput?.size?.width ??
        sectionInput?.size?.x ??
        sectionInput?.extentX ??
        sectionInput?.dimensions?.x,
      Number.NaN,
    );
    let height = toNumber(
      sectionInput?.height ??
        sectionInput?.h ??
        sectionInput?.size?.height ??
        sectionInput?.size?.y ??
        sectionInput?.extentY ??
        sectionInput?.dimensions?.y,
      Number.NaN,
    );

    if (!Number.isFinite(width)) {
      const fallback = toNumber(sectionInput?.size ?? sectionInput?.radius ?? sectionInput?.diameter ?? 10, 10);
      width = Number.isFinite(fallback) ? fallback : 10;
    }

    if (!Number.isFinite(height)) {
      const fallback = toNumber(sectionInput?.size ?? sectionInput?.radius ?? sectionInput?.diameter ?? width, width);
      height = Number.isFinite(fallback) ? fallback : width;
    }

    return {
      plane: createPlaneInstance(plane, center),
      center,
      width: Math.max(EPSILON, Math.abs(width)),
      height: Math.max(EPSILON, Math.abs(height)),
    };
  }

  function resolveSampleCounts(samplesInput) {
    if (samplesInput === undefined || samplesInput === null) {
      return { x: 10, y: 10 };
    }
    if (Array.isArray(samplesInput)) {
      if (!samplesInput.length) {
        return { x: 10, y: 10 };
      }
      if (samplesInput.length === 1) {
        const count = Math.max(2, Math.round(toNumber(samplesInput[0], 10)));
        return { x: count, y: count };
      }
      const x = Math.max(2, Math.round(toNumber(samplesInput[0], 10)));
      const y = Math.max(2, Math.round(toNumber(samplesInput[1], x)));
      return { x, y };
    }
    if (typeof samplesInput === 'object') {
      const resolvedX = Math.max(
        2,
        Math.round(
          toNumber(
            samplesInput.x ?? samplesInput.u ?? samplesInput.width ?? samplesInput.columns ?? samplesInput.count,
            10,
          ),
        ),
      );
      const resolvedY = Math.max(
        2,
        Math.round(
          toNumber(
            samplesInput.y ?? samplesInput.v ?? samplesInput.height ?? samplesInput.rows ?? samplesInput.count,
            resolvedX,
          ),
        ),
      );
      return { x: resolvedX, y: resolvedY };
    }
    const count = Math.max(2, Math.round(toNumber(samplesInput, 10)));
    return { x: count, y: count };
  }

  function createSectionGrid(section, sampleCounts) {
    const xCount = Math.max(2, Math.round(sampleCounts?.x ?? sampleCounts?.width ?? sampleCounts?.columns ?? 10));
    const yCount = Math.max(2, Math.round(sampleCounts?.y ?? sampleCounts?.height ?? sampleCounts?.rows ?? sampleCounts?.x ?? 10));
    const points = [];
    for (let j = 0; j < yCount; j += 1) {
      const v = yCount === 1 ? 0.5 : j / (yCount - 1);
      const offsetV = (v - 0.5) * section.height;
      for (let i = 0; i < xCount; i += 1) {
        const u = xCount === 1 ? 0.5 : i / (xCount - 1);
        const offsetU = (u - 0.5) * section.width;
        const point = section.center.clone();
        point.add(section.plane.xAxis.clone().multiplyScalar(offsetU));
        point.add(section.plane.yAxis.clone().multiplyScalar(offsetV));
        points.push(point);
      }
    }
    return { points, xCount, yCount };
  }

  function defaultFieldEvaluation() {
    return {
      vector: new THREE.Vector3(),
      strength: 0,
      scalar: 0,
      tensor: {
        direction: new THREE.Vector3(),
        magnitude: 0,
        contributions: [],
      },
    };
  }

  function isField(candidate) {
    return Boolean(candidate && typeof candidate.evaluate === 'function' && candidate.type === 'field');
  }

  function createField(influences = [], options = {}) {
    const normalizedInfluences = [];
    for (const influence of influences) {
      if (!influence || typeof influence.evaluate !== 'function') {
        continue;
      }
      normalizedInfluences.push({
        type: influence.type ?? 'custom',
        evaluate: influence.evaluate,
        bounds: influence.bounds ? ensureBounds(influence.bounds) : null,
        params: influence.params ? { ...influence.params } : {},
        metadata: influence.metadata ? { ...influence.metadata } : {},
      });
    }

    const fieldBounds = options.bounds ? ensureBounds(options.bounds) : null;
    const metadata = { ...(options.metadata ?? {}) };

    return {
      type: 'field',
      influences: normalizedInfluences,
      bounds: fieldBounds,
      metadata,
      evaluate(pointInput) {
        if (pointInput === undefined || pointInput === null) {
          return defaultFieldEvaluation();
        }
        const point = toVector3(pointInput, new THREE.Vector3());
        if (fieldBounds && !pointInBounds(point, fieldBounds)) {
          return defaultFieldEvaluation();
        }
        const totalVector = new THREE.Vector3();
        const contributions = [];
        let aggregatedStrength = 0;

        for (const influence of normalizedInfluences) {
          if (influence.bounds && !pointInBounds(point, influence.bounds)) {
            continue;
          }
          let influenceResult = null;
          try {
            influenceResult = influence.evaluate(point);
          } catch (error) {
            influenceResult = null;
          }
          if (!influenceResult) {
            continue;
          }
          const vector = influenceResult.vector?.clone?.() ?? new THREE.Vector3();
          const strengthCandidate = influenceResult.strength ?? influenceResult.scalar ?? influenceResult.magnitude ?? vector.length();
          const strength = Number.isFinite(strengthCandidate) ? strengthCandidate : vector.length();
          totalVector.add(vector);
          if (Number.isFinite(strength)) {
            aggregatedStrength += strength;
          }
          contributions.push({
            type: influence.type,
            vector: vector.clone(),
            strength: Number.isFinite(strength) ? strength : vector.length(),
          });
        }

        const magnitude = totalVector.length();
        const direction = magnitude > EPSILON ? totalVector.clone().divideScalar(magnitude) : new THREE.Vector3(0, 0, 0);
        const strength = aggregatedStrength > 0 ? aggregatedStrength : magnitude;

        return {
          vector: totalVector.clone(),
          strength,
          scalar: strength,
          tensor: {
            direction,
            magnitude,
            contributions,
          },
        };
      },
    };
  }

  function collectFields(input) {
    const fields = [];

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
      if (isField(value)) {
        fields.push(value);
        return;
      }
      if (typeof value === 'object') {
        if ('field' in value) {
          visit(value.field);
          return;
        }
        if ('fields' in value) {
          visit(value.fields);
          return;
        }
        if (typeof value.evaluate === 'function') {
          const bounds = ensureBounds(value.bounds);
          const influence = {
            type: value.type ?? 'custom',
            bounds,
            evaluate: (point) => value.evaluate(point),
            params: value.params ? { ...value.params } : {},
            metadata: value.metadata ? { ...value.metadata } : {},
          };
          fields.push(createField([influence], { bounds, metadata: influence.metadata }));
        }
      }
    }

    visit(input);
    return fields;
  }

  function evaluateFieldAtPoint(field, point) {
    const fallback = defaultFieldEvaluation();
    if (!field || typeof field.evaluate !== 'function') {
      return fallback;
    }
    if (point === undefined || point === null) {
      return fallback;
    }
    try {
      const evaluation = field.evaluate(point);
      if (!evaluation) {
        return fallback;
      }
      const vector = evaluation.vector?.clone?.() ?? new THREE.Vector3();
      const strength = Number.isFinite(evaluation.strength)
        ? evaluation.strength
        : Number.isFinite(evaluation.scalar)
          ? evaluation.scalar
          : vector.length();
      const magnitude = vector.length();
      const direction = magnitude > EPSILON ? vector.clone().divideScalar(magnitude) : new THREE.Vector3(0, 0, 0);
      const tensor = evaluation.tensor
        ? {
            direction: evaluation.tensor.direction?.clone?.() ?? direction.clone(),
            magnitude: Number.isFinite(evaluation.tensor.magnitude) ? evaluation.tensor.magnitude : magnitude,
            contributions: Array.isArray(evaluation.tensor.contributions)
              ? evaluation.tensor.contributions.map((entry) => ({
                type: entry.type ?? 'contribution',
                vector: entry.vector?.clone?.() ?? new THREE.Vector3(),
                strength: Number.isFinite(entry.strength)
                  ? entry.strength
                  : entry.vector?.length?.() ?? 0,
              }))
              : [],
          }
        : {
            direction,
            magnitude,
            contributions: [],
          };
      return {
        vector,
        strength,
        scalar: Number.isFinite(evaluation.scalar) ? evaluation.scalar : strength,
        tensor,
      };
    } catch (error) {
      return fallback;
    }
  }

  function createFieldDisplayMesh(field, sectionInput, samplesInput, mode, options = {}) {
    const section = ensureSection(sectionInput);
    const sampleCounts = resolveSampleCounts(samplesInput);
    const grid = createSectionGrid(section, sampleCounts);
    if (!grid.points.length) {
      return null;
    }

    const evaluations = grid.points.map((point) => evaluateFieldAtPoint(field, point));
    const strengthValues = evaluations.map((entry) => {
      if (!entry) return 0;
      if (Number.isFinite(entry.strength)) return entry.strength;
      if (Number.isFinite(entry.scalar)) return entry.scalar;
      return entry.vector?.length?.() ?? 0;
    });

    let minStrength = Number.POSITIVE_INFINITY;
    let maxStrength = Number.NEGATIVE_INFINITY;
    for (const value of strengthValues) {
      if (value < minStrength) minStrength = value;
      if (value > maxStrength) maxStrength = value;
    }
    if (!Number.isFinite(minStrength)) minStrength = 0;
    if (!Number.isFinite(maxStrength)) maxStrength = minStrength;

    const vertexCount = grid.points.length;
    const positions = new Float32Array(vertexCount * 3);
    const colors = new Float32Array(vertexCount * 3);
    const color = new THREE.Color();
    const baseColor = ensureColor(options.baseColor ?? '#666666', new THREE.Color(0.4, 0.4, 0.4));
    const positiveColour = ensureColor(options.positiveColor ?? '#ff7043', new THREE.Color(1, 0.45, 0.26));
    const negativeColour = ensureColor(options.negativeColor ?? '#4fc3f7', new THREE.Color(0.31, 0.76, 0.97));
    const normal = section.plane.zAxis.clone().normalize();
    const sampleScale = Math.max(sampleCounts.x, sampleCounts.y, 2);
    const vectorScale = options.vectorScale ?? (Math.min(section.width, section.height) / sampleScale) * 0.35;

    for (let index = 0; index < vertexCount; index += 1) {
      const basePoint = grid.points[index];
      const evaluation = evaluations[index] ?? defaultFieldEvaluation();
      const strength = Number.isFinite(strengthValues[index]) ? strengthValues[index] : 0;
      const normalizedStrength = maxStrength > minStrength
        ? THREE.MathUtils.clamp((strength - minStrength) / (maxStrength - minStrength || 1), 0, 1)
        : 0;
      const vector = evaluation.vector?.clone?.() ?? new THREE.Vector3();
      const magnitude = vector.length();
      let position = basePoint.clone();

      switch (mode) {
        case 'tensor': {
          if (magnitude > EPSILON) {
            const scaledVector = vector.clone();
            const limit = Math.max(vectorScale, EPSILON);
            const scaledLength = Math.min(magnitude * vectorScale, limit * 5);
            scaledVector.setLength(scaledLength);
            position = basePoint.clone().add(scaledVector);
          }
          color.setHSL(
            THREE.MathUtils.lerp(0.6, 0.05, normalizedStrength),
            0.8,
            THREE.MathUtils.lerp(0.35, 0.65, normalizedStrength),
          );
          break;
        }
        case 'direction': {
          if (magnitude > EPSILON) {
            const dir = vector.clone().divideScalar(magnitude);
            color.setRGB(
              THREE.MathUtils.clamp((dir.x + 1) / 2, 0, 1),
              THREE.MathUtils.clamp((dir.y + 1) / 2, 0, 1),
              THREE.MathUtils.clamp((dir.z + 1) / 2, 0, 1),
            );
          } else {
            color.copy(baseColor);
          }
          break;
        }
        case 'perpendicular': {
          const component = vector.dot(normal);
          const intensity = magnitude > EPSILON ? THREE.MathUtils.clamp(Math.abs(component) / magnitude, 0, 1) : 0;
          if (component >= 0) {
            color.copy(positiveColour).lerp(baseColor, 1 - intensity);
          } else {
            color.copy(negativeColour).lerp(baseColor, 1 - intensity);
          }
          break;
        }
        default: {
          color.setHSL(
            THREE.MathUtils.lerp(0.6, 0.05, normalizedStrength),
            0.8,
            THREE.MathUtils.lerp(0.35, 0.65, normalizedStrength),
          );
          break;
        }
      }

      positions[index * 3 + 0] = position.x;
      positions[index * 3 + 1] = position.y;
      positions[index * 3 + 2] = position.z;

      colors[index * 3 + 0] = color.r;
      colors[index * 3 + 1] = color.g;
      colors[index * 3 + 2] = color.b;
    }

    const indices = [];
    for (let y = 0; y < grid.yCount - 1; y += 1) {
      for (let x = 0; x < grid.xCount - 1; x += 1) {
        const a = y * grid.xCount + x;
        const b = a + 1;
        const c = a + grid.xCount;
        const d = c + 1;
        indices.push(a, b, d, a, d, c);
      }
    }

    const geometry = new THREE.BufferGeometry();
    geometry.setIndex(indices);
    geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
    geometry.setAttribute('color', new THREE.Float32BufferAttribute(colors, 3));
    geometry.computeVertexNormals();

    const material = new THREE.MeshStandardMaterial({
      vertexColors: true,
      side: THREE.DoubleSide,
      flatShading: mode !== 'tensor',
      transparent: options.transparent ?? false,
      opacity: options.opacity ?? 1,
    });

    const mesh = new THREE.Mesh(geometry, material);
    mesh.name = options.name ?? `field-${mode}-display`;
    mesh.userData = {
      type: 'field-display',
      mode,
      section,
      samples: sampleCounts,
      evaluations,
    };

    return mesh;
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

  function clonePlane(plane) {
    return {
      origin: plane.origin.clone(),
      xAxis: plane.xAxis.clone(),
      yAxis: plane.yAxis.clone(),
      zAxis: plane.zAxis.clone(),
    };
  }

  function collectPlanes(input) {
    const planes = [];

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
        if (isPlaneLike(value)) {
          planes.push(ensurePlane(value));
          return;
        }
      }
      if (isPlaneLike(value)) {
        planes.push(ensurePlane(value));
      }
    }

    visit(input);
    return planes;
  }

  function rotatePlaneAroundNormal(plane, angle) {
    if (!angle) {
      return clonePlane(plane);
    }
    const cos = Math.cos(angle);
    const sin = Math.sin(angle);
    const xAxis = plane.xAxis.clone().multiplyScalar(cos).add(plane.yAxis.clone().multiplyScalar(-sin));
    const yAxis = plane.xAxis.clone().multiplyScalar(sin).add(plane.yAxis.clone().multiplyScalar(cos));
    return normalizePlaneAxes(plane.origin.clone(), xAxis, yAxis, plane.zAxis.clone());
  }

  function alignPlaneOrientation(currentPlane, referencePlane) {
    const reference = clonePlane(referencePlane);
    let working = normalizePlaneAxes(
      currentPlane.origin.clone(),
      currentPlane.xAxis.clone(),
      currentPlane.yAxis.clone(),
      currentPlane.zAxis.clone()
    );

    if (working.zAxis.dot(reference.zAxis) < 0) {
      working = {
        origin: working.origin.clone(),
        xAxis: working.xAxis.clone().multiplyScalar(-1),
        yAxis: working.yAxis.clone().multiplyScalar(-1),
        zAxis: working.zAxis.clone().multiplyScalar(-1),
      };
    }

    const rotation = Math.atan2(reference.xAxis.dot(working.yAxis), reference.xAxis.dot(working.xAxis));
    working = rotatePlaneAroundNormal(working, -rotation);

    if (working.xAxis.dot(reference.xAxis) < 0) {
      working = rotatePlaneAroundNormal(working, Math.PI);
    }

    return working;
  }

  function jacobiEigenDecomposition(matrix) {
    const a = [
      [matrix[0][0], matrix[0][1], matrix[0][2]],
      [matrix[1][0], matrix[1][1], matrix[1][2]],
      [matrix[2][0], matrix[2][1], matrix[2][2]],
    ];
    const v = [
      [1, 0, 0],
      [0, 1, 0],
      [0, 0, 1],
    ];

    for (let iteration = 0; iteration < 32; iteration += 1) {
      let p = 0;
      let q = 1;
      let max = Math.abs(a[p][q]);
      for (let i = 0; i < 3; i += 1) {
        for (let j = i + 1; j < 3; j += 1) {
          const value = Math.abs(a[i][j]);
          if (value > max) {
            max = value;
            p = i;
            q = j;
          }
        }
      }

      if (max < 1e-12) {
        break;
      }

      const app = a[p][p];
      const aqq = a[q][q];
      const apq = a[p][q];
      const tau = (aqq - app) / (2 * apq);
      const t = Math.sign(tau) / (Math.abs(tau) + Math.sqrt(1 + tau * tau));
      const c = 1 / Math.sqrt(1 + t * t);
      const s = t * c;
      const tauPrime = s / (1 + c);

      a[p][p] = app - t * apq;
      a[q][q] = aqq + t * apq;
      a[p][q] = 0;
      a[q][p] = 0;

      for (let i = 0; i < 3; i += 1) {
        if (i !== p && i !== q) {
          const aip = a[i][p];
          const aiq = a[i][q];
          a[i][p] = aip - s * (aiq + tauPrime * aip);
          a[p][i] = a[i][p];
          a[i][q] = aiq + s * (aip - tauPrime * aiq);
          a[q][i] = a[i][q];
        }
        const vip = v[i][p];
        const viq = v[i][q];
        v[i][p] = vip - s * (viq + tauPrime * vip);
        v[i][q] = viq + s * (vip - tauPrime * viq);
      }
    }

    const eigenvalues = [a[0][0], a[1][1], a[2][2]];
    const eigenvectors = [
      new THREE.Vector3(v[0][0], v[1][0], v[2][0]),
      new THREE.Vector3(v[0][1], v[1][1], v[2][1]),
      new THREE.Vector3(v[0][2], v[1][2], v[2][2]),
    ];
    return { eigenvalues, eigenvectors };
  }

  function fitPlaneToPoints(points) {
    if (!points.length) {
      const plane = defaultPlane();
      return { plane, deviation: 0 };
    }
    const centroid = new THREE.Vector3();
    for (const point of points) {
      centroid.add(point);
    }
    centroid.multiplyScalar(1 / points.length);

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

    const covariance = [
      [xx, xy, xz],
      [xy, yy, yz],
      [xz, yz, zz],
    ];

    const { eigenvalues, eigenvectors } = jacobiEigenDecomposition(covariance);
    let smallestIndex = 0;
    if (eigenvalues[1] < eigenvalues[smallestIndex]) smallestIndex = 1;
    if (eigenvalues[2] < eigenvalues[smallestIndex]) smallestIndex = 2;

    let normal = eigenvectors[smallestIndex];
    if (!normal || normal.lengthSq() < EPSILON) {
      normal = new THREE.Vector3(0, 0, 1);
    } else {
      normal = normal.clone().normalize();
    }

    const fallback = Math.abs(normal.dot(new THREE.Vector3(1, 0, 0))) > 0.999
      ? new THREE.Vector3(0, 1, 0)
      : new THREE.Vector3(1, 0, 0);
    const xAxis = fallback.clone().sub(normal.clone().multiplyScalar(fallback.dot(normal))).normalize();
    const yAxis = normal.clone().cross(xAxis).normalize();

    const plane = normalizePlaneAxes(centroid, xAxis, yAxis, normal);
    let deviation = 0;
    for (const point of points) {
      const distance = Math.abs(normal.dot(point.clone().sub(plane.origin)));
      if (distance > deviation) {
        deviation = distance;
      }
    }
    return { plane, deviation };
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
        new THREE.Vector3()
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
      const resolvedEnd = toVector3(end, direction ? start.clone().add(direction) : start.clone().add(new THREE.Vector3(1, 0, 0)));
      const resolvedDirection = direction
        ? direction.clone()
        : resolvedEnd.clone().sub(start);
      if (resolvedDirection.lengthSq() < EPSILON) {
        resolvedDirection.set(1, 0, 0);
      }
      return { start, end: resolvedEnd, direction: resolvedDirection };
    }
    const start = toVector3(input, new THREE.Vector3());
    const end = start.clone().add(new THREE.Vector3(1, 0, 0));
    return { start, end, direction: end.clone().sub(start) };
  }
  if (includePlaneComponents) {
    register(['{17b7152b-d30d-4d50-b9ef-c9fe25576fc2}', 'xy plane', 'xy'], {
      type: 'plane',
      pinMap: {
        inputs: { O: 'origin', Origin: 'origin', origin: 'origin' },
        outputs: { P: 'plane', Plane: 'plane', Pl: 'plane' },
      },
      eval: ({ inputs }) => {
        const origin = toVector3(inputs.origin, new THREE.Vector3());
        const plane = normalizePlaneAxes(origin, new THREE.Vector3(1, 0, 0), new THREE.Vector3(0, 1, 0));
        return { plane: createPlaneInstance(plane) };
      },
    });

    register(['{fad344bc-09b1-4855-a2e6-437ef5715fe3}', 'yz plane', 'yz'], {
      type: 'plane',
      pinMap: {
        inputs: { O: 'origin', Origin: 'origin', origin: 'origin' },
        outputs: { P: 'plane', Plane: 'plane', Pl: 'plane' },
      },
      eval: ({ inputs }) => {
        const origin = toVector3(inputs.origin, new THREE.Vector3());
        const plane = normalizePlaneAxes(
          origin,
          new THREE.Vector3(0, 1, 0),
          new THREE.Vector3(0, 0, 1),
          new THREE.Vector3(1, 0, 0)
        );
        return { plane: createPlaneInstance(plane) };
      },
    });

    register(['{8cc3a196-f6a0-49ea-9ed9-0cb343a3ae64}', 'xz plane', 'xz'], {
      type: 'plane',
      pinMap: {
        inputs: { O: 'origin', Origin: 'origin', origin: 'origin' },
        outputs: { P: 'plane', Plane: 'plane', Pl: 'plane' },
      },
      eval: ({ inputs }) => {
        const origin = toVector3(inputs.origin, new THREE.Vector3());
        const plane = normalizePlaneAxes(
          origin,
          new THREE.Vector3(1, 0, 0),
          new THREE.Vector3(0, 0, 1),
          new THREE.Vector3(0, 1, 0)
        );
        return { plane: createPlaneInstance(plane) };
      },
    });

    register(['{33bfc73c-19b2-480b-81e6-f3523a012ea6}', 'plane fit', 'plfit'], {
      type: 'plane',
      pinMap: {
        inputs: { P: 'points', Points: 'points', points: 'points' },
        outputs: { Pl: 'plane', plane: 'plane', dx: 'deviation', Deviation: 'deviation' },
      },
      eval: ({ inputs }) => {
        const points = collectPoints(inputs.points);
        const { plane, deviation } = fitPlaneToPoints(points);
        return { plane: createPlaneInstance(plane), deviation };
      },
    });

    register(['{3a0c7bda-3d22-4588-8bab-03f57a52a6ea}', 'plane offset', 'pl offset'], {
      type: 'plane',
      pinMap: {
        inputs: { P: 'plane', plane: 'plane', Plane: 'plane', O: 'offset', offset: 'offset', Offset: 'offset' },
        outputs: { Pl: 'plane', plane: 'plane', P: 'plane' },
      },
      eval: ({ inputs }) => {
        const plane = ensurePlane(inputs.plane);
        const offset = toNumber(inputs.offset, 0);
        const normal = plane.zAxis.clone().normalize();
        const origin = plane.origin.clone().add(normal.multiplyScalar(offset));
        const result = normalizePlaneAxes(origin, plane.xAxis.clone(), plane.yAxis.clone(), plane.zAxis.clone());
        return { plane: createPlaneInstance(result) };
      },
    });

    register(['{3cd2949b-4ea8-4ffb-a70c-5c380f9f46ea}', 'deconstruct plane', 'deplane'], {
      type: 'plane',
      pinMap: {
        inputs: { P: 'plane', plane: 'plane', Plane: 'plane' },
        outputs: { O: 'origin', X: 'xAxis', Y: 'yAxis', Z: 'zAxis', origin: 'origin', x: 'xAxis', y: 'yAxis', z: 'zAxis' },
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
        inputs: { P: 'point', point: 'point', Point: 'point', S: 'system', System: 'system', system: 'system' },
        outputs: { X: 'x', Y: 'y', Z: 'z', x: 'x', y: 'y', z: 'z' },
      },
      eval: ({ inputs }) => {
        const point = toVector3(inputs.point, new THREE.Vector3());
        const plane = ensurePlane(inputs.system);
        const relative = point.clone().sub(plane.origin);
        return {
          x: relative.dot(plane.xAxis),
          y: relative.dot(plane.yAxis),
          z: relative.dot(plane.zAxis),
        };
      },
    });

    register(['{75eec078-a905-47a1-b0d2-0934182b1e3d}', 'plane origin', 'pl origin'], {
      type: 'plane',
      pinMap: {
        inputs: { B: 'plane', base: 'plane', basePlane: 'plane', Plane: 'plane', plane: 'plane', O: 'origin', Origin: 'origin', origin: 'origin' },
        outputs: { Pl: 'plane', plane: 'plane', P: 'plane' },
      },
      eval: ({ inputs }) => {
        const plane = ensurePlane(inputs.plane);
        const origin = toVector3(inputs.origin, plane.origin.clone());
        const result = normalizePlaneAxes(origin, plane.xAxis.clone(), plane.yAxis.clone(), plane.zAxis.clone());
        return { plane: createPlaneInstance(result) };
      },
    });

    register(['{9ce34996-d8c6-40d3-b442-1a7c8c093614}', 'adjust plane', 'padjust'], {
      type: 'plane',
      pinMap: {
        inputs: { P: 'plane', plane: 'plane', Plane: 'plane', N: 'normal', normal: 'normal', Normal: 'normal' },
        outputs: { P: 'plane', plane: 'plane', Pl: 'plane' },
      },
      eval: ({ inputs }) => {
        const plane = ensurePlane(inputs.plane);
        let target = ensureUnitVector(inputs.normal, plane.zAxis.clone());
        if (target.lengthSq() < EPSILON) {
          return { plane: createPlaneInstance(plane) };
        }
        if (target.dot(plane.zAxis) < 0) {
          target = target.clone().multiplyScalar(-1);
        }
        const startZ = plane.zAxis.clone().normalize();
        const dot = THREE.MathUtils.clamp(startZ.dot(target), -1, 1);
        let angle = Math.acos(dot);
        if (!Number.isFinite(angle) || angle < EPSILON) {
          const result = normalizePlaneAxes(plane.origin.clone(), plane.xAxis.clone(), plane.yAxis.clone(), target.clone());
          return { plane: createPlaneInstance(result) };
        }
        let axis = startZ.clone().cross(target);
        if (axis.lengthSq() < EPSILON) {
          axis = plane.xAxis.clone();
          if (axis.lengthSq() < EPSILON) {
            axis = new THREE.Vector3(1, 0, 0);
          }
        }
        axis.normalize();
        const quaternion = new THREE.Quaternion().setFromAxisAngle(axis, angle);
        const xAxis = plane.xAxis.clone().applyQuaternion(quaternion);
        const yAxis = plane.yAxis.clone().applyQuaternion(quaternion);
        const result = normalizePlaneAxes(plane.origin.clone(), xAxis, yAxis, target.clone());
        return { plane: createPlaneInstance(result) };
      },
    });

    register(['{b075c065-efda-4c9f-9cc9-288362b1b4b9}', 'plane closest point', 'pl closest point', 'plane cp'], {
      type: 'plane',
      pinMap: {
        inputs: { S: 'point', Point: 'point', point: 'point', P: 'plane', plane: 'plane', Plane: 'plane' },
        outputs: { P: 'projected', Point: 'projected', uv: 'uv', UV: 'uv', D: 'distance', distance: 'distance' },
      },
      eval: ({ inputs }) => {
        const plane = ensurePlane(inputs.plane);
        const sample = toVector3(inputs.point, new THREE.Vector3());
        const normal = plane.zAxis.clone().normalize();
        const distance = normal.dot(sample.clone().sub(plane.origin));
        const projected = sample.clone().sub(normal.clone().multiplyScalar(distance));
        const relative = projected.clone().sub(plane.origin);
        const u = relative.dot(plane.xAxis);
        const v = relative.dot(plane.yAxis);
        return { projected, uv: new THREE.Vector2(u, v), distance };
      },
    });

    register(['{bc3e379e-7206-4e7b-b63a-ff61f4b38a3e}', 'construct plane', 'pl'], {
      type: 'plane',
      pinMap: {
        inputs: { O: 'origin', Origin: 'origin', origin: 'origin', X: 'xAxis', x: 'xAxis', Y: 'yAxis', y: 'yAxis' },
        outputs: { Pl: 'plane', plane: 'plane', P: 'plane' },
      },
      eval: ({ inputs }) => {
        const origin = toVector3(inputs.origin, new THREE.Vector3());
        const xAxis = toVector3(inputs.xAxis, new THREE.Vector3(1, 0, 0));
        const yAxis = toVector3(inputs.yAxis, new THREE.Vector3(0, 1, 0));
        const plane = normalizePlaneAxes(origin, xAxis, yAxis);
        return { plane: createPlaneInstance(plane) };
      },
    });

    register(['{cfb6b17f-ca82-4f5d-b604-d4f69f569de3}', 'plane normal', 'plane normal plane'], {
      type: 'plane',
      pinMap: {
        inputs: { O: 'origin', Origin: 'origin', origin: 'origin', Z: 'zAxis', z: 'zAxis', Normal: 'zAxis', normal: 'zAxis' },
        outputs: { P: 'plane', plane: 'plane', Pl: 'plane' },
      },
      eval: ({ inputs }) => {
        const origin = toVector3(inputs.origin, new THREE.Vector3());
        const normal = ensureUnitVector(inputs.zAxis, new THREE.Vector3(0, 0, 1));
        const fallback = Math.abs(normal.dot(new THREE.Vector3(1, 0, 0))) > 0.999
          ? new THREE.Vector3(0, 1, 0)
          : new THREE.Vector3(1, 0, 0);
        const xAxis = fallback.clone().sub(normal.clone().multiplyScalar(fallback.dot(normal))).normalize();
        const yAxis = normal.clone().cross(xAxis).normalize();
        const plane = normalizePlaneAxes(origin, xAxis, yAxis, normal);
        return { plane: createPlaneInstance(plane) };
      },
    });

    register(['{c98a6015-7a2f-423c-bc66-bdc505249b45}', 'plane 3pt', 'pl 3pt', 'plane three point'], {
      type: 'plane',
      pinMap: {
        inputs: { A: 'pointA', a: 'pointA', B: 'pointB', b: 'pointB', C: 'pointC', c: 'pointC' },
        outputs: { Pl: 'plane', plane: 'plane', P: 'plane' },
      },
      eval: ({ inputs }) => {
        const origin = toVector3(inputs.pointA, new THREE.Vector3());
        let xAxis = toVector3(inputs.pointB, origin.clone().add(new THREE.Vector3(1, 0, 0))).sub(origin);
        if (xAxis.lengthSq() < EPSILON) {
          xAxis = new THREE.Vector3(1, 0, 0);
        } else {
          xAxis.normalize();
        }
        let orientation = toVector3(inputs.pointC, origin.clone().add(new THREE.Vector3(0, 1, 0))).sub(origin);
        orientation.sub(xAxis.clone().multiplyScalar(orientation.dot(xAxis)));
        if (orientation.lengthSq() < EPSILON) {
          orientation = xAxis.clone().cross(new THREE.Vector3(0, 0, 1));
          if (orientation.lengthSq() < EPSILON) {
            orientation = xAxis.clone().cross(new THREE.Vector3(0, 1, 0));
          }
        }
        if (orientation.lengthSq() < EPSILON) {
          orientation = new THREE.Vector3(0, 1, 0);
        } else {
          orientation.normalize();
        }
        const plane = normalizePlaneAxes(origin, xAxis, orientation);
        return { plane: createPlaneInstance(plane) };
      },
    });

    register(['{ccc3f2ff-c9f6-45f8-aa30-8a924a9bda36}', 'line + pt', 'lnpt'], {
      type: 'plane',
      pinMap: {
        inputs: { L: 'line', line: 'line', Line: 'line', P: 'point', point: 'point', Point: 'point' },
        outputs: { Pl: 'plane', plane: 'plane', P: 'plane' },
      },
      eval: ({ inputs }) => {
        const line = ensureLine(inputs.line);
        const point = toVector3(inputs.point, line.start.clone());
        let xAxis = line.direction.clone();
        if (xAxis.lengthSq() < EPSILON) {
          xAxis = new THREE.Vector3(1, 0, 0);
        } else {
          xAxis.normalize();
        }
        let yAxis = point.clone().sub(line.start);
        yAxis.sub(xAxis.clone().multiplyScalar(yAxis.dot(xAxis)));
        if (yAxis.lengthSq() < EPSILON) {
          yAxis = xAxis.clone().cross(new THREE.Vector3(0, 0, 1));
          if (yAxis.lengthSq() < EPSILON) {
            yAxis = xAxis.clone().cross(new THREE.Vector3(0, 1, 0));
          }
        }
        if (yAxis.lengthSq() < EPSILON) {
          yAxis = new THREE.Vector3(0, 1, 0);
        } else {
          yAxis.normalize();
        }
        const plane = normalizePlaneAxes(line.start.clone(), xAxis, yAxis);
        return { plane: createPlaneInstance(plane) };
      },
    });

    register(['{d788ad7f-6d68-4106-8b2f-9e55e6e107c0}', 'line + line', 'lnln'], {
      type: 'plane',
      pinMap: {
        inputs: { A: 'lineA', a: 'lineA', LineA: 'lineA', B: 'lineB', b: 'lineB', LineB: 'lineB' },
        outputs: { Pl: 'plane', plane: 'plane', P: 'plane' },
      },
      eval: ({ inputs }) => {
        const lineA = ensureLine(inputs.lineA);
        const lineB = ensureLine(inputs.lineB);
        let xAxis = lineA.direction.clone();
        if (xAxis.lengthSq() < EPSILON) {
          xAxis = new THREE.Vector3(1, 0, 0);
        } else {
          xAxis.normalize();
        }
        let yAxis = lineB.direction.clone();
        if (yAxis.lengthSq() < EPSILON) {
          yAxis = lineB.start.clone().sub(lineA.start);
        }
        if (yAxis.lengthSq() < EPSILON) {
          yAxis = lineB.end.clone().sub(lineA.start);
        }
        if (yAxis.lengthSq() < EPSILON) {
          yAxis = new THREE.Vector3(0, 1, 0);
        }
        yAxis.sub(xAxis.clone().multiplyScalar(yAxis.dot(xAxis)));
        if (yAxis.lengthSq() < EPSILON) {
          yAxis = xAxis.clone().cross(new THREE.Vector3(0, 0, 1));
          if (yAxis.lengthSq() < EPSILON) {
            yAxis = xAxis.clone().cross(new THREE.Vector3(0, 1, 0));
          }
        }
        if (yAxis.lengthSq() < EPSILON) {
          yAxis = new THREE.Vector3(0, 1, 0);
        } else {
          yAxis.normalize();
        }
        const plane = normalizePlaneAxes(lineA.start.clone(), xAxis, yAxis);
        return { plane: createPlaneInstance(plane) };
      },
    });

    register(['{2318aee8-01fe-4ea8-9524-6966023fc622}', 'align planes', 'align plane list'], {
      type: 'plane',
      pinMap: {
        inputs: { P: 'planes', planes: 'planes', Planes: 'planes', M: 'master', Master: 'master', master: 'master' },
        outputs: { P: 'planes', planes: 'planes', Planes: 'planes' },
      },
      eval: ({ inputs }) => {
        const planes = collectPlanes(inputs.planes);
        if (!planes.length) {
          return { planes: [] };
        }
        let reference = inputs.master ? ensurePlane(inputs.master) : null;
        const aligned = [];
        for (let index = 0; index < planes.length; index += 1) {
          const current = clonePlane(planes[index]);
          const ref = reference ?? clonePlane(current);
          const result = alignPlaneOrientation(current, ref);
          aligned.push(createPlaneInstance(result));
          reference = result;
        }
        return { planes: aligned };
      },
    });

    register(['{e76040ec-3b91-41e1-8e00-c74c23b89391}', 'align plane', 'plane align'], {
      type: 'plane',
      pinMap: {
        inputs: { P: 'plane', plane: 'plane', Plane: 'plane', D: 'direction', direction: 'direction', Direction: 'direction' },
        outputs: { P: 'plane', plane: 'plane', Pl: 'plane', A: 'angle', Angle: 'angle' },
      },
      eval: ({ inputs }) => {
        const plane = ensurePlane(inputs.plane);
        let direction = ensureUnitVector(inputs.direction, plane.zAxis.clone());
        if (direction.lengthSq() < EPSILON) {
          return { plane: createPlaneInstance(plane), angle: 0 };
        }
        if (direction.dot(plane.zAxis) < 0) {
          direction = direction.clone().multiplyScalar(-1);
        }
        const startZ = plane.zAxis.clone().normalize();
        const dot = THREE.MathUtils.clamp(startZ.dot(direction), -1, 1);
        let angle = Math.acos(dot);
        let axis = startZ.clone().cross(direction);
        if (axis.lengthSq() < EPSILON) {
          if (angle < EPSILON) {
            return { plane: createPlaneInstance(plane), angle: 0 };
          }
          axis = plane.xAxis.clone();
          if (axis.lengthSq() < EPSILON) {
            axis = new THREE.Vector3(1, 0, 0);
          }
        }
        axis.normalize();
        const quaternion = new THREE.Quaternion().setFromAxisAngle(axis, angle);
        const xAxis = plane.xAxis.clone().applyQuaternion(quaternion);
        const yAxis = plane.yAxis.clone().applyQuaternion(quaternion);
        const result = normalizePlaneAxes(plane.origin.clone(), xAxis, yAxis, direction.clone());
        return { plane: createPlaneInstance(result), angle };
      },
    });

    register(['{f6f14b09-6497-4564-8403-09e4eb5a6b82}', 'rotate plane', 'prot'], {
      type: 'plane',
      pinMap: {
        inputs: { P: 'plane', plane: 'plane', Plane: 'plane', A: 'angle', Angle: 'angle', angle: 'angle' },
        outputs: { P: 'plane', plane: 'plane', Pl: 'plane' },
      },
      eval: ({ inputs }) => {
        const plane = ensurePlane(inputs.plane);
        const angle = toNumber(inputs.angle, 0);
        const rotated = rotatePlaneAroundNormal(plane, angle);
        return { plane: createPlaneInstance(rotated) };
      },
    });

    register(['{c73e1ed0-82a2-40b0-b4df-8f10e445d60b}', 'flip plane', 'pflip'], {
      type: 'plane',
      pinMap: {
        inputs: {
          P: 'plane',
          plane: 'plane',
          Plane: 'plane',
          X: 'reverseX',
          x: 'reverseX',
          Y: 'reverseY',
          y: 'reverseY',
          S: 'swap',
          s: 'swap',
        },
        outputs: { P: 'plane', plane: 'plane', Pl: 'plane' },
      },
      eval: ({ inputs }) => {
        const plane = ensurePlane(inputs.plane);
        const reverseX = toBooleanFlag(inputs.reverseX, false);
        const reverseY = toBooleanFlag(inputs.reverseY, false);
        const swap = toBooleanFlag(inputs.swap, false);
        let xAxis = plane.xAxis.clone();
        let yAxis = plane.yAxis.clone();
        if (swap) {
          const temp = xAxis;
          xAxis = yAxis;
          yAxis = temp;
        }
        if (reverseX) {
          xAxis = xAxis.clone().multiplyScalar(-1);
        }
        if (reverseY) {
          yAxis = yAxis.clone().multiplyScalar(-1);
        }
        const result = normalizePlaneAxes(plane.origin.clone(), xAxis, yAxis, plane.zAxis.clone());
        return { plane: createPlaneInstance(result) };
      },
    });
  }

  if (includeFieldComponents) {
    // Field subcategory components
    register(['{08619b6d-f9c4-4cb2-adcd-90959f08dc0d}', 'tensor display', 'ftensor'], {
      type: 'display',
      pinMap: {
        inputs: {
          F: 'field', field: 'field', Field: 'field',
          S: 'section', section: 'section', Section: 'section',
          N: 'samples', samples: 'samples', Samples: 'samples',
        },
      },
      eval: ({ inputs }) => {
        const [field] = collectFields(inputs.field);
        const mesh = createFieldDisplayMesh(field, inputs.section, inputs.samples, 'tensor', { name: 'tensor-display' });
        return mesh ? { mesh } : {};
      },
    });

    register(['{4b59e893-d4ee-4e31-ae24-a489611d1088}', 'spin force', 'fspin'], {
      type: 'field',
      pinMap: {
        inputs: {
          P: 'plane', plane: 'plane', Plane: 'plane',
          S: 'strength', strength: 'strength', Strength: 'strength',
          R: 'radius', radius: 'radius', Radius: 'radius',
          D: 'decay', decay: 'decay', Decay: 'decay',
          B: 'bounds', bounds: 'bounds', Bounds: 'bounds',
        },
        outputs: { F: 'field', field: 'field', Field: 'field' },
      },
      eval: ({ inputs }) => {
        const plane = ensurePlane(inputs.plane);
        const planeData = createPlaneInstance(plane);
        const strength = toNumber(inputs.strength, 1);
        const radius = Math.max(EPSILON, Math.abs(toNumber(inputs.radius, 5)));
        const decay = Math.max(0, toNumber(inputs.decay, 1));
        const bounds = ensureBounds(inputs.bounds);

        const influence = {
          type: 'spin',
          bounds,
          params: { plane: planeData, strength, radius, decay },
          evaluate: (point) => {
            const coords = planeCoordinates(point, planeData);
            const radial = planeData.xAxis.clone().multiplyScalar(coords.u).add(planeData.yAxis.clone().multiplyScalar(coords.v));
            const radialLength = radial.length();
            if (radialLength < EPSILON) {
              return { vector: new THREE.Vector3(), strength: 0 };
            }
            const normal = planeData.zAxis.clone().normalize();
            let tangent = normal.clone().cross(radial);
            if (tangent.lengthSq() < EPSILON) {
              tangent = planeData.xAxis.clone();
            }
            tangent.normalize();
            const normalizedDistance = radialLength / radius;
            const decayPower = decay > 0 ? decay : 1;
            const falloff = 1 / (1 + Math.pow(normalizedDistance, decayPower));
            const verticalFalloff = 1 / (1 + Math.abs(coords.w) / (radius || 1));
            const magnitude = strength * falloff * verticalFalloff;
            return {
              vector: tangent.multiplyScalar(magnitude),
              strength: Math.abs(magnitude),
            };
          },
        };

        const field = createField([influence], { bounds, metadata: { type: 'spin' } });
        return { field };
      },
    });

    register(['{55f9ce6a-490c-4f25-a536-a3d47b794752}', 'scalar display', 'fscalar'], {
      type: 'display',
      pinMap: {
        inputs: {
          F: 'field', field: 'field', Field: 'field',
          S: 'section', section: 'section', Section: 'section',
          N: 'samples', samples: 'samples', Samples: 'samples',
        },
        outputs: { D: 'display', display: 'display', Display: 'display' },
      },
      eval: ({ inputs }) => {
        const [field] = collectFields(inputs.field);
        const mesh = createFieldDisplayMesh(field, inputs.section, inputs.samples, 'scalar', { name: 'scalar-display' });
        if (!mesh) {
          return { display: null };
        }
        return { display: mesh, mesh };
      },
    });

    register(['{5ba20fab-6d71-48ea-a98f-cb034db6bbdc}', 'direction display', 'fdir'], {
      type: 'display',
      pinMap: {
        inputs: {
          F: 'field', field: 'field', Field: 'field',
          S: 'section', section: 'section', Section: 'section',
          N: 'samples', samples: 'samples', Samples: 'samples',
        },
        outputs: { D: 'display', display: 'display', Display: 'display' },
      },
      eval: ({ inputs }) => {
        const [field] = collectFields(inputs.field);
        const mesh = createFieldDisplayMesh(field, inputs.section, inputs.samples, 'direction', { name: 'direction-display' });
        if (!mesh) {
          return { display: null };
        }
        return { display: mesh, mesh };
      },
    });

    register(['{8cc9eb88-26a7-4baa-a896-13e5fc12416a}', 'line charge', 'lcharge'], {
      type: 'field',
      pinMap: {
        inputs: {
          L: 'line', line: 'line', Line: 'line',
          C: 'charge', charge: 'charge', Charge: 'charge',
          B: 'bounds', bounds: 'bounds', Bounds: 'bounds',
        },
        outputs: { F: 'field', field: 'field', Field: 'field' },
      },
      eval: ({ inputs }) => {
        const line = ensureLine(inputs.line);
        const charge = toNumber(inputs.charge, 1);
        const bounds = ensureBounds(inputs.bounds);
        const lineData = {
          start: line.start.clone(),
          end: line.end.clone(),
          direction: line.direction.clone(),
        };
        const lengthSq = lineData.direction.lengthSq();
        const unitDir = lengthSq > EPSILON ? lineData.direction.clone().normalize() : new THREE.Vector3(1, 0, 0);

        const influence = {
          type: 'line-charge',
          bounds,
          params: { line: lineData, charge },
          evaluate: (point) => {
            const ap = point.clone().sub(lineData.start);
            const denom = lengthSq > EPSILON ? lengthSq : 1;
            let t = denom ? ap.dot(lineData.direction) / denom : 0;
            t = THREE.MathUtils.clamp(t, 0, 1);
            const closest = lineData.start.clone().add(lineData.direction.clone().multiplyScalar(t));
            const offset = point.clone().sub(closest);
            let distance = offset.length();
            let direction = offset;
            if (distance < EPSILON) {
              direction = unitDir.clone().cross(new THREE.Vector3(0, 0, 1));
              if (direction.lengthSq() < EPSILON) {
                direction = unitDir.clone().cross(new THREE.Vector3(0, 1, 0));
              }
              if (direction.lengthSq() < EPSILON) {
                direction = new THREE.Vector3(0, 0, 1);
              }
              direction.normalize();
              distance = EPSILON;
            } else {
              direction = direction.clone().normalize();
            }
            const magnitude = charge / (1 + distance);
            return { vector: direction.multiplyScalar(magnitude), strength: Math.abs(magnitude) };
          },
        };

        const field = createField([influence], { bounds, metadata: { type: 'line-charge' } });
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
        outputs: { T: 'tensor', tensor: 'tensor', S: 'strength', strength: 'strength', Strength: 'strength' },
      },
      eval: ({ inputs }) => {
        const [field] = collectFields(inputs.field);
        if (inputs.point === undefined || inputs.point === null) {
          return { tensor: defaultFieldEvaluation().tensor, strength: 0 };
        }
        const samplePoint = toVector3(inputs.point, new THREE.Vector3());
        const evaluation = evaluateFieldAtPoint(field, samplePoint);
        return { tensor: evaluation.tensor, strength: evaluation.strength };
      },
    });

    register(['{add6be3e-c57f-4740-96e4-5680abaa9169}', 'field line', 'fline'], {
      type: 'field',
      pinMap: {
        inputs: {
          F: 'field', field: 'field', Field: 'field',
          P: 'point', point: 'point', Point: 'point',
          N: 'steps', steps: 'steps', Steps: 'steps',
          A: 'stepSize', accuracy: 'stepSize', Accuracy: 'stepSize',
          M: 'method', method: 'method', Method: 'method',
        },
        outputs: { C: 'curve', curve: 'curve', Curve: 'curve', P: 'points', points: 'points', Points: 'points' },
      },
      eval: ({ inputs }) => {
        const [field] = collectFields(inputs.field);
        if (!field) {
          return { curve: null, points: [] };
        }
        const start = toVector3(inputs.point, new THREE.Vector3());
        const steps = Math.max(1, Math.round(toNumber(inputs.steps, 50)));
        const stepSize = Math.max(EPSILON, Math.abs(toNumber(inputs.stepSize, 0.5)));
        const methodRaw = Math.round(toNumber(inputs.method, 4));
        const method = THREE.MathUtils.clamp(methodRaw, 1, 4);

        function normalizedDirection(point) {
          const evaluation = evaluateFieldAtPoint(field, point);
          const vector = evaluation.vector.clone();
          const length = vector.length();
          if (length < EPSILON) {
            return new THREE.Vector3();
          }
          return vector.divideScalar(length);
        }

        const points = [start.clone()];
        let current = start.clone();

        for (let index = 0; index < steps; index += 1) {
          let delta = new THREE.Vector3();
          switch (method) {
            case 1: {
              delta = normalizedDirection(current).multiplyScalar(stepSize);
              break;
            }
            case 2: {
              const k1 = normalizedDirection(current).multiplyScalar(stepSize / 2);
              const mid = current.clone().add(k1);
              const k2 = normalizedDirection(mid).multiplyScalar(stepSize);
              delta = k2;
              break;
            }
            case 3: {
              const k1 = normalizedDirection(current).multiplyScalar(stepSize);
              const predictor = current.clone().add(k1);
              const k2 = normalizedDirection(predictor).multiplyScalar(stepSize);
              delta = k1.add(k2).multiplyScalar(0.5);
              break;
            }
            default: {
              const k1 = normalizedDirection(current);
              const k2 = normalizedDirection(current.clone().add(k1.clone().multiplyScalar(stepSize / 2)));
              const k3 = normalizedDirection(current.clone().add(k2.clone().multiplyScalar(stepSize / 2)));
              const k4 = normalizedDirection(current.clone().add(k3.clone().multiplyScalar(stepSize)));
              delta = k1
                .clone()
                .add(k2.clone().multiplyScalar(2))
                .add(k3.clone().multiplyScalar(2))
                .add(k4)
                .multiplyScalar(stepSize / 6);
              break;
            }
          }

          if (delta.lengthSq() < EPSILON) {
            break;
          }
          current = current.clone().add(delta);
          points.push(current.clone());
        }

        const curve = points.length > 1 ? new THREE.CatmullRomCurve3(points) : null;
        return { curve, points };
      },
    });

    register(['{b27d53bc-e713-475d-81fd-71cdd8de2e58}', 'break field', 'breakf'], {
      type: 'field',
      pinMap: {
        inputs: { F: 'field', field: 'field', Field: 'field' },
        outputs: { F: 'fields', fields: 'fields', Fields: 'fields' },
      },
      eval: ({ inputs }) => {
        const [field] = collectFields(inputs.field);
        if (!field) {
          return { fields: [] };
        }
        const parts = field.influences?.map?.((influence) => createField([influence], {
          bounds: mergeBounds(field.bounds ?? null, influence.bounds ?? null),
          metadata: influence.metadata ?? {},
        })) ?? [];
        return { fields: parts };
      },
    });

    register(['{bf106e4c-68f4-476f-b05b-9c15fb50e078}', 'perpendicular display', 'fperp'], {
      type: 'display',
      pinMap: {
        inputs: {
          F: 'field', field: 'field', Field: 'field',
          S: 'section', section: 'section', Section: 'section',
          N: 'samples', samples: 'samples', Samples: 'samples',
          'C+': 'positiveColor', Cplus: 'positiveColor', positive: 'positiveColor',
          'C-': 'negativeColor', Cminus: 'negativeColor', negative: 'negativeColor',
        },
        outputs: { D: 'display', display: 'display', Display: 'display' },
      },
      eval: ({ inputs }) => {
        const [field] = collectFields(inputs.field);
        const mesh = createFieldDisplayMesh(field, inputs.section, inputs.samples, 'perpendicular', {
          name: 'perpendicular-display',
          positiveColor: inputs.positiveColor,
          negativeColor: inputs.negativeColor,
        });
        if (!mesh) {
          return { display: null };
        }
        return { display: mesh, mesh };
      },
    });

    register(['{cffdbaf3-8d33-4b38-9cad-c264af9fc3f4}', 'point charge', 'pcharge'], {
      type: 'field',
      pinMap: {
        inputs: {
          P: 'point', point: 'point', Point: 'point',
          C: 'charge', charge: 'charge', Charge: 'charge',
          D: 'decay', decay: 'decay', Decay: 'decay',
          B: 'bounds', bounds: 'bounds', Bounds: 'bounds',
        },
        outputs: { F: 'field', field: 'field', Field: 'field' },
      },
      eval: ({ inputs }) => {
        const position = toVector3(inputs.point, new THREE.Vector3());
        const charge = toNumber(inputs.charge, 1);
        const decay = Math.max(0, toNumber(inputs.decay, 2));
        const bounds = ensureBounds(inputs.bounds);

        const influence = {
          type: 'point-charge',
          bounds,
          params: { position, charge, decay },
          evaluate: (point) => {
            const direction = point.clone().sub(position);
            let distance = direction.length();
            if (distance < EPSILON) {
              distance = EPSILON;
            }
            const normalized = direction.divideScalar(distance);
            const falloffPower = decay > 0 ? decay : 1;
            const magnitude = charge / (1 + Math.pow(distance, falloffPower));
            return { vector: normalized.multiplyScalar(magnitude), strength: Math.abs(magnitude) };
          },
        };

        const field = createField([influence], { bounds, metadata: { type: 'point-charge' } });
        return { field };
      },
    });

    register(['{d27cc1ea-9ef7-47bf-8ee2-c6662da0e3d9}', 'vector force', 'fvector'], {
      type: 'field',
      pinMap: {
        inputs: {
          L: 'line', line: 'line', Line: 'line',
          B: 'bounds', bounds: 'bounds', Bounds: 'bounds',
        },
        outputs: { F: 'field', field: 'field', Field: 'field' },
      },
      eval: ({ inputs }) => {
        const line = ensureLine(inputs.line);
        const bounds = ensureBounds(inputs.bounds);
        const direction = line.direction.clone();
        const length = direction.length();
        const unitDir = length > EPSILON ? direction.clone().normalize() : new THREE.Vector3(1, 0, 0);
        const lengthSq = direction.lengthSq();

        const influence = {
          type: 'vector-force',
          bounds,
          params: { line: { start: line.start.clone(), end: line.end.clone(), direction: direction.clone() } },
          evaluate: (point) => {
            const ap = point.clone().sub(line.start);
            const denom = lengthSq > EPSILON ? lengthSq : 1;
            let t = denom ? ap.dot(direction) / denom : 0;
            t = THREE.MathUtils.clamp(t, 0, 1);
            const closest = line.start.clone().add(direction.clone().multiplyScalar(t));
            const distance = point.distanceTo(closest);
            const falloff = 1 / (1 + distance);
            const magnitude = (length > EPSILON ? length : 1) * falloff;
            return { vector: unitDir.clone().multiplyScalar(magnitude), strength: Math.abs(magnitude) };
          },
        };

        const field = createField([influence], { bounds, metadata: { type: 'vector-force' } });
        return { field };
      },
    });

    register(['{d9a6fbd2-2e9f-472e-8147-33bf0233a115}', 'merge fields', 'mergef'], {
      type: 'field',
      pinMap: {
        inputs: { F: 'fields', fields: 'fields', Fields: 'fields' },
        outputs: { F: 'field', field: 'field', Field: 'field' },
      },
      eval: ({ inputs }) => {
        const fields = collectFields(inputs.fields);
        if (!fields.length) {
          return { field: null };
        }
        const influences = [];
        let combinedBounds = null;
        for (const entry of fields) {
          combinedBounds = mergeBounds(combinedBounds, entry.bounds ?? null);
          if (Array.isArray(entry.influences)) {
            for (const influence of entry.influences) {
              influences.push(influence);
            }
          }
        }
        if (!influences.length) {
          return { field: null };
        }
        const field = createField(influences, { bounds: combinedBounds });
        return { field };
      },
    });
  }

  if (!includePointComponents) {
    return;
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

  if (!includeVectorComponents) {
    return;
  }

  // Vector subcategory components
  register([
    '{152a264e-fc74-40e5-88cc-d1a681cd09c3}',
    'angle',
  ], {
    type: 'vector',
    pinMap: {
      inputs: { A: 'a', a: 'a', 'Vector A': 'a', B: 'b', b: 'b', 'Vector B': 'b' },
      outputs: { A: 'angle', angle: 'angle', R: 'reflex', reflex: 'reflex' },
    },
    eval: ({ inputs }) => {
      const vectorA = toVector3(inputs.a, new THREE.Vector3());
      const vectorB = toVector3(inputs.b, new THREE.Vector3());
      const { angle, reflex } = computeAngleInfo(vectorA, vectorB);
      return { angle, reflex };
    },
  });

  register([
    '{b464fccb-50e7-41bd-9789-8438db9bea9f}',
    'angle plane',
    'angle (plane)',
  ], {
    type: 'vector',
    pinMap: {
      inputs: {
        A: 'a', a: 'a', 'Vector A': 'a',
        B: 'b', b: 'b', 'Vector B': 'b',
        P: 'plane', p: 'plane', Plane: 'plane', plane: 'plane',
      },
      outputs: { A: 'angle', angle: 'angle', R: 'reflex', reflex: 'reflex' },
    },
    eval: ({ inputs }) => {
      const vectorA = toVector3(inputs.a, new THREE.Vector3());
      const vectorB = toVector3(inputs.b, new THREE.Vector3());
      const plane = ensurePlane(inputs.plane);
      const { angle, reflex } = computeAngleInfo(vectorA, vectorB, plane.zAxis.clone().normalize());
      return { angle, reflex };
    },
  });

  register([
    '{2a5cfb31-028a-4b34-b4e1-9b20ae15312e}',
    'cross product',
    'vector cross product',
    'xprod',
  ], {
    type: 'vector',
    pinMap: {
      inputs: {
        A: 'a', a: 'a', 'Vector A': 'a',
        B: 'b', b: 'b', 'Vector B': 'b',
        U: 'unitize', u: 'unitize', Unitize: 'unitize', unitize: 'unitize',
      },
      outputs: { V: 'vector', vector: 'vector', L: 'length', length: 'length' },
    },
    eval: ({ inputs }) => {
      const vectorA = toVector3(inputs.a, new THREE.Vector3());
      const vectorB = toVector3(inputs.b, new THREE.Vector3());
      const result = vectorA.clone().cross(vectorB);
      const length = result.length();
      const shouldUnitize = toBooleanFlag(inputs.unitize, false);
      const vector = shouldUnitize && length > EPSILON ? result.divideScalar(length) : result;
      return { vector, length };
    },
  });

  register([
    '{fb012ef9-4734-4049-84a0-b92b85bb09da}',
    'vector addition',
    'addition',
    'vadd',
  ], {
    type: 'vector',
    pinMap: {
      inputs: {
        A: 'a', a: 'a', 'Vector A': 'a',
        B: 'b', b: 'b', 'Vector B': 'b',
        U: 'unitize', u: 'unitize', Unitize: 'unitize', unitize: 'unitize',
      },
      outputs: { V: 'vector', vector: 'vector', L: 'length', length: 'length' },
    },
    eval: ({ inputs }) => {
      const vectorA = toVector3(inputs.a, new THREE.Vector3());
      const vectorB = toVector3(inputs.b, new THREE.Vector3());
      const result = vectorA.clone().add(vectorB);
      const length = result.length();
      const shouldUnitize = toBooleanFlag(inputs.unitize, false);
      const vector = shouldUnitize && length > EPSILON ? result.divideScalar(length) : result;
      return { vector, length };
    },
  });

  register([
    '{310e1065-d03a-4858-bcd1-809d39c042af}',
    'vector divide',
    'divide',
    'vdiv',
  ], {
    type: 'vector',
    pinMap: {
      inputs: { V: 'vector', vector: 'vector', 'Vector': 'vector', F: 'factor', f: 'factor', Factor: 'factor' },
      outputs: { V: 'vector', vector: 'vector', L: 'length', length: 'length' },
    },
    eval: ({ inputs }) => {
      const base = toVector3(inputs.vector, new THREE.Vector3());
      const factor = toNumber(inputs.factor, 1);
      if (!Number.isFinite(factor) || Math.abs(factor) < EPSILON) {
        return { vector: new THREE.Vector3(), length: 0 };
      }
      const vector = base.divideScalar(factor);
      return { vector, length: vector.length() };
    },
  });

  register([
    '{63fff845-7c61-4dfb-ba12-44d481b4bf0f}',
    'vector multiply',
    'multiply',
    'vmul',
  ], {
    type: 'vector',
    pinMap: {
      inputs: { V: 'vector', vector: 'vector', 'Vector': 'vector', F: 'factor', f: 'factor', Factor: 'factor' },
      outputs: { V: 'vector', vector: 'vector', L: 'length', length: 'length' },
    },
    eval: ({ inputs }) => {
      const base = toVector3(inputs.vector, new THREE.Vector3());
      const factor = toNumber(inputs.factor, 1);
      const vector = base.multiplyScalar(factor);
      return { vector, length: vector.length() };
    },
  });

  register([
    '{63f79e72-36c0-4489-a0c2-9ded0b9ca41f}',
    'vector mass addition',
    'mass addition',
    'massadd',
  ], {
    type: 'vector',
    pinMap: {
      inputs: {
        V: 'vectors', v: 'vectors', vectors: 'vectors', Vectors: 'vectors',
        U: 'unitize', u: 'unitize', Unitize: 'unitize', unitize: 'unitize',
      },
      outputs: { V: 'vector', vector: 'vector', L: 'length', length: 'length' },
    },
    eval: ({ inputs }) => {
      const vectors = collectVectors(inputs.vectors);
      const sum = sumVectors(vectors);
      const length = sum.length();
      const shouldUnitize = toBooleanFlag(inputs.unitize, false);
      const vector = shouldUnitize && length > EPSILON ? sum.divideScalar(length) : sum;
      return { vector, length };
    },
  });

  register([
    '{b7f1178f-4222-47fd-9766-5d06e869362b}',
    'vector mass addition (unit)',
    'mass addition unit',
  ], {
    type: 'vector',
    pinMap: {
      inputs: {
        V: 'vectors', v: 'vectors', vectors: 'vectors', Vectors: 'vectors',
        U: 'unitize', u: 'unitize', Unitize: 'unitize', unitize: 'unitize',
      },
      outputs: { V: 'vector', vector: 'vector' },
    },
    eval: ({ inputs }) => {
      const vectors = collectVectors(inputs.vectors);
      const sum = sumVectors(vectors);
      const length = sum.length();
      const shouldUnitize = toBooleanFlag(inputs.unitize, false);
      const vector = shouldUnitize && length > EPSILON ? sum.divideScalar(length) : sum;
      return { vector };
    },
  });

  register([
    '{43b9ea8f-f772-40f2-9880-011a9c3cbbb0}',
    'dot product',
    'vector dot product',
    'dprod',
  ], {
    type: 'vector',
    pinMap: {
      inputs: {
        A: 'a', a: 'a', 'Vector A': 'a',
        B: 'b', b: 'b', 'Vector B': 'b',
        U: 'unitize', u: 'unitize', Unitize: 'unitize', unitize: 'unitize',
      },
      outputs: { D: 'dot', dot: 'dot', 'Dot product': 'dot' },
    },
    eval: ({ inputs }) => {
      const vectorA = toVector3(inputs.a, new THREE.Vector3());
      const vectorB = toVector3(inputs.b, new THREE.Vector3());
      if (toBooleanFlag(inputs.unitize, false)) {
        if (vectorA.lengthSq() > EPSILON) vectorA.normalize();
        if (vectorB.lengthSq() > EPSILON) vectorB.normalize();
      }
      return { dot: vectorA.dot(vectorB) };
    },
  });

  register([
    '{675e31bf-1775-48d7-bb8d-76b77786dd53}',
    'vector length',
    'vlen',
  ], {
    type: 'vector',
    pinMap: {
      inputs: { V: 'vector', v: 'vector', vector: 'vector', Vector: 'vector' },
      outputs: { L: 'length', length: 'length' },
    },
    eval: ({ inputs }) => {
      const vector = toVector3(inputs.vector, new THREE.Vector3());
      return { length: vector.length() };
    },
  });

  register([
    '{6ec39468-dae7-4ffa-a766-f2ab22a2c62e}',
    'amplitude',
    'vector amplitude',
    'amp',
  ], {
    type: 'vector',
    pinMap: {
      inputs: { V: 'vector', vector: 'vector', Vector: 'vector', A: 'amplitude', a: 'amplitude', Amplitude: 'amplitude' },
      outputs: { V: 'vector', vector: 'vector' },
    },
    eval: ({ inputs }) => {
      const vector = toVector3(inputs.vector, new THREE.Vector3());
      const target = toNumber(inputs.amplitude, vector.length());
      if (vector.lengthSq() < EPSILON || !Number.isFinite(target)) {
        return { vector: new THREE.Vector3() };
      }
      const scaled = vector.clone().setLength(target);
      return { vector: scaled };
    },
  });

  register([
    '{59e1f848-38d4-4cbf-ad7f-40ffc52acdf5}',
    'solar incidence',
    'solar vector',
    'solar',
  ], {
    type: 'vector',
    pinMap: {
      inputs: {
        L: 'location', l: 'location', location: 'location', Location: 'location',
        T: 'time', t: 'time', time: 'time', Time: 'time',
        P: 'orientation', p: 'orientation', Orientation: 'orientation', plane: 'orientation', Plane: 'orientation',
      },
      outputs: {
        D: 'direction', direction: 'direction',
        E: 'elevation', elevation: 'elevation',
        H: 'horizon', horizon: 'horizon',
        C: 'colour', colour: 'colour', color: 'colour', Colour: 'colour',
      },
    },
    eval: ({ inputs }) => {
      const location = ensureGeoLocation(inputs.location);
      const date = ensureDate(inputs.time);
      const timezoneHours = Number.isFinite(location.timezoneHours)
        ? location.timezoneHours
        : -date.getTimezoneOffset() / 60;
      const { elevation, azimuth } = computeSolarPosition(
        date,
        location.latitude,
        location.longitude,
        timezoneHours,
      );
      const plane = ensurePlane(inputs.orientation);
      const cosElevation = Math.cos(elevation);
      const east = cosElevation * Math.sin(azimuth);
      const north = cosElevation * Math.cos(azimuth);
      const up = Math.sin(elevation);
      const direction = plane.xAxis.clone().multiplyScalar(east)
        .add(plane.yAxis.clone().multiplyScalar(north))
        .add(plane.zAxis.clone().multiplyScalar(up));
      if (direction.lengthSq() > EPSILON) {
        direction.normalize();
      } else {
        direction.set(0, 0, 0);
      }
      const horizon = elevation > 0;
      const colour = solarColourFromElevation(elevation);
      return { direction, elevation, horizon, colour };
    },
  });

  register([
    '{934ede4a-924a-4973-bb05-0dc4b36fae75}',
    'vector 2pt',
    'vector two point',
    'vec2pt',
  ], {
    type: 'vector',
    pinMap: {
      inputs: {
        A: 'a', a: 'a', 'Point A': 'a',
        B: 'b', b: 'b', 'Point B': 'b',
        U: 'unitize', u: 'unitize', Unitize: 'unitize', unitize: 'unitize',
      },
      outputs: { V: 'vector', vector: 'vector', L: 'length', length: 'length' },
    },
    eval: ({ inputs }) => {
      const pointA = toVector3(inputs.a, new THREE.Vector3());
      const pointB = toVector3(inputs.b, new THREE.Vector3());
      const result = pointB.clone().sub(pointA);
      const length = result.length();
      const shouldUnitize = toBooleanFlag(inputs.unitize, false);
      const vector = shouldUnitize && length > EPSILON ? result.divideScalar(length) : result;
      return { vector, length };
    },
  });

  register([
    '{a50fcd4a-cf42-4c3f-8616-022761e6cc93}',
    'deconstruct vector',
    'devec',
  ], {
    type: 'vector',
    pinMap: {
      inputs: { V: 'vector', v: 'vector', vector: 'vector', Vector: 'vector' },
      outputs: { X: 'x', x: 'x', Y: 'y', y: 'y', Z: 'z', z: 'z' },
    },
    eval: ({ inputs }) => {
      const vector = toVector3(inputs.vector, new THREE.Vector3());
      return { x: vector.x, y: vector.y, z: vector.z };
    },
  });

  register([
    '{d2da1306-259a-4994-85a4-672d8a4c7805}',
    'unit vector',
    'unitize vector',
    'unit',
  ], {
    type: 'vector',
    pinMap: {
      inputs: { V: 'vector', v: 'vector', vector: 'vector', Vector: 'vector' },
      outputs: { V: 'vector', vector: 'vector' },
    },
    eval: ({ inputs }) => {
      const vector = toVector3(inputs.vector, new THREE.Vector3());
      return { vector: normalizeVector(vector) };
    },
  });

  register([
    '{d5788074-d75d-4021-b1a3-0bf992928584}',
    'vector reverse',
    'reverse',
    'rev',
  ], {
    type: 'vector',
    pinMap: {
      inputs: { V: 'vector', v: 'vector', vector: 'vector', Vector: 'vector' },
      outputs: { V: 'vector', vector: 'vector' },
    },
    eval: ({ inputs }) => {
      const vector = toVector3(inputs.vector, new THREE.Vector3());
      return { vector: vector.multiplyScalar(-1) };
    },
  });

  register([
    '{b6d7ba20-cf74-4191-a756-2216a36e30a7}',
    'vector rotate',
    'rotate',
    'vrot',
  ], {
    type: 'vector',
    pinMap: {
      inputs: {
        V: 'vector', v: 'vector', vector: 'vector', Vector: 'vector',
        X: 'axis', x: 'axis', Axis: 'axis', axis: 'axis',
        A: 'angle', a: 'angle', Angle: 'angle', angle: 'angle',
      },
      outputs: { V: 'vector', vector: 'vector' },
    },
    eval: ({ inputs }) => {
      const vector = toVector3(inputs.vector, new THREE.Vector3());
      const axis = ensureUnitVector(inputs.axis ?? new THREE.Vector3(0, 0, 1), new THREE.Vector3(0, 0, 1));
      const angle = toNumber(inputs.angle, 0);
      if (axis.lengthSq() < EPSILON) {
        return { vector };
      }
      const quaternion = new THREE.Quaternion().setFromAxisAngle(axis, angle);
      const rotated = vector.clone().applyQuaternion(quaternion);
      return { vector: rotated };
    },
  });

  register([
    '{79f9fbb3-8f1d-4d9a-88a9-f7961b1012cd}',
    'unit x',
    'unit vector x',
  ], {
    type: 'vector',
    pinMap: {
      inputs: { F: 'factor', f: 'factor', Factor: 'factor', factor: 'factor' },
      outputs: { V: 'vector', vector: 'vector' },
    },
    eval: ({ inputs }) => {
      const factor = toNumber(inputs.factor, 1);
      return { vector: new THREE.Vector3(factor, 0, 0) };
    },
  });

  register([
    '{d3d195ea-2d59-4ffa-90b1-8b7ff3369f69}',
    'unit y',
    'unit vector y',
  ], {
    type: 'vector',
    pinMap: {
      inputs: { F: 'factor', f: 'factor', Factor: 'factor', factor: 'factor' },
      outputs: { V: 'vector', vector: 'vector' },
    },
    eval: ({ inputs }) => {
      const factor = toNumber(inputs.factor, 1);
      return { vector: new THREE.Vector3(0, factor, 0) };
    },
  });

  register([
    '{9103c240-a6a9-4223-9b42-dbd19bf38e2b}',
    'unit z',
    'unit vector z',
  ], {
    type: 'vector',
    pinMap: {
      inputs: { F: 'factor', f: 'factor', Factor: 'factor', factor: 'factor' },
      outputs: { V: 'vector', vector: 'vector' },
    },
    eval: ({ inputs }) => {
      const factor = toNumber(inputs.factor, 1);
      return { vector: new THREE.Vector3(0, 0, factor) };
    },
  });

  register([
    '{56b92eab-d121-43f7-94d3-6cd8f0ddead8}',
    'vector xyz',
    'vec',
  ], {
    type: 'vector',
    pinMap: {
      inputs: {
        X: 'x', x: 'x', 'X component': 'x',
        Y: 'y', y: 'y', 'Y component': 'y',
        Z: 'z', z: 'z', 'Z component': 'z',
      },
      outputs: { V: 'vector', vector: 'vector', L: 'length', length: 'length' },
    },
    eval: ({ inputs }) => {
      const x = toNumber(inputs.x, 0);
      const y = toNumber(inputs.y, 0);
      const z = toNumber(inputs.z, 0);
      const vector = new THREE.Vector3(x, y, z);
      return { vector, length: vector.length() };
    },
  });
  }


export function registerVectorFieldComponents({ register, toNumber, toVector3 }) {
  return registerVectorPointComponents({ register, toNumber, toVector3 }, {
    includeFieldComponents: true,
    includePointComponents: false,
    includePlaneComponents: false,
    includeVectorComponents: false,
  });
}

export function registerVectorVectorComponents({ register, toNumber, toVector3 }) {
  const vectorOnlyRegister = (keys, config) => {
    if (config?.type === 'vector') {
      register(keys, config);
    }
  };

  return registerVectorPointComponents({ register: vectorOnlyRegister, toNumber, toVector3 }, {
    includeFieldComponents: false,
    includePointComponents: true,
    includePlaneComponents: false,
    includeVectorComponents: true,
  });
}

export function registerVectorPlaneComponents({ register, toNumber, toVector3 }) {
  return registerVectorPointComponents({ register, toNumber, toVector3 }, {
    includeFieldComponents: false,
    includePointComponents: false,
    includePlaneComponents: true,
    includeVectorComponents: false,
  });
}

import * as THREE from 'three/webgpu';
import { SkyMesh } from 'three/addons/objects/SkyMesh.js';

const DEG2RAD = Math.PI / 180;
const RAD2DEG = 180 / Math.PI;
const DAY_MS = 1000 * 60 * 60 * 24;
const JULIAN_EPOCH = 2440587.5;
const JULIAN_J2000 = 2451545;
const EARTH_TILT = DEG2RAD * 23.4397;
const SUN_DISTANCE = 100000;
const SKY_UP_VECTOR = new THREE.Vector3(0, 0, 1);
const DEFAULT_OPTIONS = {
  lat: 52.3676,
  lon: 4.9041,
  elevation: 0,
  datetime: null,
  utcOffset: null,
  turbidity: 3.5,
  rayleigh: 1.2,
  mieCoefficient: 0.005,
  mieDirectionalG: 0.8,
  groundAlbedo: 0.25,
  exposure: 1.0,
  intensityMultiplier: 1.0,
};

function clamp(value, min, max) {
  return Math.min(Math.max(value, min), max);
}

function toJulian(date) {
  return date / DAY_MS + JULIAN_EPOCH;
}

function toDays(date) {
  return toJulian(date) - JULIAN_J2000;
}

function solarMeanAnomaly(d) {
  return DEG2RAD * (357.5291 + 0.98560028 * d);
}

function eclipticLongitude(M) {
  const C = DEG2RAD * (1.9148 * Math.sin(M) + 0.02 * Math.sin(2 * M) + 0.0003 * Math.sin(3 * M));
  const P = DEG2RAD * 102.9372;
  return M + C + P + Math.PI;
}

function declination(L) {
  return Math.asin(Math.sin(L) * Math.sin(EARTH_TILT));
}

function rightAscension(L) {
  return Math.atan2(Math.sin(L) * Math.cos(EARTH_TILT), Math.cos(L));
}

function siderealTime(d, lw) {
  return DEG2RAD * (280.16 + 360.9856235 * d) - lw;
}

function altitude(H, phi, dec) {
  return Math.asin(Math.sin(phi) * Math.sin(dec) + Math.cos(phi) * Math.cos(dec) * Math.cos(H));
}

function azimuth(H, phi, dec) {
  return Math.atan2(Math.sin(H), Math.cos(H) * Math.sin(phi) - Math.tan(dec) * Math.cos(phi));
}

function applyAtmosphericRefraction(height) {
  if (height <= 0) {
    return 0;
  }
  // Bennett's refraction formula (height in radians, returns radians)
  return DEG2RAD * 0.0002967 / Math.tan(height + DEG2RAD * 0.00312536 / (height + DEG2RAD * 0.08901179));
}

function resolveDateTimestamp(options) {
  const { datetime, utcOffset } = options;
  const baseDate = datetime instanceof Date ? datetime : (datetime ? new Date(datetime) : new Date());
  if (Number.isFinite(utcOffset)) {
    const localOffset = -baseDate.getTimezoneOffset() / 60;
    const delta = utcOffset - localOffset;
    return baseDate.getTime() - delta * 60 * 60 * 1000;
  }
  return baseDate.getTime();
}

function computeSunAngles(options) {
  const { lat, lon } = options;
  if (!Number.isFinite(lat) || !Number.isFinite(lon)) {
    return { azimuth: Math.PI, elevation: Math.PI / 4 };
  }

  const timestamp = resolveDateTimestamp(options);
  const date = new Date(timestamp);
  const lw = DEG2RAD * -lon;
  const phi = DEG2RAD * lat;
  const d = toDays(date.getTime());
  const M = solarMeanAnomaly(d);
  const L = eclipticLongitude(M);
  const dec = declination(L);
  const RA = rightAscension(L);
  const H = siderealTime(d, lw) - RA;
  const rawAltitude = altitude(H, phi, dec);
  const correctedAltitude = rawAltitude + applyAtmosphericRefraction(Math.max(rawAltitude, 0));
  const sunAzimuth = azimuth(H, phi, dec);
  return {
    azimuth: sunAzimuth,
    elevation: correctedAltitude,
  };
}

function computeSunColor(elevation, turbidity) {
  const warm = new THREE.Color(0xffd1a4);
  const cool = new THREE.Color(0xf4f6ff);
  const haze = clamp((turbidity - 2) / 8, 0, 1);
  const normalized = clamp((RAD2DEG * elevation + 6) / 96, 0, 1);
  const mix = Math.pow(normalized, 1.2);
  const color = warm.clone().lerp(cool, mix).lerp(warm, haze * 0.35);
  return color;
}

function computeSunIlluminance(elevation) {
  const sine = Math.sin(Math.max(elevation, 0));
  if (sine <= 0) {
    return 0;
  }
  const baseLux = 120000; // midday sun on a clear day
  return baseLux * Math.pow(sine, 0.6);
}

function setScalarUniform(target, value) {
  if (!target) {
    return;
  }
  if ('value' in target) {
    target.value = value;
  } else if (typeof target.setValue === 'function') {
    target.setValue(value);
  }
}

function setVectorUniform(target, vector) {
  if (!target || !vector?.isVector3) {
    return;
  }
  if (target.value?.isVector3) {
    target.value.copy(vector);
  } else if (typeof target.copy === 'function') {
    target.copy(vector);
  }
}

export class PhysicalSunSky {
  constructor(scene, options = {}) {
    this.scene = scene;
    this.options = { ...DEFAULT_OPTIONS, ...options };
    this.sunDirection = new THREE.Vector3(0, 0, 1);
    this.sky = new SkyMesh();
    this.sky.name = 'PhysicalSunSkyDome';
    this.sky.scale.setScalar(SUN_DISTANCE * 0.9);
    this.sky.frustumCulled = false;
    this.sky.material.depthWrite = false;
    setScalarUniform(this.sky.turbidity, this.options.turbidity);
    setScalarUniform(this.sky.rayleigh, this.options.rayleigh);
    setScalarUniform(this.sky.mieCoefficient, this.options.mieCoefficient);
    setScalarUniform(this.sky.mieDirectionalG, this.options.mieDirectionalG);
    setVectorUniform(this.sky.upUniform, SKY_UP_VECTOR);

    this.scene.add(this.sky);

    this.sunLight = new THREE.DirectionalLight(0xffffff, 1);
    this.sunLight.name = 'PhysicalSunLight';
    this.sunLight.castShadow = true;
    this.sunLight.shadow.bias = -0.0005;
    this.sunLight.shadow.mapSize.set(2048, 2048);
    const shadowCamera = this.sunLight.shadow.camera;
    if (shadowCamera && shadowCamera.isOrthographicCamera) {
      shadowCamera.left = -2000;
      shadowCamera.right = 2000;
      shadowCamera.top = 2000;
      shadowCamera.bottom = -2000;
      shadowCamera.near = 1;
      shadowCamera.far = SUN_DISTANCE;
    }
    this.sunTarget = new THREE.Object3D();
    this.sunTarget.name = 'PhysicalSunTarget';
    this.scene.add(this.sunTarget);
    this.sunLight.target = this.sunTarget;
    this.scene.add(this.sunLight);

    this.fillLight = new THREE.HemisphereLight(0xffffff, 0x444444, 0.25);
    this.fillLight.name = 'PhysicalSkyHemisphere';
    this.scene.add(this.fillLight);

    this.renderer = null;
    this.pmremGenerator = null;
    this.environmentTarget = null;
    this.needsEnvironmentUpdate = true;

    this.update();
  }

  setRenderer(renderer) {
    if (!renderer || renderer === this.renderer) {
      return;
    }
    this.renderer = renderer;
    try {
      this.pmremGenerator?.dispose?.();
      this.pmremGenerator = new THREE.PMREMGenerator(renderer);
    } catch (error) {
      console.warn('PhysicalSunSky: kon PMREMGenerator niet initialiseren', error);
      this.pmremGenerator = null;
    }
    this.needsEnvironmentUpdate = true;
    this.applyExposure();
    this.updateEnvironment();
  }

  applyExposure() {
    if (this.renderer && 'toneMappingExposure' in this.renderer) {
      this.renderer.toneMappingExposure = this.options.exposure;
    }
  }

  update(options = {}) {
    this.options = { ...this.options, ...options };

    setScalarUniform(this.sky.turbidity, this.options.turbidity);
    setScalarUniform(this.sky.rayleigh, this.options.rayleigh);
    setScalarUniform(this.sky.mieCoefficient, this.options.mieCoefficient);
    setScalarUniform(this.sky.mieDirectionalG, this.options.mieDirectionalG);

    const { azimuth, elevation } = computeSunAngles(this.options);
    const phi = clamp(Math.PI / 2 - elevation, 0.0001, Math.PI - 0.0001);
    const theta = azimuth;
    this.sunDirection.setFromSphericalCoords(1, phi, theta);
    setVectorUniform(this.sky.sunPosition, this.sunDirection);

    const sunColor = computeSunColor(elevation, this.options.turbidity);
    this.sunLight.color.copy(sunColor);
    const lux = computeSunIlluminance(elevation) * this.options.intensityMultiplier;
    this.sunLight.intensity = lux;
    this.sunLight.position.copy(this.sunDirection).multiplyScalar(-SUN_DISTANCE);
    this.sunTarget.position.set(0, 0, 0);
    this.sunLight.updateMatrixWorld();
    this.sunTarget.updateMatrixWorld();

    const hemiSky = sunColor.clone().lerp(new THREE.Color(0x87ceeb), 0.35);
    const ground = new THREE.Color().setScalar(clamp(this.options.groundAlbedo, 0, 1));
    this.fillLight.color.copy(hemiSky);
    this.fillLight.groundColor.copy(ground);
    this.fillLight.intensity = 0.15 + 0.55 * clamp(Math.sin(Math.max(elevation, 0)), 0, 1);

    this.needsEnvironmentUpdate = true;
    this.applyExposure();
    this.updateEnvironment();
  }

  updateEnvironment() {
    if (!this.needsEnvironmentUpdate) {
      return;
    }
    if (!this.pmremGenerator || !this.renderer) {
      return;
    }
    this.needsEnvironmentUpdate = false;
    try {
      const target = this.pmremGenerator.fromScene(this.sky);
      if (target) {
        this.environmentTarget?.dispose?.();
        this.environmentTarget = target;
        this.scene.environment = target.texture;
      }
    } catch (error) {
      console.warn('PhysicalSunSky: kon environment niet bijwerken', error);
    }
  }

  updateFrame(camera) {
    if (camera && camera.position) {
      this.sky.position.copy(camera.position);
      this.sky.updateMatrixWorld();
    }
    this.updateEnvironment();
  }

  dispose() {
    this.scene.remove(this.sky);
    this.scene.remove(this.sunLight);
    this.scene.remove(this.sunTarget);
    this.scene.remove(this.fillLight);
    this.environmentTarget?.dispose?.();
    this.pmremGenerator?.dispose?.();
    this.sky.material.dispose?.();
    this.sky.geometry?.dispose?.();
  }
}

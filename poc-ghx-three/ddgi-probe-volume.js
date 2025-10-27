import * as THREE from 'three/webgpu';

const DEFAULT_OPTIONS = {
  probeSpacing: 750,
  updateBudget: 64,
  hysteresis: 0.92,
  maxDistance: 6000,
  boundsPadding: 500,
  sampleCount: 24,
};

const SH_BASIS_COUNT = 9;
const FOUR_PI = 4 * Math.PI;

function generateDirections(samples) {
  const directions = [];
  if (!Number.isFinite(samples) || samples <= 0) {
    return directions;
  }
  const goldenAngle = Math.PI * (3 - Math.sqrt(5));
  for (let i = 0; i < samples; i += 1) {
    const y = 1 - (i / (samples - 1)) * 2;
    const radius = Math.sqrt(Math.max(1 - y * y, 0));
    const theta = goldenAngle * i;
    const x = Math.cos(theta) * radius;
    const z = Math.sin(theta) * radius;
    directions.push(new THREE.Vector3(x, y, z));
  }
  return directions;
}

const TEMP_VECTOR = new THREE.Vector3();
const TEMP_VECTOR_2 = new THREE.Vector3();
const TEMP_VECTOR_3 = new THREE.Vector3();
const TEMP_SH = new THREE.SphericalHarmonics3();

const SH_BASIS_FUNCTIONS = [
  (dir) => 0.282095,
  (dir) => 0.488603 * dir.y,
  (dir) => 0.488603 * dir.z,
  (dir) => 0.488603 * dir.x,
  (dir) => 1.092548 * dir.x * dir.y,
  (dir) => 1.092548 * dir.y * dir.z,
  (dir) => 0.315392 * (3 * dir.z * dir.z - 1),
  (dir) => 1.092548 * dir.x * dir.z,
  (dir) => 0.546274 * (dir.x * dir.x - dir.y * dir.y),
];

function addScaledColor(target, color, scale) {
  if (!color || scale === 0) {
    return;
  }
  target.r += color.r * scale;
  target.g += color.g * scale;
  target.b += color.b * scale;
}

function cloneMaterialColor(material) {
  if (!material) {
    return new THREE.Color(1, 1, 1);
  }
  if (Array.isArray(material)) {
    const entry = material[0];
    return cloneMaterialColor(entry);
  }
  if (material.color && material.color.isColor) {
    return material.color.clone();
  }
  return new THREE.Color(1, 1, 1);
}

function resolveIntersectionMaterial(intersection) {
  if (!intersection) {
    return null;
  }
  const object = intersection.object;
  if (!object) {
    return null;
  }
  const material = object.material;
  if (!material) {
    return null;
  }
  if (!Array.isArray(material)) {
    return material;
  }
  if (!intersection.face || !Number.isInteger(intersection.face.materialIndex)) {
    return material[0] ?? null;
  }
  return material[intersection.face.materialIndex] ?? material[0] ?? null;
}

function getWorldFaceNormal(intersection) {
  if (!intersection) {
    return new THREE.Vector3(0, 0, 1);
  }
  if (intersection.normal) {
    return intersection.normal.clone();
  }
  if (!intersection.face) {
    return new THREE.Vector3(0, 0, 1);
  }
  const normal = intersection.face.normal.clone();
  const object = intersection.object;
  if (object && object.isObject3D) {
    const normalMatrix = new THREE.Matrix3();
    normalMatrix.getNormalMatrix(object.matrixWorld);
    normal.applyMatrix3(normalMatrix).normalize();
  }
  return normal;
}

export class DDGIProbeVolume {
  constructor(scene, sunSky, options = {}) {
    this.scene = scene;
    this.sunSky = sunSky;
    this.options = { ...DEFAULT_OPTIONS, ...options };
    this.enabled = true;

    this.bounds = new THREE.Box3();
    this.volumeMin = new THREE.Vector3(-1000, -1000, -1000);
    this.volumeMax = new THREE.Vector3(1000, 1000, 1000);
    this.gridSize = new THREE.Vector3(1, 1, 1);
    this.gridSteps = new THREE.Vector3(1000, 1000, 1000);
    this.probes = [];
    this.probeDirections = generateDirections(this.options.sampleCount);
    this.updateCursor = 0;

    this.staticMeshes = [];
    this.directionalLights = [];
    this.pointLights = [];
    this.hemisphereLight = null;
    this.ambientSkyColor = new THREE.Color(0.02, 0.02, 0.02);
    this.ambientGroundColor = new THREE.Color(0.01, 0.01, 0.01);
    this.ambientMin = new THREE.Color(0.005, 0.005, 0.005);

    this.lightProbe = new THREE.LightProbe();
    this.lightProbe.name = 'DDGIProbeVolumeLightProbe';
    this.scene.add(this.lightProbe);

    this.raycaster = new THREE.Raycaster();
    this.raycaster.firstHitOnly = true;
    this.shadowRaycaster = new THREE.Raycaster();
    this.shadowRaycaster.firstHitOnly = true;
    this.shadowRaycaster.params.Mesh = { ...this.shadowRaycaster.params.Mesh, threshold: 0.0001 };

    this.lastLightRefresh = 0;
    this.lightRefreshInterval = 0.5;

    this.rebuildVolume();
  }

  setSunSky(sunSky) {
    this.sunSky = sunSky;
  }

  dispose() {
    if (this.lightProbe) {
      this.scene.remove(this.lightProbe);
    }
  }

  setSceneRoot(root) {
    this.staticMeshes = [];
    if (root && root.isObject3D) {
      root.traverse((child) => {
        if (child?.isMesh && child.geometry) {
          this.staticMeshes.push(child);
        }
      });
      this.bounds.setFromObject(root);
    } else {
      this.bounds.makeEmpty();
    }
    if (this.bounds.isEmpty()) {
      this.bounds.set(new THREE.Vector3(-1500, -1500, -1500), new THREE.Vector3(1500, 1500, 1500));
    } else {
      this.bounds.min.addScalar(-this.options.boundsPadding);
      this.bounds.max.addScalar(this.options.boundsPadding);
    }
    this.rebuildVolume();
  }

  rebuildVolume() {
    const min = this.bounds.min.clone();
    const max = this.bounds.max.clone();
    const size = new THREE.Vector3().subVectors(max, min);
    const spacing = Math.max(this.options.probeSpacing, 1);

    const dimX = Math.max(2, Math.ceil(size.x / spacing) + 1);
    const dimY = Math.max(2, Math.ceil(size.y / spacing) + 1);
    const dimZ = Math.max(2, Math.ceil(size.z / spacing) + 1);

    this.gridSize.set(dimX, dimY, dimZ);
    this.gridSteps.set(
      dimX > 1 ? size.x / (dimX - 1) : spacing,
      dimY > 1 ? size.y / (dimY - 1) : spacing,
      dimZ > 1 ? size.z / (dimZ - 1) : spacing,
    );
    this.volumeMin.copy(min);
    this.volumeMax.copy(max);

    const total = dimX * dimY * dimZ;
    this.probes = new Array(total).fill(null).map((_, index) => {
      const ix = index % dimX;
      const iy = Math.floor(index / dimX) % dimY;
      const iz = Math.floor(index / (dimX * dimY));
      const position = new THREE.Vector3(
        this.volumeMin.x + ix * this.gridSteps.x,
        this.volumeMin.y + iy * this.gridSteps.y,
        this.volumeMin.z + iz * this.gridSteps.z,
      );
      return {
        index,
        ix,
        iy,
        iz,
        position,
        coefficients: Array.from({ length: SH_BASIS_COUNT }, () => new THREE.Vector3()),
        validity: 0,
      };
    });

    this.updateCursor = 0;
  }

  refreshLights(scene, time) {
    if (time !== undefined && time - this.lastLightRefresh < this.lightRefreshInterval) {
      return;
    }
    this.directionalLights = [];
    this.pointLights = [];
    this.hemisphereLight = null;

    const visit = (object) => {
      if (!object || object === this.lightProbe) {
        return;
      }
      if (object.isDirectionalLight) {
        this.directionalLights.push(object);
      } else if (object.isPointLight) {
        this.pointLights.push(object);
      } else if (object.isHemisphereLight) {
        this.hemisphereLight = object;
      }
    };

    if (scene?.isScene) {
      scene.traverse(visit);
    }
    if (this.sunSky) {
      visit(this.sunSky.sunLight);
      visit(this.sunSky.fillLight);
    }

    if (this.hemisphereLight) {
      this.ambientSkyColor.copy(this.hemisphereLight.color).multiplyScalar(this.hemisphereLight.intensity * 0.5);
      this.ambientGroundColor.copy(this.hemisphereLight.groundColor).multiplyScalar(this.hemisphereLight.intensity * 0.5);
    }
    this.lastLightRefresh = time ?? 0;
  }

  addAmbientRadiance(direction, target) {
    if (direction.z >= 0) {
      addScaledColor(target, this.ambientSkyColor, 1);
    } else {
      addScaledColor(target, this.ambientGroundColor, 1);
    }
  }

  isOccluded(origin, direction, maxDistance, skipObject = null) {
    this.shadowRaycaster.set(origin, direction);
    this.shadowRaycaster.near = 0.01;
    this.shadowRaycaster.far = Math.max(maxDistance, 0.01);
    const intersections = this.shadowRaycaster.intersectObjects(this.staticMeshes, true);
    if (!intersections.length) {
      return false;
    }
    const first = intersections[0];
    if (!first || first.distance < 1e-4) {
      return false;
    }
    if (skipObject && first.object === skipObject) {
      return intersections.length > 1;
    }
    return first.distance < maxDistance - 1e-3;
  }

  computeDirectLighting(point, normal, skipObject = null) {
    const lighting = this.ambientMin.clone();

    for (const light of this.directionalLights) {
      if (!light?.isDirectionalLight || light.intensity <= 0) {
        continue;
      }
      const targetPosition = light.target ? light.target.getWorldPosition(TEMP_VECTOR) : new THREE.Vector3();
      const lightPosition = light.getWorldPosition(TEMP_VECTOR_2);
      const toTarget = TEMP_VECTOR_3.subVectors(targetPosition, lightPosition).normalize();
      const ndotl = normal.dot(toTarget);
      if (ndotl <= 0) {
        continue;
      }
      if (this.isOccluded(point, toTarget, this.options.maxDistance, skipObject)) {
        continue;
      }
      addScaledColor(lighting, light.color, light.intensity * ndotl);
    }

    for (const light of this.pointLights) {
      if (!light?.isPointLight || light.intensity <= 0) {
        continue;
      }
      const lightPosition = light.getWorldPosition(TEMP_VECTOR);
      const toLight = TEMP_VECTOR_2.subVectors(lightPosition, point);
      const distanceSq = toLight.lengthSq();
      if (distanceSq <= 1e-4) {
        continue;
      }
      const distance = Math.sqrt(distanceSq);
      toLight.divideScalar(distance);
      const ndotl = normal.dot(toLight);
      if (ndotl <= 0) {
        continue;
      }
      const range = light.distance > 0 ? light.distance : this.options.maxDistance;
      if (distance > range) {
        continue;
      }
      const attenuationBase = Math.max(1 - distance / range, 0);
      const attenuation = attenuationBase ** (light.decay > 0 ? light.decay : 1);
      if (attenuation <= 0) {
        continue;
      }
      if (this.isOccluded(point, toLight, distance - 0.05, skipObject)) {
        continue;
      }
      addScaledColor(lighting, light.color, light.intensity * ndotl * attenuation);
    }

    return lighting;
  }

  traceRadiance(origin, direction) {
    if (!this.staticMeshes.length) {
      const ambient = new THREE.Color();
      this.addAmbientRadiance(direction, ambient);
      return ambient;
    }

    this.raycaster.set(origin, direction);
    this.raycaster.near = 0.01;
    this.raycaster.far = this.options.maxDistance;
    const intersections = this.raycaster.intersectObjects(this.staticMeshes, true);
    if (!intersections.length) {
      const ambient = new THREE.Color();
      this.addAmbientRadiance(direction, ambient);
      return ambient;
    }

    const hit = intersections[0];
    const point = hit.point.clone().addScaledVector(direction, -0.01);
    const normal = getWorldFaceNormal(hit);
    const material = resolveIntersectionMaterial(hit);
    const albedo = cloneMaterialColor(material);
    const lighting = this.computeDirectLighting(point, normal, hit.object);
    const shaded = lighting.multiply(albedo);
    return shaded;
  }

  updateProbes(deltaTime, scene, time) {
    if (!this.enabled || !this.probes.length) {
      return;
    }

    this.refreshLights(scene, time);

    const probesPerFrame = Math.min(this.options.updateBudget, this.probes.length);
    for (let i = 0; i < probesPerFrame; i += 1) {
      const probe = this.probes[this.updateCursor];
      this.updateCursor = (this.updateCursor + 1) % this.probes.length;
      if (!probe) {
        continue;
      }

      const accum = Array.from({ length: SH_BASIS_COUNT }, () => new THREE.Vector3());

      for (const direction of this.probeDirections) {
        const radiance = this.traceRadiance(probe.position, direction);
        for (let c = 0; c < SH_BASIS_COUNT; c += 1) {
          const basis = SH_BASIS_FUNCTIONS[c](direction);
          addScaledColor(accum[c], radiance, basis);
        }
      }

      const scale = FOUR_PI / Math.max(this.probeDirections.length, 1);
      for (let c = 0; c < SH_BASIS_COUNT; c += 1) {
        accum[c].multiplyScalar(scale);
        probe.coefficients[c].lerp(accum[c], 1 - this.options.hysteresis);
      }
      probe.validity = Math.min(probe.validity + deltaTime * this.options.updateBudget, 1);
    }
  }

  sampleSphericalHarmonics(position, target = TEMP_SH) {
    if (!this.probes.length) {
      target.zero();
      return target;
    }

    const relative = new THREE.Vector3(
      (position.x - this.volumeMin.x) / Math.max(this.volumeMax.x - this.volumeMin.x, 1e-6),
      (position.y - this.volumeMin.y) / Math.max(this.volumeMax.y - this.volumeMin.y, 1e-6),
      (position.z - this.volumeMin.z) / Math.max(this.volumeMax.z - this.volumeMin.z, 1e-6),
    );

    const clampRelative = (value) => THREE.MathUtils.clamp(value, 0, 0.9999);
    const ux = clampRelative(relative.x) * (this.gridSize.x - 1);
    const uy = clampRelative(relative.y) * (this.gridSize.y - 1);
    const uz = clampRelative(relative.z) * (this.gridSize.z - 1);

    const ix = Math.floor(ux);
    const iy = Math.floor(uy);
    const iz = Math.floor(uz);

    const tx = ux - ix;
    const ty = uy - iy;
    const tz = uz - iz;

    const ix1 = Math.min(ix + 1, this.gridSize.x - 1);
    const iy1 = Math.min(iy + 1, this.gridSize.y - 1);
    const iz1 = Math.min(iz + 1, this.gridSize.z - 1);

    const weight = (wx, wy, wz) => {
      const vx = wx ? tx : 1 - tx;
      const vy = wy ? ty : 1 - ty;
      const vz = wz ? tz : 1 - tz;
      return vx * vy * vz;
    };

    const indexAt = (x, y, z) => ((z * this.gridSize.y + y) * this.gridSize.x + x);

    const neighbors = [
      { index: indexAt(ix, iy, iz), weight: weight(0, 0, 0) },
      { index: indexAt(ix1, iy, iz), weight: weight(1, 0, 0) },
      { index: indexAt(ix, iy1, iz), weight: weight(0, 1, 0) },
      { index: indexAt(ix1, iy1, iz), weight: weight(1, 1, 0) },
      { index: indexAt(ix, iy, iz1), weight: weight(0, 0, 1) },
      { index: indexAt(ix1, iy, iz1), weight: weight(1, 0, 1) },
      { index: indexAt(ix, iy1, iz1), weight: weight(0, 1, 1) },
      { index: indexAt(ix1, iy1, iz1), weight: weight(1, 1, 1) },
    ];

    for (let c = 0; c < SH_BASIS_COUNT; c += 1) {
      target.coefficients[c].set(0, 0, 0);
    }

    for (const neighbor of neighbors) {
      const probe = this.probes[neighbor.index];
      if (!probe || neighbor.weight <= 0) {
        continue;
      }
      for (let c = 0; c < SH_BASIS_COUNT; c += 1) {
        addScaledColor(target.coefficients[c], probe.coefficients[c], neighbor.weight);
      }
    }

    return target;
  }

  updateLightProbe(position) {
    if (!this.lightProbe) {
      return;
    }
    const sh = this.sampleSphericalHarmonics(position, TEMP_SH);
    const coefficients = this.lightProbe.sh.coefficients;
    for (let i = 0; i < coefficients.length; i += 1) {
      coefficients[i].copy(sh.coefficients[i]);
    }
    this.lightProbe.intensity = 1;
  }

  update(deltaTime, scene, time, cameraPosition) {
    const now = time ?? 0;
    this.updateProbes(deltaTime ?? 0.016, scene, now);
    if (cameraPosition) {
      this.updateLightProbe(cameraPosition);
    }
  }
}

export function createDDGIProbeVolume(scene, sunSky, options = {}) {
  return new DDGIProbeVolume(scene, sunSky, options);
}

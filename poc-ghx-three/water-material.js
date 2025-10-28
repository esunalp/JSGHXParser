import * as THREE from 'three/webgpu';
import {
  Fn,
  Loop,
  abs,
  clamp,
  color,
  cross,
  float,
  length,
  mix,
  mx_noise_float,
  normalize,
  normalLocal,
  normalView,
  positionWorld,
  pow,
  sin,
  time,
  vec2,
  vec3,
  uniform,
} from 'three/tsl';

export const WATER_PREVIEW_COLOR = new THREE.Color(201 / 255, 233 / 255, 245 / 255);

export function isWaterPreviewColor(colour, tolerance = 1 / 255) {
  if (!colour?.isColor) {
    return false;
  }
  return Math.abs(colour.r - WATER_PREVIEW_COLOR.r) <= tolerance
    && Math.abs(colour.g - WATER_PREVIEW_COLOR.g) <= tolerance
    && Math.abs(colour.b - WATER_PREVIEW_COLOR.b) <= tolerance;
}

export function createWaterSurfaceMaterial(options = {}) {
  const {
    side = THREE.DoubleSide,
    unitsPerMeter = 10,
    largeWavesFrequency: largeWavesFrequencyOption = new THREE.Vector2(0.1, 0.4),
    largeWavesSpeed: largeWavesSpeedOption = 1.25,
    largeWavesMultiplier: largeWavesMultiplierOption = 0.1,
    smallWavesIterations: smallWavesIterationsOption = 3,
    smallWavesFrequency: smallWavesFrequencyOption = 0.5,
    smallWavesSpeed: smallWavesSpeedOption = 0.3,
    smallWavesMultiplier: smallWavesMultiplierOption = 0.02,
    normalComputeShift: normalComputeShiftOption = 0.01,
  } = options;

  const toVector2 = (value, fallback) => {
    if (value?.isVector2) {
      return value.clone();
    }
    if (Array.isArray(value)) {
      const [x = fallback.x, y = fallback.y] = value;
      return new THREE.Vector2(x, y);
    }
    if (typeof value === 'number') {
      return new THREE.Vector2(value, value);
    }
    if (value && typeof value === 'object' && 'x' in value && 'y' in value) {
      return new THREE.Vector2(value.x, value.y);
    }
    return fallback.clone();
  };

  const toNumber = (value, fallback) => (Number.isFinite(value) ? value : fallback);

  const largeWavesFrequencyValue = toVector2(largeWavesFrequencyOption, new THREE.Vector2(1.6, 1));
  const largeWavesSpeedValue = toNumber(largeWavesSpeedOption, 1.25);
  const largeWavesMultiplierValue = toNumber(largeWavesMultiplierOption, 0.05);
  const smallWavesIterationsValue = Math.max(1, Math.floor(toNumber(smallWavesIterationsOption, 1)));
  const smallWavesFrequencyValue = toNumber(smallWavesFrequencyOption, 1.5);
  const smallWavesSpeedValue = toNumber(smallWavesSpeedOption, 0.3);
  const smallWavesMultiplierValue = toNumber(smallWavesMultiplierOption, 0.02);
  const normalComputeShiftValue = Math.max(0, toNumber(normalComputeShiftOption, 0.01));

  const largeWavesFrequency = uniform(largeWavesFrequencyValue);
  const largeWavesSpeed = uniform(largeWavesSpeedValue);
  const largeWavesMultiplier = uniform(largeWavesMultiplierValue);
  const smallWavesIterations = uniform(smallWavesIterationsValue);
  const smallWavesFrequency = uniform(smallWavesFrequencyValue);
  const smallWavesSpeed = uniform(smallWavesSpeedValue);
  const smallWavesMultiplier = uniform(smallWavesMultiplierValue);
  const normalComputeShiftUniform = uniform(normalComputeShiftValue);

  const material = new THREE.MeshPhysicalNodeMaterial({
    metalness: 1,
    roughness: 0.15,
    clearcoat: 1,
    clearcoatRoughness: 0.15,
    transmission: 0.8,
    thickness: 250,
    ior: 1.33,
    attenuationDistance: 1200,
    attenuationColor: new THREE.Color(0x000000),
    transparent: false,
    side,
  });

  material.shadowSide = side;

  const worldPosition = positionWorld;
  const surfaceCoordinates = vec2(worldPosition.x, worldPosition.y);
  const unitsPerMeterNode = float(unitsPerMeter);

  const normalComputeShift = normalComputeShiftUniform.mul(unitsPerMeterNode);
  const offsetX = vec2(normalComputeShift, float(0));
  const offsetY = vec2(float(0), normalComputeShift);

  const wavesElevation = Fn(([coords]) => {
    const coordsMeters = coords.div(unitsPerMeterNode).toVar();
    const largeWaveTime = time.mul(largeWavesSpeed);
    const largeWave = sin(coordsMeters.x.mul(largeWavesFrequency.x).add(largeWaveTime))
      .mul(sin(coordsMeters.y.mul(largeWavesFrequency.y).add(largeWaveTime)))
      .mul(largeWavesMultiplier)
      .toVar();

    Loop({ start: float(1), end: smallWavesIterations.add(float(1)) }, ({ i }) => {
      const noiseInput = vec3(
        coordsMeters.add(vec2(float(2), float(2))).mul(smallWavesFrequency).mul(i),
        time.mul(smallWavesSpeed),
      );
      const smallWave = mx_noise_float(noiseInput, float(1), float(0))
        .mul(smallWavesMultiplier)
        .div(i)
        .abs();
      largeWave.subAssign(smallWave);
    });

    return largeWave.mul(unitsPerMeterNode);
  });

  const heightCenter = wavesElevation(surfaceCoordinates);
  const heightPositiveX = wavesElevation(surfaceCoordinates.add(offsetX));
  const heightNegativeX = wavesElevation(surfaceCoordinates.sub(offsetX));
  const heightPositiveY = wavesElevation(surfaceCoordinates.add(offsetY));
  const heightNegativeY = wavesElevation(surfaceCoordinates.sub(offsetY));

  const doubleStep = normalComputeShift.mul(float(2));
  const gradientX = heightPositiveX.sub(heightNegativeX).div(doubleStep);
  const gradientY = heightPositiveY.sub(heightNegativeY).div(doubleStep);

  const tangentSpaceNormal = normalize(vec3(
    gradientX.mul(float(-1)),
    gradientY.mul(float(-1)),
    float(1),
  ));

  const baseNormal = normalize(normalLocal);
  const tangentCandidateA = cross(vec3(0, 0, 1), baseNormal);
  const tangentCandidateB = cross(vec3(0, 1, 0), baseNormal);
  const tangent = normalize(tangentCandidateA.add(tangentCandidateB));
  const bitangent = normalize(cross(baseNormal, tangent));

  const normalStrength = float(1.2);
  const perturbedNormal = normalize(
    tangent.mul(tangentSpaceNormal.x.mul(normalStrength))
      .add(bitangent.mul(tangentSpaceNormal.y.mul(normalStrength)))
      .add(baseNormal.mul(tangentSpaceNormal.z)),
  );

  material.normalNode = perturbedNormal;

  const slopeIntensity = clamp(length(tangentSpaceNormal.xy), 0, 1);
  const foamStrength = clamp(slopeIntensity.mul(float(1.2)), 0, 1);

  const fresnelBase = clamp(float(1).sub(abs(normalView.z)), 0, 1);
  const fresnel = pow(fresnelBase, float(3));
  const colourBlend = clamp(
    slopeIntensity.mul(float(0.45)).add(fresnel.mul(float(0.45))).add(float(0.1)),
    0,
    1,
  );
  const deepWaterColour = color(0x134f5c);
  const shallowWaterColour = color(0x76a5af);
  const foamColour = color(0x76a5af);

  const baseColour = mix(deepWaterColour, shallowWaterColour, colourBlend);
  material.colorNode = mix(baseColour, foamColour, foamStrength.mul(float(0.55)));

  material.metalnessNode = float(1);
  material.roughnessNode = float(0.15);
  material.clearcoatNode = float(1);
  material.clearcoatRoughnessNode = clamp(float(0.03).add(foamStrength.mul(float(0.08))), 0.02, 0.12);
  material.transmissionNode = float(0.8);
  material.thicknessNode = float(280);
  material.attenuationDistanceNode = float(1200);
  material.attenuationColorNode = mix(
    color(0x134f5c),
    color(0x76a5af),
    clamp(colourBlend.add(foamStrength.mul(float(0.15))), 0, 1),
  );
  material.iorNode = float(1.33);
  material.opacityNode = float(1);

  material.userData.isProceduralWater = true;
  material.userData.waveUniforms = {
    largeWavesFrequency,
    largeWavesSpeed,
    largeWavesMultiplier,
    smallWavesIterations,
    smallWavesFrequency,
    smallWavesSpeed,
    smallWavesMultiplier,
    normalComputeShift: normalComputeShiftUniform,
  };
  material.userData.previewColor = WATER_PREVIEW_COLOR.clone();
  material.needsUpdate = true;

  return material;
}

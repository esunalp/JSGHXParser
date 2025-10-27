import * as THREE from 'three/webgpu';
import {
  abs,
  cameraPosition,
  clamp,
  color,
  cos,
  float,
  mix,
  normalize,
  normalLocal,
  normalView,
  pmremTexture,
  positionLocal,
  positionWorld,
  pow,
  reflect,
  sin,
  vec3,
  mx_timer,
  materialEnvIntensity,
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
    amplitude = 18,
    frequency = 0.0048,
  } = options;

  const material = new THREE.MeshPhysicalNodeMaterial({
    metalness: 0.02,
    roughness: 0.08,
    clearcoat: 0.85,
    clearcoatRoughness: 0.08,
    transmission: 0.78,
    thickness: 250,
    ior: 1.33,
    attenuationDistance: 1200,
    attenuationColor: new THREE.Color(0x8fdcff),
    transparent: true,
    side,
  });

  material.shadowSide = side;

  const time = mx_timer();
  const worldPosition = positionWorld;

  const baseFrequency = float(frequency);
  const frequencyX = baseFrequency.mul(2.1);
  const frequencyY = baseFrequency.mul(1.6);
  const frequencyDiagonal = baseFrequency.mul(1.25);
  const frequencyCrossX = baseFrequency.mul(1.4);
  const frequencyCrossY = baseFrequency.mul(0.9);
  const frequencyRippleX = baseFrequency.mul(3.2);
  const frequencyRippleY = baseFrequency.mul(2.6);
  const frequencyZ = baseFrequency.mul(0.7);

  const waveArgX = worldPosition.x.mul(frequencyX).add(time.mul(0.62));
  const waveArgY = worldPosition.y.mul(frequencyY).add(time.mul(0.47));
  const waveArgDiagonal = worldPosition.x.add(worldPosition.y).mul(frequencyDiagonal).add(time.mul(0.55));
  const waveArgCross = worldPosition.x.mul(frequencyCrossX).sub(worldPosition.y.mul(frequencyCrossY)).add(time.mul(0.38));
  const waveArgRipple = worldPosition.x.mul(frequencyRippleX).add(worldPosition.y.mul(frequencyRippleY)).add(time.mul(1.12));
  const waveArgZ = worldPosition.z.mul(frequencyZ).add(time.mul(0.29));

  const waveX = sin(waveArgX);
  const waveY = sin(waveArgY);
  const waveDiagonal = sin(waveArgDiagonal);
  const waveCross = sin(waveArgCross);
  const waveRipple = sin(waveArgRipple);
  const waveZ = sin(waveArgZ);

  const combinedWave = waveX.mul(0.28)
    .add(waveY.mul(0.23))
    .add(waveDiagonal.mul(0.19))
    .add(waveCross.mul(0.17))
    .add(waveRipple.mul(0.11))
    .add(waveZ.mul(0.08));

  const amplitudeNode = float(amplitude);
  const displacement = combinedWave.mul(amplitudeNode);
  material.positionNode = positionLocal.add(normalLocal.mul(displacement));

  const derivativeX = cos(waveArgX).mul(frequencyX).mul(0.28)
    .add(cos(waveArgDiagonal).mul(frequencyDiagonal).mul(0.19))
    .add(cos(waveArgCross).mul(frequencyCrossX).mul(0.17))
    .add(cos(waveArgRipple).mul(frequencyRippleX).mul(0.11));
  const derivativeY = cos(waveArgY).mul(frequencyY).mul(0.23)
    .add(cos(waveArgDiagonal).mul(frequencyDiagonal).mul(0.19))
    .sub(cos(waveArgCross).mul(frequencyCrossY).mul(0.17))
    .add(cos(waveArgRipple).mul(frequencyRippleY).mul(0.11));
  const derivativeZ = cos(waveArgZ).mul(frequencyZ).mul(0.08);

  const gradient = vec3(derivativeX, derivativeY, derivativeZ).mul(amplitudeNode);
  const perturbedNormal = normalize(normalLocal.sub(gradient));
  material.normalNode = perturbedNormal;

  const viewDirection = normalize(cameraPosition.sub(worldPosition));
  const incidentDirection = viewDirection.mul(float(-1));

  const waveNormalized = combinedWave.mul(0.5).add(0.5);
  const foamScale = amplitudeNode.mul(baseFrequency).mul(64);
  const foamStrength = clamp(
    abs(derivativeX).add(abs(derivativeY)).mul(foamScale)
      .add(abs(waveRipple).mul(0.18))
      .add(abs(waveDiagonal).mul(0.15))
      .sub(0.1),
    0,
    1,
  );

  const fresnelBase = clamp(float(1).sub(abs(normalView.z)), 0, 1);
  const fresnel = pow(fresnelBase, float(3));
  const colourBlend = clamp(waveNormalized.mul(0.3).add(fresnel.mul(0.6)), 0, 1);
  const reflectionMix = clamp(fresnel.mul(float(0.85)).add(float(0.05)), 0, 1);

  const deepWaterColour = color(0x0f3a63);
  const shallowWaterColour = color(0x8dddf9);
  const foamColour = color(0xf6fdff);

  const baseColour = mix(deepWaterColour, shallowWaterColour, colourBlend);
  const roughnessBase = clamp(
    float(0.04)
      .add(waveNormalized.mul(0.06))
      .add(foamStrength.mul(0.12)),
    0.03,
    0.28,
  );
  const reflectionVector = normalize(reflect(incidentDirection, perturbedNormal));
  const environmentReflection = pmremTexture(reflectionVector, roughnessBase).mul(materialEnvIntensity);
  const colourWithReflection = mix(baseColour, environmentReflection, reflectionMix);
  material.colorNode = mix(colourWithReflection, foamColour, foamStrength.mul(0.6));

  material.metalnessNode = float(0.02);
  material.roughnessNode = roughnessBase;
  material.clearcoatNode = float(0.85);
  material.clearcoatRoughnessNode = clamp(float(0.02).add(foamStrength.mul(0.09)), 0.02, 0.12);
  material.transmissionNode = float(0.82);
  material.thicknessNode = float(280);
  material.attenuationDistanceNode = float(1200);
  material.attenuationColorNode = mix(
    color(0x3fb7ff),
    color(0xbef1ff),
    clamp(colourBlend.add(foamStrength.mul(0.2)), 0, 1),
  );
  material.iorNode = float(1.33);
  material.opacityNode = float(1);

  material.userData.isProceduralWater = true;
  material.userData.previewColor = WATER_PREVIEW_COLOR.clone();
  material.needsUpdate = true;

  return material;
}

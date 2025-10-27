import * as THREE from 'three/webgpu';
import {
  abs,
  clamp,
  color,
  cos,
  float,
  mix,
  normalize,
  normalLocal,
  normalView,
  positionLocal,
  positionWorld,
  pow,
  sin,
  vec3,
  mx_timer,
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
    amplitude = 25,
    frequency = 0.0032,
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
  const frequencyX = baseFrequency.mul(1.8);
  const frequencyY = baseFrequency.mul(1.35);
  const frequencyDiagonal = baseFrequency.mul(1.12);
  const frequencyZ = baseFrequency.mul(0.8);

  const waveArgX = worldPosition.x.mul(frequencyX).add(time.mul(0.62));
  const waveArgY = worldPosition.y.mul(frequencyY).add(time.mul(0.45));
  const waveArgDiagonal = worldPosition.x.add(worldPosition.y).mul(frequencyDiagonal).add(time.mul(0.54));
  const waveArgZ = worldPosition.z.mul(frequencyZ).add(time.mul(0.32));

  const waveX = sin(waveArgX);
  const waveY = sin(waveArgY);
  const waveDiagonal = sin(waveArgDiagonal);
  const waveZ = sin(waveArgZ);

  const combinedWave = waveX.mul(0.55)
    .add(waveY.mul(0.35))
    .add(waveDiagonal.mul(0.25))
    .add(waveZ.mul(0.15));

  const amplitudeNode = float(amplitude);
  const displacement = combinedWave.mul(amplitudeNode);
  material.positionNode = positionLocal.add(normalLocal.mul(displacement));

  const derivativeX = cos(waveArgX).mul(frequencyX).mul(0.55)
    .add(cos(waveArgDiagonal).mul(frequencyDiagonal).mul(0.25));
  const derivativeY = cos(waveArgY).mul(frequencyY).mul(0.35)
    .add(cos(waveArgDiagonal).mul(frequencyDiagonal).mul(0.25));
  const derivativeZ = cos(waveArgZ).mul(frequencyZ).mul(0.15);

  const gradient = vec3(derivativeX, derivativeY, derivativeZ).mul(amplitudeNode);
  const perturbedNormal = normalize(normalLocal.sub(gradient));
  material.normalNode = perturbedNormal;

  const waveNormalized = combinedWave.mul(0.5).add(0.5);
  const foamScale = amplitudeNode.mul(baseFrequency).mul(55);
  const foamStrength = clamp(
    abs(derivativeX).add(abs(derivativeY)).mul(foamScale)
      .add(abs(waveDiagonal).mul(0.2))
      .sub(0.08),
    0,
    1,
  );

  const fresnelBase = clamp(float(1).sub(abs(normalView.z)), 0, 1);
  const fresnel = pow(fresnelBase, float(3));
  const colourBlend = clamp(waveNormalized.mul(0.3).add(fresnel.mul(0.6)), 0, 1);

  const deepWaterColour = color(0x0f3a63);
  const shallowWaterColour = color(0x8dddf9);
  const foamColour = color(0xf6fdff);

  const baseColour = mix(deepWaterColour, shallowWaterColour, colourBlend);
  material.colorNode = mix(baseColour, foamColour, foamStrength.mul(0.6));

  material.metalnessNode = float(0.02);
  material.roughnessNode = clamp(float(0.04).add(waveNormalized.mul(0.06)).add(foamStrength.mul(0.12)), 0.03, 0.28);
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

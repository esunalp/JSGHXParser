import * as THREE from 'three/webgpu';
import {
  abs,
  cameraPosition,
  clamp,
  color,
  cos,
  float,
  length,
  max,
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
  vec2,
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
  const frequencyX = baseFrequency.mul(1.2);
  const frequencyY = baseFrequency.mul(0.9);
  const frequencyRipple = baseFrequency.mul(1.6);

  const surfaceCoords = vec2(worldPosition.x, worldPosition.y);
  const rippleDistance = length(surfaceCoords);
  const safeDistance = max(rippleDistance, float(1e-3));

  const waveArgX = worldPosition.x.mul(frequencyX).add(time.mul(0.62));
  const waveArgY = worldPosition.y.mul(frequencyY).add(time.mul(0.47));
  const waveArgRipple = rippleDistance.mul(frequencyRipple).sub(time.mul(0.85));

  const waveX = sin(waveArgX);
  const waveY = sin(waveArgY);
  const waveRipple = sin(waveArgRipple);

  const combinedWave = waveX.mul(0.5)
    .add(waveY.mul(0.35))
    .add(waveRipple.mul(0.25));

  const amplitudeNode = float(amplitude);
  const displacement = combinedWave.mul(amplitudeNode);
  material.positionNode = positionLocal.add(normalLocal.mul(displacement));

  const rippleDerivative = cos(waveArgRipple).mul(frequencyRipple);
  const rippleDirectionX = worldPosition.x.div(safeDistance);
  const rippleDirectionY = worldPosition.y.div(safeDistance);

  const derivativeX = cos(waveArgX).mul(frequencyX).mul(0.5)
    .add(rippleDerivative.mul(rippleDirectionX).mul(0.25));
  const derivativeY = cos(waveArgY).mul(frequencyY).mul(0.35)
    .add(rippleDerivative.mul(rippleDirectionY).mul(0.25));
  const derivativeZ = float(0);

  const gradient = vec3(derivativeX, derivativeY, derivativeZ).mul(amplitudeNode);
  const perturbedNormal = normalize(normalLocal.sub(gradient));
  material.normalNode = perturbedNormal;

  const viewDirection = normalize(cameraPosition.sub(worldPosition));
  const incidentDirection = viewDirection.mul(float(-1));

  const waveNormalized = combinedWave.mul(0.5).add(0.5);
  const foamScale = amplitudeNode.mul(baseFrequency).mul(24);
  const foamStrength = clamp(
    abs(derivativeX).add(abs(derivativeY)).mul(foamScale)
      .add(abs(waveRipple).mul(0.18))
      .sub(0.08),
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
  const hasEnvironmentMap = Boolean(material.envMap);
  const environmentReflection = hasEnvironmentMap
    ? pmremTexture(reflectionVector, roughnessBase).mul(materialEnvIntensity)
    : baseColour;
  const environmentReflectionMix = hasEnvironmentMap ? reflectionMix : float(0);
  const colourWithReflection = mix(baseColour, environmentReflection, environmentReflectionMix);
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

import * as THREE from 'three/webgpu';
import {
  abs,
  cameraPosition,
  clamp,
  color,
  cross,
  float,
  fract,
  length,
  mix,
  normalize,
  normalLocal,
  normalView,
  pmremTexture,
  positionWorld,
  pow,
  reflect,
  reflector,
  texture,
  vec2,
  vec3,
  materialEnvIntensity,
} from 'three/tsl';

const WATER_NORMAL_TEXTURE = new THREE.TextureLoader().load(
  new URL('./assets/waternormal1.jpg', import.meta.url).href,
);
WATER_NORMAL_TEXTURE.wrapS = THREE.RepeatWrapping;
WATER_NORMAL_TEXTURE.wrapT = THREE.RepeatWrapping;
WATER_NORMAL_TEXTURE.colorSpace = THREE.NoColorSpace;

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
    reflectionResolution = 0.35,
  } = options;

  const material = new THREE.MeshPhysicalNodeMaterial({
    metalness: 0.85,
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

  const worldPosition = positionWorld;
  const metersToUV = float(1 / 4);
  const planarUV = vec2(worldPosition.x, worldPosition.y).mul(metersToUV);
  const wrappedUV = fract(planarUV);

  const blurStep = float(0.02);
  const offsetX = vec2(blurStep, float(0));
  const offsetY = vec2(float(0), blurStep);

  const sampleCenter = texture(WATER_NORMAL_TEXTURE, wrappedUV);
  const samplePositiveX = texture(WATER_NORMAL_TEXTURE, fract(wrappedUV.add(offsetX)));
  const sampleNegativeX = texture(WATER_NORMAL_TEXTURE, fract(wrappedUV.sub(offsetX)));
  const samplePositiveY = texture(WATER_NORMAL_TEXTURE, fract(wrappedUV.add(offsetY)));
  const sampleNegativeY = texture(WATER_NORMAL_TEXTURE, fract(wrappedUV.sub(offsetY)));

  const blurredNormalSample = sampleCenter
    .add(samplePositiveX)
    .add(sampleNegativeX)
    .add(samplePositiveY)
    .add(sampleNegativeY)
    .mul(float(0.2));

  const tangentSpaceNormal = blurredNormalSample.xyz.mul(float(2)).sub(float(1));

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

  const planarReflection = reflector({ resolutionScale: reflectionResolution });
  planarReflection.target.name = 'ProceduralWaterReflectionTarget';
  planarReflection.target.matrixAutoUpdate = true;
  planarReflection.target.frustumCulled = false;
  planarReflection.target.userData.isProceduralWaterReflectionTarget = true;

  const reflectionDistortion = vec2(
    tangentSpaceNormal.x.mul(float(0.045)),
    tangentSpaceNormal.y.mul(float(0.045)),
  );
  planarReflection.uvNode = planarReflection.uvNode.add(reflectionDistortion);

  const viewDirection = normalize(cameraPosition.sub(worldPosition));
  const incidentDirection = viewDirection.mul(float(-1));

  const slopeIntensity = clamp(length(tangentSpaceNormal.xy), 0, 1);
  const foamStrength = clamp(slopeIntensity.mul(float(1.2)), 0, 1);

  const fresnelBase = clamp(float(1).sub(abs(normalView.z)), 0, 1);
  const fresnel = pow(fresnelBase, float(3));
  const colourBlend = clamp(
    slopeIntensity.mul(float(0.45)).add(fresnel.mul(float(0.45))).add(float(0.1)),
    0,
    1,
  );
  const reflectionMix = clamp(fresnel.mul(float(0.85)).add(float(0.05)), 0, 1);

  const deepWaterColour = color(0x0f3a63);
  const shallowWaterColour = color(0x8dddf9);
  const foamColour = color(0xf6fdff);

  const baseColour = mix(deepWaterColour, shallowWaterColour, colourBlend);
  const roughnessBase = clamp(
    float(0.05)
      .add(slopeIntensity.mul(float(0.12)))
      .add(foamStrength.mul(float(0.1))),
    0.04,
    0.28,
  );
  const reflectionVector = normalize(reflect(incidentDirection, perturbedNormal));
  const hasEnvironmentMap = Boolean(material.envMap);
  let combinedReflection = planarReflection;
  if (hasEnvironmentMap) {
    const environmentReflection = pmremTexture(reflectionVector, roughnessBase)
      .mul(materialEnvIntensity);
    combinedReflection = mix(planarReflection, environmentReflection, float(0.35));
  }
  const colourWithReflection = mix(baseColour, combinedReflection, reflectionMix);
  material.colorNode = mix(colourWithReflection, foamColour, foamStrength.mul(float(0.55)));

  material.metalnessNode = float(0.85);
  material.roughnessNode = roughnessBase;
  material.clearcoatNode = float(0.85);
  material.clearcoatRoughnessNode = clamp(float(0.03).add(foamStrength.mul(float(0.08))), 0.02, 0.12);
  material.transmissionNode = float(0.82);
  material.thicknessNode = float(280);
  material.attenuationDistanceNode = float(1200);
  material.attenuationColorNode = mix(
    color(0x3fb7ff),
    color(0xbef1ff),
    clamp(colourBlend.add(foamStrength.mul(float(0.15))), 0, 1),
  );
  material.iorNode = float(1.33);
  material.opacityNode = float(1);

  material.userData.isProceduralWater = true;
  material.userData.planarReflection = planarReflection;
  material.userData.setupProceduralWater = (mesh) => {
    if (!mesh?.isMesh) {
      return;
    }

    if (mesh.userData?.proceduralWaterReflection) {
      return;
    }

    const target = planarReflection?.target;
    if (!target) {
      return;
    }

    if (target.parent && target.parent !== mesh) {
      target.parent.remove(target);
    }

    target.visible = true;
    target.position.set(0, 0, 0);
    target.rotation.set(0, 0, 0);
    target.scale.setScalar(1);

    mesh.add(target);

    const geometry = mesh.geometry;
    let scale = 1;
    if (geometry) {
      if (geometry.boundingSphere) {
        scale = geometry.boundingSphere.radius * 2.2 || scale;
      } else if (typeof geometry.computeBoundingSphere === 'function') {
        geometry.computeBoundingSphere();
        scale = geometry.boundingSphere?.radius * 2.2 || scale;
      }
    }
    if (!Number.isFinite(scale) || scale <= 0) {
      scale = 1;
    }
    target.scale.set(scale, scale, scale);
    target.updateMatrixWorld(true);

    const previousDispose = mesh.userData?.dispose;
    mesh.userData.dispose = () => {
      if (target.parent === mesh) {
        mesh.remove(target);
      }
      mesh.userData.proceduralWaterReflection = false;
      if (typeof previousDispose === 'function') {
        previousDispose.call(mesh);
      }
    };

    mesh.userData.proceduralWaterReflection = true;
  };
  material.userData.previewColor = WATER_PREVIEW_COLOR.clone();
  material.needsUpdate = true;

  return material;
}

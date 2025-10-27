import * as THREE from 'three/webgpu';

const SURFACE_PROPERTY_KEYS = [
  'color',
  'map',
  'lightMap',
  'lightMapIntensity',
  'aoMap',
  'aoMapIntensity',
  'emissive',
  'emissiveIntensity',
  'emissiveMap',
  'bumpMap',
  'bumpScale',
  'normalMap',
  'normalMapType',
  'normalScale',
  'displacementMap',
  'displacementScale',
  'displacementBias',
  'roughness',
  'roughnessMap',
  'metalness',
  'metalnessMap',
  'metalnessRoughnessMap',
  'alphaMap',
  'alphaTest',
  'envMap',
  'envMapIntensity',
  'clearcoat',
  'clearcoatMap',
  'clearcoatRoughness',
  'clearcoatRoughnessMap',
  'clearcoatNormalMap',
  'clearcoatNormalScale',
  'sheen',
  'sheenColor',
  'sheenColorMap',
  'sheenRoughness',
  'sheenRoughnessMap',
  'ior',
  'transmission',
  'transmissionMap',
  'thickness',
  'thicknessMap',
  'attenuationDistance',
  'attenuationColor',
  'specularIntensity',
  'specularIntensityMap',
  'specularColor',
  'specularColorMap',
  'iridescence',
  'iridescenceMap',
  'iridescenceIOR',
  'iridescenceThicknessRange',
  'iridescenceThicknessMap',
  'anisotropy',
  'anisotropyMap',
  'anisotropyRotation',
  'sheenIntensity',
  'sheenIntensityMap',
  'depthWrite',
  'depthTest',
  'transparent',
  'opacity',
  'vertexColors',
  'flatShading',
  'wireframe',
  'wireframeLinewidth',
  'toneMapped',
  'blending',
  'blendSrc',
  'blendDst',
  'blendEquation',
  'blendSrcAlpha',
  'blendDstAlpha',
  'blendEquationAlpha',
  'premultipliedAlpha',
  'polygonOffset',
  'polygonOffsetFactor',
  'polygonOffsetUnits',
  'dithering',
  'fog',
  'visible',
  'side',
  'shadowSide',
  'name',
  'userData',
];

const CLONEABLE_VALUE_CHECKS = ['isColor', 'isVector2', 'isVector3', 'isVector4', 'isQuaternion', 'isMatrix3', 'isMatrix4'];

const DEFAULT_PHONG_DIFFUSE = new THREE.Color(0x9aa5b1);
const DEFAULT_PHONG_SPECULAR = new THREE.Color(0x111111);
const DEFAULT_PHONG_EMISSIVE = new THREE.Color(0x000000);
const MIN_ROUGHNESS = 0.02;

function cloneColorLike(value, fallback = DEFAULT_PHONG_DIFFUSE) {
  if (!value && value !== 0) {
    return fallback.clone();
  }
  if (value.isColor) {
    return value.clone();
  }
  if (typeof value === 'number') {
    return new THREE.Color(value);
  }
  if (Array.isArray(value) && value.length >= 3) {
    return new THREE.Color(value[0], value[1], value[2]);
  }
  if (typeof value === 'object') {
    const { r, g, b } = value;
    if ([r, g, b].every((component) => typeof component === 'number')) {
      return new THREE.Color(r, g, b);
    }
  }
  return fallback.clone();
}

function colorLuminance(color) {
  if (!color?.isColor) {
    return 1;
  }
  const r = color.r;
  const g = color.g;
  const b = color.b;
  const maxComponent = Math.max(r, g, b);
  if (!Number.isFinite(maxComponent)) {
    return 1;
  }
  return THREE.MathUtils.clamp(maxComponent, 0, 1);
}

function convertShininessToRoughness(shininess) {
  const numeric = Number.isFinite(shininess) ? shininess : 30;
  const safe = THREE.MathUtils.clamp(numeric, 0, 512);
  const roughness = Math.sqrt(2 / (safe + 2));
  return THREE.MathUtils.clamp(roughness, MIN_ROUGHNESS, 1);
}

function cloneMaterialValue(value) {
  if (value === undefined || value === null) {
    return value;
  }
  if (Array.isArray(value)) {
    return value.map((entry) => cloneMaterialValue(entry));
  }
  if (typeof value === 'object') {
    if (value.isTexture || value.isCubeTexture) {
      return value;
    }
    for (const check of CLONEABLE_VALUE_CHECKS) {
      if (value[check]) {
        return value.clone();
      }
    }
    if (value instanceof Date) {
      return new Date(value.getTime());
    }
    return { ...value };
  }
  return value;
}

function cloneSurfaceProperty(source, target, key) {
  if (!source || !target) {
    return;
  }

  if (!(key in source)) {
    return;
  }

  const value = source[key];
  if (value === undefined) {
    return;
  }

  const clonedValue = cloneMaterialValue(value);

  if (clonedValue?.isColor && target[key]?.isColor) {
    target[key].copy(clonedValue);
    return;
  }

  target[key] = clonedValue;
}

export function cloneSurfaceMaterial(material) {
  if (!material?.isMaterial || typeof material.clone !== 'function') {
    return material ?? null;
  }

  const cloned = material.clone();

  for (const key of SURFACE_PROPERTY_KEYS) {
    cloneSurfaceProperty(material, cloned, key);
  }

  return cloned;
}

export function applySurfaceMaterialDefaults(material, options = {}) {
  if (!material) {
    return material;
  }
  const { side = THREE.DoubleSide } = options;
  if (side !== undefined && 'side' in material && material.side !== side) {
    material.side = side;
  }
  if ('shadowSide' in material) {
    material.shadowSide = material.side;
  }
  material.needsUpdate = true;
  return material;
}

function copySurfaceMaterialParameters(source, target) {
  if (!source || !target) {
    return;
  }
  for (const key of SURFACE_PROPERTY_KEYS) {
    if (!(key in target)) {
      continue;
    }
    if (source[key] === undefined) {
      continue;
    }
    const value = cloneMaterialValue(source[key]);
    if (key === 'userData' && value && typeof value === 'object') {
      target.userData = { ...value };
      continue;
    }
    target[key] = value;
  }
  target.needsUpdate = true;
}

export function createTlsMaterial(parameters = {}, options = {}) {
  const {
    diffuse = DEFAULT_PHONG_DIFFUSE,
    specular = DEFAULT_PHONG_SPECULAR,
    emissive = DEFAULT_PHONG_EMISSIVE,
    transparency = 0,
    shininess = 30,
  } = parameters;

  const color = cloneColorLike(diffuse, DEFAULT_PHONG_DIFFUSE);
  const specularColor = cloneColorLike(specular, DEFAULT_PHONG_SPECULAR);
  const emissiveColor = cloneColorLike(emissive, DEFAULT_PHONG_EMISSIVE);
  const clampedTransparency = THREE.MathUtils.clamp(Number(transparency) || 0, 0, 1);
  const opacity = THREE.MathUtils.clamp(1 - clampedTransparency, 0, 1);
  const transparent = clampedTransparency > 0 && opacity < 1;
  const roughness = convertShininessToRoughness(shininess);

  const material = new THREE.MeshPhysicalNodeMaterial({
    color,
    emissive: emissiveColor,
    specularColor,
    specularIntensity: colorLuminance(specularColor),
    metalness: 0,
    roughness,
    opacity,
    transparent,
    transmission: clampedTransparency,
  });

  return applySurfaceMaterialDefaults(material, options);
}

export function createStandardSurfaceMaterial(parameters = {}, options = {}) {
  const material = new THREE.MeshStandardNodeMaterial({
    roughness: 0.6,
    metalness: 0.05,
    ...parameters,
  });
  return applySurfaceMaterialDefaults(material, options);
}

export function convertMaterialToNode(material, options = {}) {
  if (!material) {
    return material;
  }

  if (Array.isArray(material)) {
    return material.map((entry) => convertMaterialToNode(entry, options));
  }

  if (material.isMeshStandardNodeMaterial || material.isMeshPhysicalNodeMaterial) {
    return applySurfaceMaterialDefaults(material, options);
  }

  if (material.isMeshPhysicalMaterial) {
    const converted = new THREE.MeshPhysicalNodeMaterial();
    copySurfaceMaterialParameters(material, converted);
    return applySurfaceMaterialDefaults(converted, options);
  }

  if (material.isMeshStandardMaterial) {
    const converted = new THREE.MeshStandardNodeMaterial();
    copySurfaceMaterialParameters(material, converted);
    return applySurfaceMaterialDefaults(converted, options);
  }

  if (material.isMeshPhongMaterial) {
    const transparency = material.transparent ? 1 - (material.opacity ?? 1) : 0;
    const converted = createTlsMaterial(
      {
        diffuse: cloneMaterialValue(material.color) ?? DEFAULT_PHONG_DIFFUSE,
        specular: cloneMaterialValue(material.specular) ?? DEFAULT_PHONG_SPECULAR,
        emissive: cloneMaterialValue(material.emissive) ?? DEFAULT_PHONG_EMISSIVE,
        transparency,
        shininess: material.shininess,
      },
      options,
    );

    copySurfaceMaterialParameters(material, converted);

    converted.specularColor = cloneMaterialValue(material.specular) ?? converted.specularColor;
    converted.specularIntensity = colorLuminance(converted.specularColor);
    converted.roughness = convertShininessToRoughness(material.shininess);
    if (material.opacity !== undefined) {
      converted.opacity = material.opacity;
      converted.transparent = material.transparent ?? material.opacity < 1;
    }

    converted.needsUpdate = true;
    return converted;
  }

  if (material.isMaterial) {
    return applySurfaceMaterialDefaults(material, options);
  }

  return material;
}

export function ensureGeometryHasVertexNormals(geometry) {
  if (!geometry) {
    return geometry;
  }
  if (geometry.isBufferGeometry) {
    const normalAttribute = geometry.getAttribute?.('normal');
    if (!normalAttribute || normalAttribute.count === 0) {
      geometry.computeVertexNormals();
    }
    return geometry;
  }
  if (geometry.isGeometry && typeof geometry.computeVertexNormals === 'function') {
    geometry.computeVertexNormals();
  }
  return geometry;
}

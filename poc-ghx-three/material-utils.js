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

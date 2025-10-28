import * as THREE from 'three/webgpu';
import { abs, clamp, float, normalView, positionWorld, texture as textureNode, vec2 } from 'three/tsl';

export const GRASS_PREVIEW_COLOR = new THREE.Color(124 / 255, 252 / 255, 0 / 255);

export function isGrassPreviewColor(colour, tolerance = 1 / 255) {
  if (!colour?.isColor) {
    return false;
  }
  return Math.abs(colour.r - GRASS_PREVIEW_COLOR.r) <= tolerance
    && Math.abs(colour.g - GRASS_PREVIEW_COLOR.g) <= tolerance
    && Math.abs(colour.b - GRASS_PREVIEW_COLOR.b) <= tolerance;
}

let grassTextureCache = null;

function getGrassTexture() {
  if (grassTextureCache?.isTexture) {
    return grassTextureCache;
  }

  const loader = new THREE.TextureLoader();
  const textureUrl = new URL('./assets/grasstexture1.png', import.meta.url).href;
  const texture = loader.load(textureUrl);
  texture.wrapS = THREE.RepeatWrapping;
  texture.wrapT = THREE.RepeatWrapping;
  texture.colorSpace = THREE.SRGBColorSpace;
  texture.generateMipmaps = true;
  texture.anisotropy = 8;

  grassTextureCache = texture;
  return grassTextureCache;
}

export function createGrassSurfaceMaterial(options = {}) {
  const {
    side = THREE.DoubleSide,
    unitsPerTile: unitsPerTileOption = 1000,
    shadingStrength: shadingStrengthOption = 0.4,
  } = options;

  const grassTexture = getGrassTexture();
  const tileSizeValue = Number(unitsPerTileOption);
  const tileSize = Math.max(Number.isFinite(tileSizeValue) ? tileSizeValue : 1000, 0.001);
  const shadingStrength = THREE.MathUtils.clamp(Number(shadingStrengthOption) || 0, 0, 1);

  const material = new THREE.MeshPhysicalNodeMaterial({
    metalness: 0,
    roughness: 0.85,
    sheen: 0.18,
    sheenColor: new THREE.Color(0x4a6c2b),
    sheenRoughness: 0.85,
    transmission: 0,
    envMapIntensity: 0.35,
    side,
  });

  material.shadowSide = side;
  material.map = grassTexture;

  const worldXY = vec2(positionWorld.x, positionWorld.y);
  const scale = float(1 / tileSize);
  const planarUV = worldXY.mul(scale);
  const baseColour = textureNode(grassTexture, planarUV);
  const normalShade = clamp(abs(normalView.z).mul(float(shadingStrength)).add(float(1 - shadingStrength / 2)), 0.35, 1);

  material.colorNode = baseColour.mul(normalShade);
  material.userData = {
    ...(material.userData ?? {}),
    isProceduralGrass: true,
    source: 'procedural-grass',
    unitsPerTile: tileSize,
    texture: 'assets/grasstexture1.png',
    previewColor: GRASS_PREVIEW_COLOR.clone(),
  };

  material.needsUpdate = true;
  return material;
}

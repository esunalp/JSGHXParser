import * as THREE from 'three/webgpu';
import { abs, clamp, float, mix, normalView, positionWorld, texture as textureNode, vec2 } from 'three/tsl';

export const GRASS_PREVIEW_COLOR = new THREE.Color(124 / 255, 252 / 255, 0 / 255);

export function isGrassPreviewColor(colour, tolerance = 1 / 255) {
  if (!colour?.isColor) {
    return false;
  }
  return Math.abs(colour.r - GRASS_PREVIEW_COLOR.r) <= tolerance
    && Math.abs(colour.g - GRASS_PREVIEW_COLOR.g) <= tolerance
    && Math.abs(colour.b - GRASS_PREVIEW_COLOR.b) <= tolerance;
}

const textureLoader = new THREE.TextureLoader();

let grassTexture1Cache = null;
let grassTexture2Cache = null;
let grassNoiseTextureCache = null;

function loadRepeatingTexture(path, {
  colorSpace = THREE.SRGBColorSpace,
  generateMipmaps = true,
  anisotropy = 8,
} = {}) {
  const textureUrl = new URL(path, import.meta.url).href;
  const texture = textureLoader.load(textureUrl);
  texture.wrapS = THREE.RepeatWrapping;
  texture.wrapT = THREE.RepeatWrapping;
  texture.colorSpace = colorSpace;
  texture.generateMipmaps = generateMipmaps;
  texture.anisotropy = anisotropy;
  return texture;
}

function getGrassTexture1() {
  if (grassTexture1Cache?.isTexture) {
    return grassTexture1Cache;
  }

  grassTexture1Cache = loadRepeatingTexture('./assets/grasstexture1.png');
  return grassTexture1Cache;
}

function getGrassTexture2() {
  if (grassTexture2Cache?.isTexture) {
    return grassTexture2Cache;
  }

  grassTexture2Cache = loadRepeatingTexture('./assets/grasstexture2.png');
  return grassTexture2Cache;
}

function getGrassNoiseTexture() {
  if (grassNoiseTextureCache?.isTexture) {
    return grassNoiseTextureCache;
  }

  grassNoiseTextureCache = loadRepeatingTexture('./assets/noisemap.png', {
    colorSpace: THREE.LinearSRGBColorSpace,
    anisotropy: 1,
  });
  return grassNoiseTextureCache;
}

export function createGrassSurfaceMaterial(options = {}) {
  const {
    side = THREE.DoubleSide,
    unitsPerTile: unitsPerTileOption = 2000,
    shadingStrength: shadingStrengthOption = 0.4,
  } = options;

  const grassTexturePrimary = getGrassTexture1();
  const grassTextureSecondary = getGrassTexture2();
  const grassNoiseTexture = getGrassNoiseTexture();
  const tileSizeValue = Number(unitsPerTileOption);
  const tileSize = Math.max(Number.isFinite(tileSizeValue) ? tileSizeValue : 2000, 0.001);
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
  material.map = grassTexturePrimary;

  const worldXY = vec2(positionWorld.x, positionWorld.y);
  const scale = float(1 / tileSize);
  const planarUV = worldXY.mul(scale);
  const baseColourPrimary = textureNode(grassTexturePrimary, planarUV);
  const baseColourSecondary = textureNode(grassTextureSecondary, planarUV);
  const noiseScale = float(1 / 50000);
  const noiseUV = worldXY.mul(noiseScale);
  const noiseSample = textureNode(grassNoiseTexture, noiseUV).r;
  const noiseFactor = clamp(noiseSample, float(0), float(1));
  const blendedBaseColour = mix(baseColourPrimary, baseColourSecondary, noiseFactor);
  const normalShade = clamp(abs(normalView.z).mul(float(shadingStrength)).add(float(1 - shadingStrength / 2)), 0.35, 1);

  material.colorNode = blendedBaseColour.mul(normalShade);
  material.userData = {
    ...(material.userData ?? {}),
    isProceduralGrass: true,
    source: 'procedural-grass',
    unitsPerTile: tileSize,
    texture: 'assets/grasstexture1.png',
    textures: ['assets/grasstexture1.png', 'assets/grasstexture2.png'],
    noiseMap: 'assets/noisemap.png',
    previewColor: GRASS_PREVIEW_COLOR.clone(),
  };

  material.needsUpdate = true;
  return material;
}

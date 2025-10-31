import * as THREE from 'three/webgpu';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import {
  pass,
  mrt,
  output,
  normalView,
  metalness,
  roughness,
  blendColor,
  sample,
  directionToColor,
  colorToDirection,
  vec2,
} from 'three/tsl';
import { ssr } from 'three/addons/tsl/display/SSRNode.js';
import { smaa } from 'three/addons/tsl/display/SMAANode.js';
import { PhysicalSunSky } from './shaders/physical-sun-sky.js';
import {
  cloneSurfaceMaterial,
  createStandardSurfaceMaterial,
  convertMaterialToNode,
  ensureGeometryHasVertexNormals,
} from './shaders/material-utils.js';

THREE.Object3D.DEFAULT_UP.set(0, 0, 1);

const DEFAULT_CAMERA_POSITION = new THREE.Vector3(600, -600, 400);
const DEFAULT_CAMERA_TARGET = new THREE.Vector3(0, 0, 0);
const TEMP_BOX = new THREE.Box3();
const TEMP_SPHERE = new THREE.Sphere();

function isWebGPUAvailable() {
  if (typeof navigator === 'undefined') {
    return false;
  }
  return Boolean(navigator.gpu) && typeof THREE.WebGPURenderer === 'function';
}

function getViewportSize(canvas) {
  const width = canvas?.clientWidth ?? canvas?.parentElement?.clientWidth ?? window.innerWidth ?? 1;
  const height = canvas?.clientHeight ?? canvas?.parentElement?.clientHeight ?? window.innerHeight ?? 1;
  return {
    width: Math.max(1, Math.floor(width)),
    height: Math.max(1, Math.floor(height)),
    pixelRatio: Math.min(window.devicePixelRatio ?? 1, 2),
  };
}

function applyRendererDefaults(renderer) {
  if (!renderer) {
    return;
  }
  if ('outputColorSpace' in renderer && THREE.SRGBColorSpace) {
    renderer.outputColorSpace = THREE.SRGBColorSpace;
  }
  if ('physicallyCorrectLights' in renderer) {
    renderer.physicallyCorrectLights = true;
  }
  if ('toneMapping' in renderer && THREE.ACESFilmicToneMapping) {
    renderer.toneMapping = THREE.ACESFilmicToneMapping;
  }
  if ('shadowMap' in renderer && renderer.shadowMap) {
    renderer.shadowMap.enabled = true;
  }
  if (typeof renderer.setClearColor === 'function') {
    renderer.setClearColor(0x11161f, 1);
  }
}

function disposeMaterial(material) {
  if (!material) {
    return;
  }
  if (Array.isArray(material)) {
    material.forEach(disposeMaterial);
    return;
  }
  if (typeof material.dispose === 'function') {
    material.dispose();
  }
}

function disposeObject(object) {
  if (!object) {
    return;
  }
  if (object.children?.length) {
    [...object.children].forEach((child) => {
      disposeObject(child);
    });
  }
  if (object.geometry && typeof object.geometry.dispose === 'function') {
    object.geometry.dispose();
  }
  disposeMaterial(object.material);
}

function createSurfaceMesh(item) {
  if (!Array.isArray(item.vertices) || item.vertices.length === 0) {
    return null;
  }

  const positions = new Float32Array(item.vertices.flat());
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3));

  if (Array.isArray(item.faces) && item.faces.length) {
    const indices = item.faces.flat();
    let maxIndex = 0;
    for (const value of indices) {
      if (typeof value === 'number' && value > maxIndex) {
        maxIndex = value;
      }
    }
    const typedArray =
      maxIndex > 65535 ? new Uint32Array(indices) : new Uint16Array(indices);
    geometry.setIndex(new THREE.BufferAttribute(typedArray, 1));
  }

  geometry.computeVertexNormals();
  geometry.computeBoundingSphere();

  const material = new THREE.MeshStandardMaterial({
    color: 0x3c82ff,
    metalness: 0.1,
    roughness: 0.65,
    side: THREE.DoubleSide,
  });

  const mesh = new THREE.Mesh(geometry, material);
  mesh.castShadow = true;
  mesh.receiveShadow = true;
  mesh.userData.generatedByGHX = true;
  return mesh;
}

function createLineObject(item) {
  if (!Array.isArray(item.points) || item.points.length < 2) {
    return null;
  }
  const points = item.points
    .map((point) => (Array.isArray(point) && point.length >= 3 ? new THREE.Vector3(point[0], point[1], point[2]) : null))
    .filter(Boolean);
  if (points.length < 2) {
    return null;
  }
  const geometry = new THREE.BufferGeometry().setFromPoints(points);
  const material = new THREE.LineBasicMaterial({ color: 0xffffff, linewidth: 1 });
  const line = new THREE.Line(geometry, material);
  line.userData.generatedByGHX = true;
  line.userData.overlay = true;
  return line;
}

function createPointObject(item) {
  if (!Array.isArray(item.coordinates) || item.coordinates.length < 3) {
    return null;
  }
  const geometry = new THREE.SphereGeometry(12, 18, 14);
  const material = new THREE.MeshStandardMaterial({ color: 0xffaa33, emissive: 0x111111 });
  const point = new THREE.Mesh(geometry, material);
  point.position.set(item.coordinates[0], item.coordinates[1], item.coordinates[2]);
  point.castShadow = true;
  point.userData.generatedByGHX = true;
  point.userData.overlay = true;
  return point;
}

function focusCamera(camera, controls, groups) {
  TEMP_BOX.makeEmpty();
  for (const group of groups) {
    if (group) {
      TEMP_BOX.expandByObject(group);
    }
  }

  if (TEMP_BOX.isEmpty()) {
    camera.position.copy(DEFAULT_CAMERA_POSITION);
    controls.target.copy(DEFAULT_CAMERA_TARGET);
    controls.update();
    return;
  }

  TEMP_BOX.getBoundingSphere(TEMP_SPHERE);
  const center = TEMP_SPHERE.center;
  const radius = Math.max(TEMP_SPHERE.radius, 1);
  const offset = new THREE.Vector3(1.2, -1.0, 0.75).normalize().multiplyScalar(radius * 3.2);

  camera.position.copy(center.clone().add(offset));
  controls.target.copy(center);
  controls.update();
}

export function createThreeApp(canvas) {
  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0x11161f);

  const ambient = new THREE.AmbientLight(0xffffff, 0.55);
  const directional = new THREE.DirectionalLight(0xffffff, 0.9);
  directional.position.set(450, -320, 520);
  directional.castShadow = true;
  directional.shadow.mapSize.set(1024, 1024);
  directional.shadow.camera.near = 0.1;
  directional.shadow.camera.far = 5000;

  scene.add(ambient);
  scene.add(directional);

  const grid = new THREE.GridHelper(2000, 40, 0x2c3646, 0x1a2130);
  if (Array.isArray(grid.material)) {
    for (const material of grid.material) {
      material.transparent = true;
      material.opacity = 0.35;
    }
  } else if (grid.material) {
    grid.material.transparent = true;
    grid.material.opacity = 0.35;
  }
  grid.rotation.x = Math.PI / 2;
  scene.add(grid);

  const camera = new THREE.PerspectiveCamera(60, 1, 0.1, 100000);
  camera.position.copy(DEFAULT_CAMERA_POSITION);
  camera.up.set(0, 0, 1);

  const controls = new OrbitControls(camera, canvas);
  controls.enableDamping = true;
  controls.dampingFactor = 0.08;
  controls.target.copy(DEFAULT_CAMERA_TARGET);
  controls.update();

  const geometryGroup = new THREE.Group();
  geometryGroup.name = 'ghx-geometry';
  const overlayGroup = new THREE.Group();
  overlayGroup.name = 'ghx-overlay';
  overlayGroup.visible = true;

  scene.add(geometryGroup);
  scene.add(overlayGroup);

  const overlayObjects = new Set();
  const state = {
    renderer: null,
    overlayEnabled: true,
  };

  const handleResize = () => {
    if (!state.renderer) {
      return;
    }
    const viewport = getViewportSize(canvas);
    camera.aspect = viewport.width / viewport.height;
    camera.updateProjectionMatrix();
    state.renderer.setPixelRatio(viewport.pixelRatio);
    state.renderer.setSize(viewport.width, viewport.height, false);
  };

  const ready = (async () => {
    if (!isWebGPUAvailable()) {
      throw new Error('WebGPU wordt niet ondersteund in deze omgeving.');
    }
    const renderer = new THREE.WebGPURenderer({ canvas, antialias: true });
    await renderer.init();
    applyRendererDefaults(renderer);
    state.renderer = renderer;
    handleResize();
    renderer.setAnimationLoop(() => {
      controls.update();
      renderer.render(scene, camera);
    });
  })();

  ready.catch(() => {
    // Laat de foutafhandeling over aan de aanroeper
  });

  if (typeof window !== 'undefined') {
    window.addEventListener('resize', handleResize);
  }

  const clearGroup = (group) => {
    if (!group) {
      return;
    }
    const children = [...group.children];
    for (const child of children) {
      group.remove(child);
      disposeObject(child);
    }
  };

  const updateGeometry = (items) => {
    clearGroup(geometryGroup);
    clearGroup(overlayGroup);
    overlayObjects.clear();

    if (!Array.isArray(items) || !items.length) {
      focusCamera(camera, controls, [geometryGroup, overlayGroup]);
      return;
    }

    for (const item of items) {
      if (!item || typeof item !== 'object') {
        continue;
      }
      if (item.type === 'Surface') {
        const mesh = createSurfaceMesh(item);
        if (mesh) {
          geometryGroup.add(mesh);
        }
      } else if (item.type === 'CurveLine') {
        const line = createLineObject(item);
        if (line) {
          overlayGroup.add(line);
          overlayObjects.add(line);
        }
      } else if (item.type === 'Point') {
        const point = createPointObject(item);
        if (point) {
          overlayGroup.add(point);
          overlayObjects.add(point);
        }
      }
    }

    overlayGroup.visible = state.overlayEnabled;
    for (const object of overlayObjects) {
      object.visible = state.overlayEnabled;
    }

    focusCamera(camera, controls, [geometryGroup, overlayGroup]);
  };

  const setOverlayEnabled = (enabled) => {
    state.overlayEnabled = Boolean(enabled);
    overlayGroup.visible = state.overlayEnabled;
    for (const object of overlayObjects) {
      object.visible = state.overlayEnabled;
    }
  };

  const dispose = () => {
    if (typeof window !== 'undefined') {
      window.removeEventListener('resize', handleResize);
    }
    if (state.renderer) {
      state.renderer.setAnimationLoop(null);
      state.renderer.dispose();
      state.renderer = null;
    }
    clearGroup(geometryGroup);
    clearGroup(overlayGroup);
    overlayObjects.clear();
  };

  // Standaard overlay state gelijk houden met UI
  setOverlayEnabled(true);

  return {
    ready,
    updateGeometry,
    setOverlayEnabled,
    isWebGPUSupported: () => Boolean(state.renderer),
    dispose,
  };
}

//# sourceMappingURL=three_integration.js.map

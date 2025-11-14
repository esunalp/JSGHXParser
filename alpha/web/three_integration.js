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
  createTlsMaterial,
  convertMaterialToNode,
  ensureGeometryHasVertexNormals,
} from './shaders/material-utils.js';

THREE.Object3D.DEFAULT_UP.set(0, 0, 1);

const DEFAULT_CAMERA_POSITION = new THREE.Vector3(600, -600, 400);
const DEFAULT_CAMERA_TARGET = new THREE.Vector3(0, 0, 0);

// Toggle to enable or disable double sided rendering for viewport meshes.
const ENABLE_DOUBLE_SIDED_MESHES = false;
const DEFAULT_MESH_SIDE = ENABLE_DOUBLE_SIDED_MESHES ? THREE.DoubleSide : THREE.FrontSide;

const AXES_LENGTH_MM = 5000;
const MAX_DRAW_DISTANCE_MM = 100000;
const SCRATCH_BOUNDING_BOX = new THREE.Box3();
const SCRATCH_BOUNDING_BOX_TEMP = new THREE.Box3();
const SCRATCH_BOUNDING_SPHERE = new THREE.Sphere();

function getViewportSize(canvas) {
  const width = canvas.clientWidth || canvas.parentElement?.clientWidth || window.innerWidth || 1;
  const height = canvas.clientHeight || canvas.parentElement?.clientHeight || window.innerHeight || 1;
  return {
    width: Math.max(width, 1),
    height: Math.max(height, 1),
    pixelRatio: Math.min(window.devicePixelRatio || 1, 2),
  };
}

function applyRendererDefaults(renderer) {
  if (!renderer) {
    return;
  }

  if ('outputColorSpace' in renderer && THREE.SRGBColorSpace) {
    renderer.outputColorSpace = THREE.SRGBColorSpace;
  } else if ('outputEncoding' in renderer) {
    renderer.outputEncoding = THREE.sRGBEncoding;
  }

  if ('physicallyCorrectLights' in renderer) {
    renderer.physicallyCorrectLights = true;
  }

  if ('toneMapping' in renderer && THREE.ACESFilmicToneMapping) {
    renderer.toneMapping = THREE.ACESFilmicToneMapping;
  }

  if (renderer.shadowMap) {
    renderer.shadowMap.enabled = true;
    if ('type' in renderer.shadowMap && THREE.PCFSoftShadowMap) {
      renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    }
  }

  if (typeof renderer.setClearColor === 'function') {
    renderer.setClearColor(0x060910, 1);
  }

  if ('useLegacyLights' in renderer) {
    renderer.useLegacyLights = false;
  }
}

function applyViewportToRenderer(renderer, viewport) {
  if (!renderer || !viewport) {
    return;
  }

  if (typeof renderer.setPixelRatio === 'function') {
    renderer.setPixelRatio(viewport.pixelRatio);
  }

  if (typeof renderer.setSize === 'function') {
    renderer.setSize(viewport.width, viewport.height, false);
  }
}

async function createWebGPURenderer(canvas, viewport) {
  const renderer = new THREE.WebGPURenderer({ canvas, antialias: true });
  await renderer.init();
  applyRendererDefaults(renderer);
  applyViewportToRenderer(renderer, viewport);
  return renderer;
}

function addHelpers(scene) {
  const axes = new THREE.AxesHelper(AXES_LENGTH_MM);
  ensureGeometryHasVertexNormals(axes.geometry, { compute: false });
  scene.add(axes);
}

const OVERLAY_LINE_COLOR = new THREE.Color(0x000000);
const OVERLAY_POINT_COLOR = new THREE.Color(0x000000);
const POINT_SPHERE_RADIUS_MM = 10;
const POINT_SPHERE_WIDTH_SEGMENTS = 16;
const POINT_SPHERE_HEIGHT_SEGMENTS = 12;

function createMeshObject(item) {
  if (!Array.isArray(item.vertices) || item.vertices.length === 0) {
    return null;
  }

  const positions = new Float32Array(item.vertices.flat());
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3));

  if (Array.isArray(item.faces) && item.faces.length) {
    const flippedFaces = item.faces.map((face) => {
      if (!Array.isArray(face) || face.length < 3) {
        return face;
      }

      const flipped = face.slice();
      for (let left = 1, right = flipped.length - 1; left < right; left += 1, right -= 1) {
        const swap = flipped[left];
        flipped[left] = flipped[right];
        flipped[right] = swap;
      }

      return flipped;
    });

    const indices = flippedFaces.flat();
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

  const createPreviewMaterial = (materialData) => {
    if (!materialData || typeof materialData !== 'object') {
      return createStandardSurfaceMaterial(
        { color: 0x3c82ff, metalness: 0.1, roughness: 0.65 },
        { side: DEFAULT_MESH_SIDE },
      );
    }

    const toColorLike = (value, fallback) => {
      if (Array.isArray(value) && value.length >= 3) {
        return [
          Number.isFinite(value[0]) ? value[0] : fallback[0],
          Number.isFinite(value[1]) ? value[1] : fallback[1],
          Number.isFinite(value[2]) ? value[2] : fallback[2],
        ];
      }

      if (value && typeof value === 'object') {
        const { r, g, b } = value;
        if ([r, g, b].every((component) => Number.isFinite(component))) {
          return [r, g, b];
        }
      }

      if (typeof value === 'number' && Number.isFinite(value)) {
        return value;
      }

      return fallback;
    };

    const transparency = Number(materialData.transparency);
    const shininess = Number(materialData.shine);

    return createTlsMaterial(
      {
        diffuse: toColorLike(materialData.diffuse, [0.6, 0.65, 0.69]),
        specular: toColorLike(materialData.specular, [0.2, 0.2, 0.2]),
        emissive: toColorLike(materialData.emission, [0, 0, 0]),
        transparency: Number.isFinite(transparency)
          ? THREE.MathUtils.clamp(transparency, 0, 1)
          : 0,
        shininess: Number.isFinite(shininess) ? shininess : 30,
      },
      { side: DEFAULT_MESH_SIDE },
    );
  };

  const material = createPreviewMaterial(item.material);

  const mesh = new THREE.Mesh(geometry, material);
  mesh.castShadow = true;
  mesh.receiveShadow = true;
  mesh.userData.generatedByGHX = true;
  return mesh;
}

function createSegmentsObject(points) {
    if (!points || points.length < 2) {
        return null;
    }
    const geometry = new THREE.BufferGeometry().setFromPoints(points);
    ensureGeometryHasVertexNormals(geometry, { compute: false });
    const material = new THREE.LineBasicMaterial({
        color: OVERLAY_LINE_COLOR,
        transparent: true,
        opacity: 0.95,
        depthWrite: false,
    });
    const object = new THREE.Line(geometry, material);
    return { object, disposables: [geometry, material] };
}

function createPointsObject(points) {
    if (!Array.isArray(points) || !points.length) {
        return null;
    }
    const geometry = new THREE.SphereGeometry(
        POINT_SPHERE_RADIUS_MM,
        POINT_SPHERE_WIDTH_SEGMENTS,
        POINT_SPHERE_HEIGHT_SEGMENTS,
    );
    const material = createStandardSurfaceMaterial(
        {
            color: OVERLAY_POINT_COLOR,
            metalness: 0,
            roughness: 0.55,
        },
        { side: DEFAULT_MESH_SIDE },
    );
    const object = new THREE.InstancedMesh(geometry, material, points.length);
    const matrix = new THREE.Matrix4();
    points.forEach((point, index) => {
        matrix.makeTranslation(point.x, point.y, point.z);
        object.setMatrixAt(index, matrix);
    });
    object.instanceMatrix.needsUpdate = true;
    object.castShadow = false;
    object.receiveShadow = false;
    return { object, disposables: [geometry, material] };
}

export function createThreeApp(canvas) {
  const scene = new THREE.Scene();
  scene.background = null;

  const camera = new THREE.PerspectiveCamera(50, 1, 0.1, MAX_DRAW_DISTANCE_MM);
  camera.up.set(0, 0, 1);
  camera.position.set(6, 4, 8);

  let viewportState = getViewportSize(canvas);

  let webgpuRenderer = null;
  let postProcessing = null;
  let ssrPass = null;

  const webgpuSupported = typeof navigator !== 'undefined'
    && 'gpu' in navigator
    && (typeof THREE.WebGPURenderer.isAvailable !== 'function' || THREE.WebGPURenderer.isAvailable());

  const MIN_CAMERA_Z = 0;

  const controls = new OrbitControls(camera, canvas);
  controls.enableDamping = true;
  controls.screenSpacePanning = false;

  const raycaster = new THREE.Raycaster();
  const pointerNdc = new THREE.Vector2();

  const eventTarget = canvas;

  const sunSky = new PhysicalSunSky(scene);
  sunSky.setTarget(controls.target);
  addHelpers(scene);

  function setupPostProcessing(renderer) {
    if (!renderer || typeof THREE.PostProcessing !== 'function') {
      postProcessing = null;
      ssrPass = null;
      return;
    }

    postProcessing = new THREE.PostProcessing(renderer);

    const scenePass = pass(scene, camera);
    scenePass.setMRT(mrt({
      output,
      normal: directionToColor(normalView),
      metalrough: vec2(metalness, roughness),
    }));

    const scenePassColor = scenePass.getTextureNode('output');
    const scenePassNormal = scenePass.getTextureNode('normal');
    const scenePassDepth = scenePass.getTextureNode('depth');
    const scenePassMetalRough = scenePass.getTextureNode('metalrough');

    const normalTexture = scenePass.getTexture('normal');
    if (normalTexture) {
      normalTexture.type = THREE.UnsignedByteType;
    }

    const metalRoughTexture = scenePass.getTexture('metalrough');
    if (metalRoughTexture) {
      metalRoughTexture.type = THREE.UnsignedByteType;
    }

    const sceneNormal = sample((uv) => colorToDirection(scenePassNormal.sample(uv)));

    ssrPass = ssr(
      scenePassColor,
      scenePassDepth,
      sceneNormal,
      scenePassMetalRough.r,
      scenePassMetalRough.g,
    );

    // Tune the SSR pass so reflections read stronger in the viewport.
    ssrPass.quality.value = 0.75; // march more samples for crisper reflections
    ssrPass.blurQuality.value = 0.75; // keep reflections sharper after the blur stage
    ssrPass.maxDistance.value = 40000; // allow reflections to travel further across the scene
    ssrPass.opacity.value = 1.5; // boost the contribution of the reflection colour
    ssrPass.thickness.value = 0.25; // widen the hit threshold to catch more surfaces

    const outputNode = smaa(blendColor(scenePassColor, ssrPass));
    postProcessing.outputNode = outputNode;
    postProcessing.needsUpdate = true;
  }

  function updateRendererViewports() {
    if (webgpuRenderer) {
      applyViewportToRenderer(webgpuRenderer, viewportState);
    }
    if (postProcessing?.setSize) {
      postProcessing.setSize(viewportState.width, viewportState.height);
    }
  }

  const resize = () => {
    viewportState = getViewportSize(canvas);
    const width = viewportState.width;
    const height = viewportState.height;
    camera.aspect = width / height;
    camera.updateProjectionMatrix();
    updateRendererViewports();
  };
  resize();
  window.addEventListener('resize', resize);

  let currentObject = null;
  let needsFit = true;
  let overlayEnabled = false;
  let currentOverlayGroup = null;
  const overlayItemsByNode = new Map();

  function rebuildOverlayGroup() {
      if (currentOverlayGroup) {
          scene.remove(currentOverlayGroup);
          disposeSceneObject(currentOverlayGroup);
          currentOverlayGroup = null;
      }
      if (!overlayEnabled || overlayItemsByNode.size === 0) {
          return;
      }
      const group = new THREE.Group();
      group.name = 'GHXCurveOverlay';

      for (const items of overlayItemsByNode.values()) {
          items.forEach(item => {
              if (item.type === 'Line') {
                  const points = [item.start, item.end]
                      .filter(Array.isArray)
                      .map(p => new THREE.Vector3(p[0], p[1], p[2]));
                  const segmentObject = createSegmentsObject(points);
                  if (segmentObject) {
                      group.add(segmentObject.object);
                  }
              } else if (item.type === 'Polyline') {
                  const points = Array.isArray(item.points)
                      ? item.points.map(p => new THREE.Vector3(p[0], p[1], p[2]))
                      : [];
                  const segmentObject = createSegmentsObject(points);
                  if (segmentObject) {
                      group.add(segmentObject.object);
                  }
              } else if (item.type === 'Point') {
                  const point = new THREE.Vector3(item.coordinates[0], item.coordinates[1], item.coordinates[2]);
                  const pointObject = createPointsObject([point]);
                  if (pointObject) {
                      group.add(pointObject.object);
                  }
              }
          });
      }

      if (group.children.length > 0) {
          group.traverse(child => {
              if (child.geometry) {
                  ensureGeometryHasVertexNormals(child.geometry, { compute: false });
              }
          });
          currentOverlayGroup = group;
          scene.add(currentOverlayGroup);
      }
  }


  function setOverlayEnabled(value) {
    overlayEnabled = Boolean(value);
    rebuildOverlayGroup();
  }

  function computeGeometryBounds(objects, boxTarget, sphereTarget) {
    boxTarget.makeEmpty();
    let hasBounds = false;
    for (const object of objects) {
      if (!object?.isObject3D) {
        continue;
      }
      object.updateWorldMatrix(true, false);
      SCRATCH_BOUNDING_BOX_TEMP.setFromObject(object);
      if (SCRATCH_BOUNDING_BOX_TEMP.isEmpty()) {
        continue;
      }
      if (!hasBounds) {
        boxTarget.copy(SCRATCH_BOUNDING_BOX_TEMP);
        hasBounds = true;
      } else {
        boxTarget.union(SCRATCH_BOUNDING_BOX_TEMP);
      }
    }
    if (!hasBounds) {
      if (sphereTarget) {
        sphereTarget.center.set(0, 0, 0);
        sphereTarget.radius = 0;
      }
      return false;
    }
    if (sphereTarget) {
      boxTarget.getBoundingSphere(sphereTarget);
    }
    return true;
  }

  function fitCameraToSphere(sphere) {
    if (!sphere) {
      return;
    }

    const center = sphere.center.clone();
    const radius = Math.max(sphere.radius, 0.0001);

    const previousTarget = controls.target.clone();
    let direction = camera.position.clone().sub(previousTarget);
    if (!Number.isFinite(direction.lengthSq()) || direction.lengthSq() < 1e-6) {
      direction.set(0, 0, 1);
    }
    direction.normalize();

    const aspect = camera.aspect || 1;
    const halfVerticalFov = THREE.MathUtils.degToRad(camera.fov) / 2;
    const safeHalfVertical = halfVerticalFov > 1e-4 ? halfVerticalFov : 1e-4;
    const halfHorizontalFov = Math.atan(Math.tan(safeHalfVertical) * aspect);
    const safeHalfHorizontal = halfHorizontalFov > 1e-4 ? halfHorizontalFov : 1e-4;

    const verticalDistance = radius / Math.tan(safeHalfVertical);
    const horizontalDistance = radius / Math.tan(safeHalfHorizontal);
    const distance = Math.max(verticalDistance, horizontalDistance, radius * 1.5, 1);

    const newPosition = center.clone().add(direction.multiplyScalar(distance));
    newPosition.z = Math.max(newPosition.z, MIN_CAMERA_Z);

    controls.target.copy(center);
    sunSky.setTarget(center);
    camera.position.copy(newPosition);
    camera.near = Math.max(distance / 100, 0.01);
    camera.far = Math.max(distance * 4, distance + radius * 4, MAX_DRAW_DISTANCE_MM);
    camera.updateProjectionMatrix();
    controls.update();
  }


  function updateOrbitTarget(targetPoint) {
    if (!targetPoint?.isVector3) {
      return;
    }
    controls.target.copy(targetPoint);
    sunSky.setTarget(targetPoint);
    controls.update();
  }

  function clampCameraHeight() {
    if (camera.position.z < MIN_CAMERA_Z) {
      camera.position.z = MIN_CAMERA_Z;
    }
  }

  function findPointerIntersection(event) {
    const bounds = eventTarget.getBoundingClientRect();
    pointerNdc.x = ((event.clientX - bounds.left) / bounds.width) * 2 - 1;
    pointerNdc.y = -((event.clientY - bounds.top) / bounds.height) * 2 + 1;
    raycaster.setFromCamera(pointerNdc, camera);
    if (!currentObject) return null;
    const intersections = raycaster.intersectObject(currentObject, true);
    return intersections.length > 0 ? intersections[0].point.clone() : null;
  }

  const ORBIT_CLICK_DISTANCE_SQ = 4 * 4;
  let pendingOrbitTarget = null;

  function handlePointerDown(event) {
    if (event.button !== 0) return;
    pendingOrbitTarget = {
      pointerId: event.pointerId,
      clientX: event.clientX,
      clientY: event.clientY,
      moved: false,
      targetPoint: findPointerIntersection(event),
    };
  }

  function handlePointerMove(event) {
    if (!pendingOrbitTarget || event.pointerId !== pendingOrbitTarget.pointerId) return;
    const dx = event.clientX - pendingOrbitTarget.clientX;
    const dy = event.clientY - pendingOrbitTarget.clientY;
    if (dx * dx + dy * dy > ORBIT_CLICK_DISTANCE_SQ) {
      pendingOrbitTarget.moved = true;
    }
  }

  function handlePointerUp(event) {
    if (!pendingOrbitTarget || event.pointerId !== pendingOrbitTarget.pointerId) return;
    if (!pendingOrbitTarget.moved) {
      updateOrbitTarget(pendingOrbitTarget.targetPoint);
    }
    pendingOrbitTarget = null;
  }

  eventTarget.addEventListener('pointerdown', handlePointerDown);
  eventTarget.addEventListener('pointermove', handlePointerMove);
  eventTarget.addEventListener('pointerup', handlePointerUp);
  eventTarget.addEventListener('pointerleave', () => pendingOrbitTarget = null);

  let webgpuInitPromise = null;
  async function ensureWebGPURenderer() {
    if (!webgpuSupported) {
      throw new Error('WebGPU wordt niet ondersteund in deze omgeving.');
    }
    if (webgpuRenderer) return webgpuRenderer;
    if (!webgpuInitPromise) {
      webgpuInitPromise = createWebGPURenderer(canvas, viewportState)
        .then(renderer => {
          webgpuRenderer = renderer;
          sunSky.setRenderer(renderer);
          setupPostProcessing(renderer);
          return renderer;
        })
        .catch(error => {
          webgpuInitPromise = null;
          throw error;
        });
    }
    return webgpuInitPromise;
  }

  function disposeSceneObject(object) {
      if (!object) return;
      if (typeof object.userData?.dispose === 'function') {
          object.userData.dispose();
          return;
      }
      if (object.geometry) object.geometry.dispose();
      if (object.material) {
          if (Array.isArray(object.material)) {
              object.material.forEach(m => m.dispose());
          } else {
              object.material.dispose();
          }
      }
      while(object.children.length > 0){
          disposeSceneObject(object.children[0]);
          object.remove(object.children[0]);
      }
  }


  function applyShadowDefaults(object) {
    if (!object?.isObject3D) return;
    object.traverse(child => {
      if (child.isMesh) {
        const preparedMaterial = convertMaterialToNode(child.material, { side: DEFAULT_MESH_SIDE });
        if (preparedMaterial) child.material = preparedMaterial;
        ensureGeometryHasVertexNormals(child.geometry);
      }
      child.castShadow = true;
      child.receiveShadow = true;
    });
  }

  const geometryObjects = new Map();

  function updateGeometry(diff, options = {}) {
      const { added = [], updated = [], removed = [] } = diff ?? {};
      const { preserveCamera = false, refitCamera = false } = options;

      removed.forEach(id => {
          const existing = geometryObjects.get(id);
          if (existing) {
              scene.remove(existing);
              disposeSceneObject(existing);
              geometryObjects.delete(id);
          }
          overlayItemsByNode.delete(id);
      });

      const processItem = (item) => {
          if (!item || !item.type) return null;

          if (item.type === 'Mesh') {
              return createMeshObject(item);
          }
          // Non-mesh items are handled by the overlay group.
          return null;
      };

      const createGroupFromItems = (items) => {
          if (!Array.isArray(items) || items.length === 0) {
              return null;
          }
          const group = new THREE.Group();
          items.forEach(geometryItem => {
              const obj = processItem(geometryItem);
              if (obj) {
                  group.add(obj);
              }
          });
          return group.children.length > 0 ? group : null;
      };

      const updateNode = (nodeOutput) => {
          const { id, items = [] } = nodeOutput;
          const existing = geometryObjects.get(id);
          if (existing) {
              scene.remove(existing);
              disposeSceneObject(existing);
              geometryObjects.delete(id);
          }

          const meshItems = items.filter(item => item.type === 'Mesh');
          const overlayItems = items.filter(item => item.type !== 'Mesh');

          const newGroup = createGroupFromItems(meshItems);
          if (newGroup) {
              geometryObjects.set(id, newGroup);
              scene.add(newGroup);
          }

          if (overlayItems.length > 0) {
              overlayItemsByNode.set(id, overlayItems);
          } else {
              overlayItemsByNode.delete(id);
          }
      };

      updated.forEach(updateNode);
      added.forEach(updateNode);

      rebuildOverlayGroup();

      const shouldPreserveView = preserveCamera && !refitCamera && !needsFit;

      const hasBounds = computeGeometryBounds(geometryObjects.values(), SCRATCH_BOUNDING_BOX, SCRATCH_BOUNDING_SPHERE);
      sunSky.updateShadowBounds(hasBounds ? SCRATCH_BOUNDING_BOX : null);

      let sphere = null;
      if (!shouldPreserveView && hasBounds) {
          sphere = SCRATCH_BOUNDING_SPHERE;
      }

      if (sphere && (refitCamera || needsFit)) {
          fitCameraToSphere(sphere);
          needsFit = false;
      } else if (!hasBounds) {
          controls.target.copy(DEFAULT_CAMERA_TARGET);
          camera.position.copy(DEFAULT_CAMERA_POSITION);
          controls.update();
          needsFit = true;
      } else if (refitCamera && !sphere) {
          needsFit = true;
      }
  }

  function animate() {
    requestAnimationFrame(animate);
    controls.update();
    clampCameraHeight();
    sunSky.updateFrame(camera);
    if (postProcessing) {
      postProcessing.render();
    } else if (webgpuRenderer) {
      webgpuRenderer.render(scene, camera);
    }
  }

  const ready = ensureWebGPURenderer().catch(err => {
      console.error("Renderer initialisatie mislukt:", err);
      throw err;
  });

  animate();

  const dispose = () => {
    window.removeEventListener('resize', resize);
    eventTarget.removeEventListener('pointerdown', handlePointerDown);
    eventTarget.removeEventListener('pointermove', handlePointerMove);
    eventTarget.removeEventListener('pointerup', handlePointerUp);
    eventTarget.removeEventListener('pointerleave', () => pendingOrbitTarget = null);

    if (webgpuRenderer) {
        webgpuRenderer.setAnimationLoop(null);
        webgpuRenderer.dispose();
    }
    disposeSceneObject(scene);
  };

  return {
    ready,
    updateGeometry,
    setOverlayEnabled,
    isWebGPUSupported: () => webgpuSupported,
    dispose,
    // For debugging
    scene,
    camera,
    controls,
  };
}

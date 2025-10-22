import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';

function createRenderer(canvas) {
  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  renderer.setSize(canvas.clientWidth || window.innerWidth, canvas.clientHeight || window.innerHeight, false);
  renderer.outputEncoding = THREE.sRGBEncoding;
  renderer.setClearColor(0x111111, 1);
  return renderer;
}

function addDefaultLights(scene) {
  const ambient = new THREE.AmbientLight(0xffffff, 0.4);
  scene.add(ambient);

  const dirLight = new THREE.DirectionalLight(0xffffff, 0.8);
  dirLight.position.set(5, 8, 4);
  scene.add(dirLight);
}

function addHelpers(scene) {
  const grid = new THREE.GridHelper(20, 20, 0x888888, 0x444444);
  grid.position.y = -0.5;
  scene.add(grid);

  const axes = new THREE.AxesHelper(5);
  scene.add(axes);
}

export function initScene(canvas) {
  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0x050505);

  const camera = new THREE.PerspectiveCamera(50, 1, 0.1, 1000);
  camera.position.set(6, 4, 8);

  const renderer = createRenderer(canvas);
  const controls = new OrbitControls(camera, renderer.domElement);
  controls.enableDamping = true;

  addDefaultLights(scene);
  addHelpers(scene);

  const resize = () => {
    const width = canvas.clientWidth || canvas.parentElement?.clientWidth || window.innerWidth;
    const height = canvas.clientHeight || canvas.parentElement?.clientHeight || window.innerHeight;
    camera.aspect = width / height;
    camera.updateProjectionMatrix();
    renderer.setSize(width, height, false);
  };
  resize();
  window.addEventListener('resize', resize);

  let currentMesh = null;
  let needsFit = true;

  function computeWorldBoundingSphere(mesh) {
    const geometry = mesh?.geometry;
    if (!geometry) {
      return null;
    }

    if (typeof geometry.computeBoundingSphere === 'function') {
      geometry.computeBoundingSphere();
    }

    if (!geometry.boundingSphere) {
      return null;
    }

    const sphere = geometry.boundingSphere.clone();
    if (!sphere) {
      return null;
    }

    if (typeof mesh.updateWorldMatrix === 'function') {
      mesh.updateWorldMatrix(true, false);
    } else {
      mesh.updateMatrixWorld(true);
    }

    sphere.center.applyMatrix4(mesh.matrixWorld);

    const scale = new THREE.Vector3();
    const position = new THREE.Vector3();
    const quaternion = new THREE.Quaternion();
    mesh.matrixWorld.decompose(position, quaternion, scale);
    const maxScale = Math.max(scale.x, scale.y, scale.z);
    if (Number.isFinite(maxScale) && maxScale > 0) {
      sphere.radius *= maxScale;
    }

    return sphere;
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

    controls.target.copy(center);
    camera.position.copy(newPosition);
    camera.near = Math.max(distance / 100, 0.01);
    camera.far = Math.max(distance * 4, distance + radius * 4);
    camera.updateProjectionMatrix();
    controls.update();
  }

  function updateMesh(geometryOrMesh) {
    if (currentMesh) {
      scene.remove(currentMesh);
      if (geometryOrMesh !== currentMesh) {
        currentMesh.geometry?.dispose?.();
        currentMesh.material?.dispose?.();
      }
      currentMesh = null;
    }

    if (!geometryOrMesh) {
      needsFit = true;
      return;
    }

    if (geometryOrMesh.isMesh) {
      currentMesh = geometryOrMesh;
    } else if (geometryOrMesh.isBufferGeometry || geometryOrMesh.isGeometry) {
      const material = new THREE.MeshStandardMaterial({ color: 0x2c9cf5, metalness: 0.1, roughness: 0.65 });
      currentMesh = new THREE.Mesh(geometryOrMesh, material);
    } else if (geometryOrMesh.geometry) {
      const material = geometryOrMesh.material || new THREE.MeshStandardMaterial({ color: 0x2c9cf5 });
      currentMesh = new THREE.Mesh(geometryOrMesh.geometry, material);
    } else {
      console.warn('updateMesh: onbekend objecttype', geometryOrMesh);
      return;
    }

    currentMesh.castShadow = true;
    currentMesh.receiveShadow = true;
    scene.add(currentMesh);

    const sphere = computeWorldBoundingSphere(currentMesh);
    if (sphere) {
      fitCameraToSphere(sphere);
      needsFit = false;
    } else if (needsFit) {
      controls.target.set(0, 0, 0);
      controls.update();
      needsFit = false;
    }
  }

  function animate() {
    requestAnimationFrame(animate);
    controls.update();
    renderer.render(scene, camera);
  }
  animate();

  return { scene, camera, renderer, controls, updateMesh };
}

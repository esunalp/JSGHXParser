import * as THREE from 'three/webgpu';
import { CSMShadowNode } from 'three/addons/csm/CSMShadowNode.js';

const DEFAULT_OPTIONS = {
  cascades: 4,
  maxFar: 60000,
  mode: 'practical',
  lightMargin: 400,
  shadowMapSize: 2048,
  shadowBias: -0.00045,
  shadowNormalBias: 0.04,
  shadowNear: 1,
  shadowFar: 200000,
  fade: true,
};

function isPositiveNumber(value) {
  return Number.isFinite(value) && value > 0;
}

export class CascadedShadowMaps {
  constructor(light, options = {}) {
    this.light = light;
    this.options = { ...DEFAULT_OPTIONS, ...options };
    this.shadowNode = null;
    this.camera = null;
    this._projectionMatrix = null;
    this._cameraParams = { near: null, far: null, zoom: null };
    this._needsFrustumUpdate = true;
    this._shadowNodeValidated = false;
    this._shadowNodeFailed = false;
    this._desiredFade = Boolean(this.options.fade);
    this._stateChangeCallback = null;

    this.configureLight();
  }

  setStateChangeCallback(callback) {
    if (typeof callback === 'function') {
      this._stateChangeCallback = callback;
    } else {
      this._stateChangeCallback = null;
    }

    if (this._stateChangeCallback) {
      this._emitStateChange(Boolean(this.shadowNode));
    }
  }

  _emitStateChange(hasShadowNode) {
    if (!this._stateChangeCallback) {
      return;
    }

    try {
      this._stateChangeCallback({
        hasShadowNode: Boolean(hasShadowNode),
        shadowNode: hasShadowNode ? this.shadowNode : null,
      });
    } catch (error) {
      console.warn('CascadedShadowMaps: state change callback error', error);
    }
  }

  configureLight() {
    if (!this.light?.shadow) {
      return;
    }

    const shadow = this.light.shadow;
    const { shadowMapSize, shadowBias, shadowNormalBias, shadowNear, shadowFar } = this.options;

    if (isPositiveNumber(shadowMapSize)) {
      shadow.mapSize.set(shadowMapSize, shadowMapSize);
    }

    if (Number.isFinite(shadowBias)) {
      shadow.bias = shadowBias;
    }

    if (Number.isFinite(shadowNormalBias) && 'normalBias' in shadow) {
      shadow.normalBias = shadowNormalBias;
    }

    if (Number.isFinite(shadowNear) && shadow.camera) {
      shadow.camera.near = Math.max(shadowNear, 0.01);
      shadow.camera.updateProjectionMatrix?.();
    }

    if (Number.isFinite(shadowFar) && shadow.camera) {
      shadow.camera.far = Math.max(shadowFar, shadow.camera.near + 1);
      shadow.camera.updateProjectionMatrix?.();
    }
  }

  disposeShadowNode({ preserveFailure = false } = {}) {
    const hadShadowNode = Boolean(this.shadowNode);

    if (this.shadowNode) {
      try {
        this.shadowNode.dispose();
      } catch (error) {
        console.warn('CascadedShadowMaps: failed to dispose shadow node', error);
      }
      if (this.light?.shadow) {
        this.light.shadow.shadowNode = null;
      }
      this.shadowNode = null;
    }
    this._shadowNodeValidated = false;
    if (!preserveFailure) {
      this._shadowNodeFailed = false;
    }

    if (hadShadowNode) {
      this._emitStateChange(false);
    }
  }

  dispose() {
    this.disposeShadowNode();
    this.camera = null;
    this._projectionMatrix = null;
  }

  setCamera(camera) {
    if (!camera?.isCamera) {
      return;
    }

    if (this.camera !== camera) {
      this.camera = camera;
      this._projectionMatrix = null;
      this._cameraParams = { near: null, far: null, zoom: null };
      this._shadowNodeValidated = false;
    }

    this.ensureShadowNode();
    if (this.shadowNode) {
      this.shadowNode.camera = this.camera;
    }

    this.requestFrustumUpdate();
  }

  ensureShadowNode() {
    if (!this.light?.shadow || this._shadowNodeFailed) {
      return null;
    }

    if (!this.shadowNode) {
      this.shadowNode = new CSMShadowNode(this.light, {
        cascades: this.options.cascades,
        maxFar: this.options.maxFar,
        mode: this.options.mode,
        lightMargin: this.options.lightMargin,
      });
      // Delay enabling fading until the shadow node has finished initialising
      // its internal frustums. The `fade` setter internally accesses
      // `_shadowNodes`, which is undefined until the first shadow pass is
      // executed. Calling it too early causes `TypeError: can't access property
      // "oneMinus", this._shadowNodes[i] is undefined` when running with the
      // WebGPU renderer. Keep track of the desired fade value and apply it once
      // validation succeeds instead of touching the setter immediately here.
      this.shadowNode.fade = false;
      if (this.camera?.isCamera) {
        this.shadowNode.camera = this.camera;
      }
      const lightShadow = this.light?.shadow;
      if (lightShadow && lightShadow.shadowNode !== this.shadowNode) {
        lightShadow.shadowNode = this.shadowNode;
        this._emitStateChange(true);
      }
      this._needsFrustumUpdate = true;
      this._shadowNodeValidated = false;
    }

    return this.shadowNode;
  }

  disableShadowNode() {
    if (this._shadowNodeFailed) {
      return;
    }

    console.warn('CascadedShadowMaps: shadow node initialisation failed, reverting to standard directional shadows.');
    this._shadowNodeFailed = true;
    this.disposeShadowNode({ preserveFailure: true });
  }

  validateShadowNode() {
    if (!this.shadowNode) {
      return false;
    }

    const cascades = Number.isInteger(this.shadowNode.cascades) ? this.shadowNode.cascades : 0;
    const nodes = this.shadowNode._shadowNodes;

    if (!Array.isArray(nodes) || nodes.length !== cascades) {
      this.disableShadowNode();
      return false;
    }

    const hasInvalidNode = nodes.some((node) => !node || typeof node.oneMinus !== 'function');
    if (hasInvalidNode) {
      this.disableShadowNode();
      return false;
    }

    return true;
  }

  applyDesiredFade() {
    if (!this.shadowNode || !this._shadowNodeValidated) {
      return;
    }

    const fade = Boolean(this._desiredFade);
    if (this.shadowNode.fade !== fade) {
      try {
        this.shadowNode.fade = fade;
      } catch (error) {
        console.warn('CascadedShadowMaps: failed to apply fade setting', error);
        this.shadowNode.fade = false;
      }
    }
  }

  setOptions(options = {}) {
    if (!options || typeof options !== 'object') {
      return;
    }

    const next = { ...this.options, ...options };
    const cascadesChanged = next.cascades !== this.options.cascades;

    this.options = next;
    this._shadowNodeFailed = false;
    this._shadowNodeValidated = false;
    this._desiredFade = Boolean(next.fade);

    if (cascadesChanged && isPositiveNumber(next.cascades)) {
      const camera = this.camera;
      this.disposeShadowNode();
      this.shadowNode = null;
      this.ensureShadowNode();
      if (this.shadowNode && camera) {
        this.shadowNode.camera = camera;
      }
    }

    if (this.shadowNode) {
      if (Number.isFinite(next.maxFar)) {
        this.shadowNode.maxFar = next.maxFar;
      }
      if (next.mode) {
        this.shadowNode.mode = next.mode;
      }
      if (Number.isFinite(next.lightMargin)) {
        this.shadowNode.lightMargin = next.lightMargin;
      }
      this.applyDesiredFade();
    }

    this.configureLight();
    this.requestFrustumUpdate();
  }

  setShadowMapSize(size) {
    if (!isPositiveNumber(size)) {
      return;
    }
    this.options.shadowMapSize = size;
    if (this.light?.shadow) {
      this.light.shadow.mapSize.set(size, size);
    }
    this.requestFrustumUpdate();
  }

  setShadowBias(value) {
    this.options.shadowBias = value;
    if (this.light?.shadow && Number.isFinite(value)) {
      this.light.shadow.bias = value;
    }
  }

  setShadowNormalBias(value) {
    this.options.shadowNormalBias = value;
    if (this.light?.shadow && Number.isFinite(value) && 'normalBias' in this.light.shadow) {
      this.light.shadow.normalBias = value;
    }
  }

  setMaxFar(value) {
    if (!Number.isFinite(value) || value <= 0) {
      return;
    }
    this.options.maxFar = value;
    if (this.shadowNode) {
      this.shadowNode.maxFar = value;
    }
    this.requestFrustumUpdate();
  }

  notifyLightChanged() {
    this.requestFrustumUpdate();
  }

  requestFrustumUpdate() {
    this._needsFrustumUpdate = true;
  }

  update(camera = null) {
    if (camera && camera !== this.camera) {
      this.setCamera(camera);
    }

    if (!this.camera || !this.ensureShadowNode()) {
      return;
    }

    const cam = this.camera;
    cam.updateProjectionMatrix?.();

    if (!this._projectionMatrix) {
      this._projectionMatrix = new THREE.Matrix4().copy(cam.projectionMatrix);
      this._cameraParams = {
        near: cam.near,
        far: cam.far,
        zoom: cam.zoom,
      };
      this._needsFrustumUpdate = true;
    } else {
      const projectionChanged = !this._projectionMatrix.equals(cam.projectionMatrix);
      const nearChanged = this._cameraParams.near !== cam.near;
      const farChanged = this._cameraParams.far !== cam.far;
      const zoomChanged = this._cameraParams.zoom !== cam.zoom;
      if (projectionChanged || nearChanged || farChanged || zoomChanged) {
        this._needsFrustumUpdate = true;
      }
    }

    if (this.shadowNode?.mainFrustum && !this._shadowNodeValidated) {
      if (!this.validateShadowNode()) {
        return;
      }
      this._shadowNodeValidated = true;
      this.applyDesiredFade();
    }

    if (this._needsFrustumUpdate) {
      if (!this.shadowNode?.mainFrustum) {
        // `CSMShadowNode` lazily initialises its internal frustums during the
        // renderer's shadow pass. When running with the WebGPU renderer the
        // shadow node might not be ready the very first time we try to update
        // it, which previously resulted in accessing `mainFrustum` while it was
        // still `null`. Wait for the shadow node to finish initialising before
        // triggering an update so we can safely compute the cascades once
        // `mainFrustum` exists.
        return;
      }

      this.shadowNode.updateFrustums();
      this._projectionMatrix.copy(cam.projectionMatrix);
      this._cameraParams = {
        near: cam.near,
        far: cam.far,
        zoom: cam.zoom,
      };
      this._needsFrustumUpdate = false;
    }
  }
}

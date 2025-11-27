const DEFAULT_SLIDER_MIN = 0;
const DEFAULT_SLIDER_MAX = 100;

function formatOutputValue(value) {
  if (value === null) return 'Null';
  if (typeof value !== 'object') return String(value);

  if ('Text' in value) return `Text: "${value.Text}"`;
  if ('Number' in value) return `Number: ${value.Number}`;
  if ('Point' in value) return `Point: (${value.Point.join(', ')})`;
  if ('List' in value) return `List (${value.List.length} items)`;

  return JSON.stringify(value);
}

function formatValue(value, step) {
  if (!Number.isFinite(value)) {
    return '';
  }
  const precision = step && !Number.isInteger(step) ? Math.min(6, Math.ceil(-Math.log10(step))) : 0;
  return Number.parseFloat(value).toFixed(Math.max(0, precision));
}

function createSliderElement(slider, handlers) {
  const wrapper = document.createElement('div');
  wrapper.className = 'slider';
  wrapper.dataset.sliderId = slider.id;

  const labelRow = document.createElement('div');
  labelRow.className = 'slider-label';

  const nameSpan = document.createElement('span');
  nameSpan.textContent = slider.name ?? slider.id ?? 'Slider';
  const valueSpan = document.createElement('span');
  labelRow.append(nameSpan, valueSpan);

  const inputsRow = document.createElement('div');
  inputsRow.className = 'slider-inputs';

  const rangeInput = document.createElement('input');
  rangeInput.type = 'range';

  const numberInput = document.createElement('input');
  numberInput.type = 'number';

  const hasMin = Number.isFinite(slider.min);
  const hasMax = Number.isFinite(slider.max);
  const hasStep = Number.isFinite(slider.step) && slider.step > 0;

  const lowerBound = hasMin ? slider.min : DEFAULT_SLIDER_MIN;
  let upperBound = hasMax ? slider.max : DEFAULT_SLIDER_MAX;
  if (upperBound < lowerBound) {
    upperBound = lowerBound;
  }

  const numericStep = hasStep ? slider.step : Math.max((upperBound - lowerBound) / 100, 0.0001);
  const formatStep = hasStep ? Math.abs(slider.step) : undefined;

  rangeInput.min = String(lowerBound);
  rangeInput.max = String(upperBound);
  rangeInput.step = String(numericStep);
  rangeInput.setAttribute('aria-label', `${nameSpan.textContent} (range)`);

  if (hasMin) {
    numberInput.min = String(lowerBound);
  }
  if (hasMax) {
    numberInput.max = String(upperBound);
  }
  numberInput.step = hasStep ? String(slider.step) : 'any';
  numberInput.setAttribute('aria-label', `${nameSpan.textContent} (nummer)`);

  const clamp = (value) => {
    let result = Number.isFinite(value) ? value : lowerBound;
    if (hasMin && result < lowerBound) {
      result = lowerBound;
    }
    if (hasMax && result > upperBound) {
      result = upperBound;
    }
    return result;
  };

  let committedValue = clamp(Number(slider.value));

  const applyValue = (value, { updateRange = true, updateNumber = true } = {}) => {
    committedValue = clamp(value);
    valueSpan.textContent = formatValue(committedValue, formatStep);
    if (updateRange) {
      rangeInput.value = String(committedValue);
    }
    if (updateNumber) {
      numberInput.value = String(committedValue);
    }
  };

  applyValue(committedValue, { updateRange: true, updateNumber: true });

  const emitChange = (value) => {
    if (typeof handlers.onSliderChange === 'function') {
      handlers.onSliderChange(slider.id, clamp(value));
    }
  };

  rangeInput.addEventListener('input', (event) => {
    const numeric = Number(event.target.value);
    if (Number.isNaN(numeric)) {
      return;
    }
    applyValue(numeric, { updateRange: false });
  });

  const commitRangeValue = (rawValue) => {
    const numeric = Number(rawValue);
    if (Number.isNaN(numeric)) {
      applyValue(committedValue);
      return;
    }
    applyValue(numeric);
    emitChange(numeric);
  };

  rangeInput.addEventListener('change', (event) => {
    commitRangeValue(event.target.value);
  });

  const commitNumberValue = (rawValue) => {
    const numeric = Number(rawValue);
    if (Number.isNaN(numeric)) {
      applyValue(committedValue);
      return;
    }
    applyValue(numeric);
    emitChange(numeric);
  };

  numberInput.addEventListener('change', (event) => {
    commitNumberValue(event.target.value);
  });

  numberInput.addEventListener('blur', (event) => {
    commitNumberValue(event.target.value);
  });

  numberInput.addEventListener('keydown', (event) => {
    if (event.key === 'Enter') {
      commitNumberValue(event.target.value);
    }
  });

  numberInput.addEventListener('input', (event) => {
    const numeric = Number(event.target.value);
    if (Number.isNaN(numeric)) {
      valueSpan.textContent = '';
      return;
    }
    valueSpan.textContent = formatValue(numeric, formatStep);
  });

  inputsRow.append(rangeInput, numberInput);
  wrapper.append(labelRow, inputsRow);

  return {
    element: wrapper,
    controller: {
      update(value) {
        applyValue(value);
      },
    },
  };
}

function createToggleElement(toggle, handlers) {
    const wrapper = document.createElement('div');
    wrapper.className = 'slider toggle-control'; // Use slider class for layout consistency
    wrapper.dataset.sliderId = toggle.id;

    const labelRow = document.createElement('div');
    labelRow.className = 'slider-label';

    const nameSpan = document.createElement('span');
    nameSpan.textContent = toggle.name ?? toggle.id ?? 'Toggle';
    const valueSpan = document.createElement('span');
    labelRow.append(nameSpan, valueSpan);

    const inputsRow = document.createElement('div');
    inputsRow.className = 'slider-inputs'; // Re-use slider layout

    const checkbox = document.createElement('input');
    checkbox.type = 'checkbox';
    checkbox.checked = Boolean(toggle.value);
    checkbox.setAttribute('aria-label', nameSpan.textContent);

    const updateDisplay = (val) => {
      valueSpan.textContent = val ? 'True' : 'False';
      checkbox.checked = val;
    };

    updateDisplay(toggle.value);

    checkbox.addEventListener('change', (event) => {
      const isChecked = event.target.checked;
      updateDisplay(isChecked);
      if (typeof handlers.onSliderChange === 'function') {
        handlers.onSliderChange(toggle.id, isChecked);
      }
    });

    inputsRow.append(checkbox);
    wrapper.append(labelRow, inputsRow);

    return {
      element: wrapper,
      controller: {
        update(value) {
          updateDisplay(Boolean(value));
        },
      },
    };
  }

function createValueListElement(list, handlers) {
  const wrapper = document.createElement('div');
  wrapper.className = 'slider value-list';
  wrapper.dataset.sliderId = list.id;

  const labelRow = document.createElement('div');
  labelRow.className = 'slider-label';

  const nameSpan = document.createElement('span');
  nameSpan.textContent = list.name ?? list.id ?? 'Value List';
  const valueSpan = document.createElement('span');
  labelRow.append(nameSpan, valueSpan);

  const inputsRow = document.createElement('div');
  inputsRow.className = 'slider-inputs value-list-inputs';

  const select = document.createElement('select');
  select.setAttribute('aria-label', nameSpan.textContent);

  const rawItems = Array.isArray(list.items) ? list.items : [];
  const options = rawItems.map((item) => {
    if (item && typeof item.label === 'string') {
      return item.label;
    }
    if (typeof item === 'string') {
      return item;
    }
    return '';
  });

  if (!options.length) {
    const placeholder = document.createElement('option');
    placeholder.value = '';
    placeholder.textContent = 'Geen opties';
    select.append(placeholder);
    select.disabled = true;
  } else {
    options.forEach((label, index) => {
      const optionElement = document.createElement('option');
      optionElement.value = String(index);
      optionElement.textContent = label;
      select.append(optionElement);
    });
  }

  const clampIndex = (raw) => {
    if (!options.length) {
      return 0;
    }
    let index = Number(raw);
    if (!Number.isFinite(index)) {
      index = 0;
    }
    index = Math.trunc(index);
    if (index < 0) {
      index = 0;
    }
    if (index >= options.length) {
      index = options.length - 1;
    }
    return index;
  };

  const updateDisplay = (value) => {
    const index = clampIndex(value);
    if (options.length) {
      select.value = String(index);
      valueSpan.textContent = options[index];
    } else {
      select.value = '';
      valueSpan.textContent = 'Geen opties';
    }
    return index;
  };

  const initialIndex = Number.isFinite(list.selectedIndex)
    ? list.selectedIndex
    : Number.isFinite(list.value)
      ? list.value
      : 0;
  updateDisplay(initialIndex);

  select.addEventListener('change', (event) => {
    if (!options.length) {
      return;
    }
    const selected = Number(event.target.value);
    const index = updateDisplay(selected);
    if (typeof handlers.onSliderChange === 'function') {
      handlers.onSliderChange(list.id, index);
    }
  });

  inputsRow.append(select);
  wrapper.append(labelRow, inputsRow);

  return {
    element: wrapper,
    controller: {
      update(value) {
        updateDisplay(value);
      },
    },
  };
}

export function setupUi() {
  const canvas = document.getElementById('viewport');
  const fileInput = document.getElementById('ghx-input');
  const statusOutput = document.getElementById('status');
  const topologyMapOutput = document.getElementById('topology-map');
  const nodeListContainer = document.getElementById('node-list');
  const sliderContainer = document.getElementById('slider-container');
  const overlayToggle = document.getElementById('overlay-toggle');
  const overlayState = document.getElementById('overlay-state');
  const loadingOverlay = document.getElementById('loading-overlay');

  const sliderElements = new Map();
  const handlers = {
    onFileSelected: null,
    onSliderChange: null,
    onOverlayToggle: null,
  };

  const setStatus = (text) => {
    if (statusOutput) {
      statusOutput.textContent = text;
    }
  };

  const showLoading = (active) => {
    if (!loadingOverlay) {
      return;
    }
    if (active) {
      loadingOverlay.classList.remove('loading-overlay--hidden');
      loadingOverlay.setAttribute('aria-hidden', 'false');
    } else {
      loadingOverlay.classList.add('loading-overlay--hidden');
      loadingOverlay.setAttribute('aria-hidden', 'true');
    }
  };

  const setOverlayState = (enabled) => {
    const isEnabled = Boolean(enabled);
    if (overlayToggle) {
      overlayToggle.checked = isEnabled;
      overlayToggle.setAttribute('aria-checked', isEnabled ? 'true' : 'false');
    }
    if (overlayState) {
      overlayState.textContent = isEnabled ? 'true' : 'false';
    }
  };

  const renderSliders = (sliders = []) => {
    sliderElements.clear();

    if (!sliderContainer) {
      return;
    }

    sliderContainer.innerHTML = '';

    if (!sliders.length) {
      const empty = document.createElement('p');
      empty.textContent = 'Geen sliders beschikbaar voor dit model.';
      empty.style.opacity = '0.65';
      empty.style.fontSize = '0.9rem';
      sliderContainer.appendChild(empty);
      return;
    }

    for (const control of sliders) {
      if (!control?.id) {
        continue;
      }
      let result;
      if (control.type === 'toggle') {
        result = createToggleElement(control, handlers);
      } else if (control.type === 'value-list') {
        result = createValueListElement(control, handlers);
      } else {
        // Default to slider
        result = createSliderElement(control, handlers);
      }
      const { element, controller } = result;
      sliderContainer.appendChild(element);
      sliderElements.set(String(control.id), controller);
    }
  };

  const updateSliderValue = (sliderId, value) => {
    const controller = sliderElements.get(String(sliderId));
    if (!controller) {
      return false;
    }
    controller.update(value);
    return true;
  };

  const setHandlers = (newHandlers = {}) => {
    handlers.onFileSelected = typeof newHandlers.onFileSelected === 'function' ? newHandlers.onFileSelected : null;
    handlers.onSliderChange = typeof newHandlers.onSliderChange === 'function' ? newHandlers.onSliderChange : null;
    handlers.onOverlayToggle = typeof newHandlers.onOverlayToggle === 'function' ? newHandlers.onOverlayToggle : null;
  };

  if (fileInput) {
    fileInput.addEventListener('change', () => {
      const file = fileInput.files && fileInput.files[0] ? fileInput.files[0] : null;
      if (typeof handlers.onFileSelected === 'function') {
        handlers.onFileSelected(file);
      }
      // Laat dezelfde bestandsselectie opnieuw toe
      fileInput.value = '';
    });
  }

  if (overlayToggle) {
    overlayToggle.addEventListener('change', () => {
      const checked = overlayToggle.checked;
      setOverlayState(checked);
      if (typeof handlers.onOverlayToggle === 'function') {
        handlers.onOverlayToggle(checked);
      }
    });
  }

  return {
    canvas,
    setHandlers,
    renderSliders,
    updateSliderValue,
    setStatus,
    showLoading,
    setOverlayState,
  };
}

//# sourceMappingURL=ui.js.map

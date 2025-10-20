function formatValue(value, step) {
  const precision = step && !Number.isInteger(step) ? Math.min(6, Math.ceil(-Math.log10(step))) : 0;
  return Number.parseFloat(value ?? 0).toFixed(precision);
}

export function createSliderUI({ container, engine }) {
  if (!container) {
    throw new Error('Slider container ontbreekt');
  }

  function render() {
    const sliders = engine.listSliders();
    container.innerHTML = '';

    if (!sliders.length) {
      const empty = document.createElement('p');
      empty.textContent = 'Geen sliders beschikbaar voor deze graph.';
      empty.style.opacity = '0.65';
      empty.style.fontSize = '0.9rem';
      container.appendChild(empty);
      return;
    }

    for (const slider of sliders) {
      const wrapper = document.createElement('div');
      wrapper.className = 'slider';
      wrapper.dataset.nodeId = slider.id;

      const label = document.createElement('div');
      label.className = 'slider-label';
      label.innerHTML = `<span>${slider.label}</span><span>${formatValue(slider.value, slider.step)}</span>`;

      const inputs = document.createElement('div');
      inputs.className = 'slider-inputs';

      const range = document.createElement('input');
      range.type = 'range';
      range.min = slider.min;
      range.max = slider.max;
      range.step = slider.step;
      range.value = slider.value;
      range.setAttribute('aria-label', `${slider.label} (range)`);

      const number = document.createElement('input');
      number.type = 'number';
      number.min = slider.min;
      number.max = slider.max;
      number.step = slider.step;
      number.value = slider.value;
      number.setAttribute('aria-label', `${slider.label} (nummer)`);

      const syncInputs = (value) => {
        label.innerHTML = `<span>${slider.label}</span><span>${formatValue(value, slider.step)}</span>`;
        range.value = value;
        number.value = value;
      };

      const handleInput = (event) => {
        const rawValue = event.target.value;
        const numeric = rawValue === '' ? slider.min : Number(rawValue);
        if (Number.isNaN(numeric)) {
          return;
        }
        syncInputs(numeric);
        engine.setSliderValue(slider.id, numeric);
      };

      range.addEventListener('input', handleInput);
      number.addEventListener('input', handleInput);

      inputs.appendChild(range);
      inputs.appendChild(number);
      wrapper.appendChild(label);
      wrapper.appendChild(inputs);
      container.appendChild(wrapper);
    }
  }

  return { render };
}

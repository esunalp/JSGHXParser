// Eenvoudige UI bootstrap; later uit te breiden met slider-controls.

export function createUi() {
  const container = document.createElement('section');
  container.id = 'ui';
  container.innerHTML = `
    <p>UI wordt later ingevuld. Gebruik deze sectie voor sliders en file inputs.</p>
  `;
  document.body.appendChild(container);
}

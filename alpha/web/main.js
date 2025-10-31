import { bootstrap } from './three_integration.js';
import { createUi } from './ui.js';

async function init() {
  createUi();
  await bootstrap();
}

init().catch((err) => {
  console.error('Kon de demo niet initialiseren:', err);
});

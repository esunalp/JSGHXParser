export async function parseGHX(file) {
  if (!file) {
    throw new Error('Geen bestand aangeleverd.');
  }
  const text = await file.text();
  const parser = new DOMParser();
  const doc = parser.parseFromString(text, 'application/xml');
  const parseError = doc.querySelector('parsererror');
  if (parseError) {
    throw new Error('Kon GHX-bestand niet parsen. Controleer of het valide XML is.');
  }

  // Placeholder: deze module wordt in latere stappen gevuld met echte selectors
  console.warn('parseGHX: Parser skeleton is nog niet geïmplementeerd.');
  return { nodes: [], wires: [] };
}

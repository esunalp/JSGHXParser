export const ASSET_VERSION = '74';

export function withVersion(path) {
  if (!path || typeof path !== 'string') {
    return path;
  }
  if (!ASSET_VERSION) {
    return path;
  }
  const separator = path.includes('?') ? '&' : '?';
  return `${path}${separator}version=${encodeURIComponent(ASSET_VERSION)}`;
}

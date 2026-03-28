import BRANDING from "../branding.generated";
import brandLogoSrc from "../assets/branding/current/app-logo.png";

export { BRANDING, brandLogoSrc };

export function storageKey(name: string): string {
  return `${BRANDING.localStoragePrefix}:${name}`;
}

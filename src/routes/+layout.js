// This disables SSR for the entire app
export const ssr = false;

// This tells SvelteKit to treat this as a SPA
export const csr = true;

// This enables prerendering for the entire app
export const prerender = true;

// This handles errors
export function load() {
  return {
    status: 200
  };
}

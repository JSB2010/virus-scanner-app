// This ensures the page is loaded client-side
export const csr = true;
export const ssr = false;
export const prerender = true;

// This ensures the page loads correctly
export function load() {
  return {
    status: 200,
    props: {
      ready: true
    }
  };
}

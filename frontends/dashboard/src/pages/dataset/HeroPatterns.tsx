export const HeroPatterns: Record<
  string,
  (color: string, opacity: number) => string
> = {
  Solid: (foregroundColor: string, foregroundOpacity: number) => {
    const encoded = btoa(
      `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1 1" width="1" height="1">
        <rect width="1" height="1" fill="${foregroundColor}" fill-opacity="${foregroundOpacity}"/>
      </svg>`,
    );
    return `data:image/svg+xml;base64,${encoded}`;
  },
  Texture: (foregroundColor: string, foregroundOpacity: number) => {
    const encoded = btoa(
      `<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 4 4' width='4' height='4'>
        <path fill='${foregroundColor}' fill-opacity='${foregroundOpacity}' d='M1 3h1v1H1V3zm2-2h1v1H3V1z'></path>
      </svg>`,
    );
    return `data:image/svg+xml;base64,${encoded}`;
  },
  Circles: (foregroundColor: string, foregroundOpacity: number) => {
    const encoded = btoa(
      `<svg width="80" height="80" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg"><g fill="${foregroundColor}" fill-opacity="${foregroundOpacity}" fill-rule="evenodd"><g fill="${foregroundColor}"><path d="M50 50c0-5.523 4.477-10 10-10s10 4.477 10 10-4.477 10-10 10c0 5.523-4.477 10-10 10s-10-4.477-10-10 4.477-10 10-10zM10 10c0-5.523 4.477-10 10-10s10 4.477 10 10-4.477 10-10 10c0 5.523-4.477 10-10 10S0 25.523 0 20s4.477-10 10-10zm10 8c4.418 0 8-3.582 8-8s-3.582-8-8-8-8 3.582-8 8 3.582 8 8 8zm40 40c4.418 0 8-3.582 8-8s-3.582-8-8-8-8 3.582-8 8 3.582 8 8 8z" /></g></g></svg>`,
    );
    return `data:image/svg+xml;base64,${encoded}`;
  },
  Wiggle: (foregroundColor: string, foregroundOpacity: number) => {
    const encoded = btoa(
      `<svg width="52" height="26" viewBox="0 0 52 26" xmlns="http://www.w3.org/2000/svg"><g fill="${foregroundColor}" fill-opacity="${foregroundOpacity}" fill-rule="evenodd"><g fill="${foregroundColor}"><path d="M10 10c0-2.21-1.79-4-4-4-3.314 0-6-2.686-6-6h2c0 2.21 1.79 4 4 4 3.314 0 6 2.686 6 6 0 2.21 1.79 4 4 4 3.314 0 6 2.686 6 6 0 2.21 1.79 4 4 4v2c-3.314 0-6-2.686-6-6 0-2.21-1.79-4-4-4-3.314 0-6-2.686-6-6zm25.464-1.95l8.486 8.486-1.414 1.414-8.486-8.486 1.414-1.414z" /></g></g></svg>`,
    );
    return `data:image/svg+xml;base64,${encoded}`;
  },
};

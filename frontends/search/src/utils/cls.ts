export function cls(...classes: (string | boolean | undefined)[]): string {
  return classes.filter(Boolean).join(" ");
}

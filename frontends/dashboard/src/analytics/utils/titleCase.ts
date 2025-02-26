// https://stackoverflow.com/a/196991
export function toTitleCase(str: string) {
  return str
    .replace(
      /\w\S*/g,
      (text) => text.charAt(0).toUpperCase() + text.substring(1).toLowerCase(),
    )
    .replace("_", " ");
}

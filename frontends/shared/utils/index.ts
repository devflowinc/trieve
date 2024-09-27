import clsx, { type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...args: ClassValue[]) {
  return twMerge(clsx(...args));
}

export function jsonToCSV(object: any[]) {
  const header = Object.keys(object[0]);
  const csv = [
    header.join(","),
    ...object.map((row) =>
      header
        .map((fieldName) => {
          if (!row[fieldName]) return null;
          if (typeof row[fieldName] === "object")
            return `"${JSON.stringify(row[fieldName]).replaceAll(`"`, `""`)}"`;
          return row[fieldName];
        })
        .join(",")
    ),
  ].join("\r\n");

  return csv;
}

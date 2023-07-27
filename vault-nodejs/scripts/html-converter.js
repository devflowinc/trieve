import { convert } from "html-to-text";

export function convertToText(html) {
  console.log(convert(html));
}

if (process.argv[2] == "-html") {
  convertToText(process.argv[3]);
}

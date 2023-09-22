import { parse } from "node-html-parser";

function convertToText(html) {
  const root = parse("<body>" + html + "</body>");
  console.log(root.text);
}

function getInnerHtml(html) {
  const root = parse(html);
  let body = root.getElementsByTagName("body")[0];
  if (!body) {
    console.error("No body tag found.");
    process.exit(1);
  }
  console.log(body.text);
}

if (process.argv[2] == "-html") {
  convertToText(process.argv[3]);
} else if (process.argv[2] == "-get-inner-html") {
  getInnerHtml(process.argv[3]);
} else {
  console.error("Invalid arguments.");
  process.exit(1);
}

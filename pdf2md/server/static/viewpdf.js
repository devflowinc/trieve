let { pdfjsViewer } = globalThis;
//
pdfjsLib.GlobalWorkerOptions.workerSrc =
  "https://cdnjs.cloudflare.com/ajax/libs/pdf.js/4.8.69/pdf.worker.mjs";

const loadingTask = pdfjsLib.getDocument(window.pdf_url);
const pdf = await loadingTask.promise;
//
// Fetch the first page
//
const page = await pdf.getPage(1);
const scale = 1;
const viewport = page.getViewport({ scale });
// Support HiDPI-screens.
const outputScale = window.devicePixelRatio || 1;

const canvas = document.getElementById("the-canvas");
const context = canvas.getContext("2d");

canvas.width = Math.floor(viewport.width * outputScale);
canvas.height = Math.floor(viewport.height * outputScale);
canvas.style.width = Math.floor(viewport.width) + "px";
canvas.style.height = Math.floor(viewport.height) + "px";

const transform =
  outputScale !== 1 ? [outputScale, 0, 0, outputScale, 0, 0] : null;

const renderContext = {
  canvasContext: context,
  transform,
  viewport,
};

page.render(renderContext);

const container = document.getElementById("viewerContainer");

const pdfViewer = new pdfjsViewer.PDFViewer({
  container,
  eventBus,
});

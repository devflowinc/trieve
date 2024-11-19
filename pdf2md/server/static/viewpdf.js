pdfjsLib.GlobalWorkerOptions.workerSrc =
  "https://cdnjs.cloudflare.com/ajax/libs/pdf.js/4.8.69/pdf.worker.mjs";

let currentPage = 1;
let totalPages;
let pdf;

async function loadPDF(pdfUrl) {
  const loadingTask = pdfjsLib.getDocument(pdfUrl);
  pdf = await loadingTask.promise;
  totalPages = pdf.numPages;
  renderPage(currentPage);
}

async function renderPage(pageNumber) {
  // Fetch the page
  const page = await pdf.getPage(pageNumber);
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

  await page.render(renderContext).promise;
}

function previousPage() {
  if (currentPage <= 1) return;
  currentPage--;
  renderPage(currentPage);
}

function nextPage() {
  if (currentPage >= totalPages) return;
  currentPage++;
  renderPage(currentPage);
}

// Add keyboard event listener
document.addEventListener("keydown", (e) => {
  if (e.key === "ArrowLeft") {
    console.log("left");
    previousPage();
  } else if (e.key === "ArrowRight") {
    console.log("right");
    nextPage();
  }
});

document.addEventListener("open-pdf", (e) => {
  loadPDF(e.detail.pdfUrl);
});

const url = new URL(window.location);
const taskId = url.searchParams.get("taskId");
if (taskId) {
  fetch(`/api/task/${taskId}`, {
    headers: {
      Authorization: window.TRIEVE_API_KEY,
    },
  })
    .then((response) => response.json())
    .then((data) => {
      upsertTaskToStorage(data);
    })
    .catch((error) => {
      console.error("Error:", error);
    });
}

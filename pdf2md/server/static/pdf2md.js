const defaultTableRowStr = `
    <td
      class="task-id whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium text-gray-900 sm:pl-0"
    >
      Lindsay Walton
    </td>
    <td
      class="task-file-name whitespace-nowrap px-3 py-4 text-sm text-gray-500"
    >
      Front-end Developer
    </td>
    <td
      class="task-status whitespace-nowrap px-3 py-4 text-sm text-gray-500"
    >
      lindsay.walton@example.com
    </td>
    <td
      class="relative whitespace-nowrap py-4 pl-3 pr-4 text-right text-sm font-medium sm:pr-0"
    >
      <button
        href="#"
        class="task-view-button text-magenta-600 hover:text-magenta-900"
      >
        View
      </button>
    </td>
`;
const defaultTableRow = document.createElement("tr");
defaultTableRow.innerHTML = defaultTableRowStr;

var notyf = new Notyf();

const upsertTaskToStorage = (task) => {
  let tasks = JSON.parse(localStorage.getItem("tasks")) || [];
  if (tasks.find((t) => t.id === task.id)) {
    tasks = tasks.map((t) => (t.id === task.id ? task : t));
  } else {
    tasks.unshift(task);
  }
  localStorage.setItem("tasks", JSON.stringify(tasks));

  updateTaskStatusTable();
};

const displayTask = (task) => {
  const markdownContainer = document.getElementById("markdown-container");
  const taskId = markdownContainer.getAttribute("data-task-id");
  const taskStatus = markdownContainer.getAttribute("data-task-status");
  const taskNumPages = markdownContainer.getAttribute("data-task-num-pages");
  if (
    taskId === task.id &&
    taskStatus === task.status &&
    taskNumPages === task.num_pages.toString()
  ) {
    console.log("Task already displayed", task.id);
    return;
  }

  PDFObject.embed(task.file_url, "#my-pdf", {
    pdfOpenParams: {
      view: "FitH",
    },
  });
  const pages = task.pages;
  const sortedPages = pages.sort((a, b) => a.metadata.page - b.metadata.page);

  while (markdownContainer.firstChild) {
    markdownContainer.removeChild(markdownContainer.firstChild);
  }

  markdownContainer.setAttribute("data-task-id", task.id);
  markdownContainer.setAttribute("data-task-status", task.status);
  markdownContainer.setAttribute("data-task-num-pages", task.num_pages);

  sortedPages.forEach((page) => {
    const pageContainer = document.createElement("div");
    pageContainer.classList.add("page-container");
    pageContainer.innerText = page.content;
    markdownContainer.appendChild(pageContainer);
    const spacerDiv = document.createElement("div");
    spacerDiv.classList.add(...["my-4", "h-1", "bg-gray-700"]);
    markdownContainer.appendChild(spacerDiv);
  });
  if (!sortedPages.length) {
    const pageContainer = document.createElement("div");
    pageContainer.classList.add(...["page-container", "animate-pulse", "pt-4"]);
    pageContainer.innerText =
      "Your file is being converted. We are pinging the server every 5 seconds to check for status updates. Please be patient!";
    markdownContainer.appendChild(pageContainer);
  }
};

const fileUploadInput = document.getElementById("file-upload");

fileUploadInput.addEventListener("change", (event) => {
  const file = event.target.files[0];
  if (!file) {
    console.error("No file selected");
    return;
  }

  const reader = new FileReader();
  reader.onload = (event) => {
    const file_name = file.name;
    const base64_file = event.target.result.split(",")[1];

    const formData = {
      file_name,
      base64_file,
    };

    fetch("/api/task", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: window.TRIEVE_API_KEY,
      },
      body: JSON.stringify(formData),
    })
      .then((response) => response.json())
      .then((data) => {
        notyf.success({
          message:
            "File uploaded! We are processing the file. Please wait. Scroll down to the table to view the status.",
          dismissable: true,
          position: { x: "center", y: "top" },
        });

        upsertTaskToStorage(data);
        const url = new URL(window.location);
        url.searchParams.set("taskId", data.id);
        window.history.pushState({}, "", url);
      })
      .catch((error) => {
        notyf.error({
          message: `Error uploading file. Please try again later. ${error}`,
          dismissable: true,
          position: { x: "center", y: "top" },
        });
      });
  };

  reader.readAsDataURL(file);
});

const updateTaskStatusTable = () => {
  const tableContainer = document.getElementById("task-status-table-container");
  const tasks = JSON.parse(localStorage.getItem("tasks")) || [];
  const tbody = tableContainer.querySelector("tbody");
  const firstRow = tbody.querySelector("tr");
  tbody.innerHTML = "";
  const htmlRows = tasks.map((task) => {
    const row = firstRow
      ? firstRow.cloneNode(true)
      : defaultTableRow.cloneNode(true);
    row.querySelector(".task-id").innerText = task.id;
    row.querySelector(".task-file-name").innerText = task.file_name;
    row.querySelector(".task-status").innerText =
      task.status.toLowerCase() === "completed"
        ? task.status
        : `${task.status} | Please wait. Checking for updates every 5 seconds.`;
    row
      .querySelector(".task-status")
      .classList.add(
        `status-${task.status.split(" ").join("-").toLowerCase()}`
      );
    row.querySelector("button").addEventListener("click", () => {
      const url = new URL(window.location);
      url.searchParams.set("taskId", task.id);
      window.history.pushState({}, "", url);

      displayTask(task);
    });
    return row;
  });
  htmlRows.forEach((row) => tbody.appendChild(row));

  if (htmlRows.length) {
    tableContainer.classList.remove("hidden");
    tableContainer.classList.add("flow-root");
    const formContainer = document.getElementById("upload-form-container");
    formContainer.classList.remove("h-[75vh]");
    formContainer.classList.add(...["mt-10", "sm:mt-14", "md:mt-24"]);
  }
};

updateTaskStatusTable();

const refreshTasks = () => {
  const tasks = JSON.parse(localStorage.getItem("tasks")) || [];
  tasks.forEach((task) => {
    if (
      task.status.toLowerCase() === "completed" &&
      task.pages &&
      task.pages.length
    ) {
      return;
    }

    fetch(`/api/task/${task.id}`, {
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
  });

  const url = new URL(window.location);
  const taskId = url.searchParams.get("taskId");
  tasks.forEach((task) => {
    if (task.id === taskId) {
      displayTask(task);
    }
  });
};

setInterval(refreshTasks, 5000);

const setActiveTaskFromUrl = () => {
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
        displayTask(data);
      })
      .catch((error) => {
        console.error("Error:", error);
      });
  }
};

setActiveTaskFromUrl();

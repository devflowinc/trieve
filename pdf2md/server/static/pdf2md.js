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

const upsertTaskToStorage = (task) => {
  const tasks = JSON.parse(localStorage.getItem("tasks")) || [];
  const filteredTasks = tasks.filter((t) => t.id !== task.id);
  filteredTasks.unshift(task);
  localStorage.setItem("tasks", JSON.stringify(filteredTasks));

  updateTaskStatusTable();
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
    const base64_file = event.target.result;

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
        upsertTaskToStorage(data);
      })
      .catch((error) => {
        console.error("Error:", error);
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
    row.querySelector(".task-status").innerText = task.status;
    row.querySelector(".task-status").classList.add(`status-${task.status}`);
    row.querySelector("button").addEventListener("click", () => {
      const url = new URL(window.location);
      url.searchParams.set("taskId", task.id);
      window.history.pushState({}, "", url);

      document.dispatchEvent(
        new CustomEvent("open-pdf", {
          detail: { pdfUrl: task.file_url },
        })
      );
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
};

setInterval(refreshTasks, 5000);

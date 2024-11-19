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
    const row = firstRow.cloneNode(true);
    row.querySelector(".task-id").innerText = task.id;
    row.querySelector(".task-file-name").innerText = task.file_name;
    row.querySelector(".task-status").innerText = task.status;
    row.querySelector(".task-status").classList.add(`status-${task.status}`);
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

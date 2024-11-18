const fileUploadInput = document.getElementById("file-upload");

fileUploadInput.addEventListener("change", (event) => {
  const file = event.target.files[0];
  if (!file) {
    console.error("No file selected");
    return;
  }

  let base64 = null;
  const reader = new FileReader();
  reader.onload = (event) => {
    base64 = event.target.result;
    console.log(base64);
  };

  const formData = {
    base64_file: base64,
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
      console.log(data);
    })
    .catch((error) => {
      console.error("Error:", error);
    });
});

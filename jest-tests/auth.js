import fetch from "node-fetch";

export const getAuthCookie = async () => {
  const api_endpoint = process.env.API_ENDPOINT || "http://localhost:8090/api";
  const email = process.env.TEST_USER_EMAIL;
  const password = process.env.TEST_USER_PASSWORD;

  const response = await fetch(`${api_endpoint}/auth`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    credentials: "include",
    body: JSON.stringify({ email, password }),
  });

  const cookies = response.headers.raw()["set-cookie"];
  const authCookie = cookies.map((cookie) => cookie.split(";")[0]).join(";");

  return authCookie;
};

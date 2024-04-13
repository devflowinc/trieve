import { createEffect } from "solid-js";
import { createToast } from "../components/ShowToasts";

export const Home = () => {
  const api_host: string = import.meta.env.VITE_API_HOST as string;

  const login = () => {
    fetch(`${api_host}/auth/me`, {
      credentials: "include",
    })
      .then((res) => {
        if (res.status === 401) {
          window.location.href = `${api_host}/auth?redirect_uri=${window.origin}/dashboard`;
        }
        return res.json();
      })
      .then(() => {
        window.location.href = `${window.origin}/dashboard`;
      })
      .catch((err) => {
        console.error(err);
        createToast({
          title: "Error",
          type: "error",
          message: "Error logging in",
        });
      });
  };

  createEffect(() => {
    login();
  });
  return <div />;
};

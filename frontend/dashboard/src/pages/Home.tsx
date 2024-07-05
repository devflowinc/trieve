import { createEffect, useContext } from "solid-js";
import { createToast } from "../components/ShowToasts";
import { UserContext } from "../contexts/UserContext";

export const Home = () => {
  const api_host: string = import.meta.env.VITE_API_HOST as string;

  const userContext = useContext(UserContext);

  const login = () => {
    fetch(`${api_host}/auth/me`, {
      credentials: "include",
    })
      .then((res) => {
        if (res.status === 401) {
          window.location.href = `${api_host}/auth?redirect_uri=${window.origin}`;
          return;
        }
        return res.json();
      })
      .then(() => {
        window.location.href = `${
          window.origin
        }/dashboard/${userContext.selectedOrganizationId?.()}`;
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

import { HomeNavbar } from "../components/Atoms/HomeNavbar";
import type { JSX } from "solid-js";
import { DatasetAndUserContextWrapper } from "../components/Contexts/DatasetAndUserContext";

export const HomeLayout = (props: { children: JSX.Element }) => {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars

  const theme = (() => {
    if (typeof localStorage !== "undefined" && localStorage.getItem("theme")) {
      return localStorage.getItem("theme");
    }
    if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
      return "dark";
    }
    return "light";
  })();

  if (theme === "light") {
    document.documentElement.classList.remove("dark");
  } else {
    document.documentElement.classList.add("dark");
  }

  if ("serviceWorker" in navigator) {
    window.addEventListener("load", function () {
      navigator.serviceWorker.register("/sw.js").then(
        function (registration) {
          console.log(
            "Service Worker registered with scope:",
            registration.scope,
          );
        },
        function (error) {
          console.log("Service Worker registration failed:", error);
        },
      );
    });
  }

  return (
    <DatasetAndUserContextWrapper>
      <div class="flex min-h-screen flex-col bg-white dark:bg-shark-800 dark:text-white">
        <HomeNavbar />
        {props.children}
      </div>
    </DatasetAndUserContextWrapper>
  );
};

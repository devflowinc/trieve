import ShowToast from "../components/ShowToasts";
import { HomeSearch } from "../HomeSearch";

export const Home = () => {
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
    <>
      <HomeSearch />
      <ShowToast />
    </>
  );
};

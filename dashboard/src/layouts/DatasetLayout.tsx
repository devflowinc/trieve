import { JSX, createEffect } from "solid-js";
import NavBar from "../components/Navbar";
import ShowToasts from "../components/ShowToasts";
import { Sidebar } from "../components/Sidebar";
import { DatasetName } from "../components/DatasetName";
import { DatasetTabs } from "../components/DatasetTabs";
import { useLocation, useNavigate } from "@solidjs/router";

interface DatasetLayoutProps {
  children?: JSX.Element;
}

export const DatasetLayout = (props: DatasetLayoutProps) => {
  const location = useLocation();
  const navigate = useNavigate();

  createEffect(() => {
    const pathname = location.pathname;
    const sectionsLength = pathname.split("/").length;
    if (sectionsLength === 4) {
      navigate(pathname + "/start", { replace: true });
    }
  });

  return (
    <>
      <ShowToasts />
      <div class="flex min-h-screen flex-col bg-white text-black">
        <div class="w-full border-b px-10 py-2">
          <NavBar />
        </div>
        <div class="flex">
          <Sidebar />
          <div class="w-full bg-neutral-50 px-8">
            <div class="my-4 flex flex-col space-y-3 border-b">
              <DatasetName />
              <DatasetTabs />
            </div>
            <div>{props.children}</div>
          </div>
        </div>
      </div>
    </>
  );
};

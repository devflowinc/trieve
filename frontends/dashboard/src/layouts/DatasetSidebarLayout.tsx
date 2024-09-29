import { JSX } from "solid-js";
import { DashboardSidebar } from "../components/Sidebar";

interface Props {
  children?: JSX.Element;
}

export const DatasetLayout = (props: Props) => {
  return (
    <div class="grid max-h-full grow grid-cols-[300px_calc(100vw-300px)] overflow-hidden -md:overflow-x-auto">
      <DashboardSidebar />
      <div class="p-4 -md:min-w-[400px]">{props.children}</div>
    </div>
  );
};

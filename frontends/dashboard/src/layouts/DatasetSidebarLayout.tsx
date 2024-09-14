import { JSX } from "solid-js";
import { DashboardSidebar } from "../components/Sidebar";

interface Props {
  children?: JSX.Element;
}

// Needs to ensure dataset and org don't desync
export const DatasetLayout = (props: Props) => {
  return (
    <div class="grid max-h-full grow grid-cols-[270px_1fr] overflow-hidden">
      <DashboardSidebar />
      <div class="overflow-scroll p-4">{props.children}</div>
    </div>
  );
};

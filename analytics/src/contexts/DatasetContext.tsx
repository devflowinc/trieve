import { SlimUser } from "shared/types";
import { createSignal, ParentProps } from "solid-js";

interface DatasetContextProps extends ParentProps {
  user: SlimUser;
}

export const DatasetContextProvider = (props: DatasetContextProps) => {
  const selectedDataset = createSignal(props.user.orgs);
};

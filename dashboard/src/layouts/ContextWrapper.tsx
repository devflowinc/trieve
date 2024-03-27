import { JSX } from "solid-js";
import { UserContextWrapper } from "../contexts/UserContext.tsx";
import { DatasetContextWrapper } from "../contexts/DatasetContext.tsx";

interface ContextWrapperProps {
  children?: JSX.Element;
}

export const ContextWrapper = (props: ContextWrapperProps) => {
  return (
    <>
      <UserContextWrapper>
        <DatasetContextWrapper>{props.children}</DatasetContextWrapper>
      </UserContextWrapper>
    </>
  );
};

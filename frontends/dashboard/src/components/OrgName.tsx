import { useContext } from "solid-js";
import { UserContext } from "../contexts/UserContext";
import { CopyButton } from "./CopyButton";

export const OrgName = () => {
  const userContext = useContext(UserContext);

  return (
    <h3 class="flex items-baseline gap-2 text-xl font-semibold text-neutral-600">
      {userContext.selectedOrg().name}
      <CopyButton size={14} text={userContext.selectedOrg().id} />
    </h3>
  );
};

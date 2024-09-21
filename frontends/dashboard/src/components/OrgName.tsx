import { useContext } from "solid-js";
import { UserContext } from "../contexts/UserContext";
import { CopyButton } from "./CopyButton";

export const OrgName = () => {
  const userContext = useContext(UserContext);

  return (
    <div>
      <h3 class="flex items-baseline gap-2 text-xl font-semibold">
        {userContext.selectedOrg().name}
      </h3>
      <p class="flex flex-row gap-1.5 text-sm text-neutral-700">
        <span class="font-medium">Org ID:</span>
        {userContext.selectedOrg().id}
        <CopyButton size={14} text={userContext.selectedOrg().id} />
      </p>
    </div>
  );
};

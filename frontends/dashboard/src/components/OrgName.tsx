import { useContext } from "solid-js";
import { UserContext } from "../contexts/UserContext";

export const OrgName = () => {
  const userContext = useContext(UserContext);

  return (
    <h3 class="text-xl font-semibold text-neutral-600">
      {userContext.selectedOrganization().name}
    </h3>
  );
};

import { useContext } from "solid-js";
import { UserContext } from "../contexts/UserContext";
import { FiChevronDown, FiUser } from "solid-icons/fi";

export const NavbarOrgWidget = () => {
  const userInfo = useContext(UserContext);
  return (
    <div class="relative">
      <div class="flex items-center gap-2 rounded-md border border-neutral-200 bg-neutral-100 p-1 px-2 text-sm">
        <FiUser class="text-neutral-500" />
        <div>{userInfo.user().email}</div>
        <FiChevronDown />
      </div>
    </div>
  );
};

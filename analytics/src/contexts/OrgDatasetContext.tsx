import { redirect } from "@solidjs/router";
import { Organization, SlimUser } from "shared/types";
import { Accessor, createContext, createSignal, ParentProps } from "solid-js";

interface OrgDatasetContextProps extends ParentProps {
  user: SlimUser;
}

interface OrgDatasetContextType {
  selectedOrg: Accessor<Organization>;
  selectOrg: (organization: Organization) => void;
}

const OrgDatasetContext = createContext<OrgDatasetContextType>();

//TODO: Read from path parameters
export const OrgDatasetContextProvider = (props: OrgDatasetContextProps) => {
  // TODO: use localstorage
  const getInitialUserOrg = () => {
    if (props.user.orgs.length === 0) {
      throw redirect("/error");
    }
    return props.user.orgs[0];
  };
  const [selectedOrg, setSelectedOrg] = createSignal(getInitialUserOrg());

  return (
    <>
      <OrgDatasetContext.Provider
        value={{
          selectedOrg: selectedOrg,
          selectOrg: (organization) => setSelectedOrg(organization),
        }}
      >
        {props.children}
      </OrgDatasetContext.Provider>
    </>
  );
};

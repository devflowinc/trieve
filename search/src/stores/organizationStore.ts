import { atom } from "nanostores";
import { isOrganizationDTO, type OrganizationDTO } from "../../utils/apiTypes";
import { currentUser } from "./userStore";

currentUser.subscribe((user) => {
  if (!user) {
    return;
  }
  const orgItem = localStorage.getItem("currentOrganization");
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
  const organization = orgItem ? JSON.parse(orgItem) : null;
  organizations.set(user.orgs);
  if (organization && isOrganizationDTO(organization)) {
    // check if user is in the organization
    const org = user.orgs.find((o) => o.id === organization.id);
    if (org) {
      currentOrganization.set(organization);
      return;
    }
  } else {
    currentOrganization.set(user.orgs[0]);
  }
});

export const currentOrganization = atom<OrganizationDTO | null>(null);
export const organizations = atom<OrganizationDTO[]>([]);

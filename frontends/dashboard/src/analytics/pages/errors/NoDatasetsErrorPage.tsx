import { FiExternalLink } from "solid-icons/fi";

interface NoDatasetsErrorPageProps {
  orgId: string;
}
export const NoDatasetsErrorPage = (props: NoDatasetsErrorPageProps) => {
  const dashboardLink = import.meta.env.VITE_DASHBOARD_URL as string;
  return (
    <div class="grid grow place-items-center pt-80">
      <div class="flex flex-col items-center rounded border border-neutral-200 bg-white p-4 text-center">
        <div>You Have No Datasets In This Organization</div>
        <a
          class="mt-2 flex items-center gap-2 rounded border border-neutral-300 bg-neutral-200 p-2 text-sm hover:underline"
          target="_blank"
          href={`${dashboardLink}/dashboard/${props.orgId}/overview`}
        >
          Create A New One
          <FiExternalLink />
        </a>
      </div>
    </div>
  );
};

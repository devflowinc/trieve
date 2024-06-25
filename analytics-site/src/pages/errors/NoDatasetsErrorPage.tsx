interface NoDatasetsErrorPageProps {
  orgId: string;
}
export const NoDatasetsErrorPage = (props: NoDatasetsErrorPageProps) => {
  const dashboardLink = import.meta.env.VITE_DASHBOARD_URL as string;
  return (
    <div class="grid grow place-items-center">
      <div class="border border-neutral-200 bg-white p-4 text-center">
        <div>You Have No Datasets</div>
        <a
          target="_blank"
          href={`${dashboardLink}/dashboard/${props.orgId}/overview`}
        >
          Create A New One
        </a>
      </div>
    </div>
  );
};

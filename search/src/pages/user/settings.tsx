import EditUserForm from "../../components/EditUserForm";
import { Footer } from "../../components/Footer";
import { SearchLayout } from "../../layouts/SearchLayout";

export const Settings = () => {
  return (
    <SearchLayout>
      <div class="mx-auto w-full max-w-6xl px-4">
        <h1 class="mb-4 mt-8 text-center text-4xl font-bold">
          Edit your profile
        </h1>
        <EditUserForm />
      </div>
      <div class="flex-1" />
      <Footer />
    </SearchLayout>
  );
};

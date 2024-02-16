import {createSignal } from 'solid-js';
import { SearchLayout } from '../../layouts/SearchLayout';
import { Footer } from '../../components/Footer';
import { GroupUserPageView } from '../../components/OrgGroupPageView';
import { useStore } from '@nanostores/solid';
import { currentUser } from '../../stores/userStore';
import { ConfirmModal } from '../../components/Atoms/ConfirmModal';



export const OrgGroups = () => {
    // Define your component logic here
    const $currentUser = useStore(currentUser);
    const [showConfirmGroupDeleteModal, setShowConfirmGroupDeleteModal] = createSignal(false);
    const [onGroupDelete, setOnGroupDelete] = createSignal(() => { });


    return (
        <SearchLayout>
            <div class="mb-4 mt-4 flex  flex-col overflow-hidden  pt-4 text-xl mx-[10rem]">
                <GroupUserPageView
                    loggedUser={$currentUser()}
                    setOnDelete={setOnGroupDelete}
                    setShowConfirmModal={setShowConfirmGroupDeleteModal}
                />
                <ConfirmModal
                    showConfirmModal={showConfirmGroupDeleteModal}
                    setShowConfirmModal={setShowConfirmGroupDeleteModal}
                    onConfirm={onGroupDelete}
                    message={"Are you sure you want to delete this group?"}
                />
            </div>
            <div class="flex-1" />
            <Footer />

        </SearchLayout>
    );
};


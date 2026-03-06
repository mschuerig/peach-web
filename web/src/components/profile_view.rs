use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use send_wrapper::SendWrapper;

use domain::{ProgressTimeline, TrainingMode, TrainingModeState};

use super::page_nav::PageNav;
use super::progress_card::ProgressCard;

#[component]
pub fn ProfileView() -> impl IntoView {
    let progress_timeline: SendWrapper<Rc<RefCell<ProgressTimeline>>> =
        use_context().expect("ProgressTimeline context");
    let is_profile_loaded: RwSignal<bool> = use_context().expect("is_profile_loaded context");

    let ptl = progress_timeline.clone();

    view! {
        <div class="py-12">
            <PageNav current="profile" />
            <h1 class="text-2xl font-bold dark:text-white">"Profile"</h1>

            {move || {
                if !is_profile_loaded.get() {
                    return view! {
                        <p class="mt-6 text-gray-500 dark:text-gray-400">"Loading\u{2026}"</p>
                    }
                    .into_any();
                }

                let tl = ptl.borrow();
                let any_active = TrainingMode::ALL
                    .iter()
                    .any(|&m| tl.state(m) == TrainingModeState::Active);
                drop(tl);

                if !any_active {
                    return view! {
                        <p class="mt-6 text-gray-500 dark:text-gray-400">
                            "No training data yet. Start a training session to see your progress."
                        </p>
                    }
                    .into_any();
                }

                view! {
                    <div class="mt-6 space-y-4">
                        {TrainingMode::ALL
                            .iter()
                            .map(|&mode| view! { <ProgressCard mode=mode /> })
                            .collect::<Vec<_>>()}
                    </div>
                }
                .into_any()
            }}
        </div>
    }
}

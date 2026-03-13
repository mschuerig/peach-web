use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use leptos_fluent::{move_tr, tr, I18n};
use send_wrapper::SendWrapper;

use crate::app::base_href;

use domain::{ProgressTimeline, TrainingMode, TrainingModeState};

use super::nav_bar::NavBar;
use super::progress_card::ProgressCard;

#[component]
pub fn ProfileView() -> impl IntoView {
    let progress_timeline: SendWrapper<Rc<RefCell<ProgressTimeline>>> =
        use_context().expect("ProgressTimeline context");
    let crate::app::IsProfileLoaded(is_profile_loaded) =
        use_context().expect("is_profile_loaded context");
    let i18n = expect_context::<I18n>();

    let ptl = progress_timeline.clone();

    view! {
        <div class="pt-4 pb-12">
            <NavBar title=move_tr!("profile-title") back_href=base_href("/")>
            </NavBar>

            {move || {
                if !is_profile_loaded.get() {
                    return view! {
                        <p class="mt-6 text-gray-500 dark:text-gray-400">{move_tr!("loading")}</p>
                    }
                    .into_any();
                }

                let tl = ptl.borrow();
                let active_names: Vec<String> = TrainingMode::ALL
                    .iter()
                    .filter(|&&m| tl.state(m) == TrainingModeState::Active)
                    .map(|&m| i18n.tr(m.config().display_name))
                    .collect();
                drop(tl);

                if active_names.is_empty() {
                    let empty_aria = tr!("profile-no-data-aria");
                    return view! {
                        <div aria-label=empty_aria>
                            <p class="mt-6 text-gray-500 dark:text-gray-400">
                                {move_tr!("no-training-data")}
                            </p>
                        </div>
                    }
                    .into_any();
                }

                let scroll_aria = tr!("profile-showing-progress", {
                    "modes" => active_names.join(", ")
                });

                view! {
                    <div class="mt-6 space-y-4" aria-label=scroll_aria>
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

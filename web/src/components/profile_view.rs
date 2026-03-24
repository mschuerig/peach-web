use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use leptos_fluent::{I18n, move_tr, tr};
use send_wrapper::SendWrapper;

use crate::app::base_href;

use domain::{PerceptualProfile, TrainingDiscipline, TrainingDisciplineState};

use super::help_content::HelpModal;
use super::nav_bar::{NavBar, NavIconButton};
use super::progress_card::ProgressCard;

use crate::help_sections::PROFILE_HELP;

#[component]
pub fn ProfileView() -> impl IntoView {
    let profile: SendWrapper<Rc<RefCell<PerceptualProfile>>> =
        use_context().expect("PerceptualProfile context");
    let crate::app::IsProfileLoaded(is_profile_loaded) =
        use_context().expect("is_profile_loaded context");
    let i18n = expect_context::<I18n>();

    let is_help_open = RwSignal::new(false);

    view! {
        <div class="pt-4 pb-12">
            <NavBar title=move_tr!("profile-title") back_href=base_href("/")>
                <NavIconButton label=Signal::derive(move || tr!("nav-help")) icon="?".to_string() on_click=Callback::new(move |_| is_help_open.set(true)) circled=true />
            </NavBar>
            <HelpModal title=move_tr!("profile-help-title") sections=PROFILE_HELP is_open=is_help_open />

            {move || {
                if !is_profile_loaded.get() {
                    return view! {
                        <p class="mt-6 text-gray-500 dark:text-gray-400">{move_tr!("loading")}</p>
                    }
                    .into_any();
                }

                let p = profile.borrow();
                let active_names: Vec<String> = TrainingDiscipline::ALL
                    .iter()
                    .filter(|&&m| p.state(m) == TrainingDisciplineState::Active)
                    .map(|&m| i18n.tr(m.config().display_name))
                    .collect();
                drop(p);

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
                        {TrainingDiscipline::ALL
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

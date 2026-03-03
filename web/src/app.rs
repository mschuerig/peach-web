use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes, A},
    path,
};
use send_wrapper::SendWrapper;
use wasm_bindgen_futures::spawn_local;

use crate::adapters::audio_context::AudioContextManager;
use crate::adapters::indexeddb_store::IndexedDbStore;
use crate::components::{
    ComparisonView, InfoView, PitchMatchingView, ProfileView, SettingsView, StartPage,
};
use domain::types::MIDINote;
use domain::{PerceptualProfile, ThresholdTimeline, TrendAnalyzer};

#[component]
pub fn App() -> impl IntoView {
    let profile = SendWrapper::new(Rc::new(RefCell::new(PerceptualProfile::new())));
    let audio_ctx_manager = SendWrapper::new(Rc::new(RefCell::new(AudioContextManager::new())));
    let trend_analyzer = SendWrapper::new(Rc::new(RefCell::new(TrendAnalyzer::new())));
    let timeline = SendWrapper::new(Rc::new(RefCell::new(ThresholdTimeline::new())));
    let is_profile_loaded = RwSignal::new(false);
    let db_store = RwSignal::new_local(None::<Rc<IndexedDbStore>>);

    provide_context(profile.clone());
    provide_context(audio_ctx_manager);
    provide_context(trend_analyzer.clone());
    provide_context(timeline.clone());
    provide_context(is_profile_loaded);
    provide_context(db_store);

    // Async hydration — runs after mount
    let profile_for_hydration = Rc::clone(&*profile);
    let trend_for_hydration = Rc::clone(&*trend_analyzer);
    let timeline_for_hydration = Rc::clone(&*timeline);

    spawn_local(async move {
        match IndexedDbStore::open().await {
            Ok(store) => {
                let store = Rc::new(store);

                match store.fetch_all_comparisons().await {
                    Ok(records) => {
                        let mut prof = profile_for_hydration.borrow_mut();
                        let mut trend = trend_for_hydration.borrow_mut();
                        let mut tl = timeline_for_hydration.borrow_mut();

                        for record in &records {
                            prof.update(
                                MIDINote::new(record.reference_note),
                                record.cent_offset.abs(),
                                record.is_correct,
                            );

                            trend.push(record.cent_offset.abs());

                            tl.push(
                                &record.timestamp,
                                record.cent_offset.abs(),
                                record.is_correct,
                                record.reference_note,
                            );
                        }

                        log::info!("Profile hydrated from {} records", records.len());
                    }
                    Err(e) => {
                        log::error!("Failed to fetch records for hydration: {e}");
                    }
                }

                db_store.set(Some(store));
            }
            Err(e) => {
                log::error!("Failed to open IndexedDB: {e}");
            }
        }

        is_profile_loaded.set(true);
    });

    view! {
        <Router>
            <a
                href="#main-content"
                class="sr-only focus:not-sr-only focus:absolute focus:z-50 focus:p-3 focus:bg-white focus:text-black dark:focus:bg-gray-900 dark:focus:text-white"
            >
                "Skip to main content"
            </a>
            <main id="main-content" class="min-h-screen bg-white dark:bg-gray-900">
                <div class="mx-auto max-w-lg px-4">
                    <Routes fallback=|| {
                        view! {
                            <div class="py-12 text-center">
                                <h1 class="text-2xl font-bold dark:text-white">"Page not found"</h1>
                                <A
                                    href="/"
                                    attr:class="mt-4 inline-block min-h-11 min-w-11 rounded px-3 py-2 text-indigo-600 hover:text-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-2 dark:text-indigo-400 dark:hover:text-indigo-300"
                                >
                                    "Back to Start"
                                </A>
                            </div>
                        }
                    }>
                        <Route path=path!("/") view=StartPage />
                        <Route path=path!("/training/comparison") view=ComparisonView />
                        <Route path=path!("/training/pitch-matching") view=PitchMatchingView />
                        <Route path=path!("/profile") view=ProfileView />
                        <Route path=path!("/settings") view=SettingsView />
                        <Route path=path!("/info") view=InfoView />
                    </Routes>
                </div>
            </main>
        </Router>
    }
}

//! One polish action: capture, build the prompt, call the provider, replace.
//!
//! The orchestration lives here rather than in the Tauri shell so every path -
//! capture failure, provider failure, replace failure, success - can be driven
//! by tests with fakes. It knows nothing about windows, threads, menus, or the
//! real clipboard.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use thiserror::Error;

use crate::config::Config;
use crate::config_store::LiveConfig;
use crate::prompt::build_prompt;
use crate::provider::ProviderError;
use crate::selection::{Selection, SelectionError};

/// What the overlay and the settings window are told about an action.
///
/// Serializes as `{"state": "working" | "success" | "error", "message": "..."}`,
/// which is what the frontend listens for.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct StateEvent {
    pub state: State,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Working,
    Success,
    Error,
}

impl serde::Serialize for State {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl State {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Working => "working",
            Self::Success => "success",
            Self::Error => "error",
        }
    }
}

impl StateEvent {
    pub fn working() -> Self {
        Self {
            state: State::Working,
            message: "Polishing…".into(),
        }
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self {
            state: State::Success,
            message: message.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            state: State::Error,
            message: message.into(),
        }
    }
}

/// Why a polish action stopped.
#[derive(Debug, Error)]
pub enum ActionError {
    #[error("another polish action is already running")]
    Busy,
    #[error(transparent)]
    Selection(#[from] SelectionError),
    #[error(transparent)]
    Provider(#[from] ProviderError),
}

impl ActionError {
    /// The text shown to the user for this failure.
    pub fn user_message(&self) -> String {
        match self {
            Self::Busy => "Refiny is already polishing.".to_string(),
            Self::Selection(error) => error.user_message(),
            Self::Provider(error) => error.to_string(),
        }
    }
}

/// Calls a provider for one piece of text.
///
/// The real implementation builds an [`crate::OpenAiCompatible`] from the
/// config in effect; tests substitute a fake.
pub trait PolishProvider: Send + Sync {
    fn polish(
        &self,
        config: &Config,
        system_prompt: &str,
        text: &str,
    ) -> Result<String, ProviderError>;
}

/// Runs a polish action end to end without any platform code.
pub struct PolishAction {
    config: Arc<LiveConfig>,
    selection: Arc<dyn Selection>,
    provider: Arc<dyn PolishProvider>,
    busy: AtomicBool,
}

impl PolishAction {
    pub fn new(
        config: Arc<LiveConfig>,
        selection: Arc<dyn Selection>,
        provider: Arc<dyn PolishProvider>,
    ) -> Self {
        Self {
            config,
            selection,
            provider,
            busy: AtomicBool::new(false),
        }
    }

    /// True while an action is running.
    pub fn is_busy(&self) -> bool {
        self.busy.load(Ordering::SeqCst)
    }

    /// Runs one action, reporting each state change through `report`.
    ///
    /// Rejects with [`ActionError::Busy`] when another action is running, so a
    /// held-down hotkey cannot start a second paste over the first one.
    pub fn run(&self, report: &mut dyn FnMut(StateEvent)) -> Result<(), ActionError> {
        let _guard = self.begin()?;

        // Capture before the overlay is shown, so the overlay can never take
        // focus away from the app that owns the selection.
        let text = match self.selection.capture() {
            Ok(text) => text,
            Err(error) => {
                let error = ActionError::from(error);
                report(StateEvent::error(error.user_message()));
                return Err(error);
            }
        };

        report(StateEvent::working());

        let result = self.polish_and_replace(&text);
        match &result {
            Ok(()) => report(StateEvent::success("Done.")),
            Err(error) => report(StateEvent::error(error.user_message())),
        }
        result
    }

    fn polish_and_replace(&self, text: &str) -> Result<(), ActionError> {
        let config = self.config.snapshot();
        let prompt = build_prompt(config.prompt_template(), &config.preserve);
        let polished = self.provider.polish(&config, &prompt, text)?;

        // A failed replace leaves the text as it was; the error says why.
        self.selection.replace(&polished)?;
        Ok(())
    }

    fn begin(&self) -> Result<BusyGuard<'_>, ActionError> {
        if self.busy.swap(true, Ordering::SeqCst) {
            return Err(ActionError::Busy);
        }
        Ok(BusyGuard { busy: &self.busy })
    }
}

/// Clears the busy flag even if the action panics.
struct BusyGuard<'a> {
    busy: &'a AtomicBool,
}

impl Drop for BusyGuard<'_> {
    fn drop(&mut self) {
        self.busy.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_store::MemoryConfigStore;
    use crate::provider::ProviderError;
    use crate::selection::MAX_CHARS;
    use std::sync::Mutex;

    /// Records what the orchestrator asked the platform layer to do.
    #[derive(Default)]
    struct FakeSelection {
        selection: Mutex<Option<String>>,
        capture_error: Mutex<Option<SelectionError>>,
        replace_error: Mutex<Option<SelectionError>>,
        replaced: Mutex<Vec<String>>,
    }

    impl FakeSelection {
        fn with_selection(text: &str) -> Arc<Self> {
            let fake = Self::default();
            *fake.selection.lock().unwrap() = Some(text.to_string());
            Arc::new(fake)
        }

        fn failing(error: SelectionError) -> Arc<Self> {
            let fake = Self::default();
            *fake.capture_error.lock().unwrap() = Some(error);
            Arc::new(fake)
        }

        fn failing_replace(text: &str, error: SelectionError) -> Arc<Self> {
            let fake = Self::with_selection(text);
            *fake.replace_error.lock().unwrap() = Some(error);
            fake
        }

        fn replaced(&self) -> Vec<String> {
            self.replaced.lock().unwrap().clone()
        }
    }

    impl Selection for FakeSelection {
        fn capture(&self) -> Result<String, SelectionError> {
            if let Some(error) = self.capture_error.lock().unwrap().take() {
                return Err(error);
            }
            self.selection
                .lock()
                .unwrap()
                .clone()
                .ok_or(SelectionError::NothingSelected)
        }

        fn replace(&self, text: &str) -> Result<(), SelectionError> {
            if let Some(error) = self.replace_error.lock().unwrap().take() {
                return Err(error);
            }
            self.replaced.lock().unwrap().push(text.to_string());
            Ok(())
        }
    }

    /// A provider that answers with a fixed string or a fixed error.
    struct FakeProvider {
        answer: Result<String, ProviderError>,
        seen: Mutex<Vec<(String, String)>>,
    }

    impl FakeProvider {
        fn answering(text: &str) -> Arc<Self> {
            Arc::new(Self {
                answer: Ok(text.to_string()),
                seen: Mutex::new(Vec::new()),
            })
        }

        fn failing(error: ProviderError) -> Arc<Self> {
            Arc::new(Self {
                answer: Err(error),
                seen: Mutex::new(Vec::new()),
            })
        }

        fn calls(&self) -> usize {
            self.seen.lock().unwrap().len()
        }
    }

    impl PolishProvider for FakeProvider {
        fn polish(
            &self,
            _config: &Config,
            system_prompt: &str,
            text: &str,
        ) -> Result<String, ProviderError> {
            self.seen
                .lock()
                .unwrap()
                .push((system_prompt.to_string(), text.to_string()));
            match &self.answer {
                Ok(answer) => Ok(answer.clone()),
                Err(error) => Err(clone_provider_error(error)),
            }
        }
    }

    fn clone_provider_error(error: &ProviderError) -> ProviderError {
        match error {
            ProviderError::Status { status, body } => ProviderError::Status {
                status: *status,
                body: body.clone(),
            },
            ProviderError::Api(message) => ProviderError::Api(message.clone()),
            ProviderError::NoChoices => ProviderError::NoChoices,
            ProviderError::Empty => ProviderError::Empty,
            ProviderError::Request(_) | ProviderError::Decode(_) => {
                ProviderError::Api("fake provider".into())
            }
        }
    }

    fn action(
        selection: Arc<FakeSelection>,
        provider: Arc<FakeProvider>,
    ) -> (PolishAction, Arc<LiveConfig>) {
        let store = Arc::new(MemoryConfigStore::new());
        let config = Arc::new(LiveConfig::load(store).unwrap());
        let action = PolishAction::new(config.clone(), selection, provider);
        (action, config)
    }

    fn record(events: &mut Vec<StateEvent>) -> impl FnMut(StateEvent) + '_ {
        move |event| events.push(event)
    }

    #[test]
    fn success_reports_working_then_done_and_pastes_the_result() {
        let selection = FakeSelection::with_selection("rough text");
        let provider = FakeProvider::answering("Polished.");
        let (action, _) = action(selection.clone(), provider.clone());

        let mut events = Vec::new();
        action.run(&mut record(&mut events)).unwrap();

        assert_eq!(
            events,
            vec![StateEvent::working(), StateEvent::success("Done.")]
        );
        assert_eq!(selection.replaced(), vec!["Polished.".to_string()]);
        assert_eq!(provider.calls(), 1);
    }

    #[test]
    fn the_prompt_includes_the_preservation_list() {
        let selection = FakeSelection::with_selection("text");
        let provider = FakeProvider::answering("text");
        let (action, config) = action(selection, provider.clone());
        config
            .update(|next| next.preserve = vec!["Refiny".into()])
            .unwrap();

        let mut events = Vec::new();
        action.run(&mut record(&mut events)).unwrap();

        let seen = provider.seen.lock().unwrap();
        assert!(seen[0].0.contains("- Refiny"));
        assert_eq!(seen[0].1, "text");
    }

    #[test]
    fn capture_failure_reports_one_error_and_touches_nothing() {
        let selection = FakeSelection::failing(SelectionError::NothingSelected);
        let provider = FakeProvider::answering("Polished.");
        let (action, _) = action(selection.clone(), provider.clone());

        let mut events = Vec::new();
        let error = action.run(&mut record(&mut events)).unwrap_err();

        assert!(matches!(error, ActionError::Selection(_)));
        assert_eq!(
            events,
            vec![StateEvent::error(error.user_message())],
            "a capture failure must not claim to be working"
        );
        assert_eq!(provider.calls(), 0);
        assert!(selection.replaced().is_empty());
    }

    #[test]
    fn a_selection_over_the_limit_never_reaches_the_provider() {
        let selection = FakeSelection::failing(SelectionError::TooLong(MAX_CHARS));
        let provider = FakeProvider::answering("Polished.");
        let (action, _) = action(selection, provider.clone());

        let mut events = Vec::new();
        let error = action.run(&mut record(&mut events)).unwrap_err();

        assert!(error.user_message().contains("20000"));
        assert_eq!(provider.calls(), 0);
    }

    #[test]
    fn provider_failure_leaves_the_text_untouched() {
        let selection = FakeSelection::with_selection("rough text");
        let provider = FakeProvider::failing(ProviderError::Status {
            status: 401,
            body: "invalid api key".into(),
        });
        let (action, _) = action(selection.clone(), provider);

        let mut events = Vec::new();
        let error = action.run(&mut record(&mut events)).unwrap_err();

        assert!(error.user_message().contains("401"));
        assert_eq!(
            events,
            vec![
                StateEvent::working(),
                StateEvent::error(error.user_message())
            ]
        );
        assert!(selection.replaced().is_empty());
        assert_eq!(
            selection.selection.lock().unwrap().as_deref(),
            Some("rough text")
        );
    }

    #[test]
    fn replace_failure_leaves_the_text_untouched_and_reports_it() {
        let selection =
            FakeSelection::failing_replace("rough text", SelectionError::Input("blocked".into()));
        let provider = FakeProvider::answering("Polished.");
        let (action, _) = action(selection.clone(), provider);

        let mut events = Vec::new();
        let error = action.run(&mut record(&mut events)).unwrap_err();

        assert!(error.user_message().contains("synthetic input"));
        assert_eq!(
            events,
            vec![
                StateEvent::working(),
                StateEvent::error(error.user_message())
            ]
        );
        assert!(selection.replaced().is_empty());
    }

    #[test]
    fn the_saved_config_is_what_the_next_action_uses() {
        let selection = FakeSelection::with_selection("text");
        let provider = FakeProvider::answering("text");
        let (action, config) = action(selection, provider.clone());

        let mut events = Vec::new();
        action.run(&mut record(&mut events)).unwrap();
        config
            .update(|next| {
                next.provider.model = "llama3.2".into();
                next.prompt = "Be terse.".into();
            })
            .unwrap();
        action.run(&mut record(&mut events)).unwrap();

        let seen = provider.seen.lock().unwrap();
        assert_eq!(seen.len(), 2);
        assert_eq!(seen[1].0, "Be terse.");
    }

    #[test]
    fn an_action_is_reported_busy_while_another_runs() {
        let selection = FakeSelection::with_selection("text");
        let provider = FakeProvider::answering("text");
        let (action, _) = action(selection.clone(), provider.clone());

        let mut events = Vec::new();
        action.run(&mut record(&mut events)).unwrap();
        assert!(!action.is_busy());
        assert_eq!(provider.calls(), 1);

        let mut nested = Vec::new();
        let mut reporter = |event: StateEvent| {
            nested.push(event);
            // A hotkey pressed during the paste must be ignored, not queued.
            assert!(matches!(action.run(&mut |_| {}), Err(ActionError::Busy)));
        };
        action.run(&mut reporter).unwrap();
        assert!(!action.is_busy());
        assert_eq!(provider.calls(), 2);
    }

    #[test]
    fn the_real_provider_satisfies_the_trait_the_app_calls_through() {
        // Guards against the trait drifting away from the real client.
        fn assert_provider<T: PolishProvider + ?Sized>() {}
        assert_provider::<FakeProvider>();
        assert_provider::<crate::provider::ConfiguredProvider>();
    }
}

use iced::keyboard;

use crate::family::Family;
use crate::state_machine::{CanvasMessage, EditorTool};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Menu {
    New,
    NewFromTabs,
    Operations,
    File,
    Export,
}

#[derive(Debug, Clone)]
pub enum Message {
    Canvas(CanvasMessage),
    SelectTool(EditorTool),
    KeyPressed {
        key: keyboard::Key,
        modifiers: keyboard::Modifiers,
        captured: bool,
    },
    KeyReleased(keyboard::Key),
    Clear,
    EditTextChanged(String),
    FinishEditing,
    ToggleMenu(Menu),
    CloseMenus,
    DismissModal,
    CloseModal(Modal),
    Noop,
    ChooseStartupModule(usize),
    StartupHover(usize),
    StartupOpenFile,
    ExitApp,
    WindowCloseRequested,
    ConfirmCloseDiscard,
    ConfirmCloseSave,
    ToggleTheme,
    WindowResized(iced::Size),
    ToggleDock,
    ToggleShortcuts,
    ToastTick,
    DismissToast(u64),
    ZoomIn,
    ZoomOut,
    ZoomReset,
    FitView,
    OpenLoadDialog,
    LoadPathChanged(String),
    LoadBrowseClicked,
    LoadBrowseResult {
        picked: Option<Result<(String, String), String>>,
    },
    LoadPathSubmitted,
    OpenSaveDialog,
    SavePathChanged(String),
    SaveBrowseClicked,
    SaveBrowseResult { result: Result<String, String> },
    SavePathSubmitted,
    CopyLlmPrompt,
    CheckInput,
    NfaToDfa,
    Minimize,
    CheckInputTextChanged(String),
    SubmitCheckInput,
    OpenRegexDialog,
    RegexTextChanged(String),
    SubmitRegex,
    NewTab(Family),
    RemoveTab(usize),
    SwitchTab(usize),
    TabRenameChanged(String),
    TabRenameSubmit,
    RunInputChanged(String),
    RunLoad,
    RunStep,
    RunReset,
    RunTogglePlay,
    RunTick,
    GrammarEditorAction(iced::widget::text_editor::Action),
    GrammarWordChanged(String),
    GrammarParse,
    GrammarCheckWord,
    GrammarDerive,
    GrammarToCnf,
    OpenLatexExport,
    CopyLatexExport,
    OpenRegexExport,
    CopyRegexExport,
    EditTransitionLabelChanged(usize, String),
    SaveEditTransitionLabels,
    DeleteEditTransitionLabel(usize),
    AddEditTransitionLabel,
}

impl Message {
    pub(crate) fn may_change_content(&self) -> bool {
        !matches!(
            self,
            Message::RunTick
                | Message::ToastTick
                | Message::KeyReleased(_)
                | Message::Noop
                | Message::StartupHover(_)
                | Message::WindowResized(_)
                | Message::Canvas(
                    CanvasMessage::MoveState { .. }
                        | CanvasMessage::Scrolled(_)
                        | CanvasMessage::Zoomed { .. }
                        | CanvasMessage::Viewport(_)
                )
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Modal {
    RenameState,
    CheckInput,
    BuildRegex,
    Load,
    Save,
    LatexExport,
    RegexExport,
    NewTransition,
    TransitionLabels,
    Shortcuts,
    Close,
    Error,
}

impl Modal {
    pub(crate) const STACK: [Modal; 12] = [
        Modal::RenameState,
        Modal::CheckInput,
        Modal::BuildRegex,
        Modal::Load,
        Modal::Save,
        Modal::LatexExport,
        Modal::RegexExport,
        Modal::NewTransition,
        Modal::TransitionLabels,
        Modal::Shortcuts,
        Modal::Close,
        Modal::Error,
    ];
}

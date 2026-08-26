use iced::keyboard;

use crate::state_machine;

#[derive(Debug, Clone)]
pub enum Message {
    Canvas(state_machine::CanvasMessage),
    KeyPressed(keyboard::Key),
    KeyReleased(keyboard::Key),
    Clear,
    EditTextChanged(String),
    FinishEditing,
    CancelEditing,
    ToggleOperationsMenu,
    CheckInput,
    DfaToNfa,
    Minimize,
    CheckInputTextChanged(String),
    SubmitCheckInput,
    CancelCheckInput,
    CloseCheckResultPopup,
    AddTab,
    RemoveTab(usize),
    SwitchTab(usize),
    CloseError,
    OpenLatexExport,
    CloseLatexExport,
    CopyLatexExport,
    #[allow(dead_code)]
    OpenEditTransition((usize, usize)),
    EditTransitionLabelChanged(usize, String),
    SaveEditTransitionLabels,
    DeleteEditTransitionLabel(usize),
    AddEditTransitionLabel,
    CancelEditTransitionLabels,
}

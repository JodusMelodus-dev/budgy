use egui::{KeyboardShortcut, Ui};

pub trait ExtraUi {
    fn button_with_shortcut(
        &mut self,
        text: impl Into<egui::WidgetText>,
        shortcut: KeyboardShortcut,
    ) -> bool;
}

impl ExtraUi for Ui {
    fn button_with_shortcut(
        &mut self,
        text: impl Into<egui::WidgetText>,
        shortcut: KeyboardShortcut,
    ) -> bool {
        let shortcut_pressed = self.input_mut(|i| i.consume_shortcut(&shortcut));
        let button = egui::Button::new(text).shortcut_text(self.ctx().format_shortcut(&shortcut));
        self.add(button).clicked() || shortcut_pressed
    }
}

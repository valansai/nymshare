// MIT License
// Copyright (c) Valan Sai 2025
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use paste::paste;

/// ---------------------- Timed message macro ----------------------
/// Generates setter and checker for a message with a duration
#[macro_export]
macro_rules! timed_message {
    ($set_fn:ident, $show_fn:ident, $field:ident, $time_field:ident, $duration:expr) => {
        pub fn $set_fn(&mut self, msg: impl Into<String>) {
            self.$field = msg.into();
            self.$time_field = Some(std::time::Instant::now());
        }

        pub fn $show_fn(&self) -> bool {
            match self.$time_field {
                Some(t) => t.elapsed().as_secs_f32() < $duration,
                None => false,
            }
        }
    };
}

/// ---------------------- Tab-specific messages ----------------------
/// Generates inline + popup messages for a tab, plus a popup renderer
#[macro_export]
macro_rules! define_tab_messages {
    ($tab:ident, $inline_dur:expr, $popup_dur:expr) => {
        paste! {
            // Inline message
            timed_message!(
                [<set_ $tab _message>],
                [<show_ $tab _message>],
                [<$tab _message>],
                [<$tab _message_time>],
                $inline_dur
            );

            // Popup message
            timed_message!(
                [<set_ $tab _popup_message>],
                [<show_ $tab _popup_message>],
                [<$tab _popup_message>],
                [<$tab _popup_message_time>],
                $popup_dur
            );

            // Popup renderer
            pub fn [<render_ $tab _popup>](&mut self, ctx: &egui::Context) {
                if self.[<show_ $tab _popup_message>]() {
                    egui::Window::new(stringify!([<$tab:upper _Message>]))
                        .collapsible(false)
                        .resizable(false)
                        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                        .show(ctx, |ui| {
                            ui.label(&self.[<$tab _popup_message>]);
                            if ui.button("OK").clicked() {
                                self.[<$tab _popup_message_time>] = None;
                            }
                        });
                }
            }
        }
    };
}

/// ---------------------- Generic active-tab messages ----------------------
/// Generates generic methods for all tabs passed: set/show/clear message & popup
#[macro_export]
macro_rules! define_generic_messages {
    ($(($enum_variant:ident, $name:ident)),+) => {
        paste! {
            impl FileSharingApp {
                pub fn set_message(&mut self, msg: impl Into<String>) {
                    match self.active_tab {
                        $(Tab::$enum_variant => self.[<set_ $name _message>](msg),)+
                    }
                }

                pub fn show_message(&self) -> bool {
                    match self.active_tab {
                        $(Tab::$enum_variant => self.[<show_ $name _message>](),)+
                    }
                }

                pub fn clear_message(&mut self) {
                    match self.active_tab {
                        $(Tab::$enum_variant => self.[<$name _message_time>] = None,)+
                    }
                }

                pub fn set_popup_message(&mut self, msg: impl Into<String>) {
                    match self.active_tab {
                        $(Tab::$enum_variant => self.[<set_ $name _popup_message>](msg),)+
                    }
                }

                pub fn show_popup_message(&self) -> bool {
                    match self.active_tab {
                        $(Tab::$enum_variant => self.[<show_ $name _popup_message>](),)+
                    }
                }

                pub fn clear_popup_message(&mut self) {
                    match self.active_tab {
                        $(Tab::$enum_variant => self.[<$name _popup_message_time>] = None,)+
                    }
                }

                pub fn render_active_popup(&mut self, ctx: &egui::Context) {
                    match self.active_tab {
                        $(Tab::$enum_variant => self.[<render_ $name _popup>](ctx),)+
                    }
                }
            }
        }
    }
}
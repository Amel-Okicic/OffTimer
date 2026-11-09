#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod actions;
mod settings;

use actions::PowerAction;
use settings::AppSettings;
use chrono::Local;
use eframe::egui;
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
struct ThemePalette {
    main_background: egui::Color32, surface_background: egui::Color32, contrast_background: egui::Color32,
    secondary_text: egui::Color32, accent: egui::Color32, control_background: egui::Color32,
    primary_text: egui::Color32, selection_background: egui::Color32, error_text: egui::Color32,
    button_hover_background: egui::Color32, button_active_background: egui::Color32,
    button_text: egui::Color32, button_hover_text: egui::Color32, button_active_text: egui::Color32,
    card_border: egui::Stroke, control_border: Option<egui::Stroke>, hover_border: Option<egui::Stroke>,
    window_border: Option<egui::Stroke>, button_corner_radius: Option<egui::CornerRadius>,
    card_corner_radius: egui::CornerRadius, use_dark_visuals: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Theme { Default, Midnight, Light, RedMoon, WindowsClassic }

impl Theme {
    const ALL: [Self; 5] = [Self::Default, Self::Midnight, Self::Light, Self::RedMoon, Self::WindowsClassic];

    fn label(self) -> &'static str {
        match self {
            Self::Default => "DEFAULT",
            Self::Midnight => "MIDNIGHT",
            Self::Light => "LIGHT",
            Self::RedMoon => "RED MOON",
            Self::WindowsClassic => "WINDOWS CLASSIC",
        }
    }

    fn settings_key(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Midnight => "midnight",
            Self::Light => "light",
            Self::RedMoon => "red_moon",
            Self::WindowsClassic => "windows_classic",
        }
    }

    fn from_settings_key(value: &str) -> Option<Self> {
        match value {
            "default" => Some(Self::Default),
            "midnight" => Some(Self::Midnight),
            "light" => Some(Self::Light),
            "windows_xp" | "red_moon" => Some(Self::RedMoon),
            "windows_classic" => Some(Self::WindowsClassic),
            _ => None,
        }
    }

    fn palette(self) -> ThemePalette {
        match self {
            Self::Default => ThemePalette {
                // Preserve the original DEFAULT colors and widget treatment.
                main_background: egui::Color32::from_rgb(0x00, 0x00, 0x00), surface_background: egui::Color32::from_rgb(0x47, 0x47, 0x47), contrast_background: egui::Color32::from_rgb(0x00, 0x00, 0x00),
                secondary_text: egui::Color32::from_rgb(0x87, 0x87, 0x87), accent: egui::Color32::from_rgb(0xf5, 0xf5, 0xf5), control_background: egui::Color32::from_rgb(0xb5, 0xb5, 0xb5),
                primary_text: egui::Color32::from_rgb(0xff, 0xff, 0xff), selection_background: egui::Color32::from_rgb(0x21, 0x21, 0x21), error_text: egui::Color32::from_rgb(0xff, 0x8b, 0x8b),
                button_hover_background: egui::Color32::from_rgb(0xf5, 0xf5, 0xf5), button_active_background: egui::Color32::from_rgb(0xff, 0xff, 0xff),
                button_text: egui::Color32::from_rgb(0x00, 0x00, 0x00), button_hover_text: egui::Color32::from_rgb(0x00, 0x00, 0x00), button_active_text: egui::Color32::from_rgb(0x00, 0x00, 0x00),
                card_border: egui::Stroke::new(0.0, egui::Color32::TRANSPARENT), control_border: None, hover_border: None, window_border: None,
                button_corner_radius: None, card_corner_radius: egui::CornerRadius::same(12), use_dark_visuals: true,
            },
            Self::Midnight => ThemePalette {
                main_background: egui::Color32::from_rgb(0x0b, 0x0f, 0x14), surface_background: egui::Color32::from_rgb(0x15, 0x1c, 0x25), contrast_background: egui::Color32::from_rgb(0x08, 0x0b, 0x10),
                secondary_text: egui::Color32::from_rgb(0x96, 0xa3, 0xb5), accent: egui::Color32::from_rgb(0x6e, 0xa8, 0xe8), control_background: egui::Color32::from_rgb(0x26, 0x33, 0x43),
                primary_text: egui::Color32::from_rgb(0xea, 0xf1, 0xfa), selection_background: egui::Color32::from_rgb(0x25, 0x49, 0x6f), error_text: egui::Color32::from_rgb(0xff, 0x85, 0x85),
                button_hover_background: egui::Color32::from_rgb(0x33, 0x46, 0x5c), button_active_background: egui::Color32::from_rgb(0x20, 0x32, 0x47),
                button_text: egui::Color32::from_rgb(0xea, 0xf1, 0xfa), button_hover_text: egui::Color32::from_rgb(0xf5, 0xf9, 0xff), button_active_text: egui::Color32::from_rgb(0xf5, 0xf9, 0xff),
                card_border: egui::Stroke::new(1.0, egui::Color32::from_rgb(0x2c, 0x39, 0x48)), control_border: Some(egui::Stroke::new(1.0, egui::Color32::from_rgb(0x42, 0x54, 0x6a))), hover_border: Some(egui::Stroke::new(1.0, egui::Color32::from_rgb(0x6e, 0xa8, 0xe8))),
                window_border: Some(egui::Stroke::new(1.0, egui::Color32::from_rgb(0x2c, 0x39, 0x48))), button_corner_radius: Some(egui::CornerRadius::same(8)),
                card_corner_radius: egui::CornerRadius::same(12), use_dark_visuals: true,
            },
            Self::Light => ThemePalette {
                main_background: egui::Color32::from_rgb(0xf3, 0xf5, 0xf8), surface_background: egui::Color32::from_rgb(0xff, 0xff, 0xff), contrast_background: egui::Color32::from_rgb(0xe8, 0xed, 0xf4),
                secondary_text: egui::Color32::from_rgb(0x63, 0x70, 0x83), accent: egui::Color32::from_rgb(0x25, 0x63, 0xeb), control_background: egui::Color32::from_rgb(0xe8, 0xee, 0xf6),
                primary_text: egui::Color32::from_rgb(0x17, 0x23, 0x38), selection_background: egui::Color32::from_rgb(0xdc, 0xe9, 0xff), error_text: egui::Color32::from_rgb(0xb4, 0x23, 0x18),
                button_hover_background: egui::Color32::from_rgb(0xd9, 0xe7, 0xff), button_active_background: egui::Color32::from_rgb(0xc2, 0xd8, 0xff),
                button_text: egui::Color32::from_rgb(0x17, 0x23, 0x38), button_hover_text: egui::Color32::from_rgb(0x15, 0x3a, 0x78), button_active_text: egui::Color32::from_rgb(0x15, 0x3a, 0x78),
                card_border: egui::Stroke::new(1.0, egui::Color32::from_rgb(0xd7, 0xdf, 0xe9)), control_border: Some(egui::Stroke::new(1.0, egui::Color32::from_rgb(0xa9, 0xbb, 0xd4))), hover_border: Some(egui::Stroke::new(1.0, egui::Color32::from_rgb(0x25, 0x63, 0xeb))),
                window_border: Some(egui::Stroke::new(1.0, egui::Color32::from_rgb(0xd0, 0xd9, 0xe5))), button_corner_radius: Some(egui::CornerRadius::same(8)),
                card_corner_radius: egui::CornerRadius::same(12), use_dark_visuals: false,
            },
            Self::RedMoon => ThemePalette {
                // Deep charcoal and burgundy surfaces with a vivid red-moon accent.
                main_background: egui::Color32::from_rgb(0x12, 0x0c, 0x11), surface_background: egui::Color32::from_rgb(0x21, 0x16, 0x1d), contrast_background: egui::Color32::from_rgb(0x0b, 0x07, 0x0a),
                secondary_text: egui::Color32::from_rgb(0xb9, 0x9e, 0xaa), accent: egui::Color32::from_rgb(0xe0, 0x4b, 0x68), control_background: egui::Color32::from_rgb(0x36, 0x24, 0x2e),
                primary_text: egui::Color32::from_rgb(0xf7, 0xea, 0xef), selection_background: egui::Color32::from_rgb(0x62, 0x27, 0x3d), error_text: egui::Color32::from_rgb(0xff, 0x8b, 0x9b),
                button_hover_background: egui::Color32::from_rgb(0x4b, 0x2a, 0x39), button_active_background: egui::Color32::from_rgb(0x68, 0x2a, 0x43),
                button_text: egui::Color32::from_rgb(0xf7, 0xea, 0xef), button_hover_text: egui::Color32::from_rgb(0xff, 0xf5, 0xf8), button_active_text: egui::Color32::from_rgb(0xff, 0xf5, 0xf8),
                card_border: egui::Stroke::new(1.0, egui::Color32::from_rgb(0x3d, 0x29, 0x34)), control_border: Some(egui::Stroke::new(1.0, egui::Color32::from_rgb(0x61, 0x3d, 0x4e))), hover_border: Some(egui::Stroke::new(1.0, egui::Color32::from_rgb(0xe0, 0x4b, 0x68))),
                window_border: Some(egui::Stroke::new(1.0, egui::Color32::from_rgb(0x3d, 0x29, 0x34))), button_corner_radius: Some(egui::CornerRadius::same(8)),
                card_corner_radius: egui::CornerRadius::same(14), use_dark_visuals: true,
            },
            Self::WindowsClassic => ThemePalette {
                // Windows 98/2000 grey, navy selection and square controls.
                main_background: egui::Color32::from_rgb(0xc0, 0xc0, 0xc0), surface_background: egui::Color32::from_rgb(0xc0, 0xc0, 0xc0), contrast_background: egui::Color32::from_rgb(0xff, 0xff, 0xff),
                secondary_text: egui::Color32::from_rgb(0x40, 0x40, 0x40), accent: egui::Color32::from_rgb(0x00, 0x00, 0x80), control_background: egui::Color32::from_rgb(0xc0, 0xc0, 0xc0),
                primary_text: egui::Color32::from_rgb(0x00, 0x00, 0x00), selection_background: egui::Color32::from_rgb(0x00, 0x00, 0x80), error_text: egui::Color32::from_rgb(0x80, 0x00, 0x00),
                button_hover_background: egui::Color32::from_rgb(0xd4, 0xd0, 0xc8), button_active_background: egui::Color32::from_rgb(0xa8, 0xa8, 0xa8),
                button_text: egui::Color32::from_rgb(0x00, 0x00, 0x00), button_hover_text: egui::Color32::from_rgb(0x00, 0x00, 0x00), button_active_text: egui::Color32::from_rgb(0x00, 0x00, 0x00),
                card_border: egui::Stroke::new(1.0, egui::Color32::from_rgb(0x80, 0x80, 0x80)), control_border: Some(egui::Stroke::new(1.0, egui::Color32::from_rgb(0x40, 0x40, 0x40))), hover_border: Some(egui::Stroke::new(1.0, egui::Color32::from_rgb(0x80, 0x80, 0x80))),
                window_border: Some(egui::Stroke::new(1.0, egui::Color32::from_rgb(0x40, 0x40, 0x40))), button_corner_radius: Some(egui::CornerRadius::ZERO),
                card_corner_radius: egui::CornerRadius::ZERO, use_dark_visuals: false,
            },
        }
    }
}

const QUICK_PRESETS: [(&str, u64, u64); 4] = [
    ("10 MINUTES", 0, 10),
    ("30 MINUTES", 0, 30),
    ("1 HOUR", 1, 0),
    ("2 HOURS", 2, 0),
];

fn main() -> eframe::Result {
    #[cfg(windows)]
    let _single_instance_guard = match actions::acquire_single_instance() {
        Ok(Some(guard)) => guard,
        Ok(None) => {
            actions::activate_existing_instance();
            return Ok(());
        }
        Err(error) => {
            actions::show_single_instance_error(&error);
            std::process::exit(1);
        }
    };

    let app_icon = egui::IconData {
        rgba: include_bytes!(concat!(env!("OUT_DIR"), "/offtimer_icon.rgba")).to_vec(),
        width: 256,
        height: 256,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("OffTimer")
            .with_icon(app_icon)
            .with_inner_size([760.0, 660.0])
            .with_min_inner_size([760.0, 560.0]),
        ..Default::default()
    };

    eframe::run_native(
        "OffTimer",
        options,
        Box::new(|creation_context| Ok(Box::new(OffTimerApp::new(&creation_context.egui_ctx)))),
    )
}

struct OffTimerApp {
    theme: Theme,
    hours_input: String,
    minutes_input: String,
    use_24_hour_clock: bool,
    custom_duration: Option<(u64, u64)>,
    quick_preset_custom_durations: [Option<(u64, u64)>; 4],
    editing_quick_preset: Option<usize>,
    custom_hours_draft: String,
    custom_minutes_draft: String,
    custom_dialog_open: bool,
    custom_dialog_recenter_on_open: bool,
    custom_dialog_error: Option<String>,
    custom_hover_started: Option<Instant>,
    selected_action: PowerAction,
    deadline: Option<Instant>,
    pending_action: Option<PowerAction>,
    windows_countdown_started: bool,
    tray_notification_visible: bool,
    final_minute_notification_sent: bool,
    status_message: Option<String>,
    status_is_error: bool,
    about_dialog_open: bool,
    close_to_tray: bool,
    settings_dirty: bool,
    last_settings_save_failure: Option<Instant>,
    settings_save_error: Option<String>,
    centered_on_launch: bool,
    exit_requested: bool,
    #[cfg(windows)]
    last_power_clock: Option<actions::PowerClockSnapshot>,
    #[cfg(windows)]
    tray_icon: Option<tray_icon::TrayIcon>,
}

impl Default for OffTimerApp {
    fn default() -> Self {
        Self {
            theme: Theme::Default,
            hours_input: String::new(),
            minutes_input: String::new(),
            use_24_hour_clock: true,
            custom_duration: None,
            quick_preset_custom_durations: [None; 4],
            editing_quick_preset: None,
            custom_hours_draft: "0".to_owned(),
            custom_minutes_draft: "30".to_owned(),
            custom_dialog_open: false,
            custom_dialog_recenter_on_open: false,
            custom_dialog_error: None,
            custom_hover_started: None,
            selected_action: PowerAction::Shutdown,
            deadline: None,
            pending_action: None,
            windows_countdown_started: false,
            tray_notification_visible: false,
            final_minute_notification_sent: false,
            status_message: None,
            status_is_error: false,
            about_dialog_open: false,
            close_to_tray: false,
            settings_dirty: false,
            last_settings_save_failure: None,
            settings_save_error: None,
            centered_on_launch: false,
            exit_requested: false,
            #[cfg(windows)]
            last_power_clock: actions::power_clock_snapshot(),
            #[cfg(windows)]
            tray_icon: None,
        }
    }
}

impl OffTimerApp {

    fn new(ctx: &egui::Context) -> Self {
        let mut app = Self::default();

        match settings::load() {
            Ok(Some(settings)) => {
                app.use_24_hour_clock = settings.use_24_hour_clock;
                app.close_to_tray = settings.close_to_tray;
                app.selected_action = settings.selected_action;
                app.theme = Theme::from_settings_key(&settings.theme).unwrap_or(Theme::Default);
                app.custom_duration = settings.custom_duration;
                app.quick_preset_custom_durations = settings.quick_preset_custom_durations;
            }
            Ok(None) => {
                app.settings_dirty = true;
            }
            Err(error) => {
                app.status_message = Some(format!("Could not load settings.json: {error}"));
                app.status_is_error = true;
            }
        }

        #[cfg(windows)]
        if app.close_to_tray {
            match create_offtimer_tray_icon(ctx) {
                Ok(icon) => app.tray_icon = Some(icon),
                Err(error) => {
                    app.close_to_tray = false;
                    app.settings_dirty = true;
                    app.status_message = Some(format!("Could not restore Close to Tray: {error}"));
                    app.status_is_error = true;
                }
            }
        }

        #[cfg(not(windows))]
        if app.close_to_tray {
            app.close_to_tray = false;
            app.settings_dirty = true;
        }

        app.save_settings_if_dirty();
        app
    }

    fn save_settings_if_dirty(&mut self) {
        self.save_settings_if_dirty_with_retry_throttle(true);
    }

    fn save_settings_before_exit(&mut self) {
        // Do not let the retry backoff skip the final attempt during shutdown.
        self.save_settings_if_dirty_with_retry_throttle(false);
    }

    fn save_settings_if_dirty_with_retry_throttle(&mut self, respect_retry_delay: bool) {
        if !self.settings_dirty {
            return;
        }

        // Back off only after a failed write. Successful saves never delay a later change.
        if respect_retry_delay
            && self.last_settings_save_failure
                .is_some_and(|last_failure| last_failure.elapsed() < Duration::from_secs(1))
        {
            return;
        }

        let settings = AppSettings {
            use_24_hour_clock: self.use_24_hour_clock,
            close_to_tray: self.close_to_tray,
            selected_action: self.selected_action,
            theme: self.theme.settings_key().to_owned(),
            custom_duration: self.custom_duration,
            quick_preset_custom_durations: self.quick_preset_custom_durations,
        };

        match settings::save(&settings) {
            Ok(()) => {
                self.settings_dirty = false;
                self.last_settings_save_failure = None;
                self.settings_save_error = None;
            }
            Err(error) => {
                // Preserve the user's current timer/status message and report this failure once.
                self.last_settings_save_failure = Some(Instant::now());
                if self.settings_save_error.is_none() {
                    self.settings_save_error =
                        Some(format!("Could not save settings.json: {error}"));
                }
            }
        }
    }

    fn apply_theme(ctx: &egui::Context, palette: &ThemePalette) {
        let mut visuals = if palette.use_dark_visuals { egui::Visuals::dark() } else { egui::Visuals::light() };
        visuals.panel_fill = palette.main_background;
        visuals.window_fill = palette.surface_background;
        visuals.extreme_bg_color = palette.contrast_background;
        visuals.faint_bg_color = palette.surface_background;
        visuals.code_bg_color = palette.contrast_background;
        visuals.override_text_color = Some(palette.primary_text);
        visuals.weak_text_color = Some(palette.secondary_text);
        visuals.hyperlink_color = palette.accent;
        visuals.selection.bg_fill = palette.selection_background;
        visuals.selection.stroke.color = palette.primary_text;

        visuals.widgets.noninteractive.bg_fill = palette.surface_background;
        visuals.widgets.noninteractive.fg_stroke.color = palette.primary_text;
        visuals.widgets.inactive.bg_fill = palette.control_background;
        visuals.widgets.inactive.weak_bg_fill = palette.control_background;
        visuals.widgets.inactive.fg_stroke.color = palette.button_text;
        visuals.widgets.hovered.bg_fill = palette.button_hover_background;
        visuals.widgets.hovered.weak_bg_fill = palette.button_hover_background;
        visuals.widgets.hovered.fg_stroke.color = palette.button_hover_text;
        visuals.widgets.active.bg_fill = palette.button_active_background;
        visuals.widgets.active.weak_bg_fill = palette.button_active_background;
        visuals.widgets.active.fg_stroke.color = palette.button_active_text;
        visuals.widgets.open.bg_fill = palette.control_background;
        visuals.widgets.open.fg_stroke.color = palette.button_text;

        if let Some(border) = palette.control_border {
            visuals.widgets.inactive.bg_stroke = border;
            visuals.widgets.open.bg_stroke = border;
        }
        if let Some(border) = palette.hover_border {
            visuals.widgets.hovered.bg_stroke = border;
            visuals.widgets.active.bg_stroke = border;
        }
        if let Some(radius) = palette.button_corner_radius {
            visuals.widgets.inactive.corner_radius = radius;
            visuals.widgets.hovered.corner_radius = radius;
            visuals.widgets.active.corner_radius = radius;
            visuals.widgets.open.corner_radius = radius;
        }
        if let Some(border) = palette.window_border {
            visuals.window_stroke = border;
        }

        ctx.set_theme(egui::Theme::Dark);
        ctx.set_visuals(visuals);
        ctx.style_mut_of(egui::Theme::Dark, |style| {
            style.spacing.item_spacing = egui::vec2(10.0, 10.0);
            style.spacing.button_padding = egui::vec2(12.0, 9.0);
        });
    }

    fn apply_menu_popup_style(ui: &mut egui::Ui, palette: &ThemePalette) {
        let visuals = &mut ui.style_mut().visuals;
        visuals.override_text_color = None;
        visuals.widgets.inactive.fg_stroke.color = palette.secondary_text;
        visuals.widgets.hovered.fg_stroke.color = palette.button_hover_text;
        visuals.widgets.active.fg_stroke.color = palette.button_active_text;
        visuals.widgets.open.fg_stroke.color = palette.button_text;
    }

    fn apply_dialog_button_hover_style(ui: &mut egui::Ui, palette: &ThemePalette) {
        let visuals = &mut ui.style_mut().visuals;
        visuals.override_text_color = None;
        visuals.widgets.inactive.fg_stroke.color = palette.primary_text;
        visuals.widgets.hovered.fg_stroke.color = palette.button_hover_text;
        visuals.widgets.active.fg_stroke.color = palette.button_active_text;
        visuals.widgets.open.fg_stroke.color = palette.button_text;
    }

    fn clock_text(&self) -> String {
        let now = Local::now();
        if self.use_24_hour_clock {
            now.format("%H:%M:%S").to_string()
        } else {
            now.format("%I:%M:%S %p").to_string()
        }
    }

    fn is_running(&self) -> bool {
        self.deadline.is_some() || self.windows_countdown_started
    }

    fn set_preset(&mut self, hours: u64, minutes: u64) {
        self.hours_input = hours.to_string();
        self.minutes_input = minutes.to_string();
        self.status_message = None;
        self.status_is_error = false;
    }

    fn open_custom_dialog(&mut self) {
        self.editing_quick_preset = None;
        if let Some((hours, minutes)) = self.custom_duration {
            self.custom_hours_draft = hours.to_string();
            self.custom_minutes_draft = minutes.to_string();
        } else {
            self.custom_hours_draft.clear();
            self.custom_minutes_draft.clear();
        }
        self.custom_dialog_error = None;
        self.custom_dialog_recenter_on_open = true;
        self.custom_dialog_open = true;
    }

    fn open_quick_preset_dialog(&mut self, index: usize) {
        let Some((_, default_hours, default_minutes)) = QUICK_PRESETS.get(index).copied() else {
            return;
        };
        let (hours, minutes) = self.quick_preset_custom_durations[index]
            .unwrap_or((default_hours, default_minutes));

        self.custom_hours_draft = hours.to_string();
        self.custom_minutes_draft = minutes.to_string();
        self.editing_quick_preset = Some(index);
        self.custom_dialog_error = None;
        self.custom_dialog_recenter_on_open = true;
        self.custom_dialog_open = true;
    }

    fn start_timer(&mut self) {
        match parse_duration(&self.hours_input, &self.minutes_input) {
            Ok(duration) => {
                self.deadline = Some(Instant::now() + duration);
                self.pending_action = Some(self.selected_action);
                self.windows_countdown_started = false;
                self.tray_notification_visible = false;
                self.final_minute_notification_sent = false;
                self.status_message = Some(match self.selected_action {
                    PowerAction::Shutdown | PowerAction::Restart => {
                        "Windows will show its own countdown during the final minute.".to_owned()
                    }
                    PowerAction::Sleep | PowerAction::SignOut => {
                        "Keep OffTimer open until the timer finishes.".to_owned()
                    }
                });
                self.status_is_error = false;
            }
            Err(message) => {
                self.status_message = Some(message);
                self.status_is_error = true;
            }
        }
    }

    fn cancel_timer(&mut self) {
        if self.windows_countdown_started {
            if let Err(error) = actions::abort_system_countdown() {
                self.status_message = Some(format!("Could not cancel the Windows shutdown: {error}"));
                self.status_is_error = true;
                return;
            }
        }

        if self.tray_notification_visible {
            actions::remove_tray_notification();
        }

        self.deadline = None;
        self.pending_action = None;
        self.windows_countdown_started = false;
        self.tray_notification_visible = false;
        self.final_minute_notification_sent = false;
        self.hours_input.clear();
        self.minutes_input.clear();
        self.status_message = Some("Timer cancelled. No action is pending.".to_owned());
        self.status_is_error = false;
    }

    #[cfg(windows)]
    fn check_for_sleep_resume(&mut self) {
        let Some(current) = actions::power_clock_snapshot() else {
            return;
        };

        if let Some(previous) = self.last_power_clock.replace(current) {
            let suspended_for = current.sleep_duration_since(previous);
            // The Windows counters have coarse tick resolution. This threshold
            // filters normal sampling jitter while still catching real suspend cycles.
            if suspended_for >= Duration::from_millis(250) && self.is_running() {
                self.cancel_after_sleep_resume();
            }
        }
    }

    #[cfg(windows)]
    fn cancel_after_sleep_resume(&mut self) {
        if self.windows_countdown_started {
            if let Err(error) = actions::abort_system_countdown() {
                // Keep the native-countdown state so the user can still retry
                // cancellation manually from OffTimer.
                self.deadline = None;
                self.status_message = Some(format!(
                    "The PC resumed from sleep, but Windows' pending shutdown could not be cancelled: {error}"
                ));
                self.status_is_error = true;
                return;
            }
        }

        if self.tray_notification_visible {
            actions::remove_tray_notification();
        }

        self.deadline = None;
        self.pending_action = None;
        self.windows_countdown_started = false;
        self.tray_notification_visible = false;
        self.final_minute_notification_sent = false;
        self.hours_input.clear();
        self.minutes_input.clear();
        self.status_message = Some(
            "Timer cancelled because the PC resumed from sleep or hibernation.".to_owned(),
        );
        self.status_is_error = false;
    }

    fn update_countdown(&mut self) {
        let Some(deadline) = self.deadline else {
            return;
        };

        let now = Instant::now();
        let remaining = deadline.saturating_duration_since(now);
        let action = self.pending_action.unwrap_or(self.selected_action);

        match action {
            PowerAction::Shutdown | PowerAction::Restart
                if !self.windows_countdown_started && remaining <= Duration::from_secs(60) =>
            {
                let seconds = remaining
                    .as_secs()
                    .saturating_add(u64::from(remaining.subsec_nanos() > 0))
                    .clamp(1, 60) as u32;

                match actions::begin_system_countdown(action, seconds) {
                    Ok(()) => {
                        self.windows_countdown_started = true;
                        self.status_message = Some(
                            "Windows' native countdown is active. Use CANCEL here to abort it."
                                .to_owned(),
                        );
                        self.status_is_error = false;
                    }
                    Err(error) => {
                        self.deadline = None;
                        self.pending_action = None;
                        self.status_message = Some(format!(
                            "Could not start the Windows countdown: {error}"
                        ));
                        self.status_is_error = true;
                        return;
                    }
                }
            }
            PowerAction::Sleep | PowerAction::SignOut
                if !self.final_minute_notification_sent
                    && !remaining.is_zero()
                    && remaining <= Duration::from_secs(60) =>
            {
                self.final_minute_notification_sent = true;
                let seconds = remaining
                    .as_secs()
                    .saturating_add(u64::from(remaining.subsec_nanos() > 0));

                match actions::show_tray_notification(action, seconds) {
                    Ok(()) => {
                        self.tray_notification_visible = true;
                        self.status_message = Some(
                            "Final-minute notification sent to the Windows notification area."
                                .to_owned(),
                        );
                        self.status_is_error = false;
                    }
                    Err(error) => {
                        self.status_message = Some(format!(
                            "Could not show the Windows tray notification: {error}"
                        ));
                        self.status_is_error = true;
                    }
                }
            }
            _ => {}
        }

        if now < deadline {
            return;
        }

        if self.windows_countdown_started {
            // Windows owns the actual shutdown/restart now. Keep the local
            // Cancel button available in case Windows is still waiting on apps.
            self.deadline = None;
            self.status_message = Some("Windows has started the shutdown sequence.".to_owned());
            self.status_is_error = false;
            return;
        }

        self.deadline = None;
        self.pending_action = None;

        if self.tray_notification_visible {
            actions::remove_tray_notification();
            self.tray_notification_visible = false;
        }

        match actions::execute(action) {
            Ok(()) => {
                self.status_message = Some(format!("{} command sent to Windows.", action.label()));
                self.status_is_error = false;
            }
            Err(error) => {
                self.status_message = Some(format!("Could not perform {}: {}", action.label(), error));
                self.status_is_error = true;
            }
        }

        self.final_minute_notification_sent = false;
    }

    fn countdown_text(&self) -> Option<String> {
        let Some(deadline) = self.deadline else {
            return self
                .windows_countdown_started
                .then(|| "WINDOWS STARTED".to_owned());
        };

        let remaining = deadline.saturating_duration_since(Instant::now());
        let total_seconds = remaining.as_secs() + u64::from(remaining.subsec_nanos() > 0);
        let hours = total_seconds / 3_600;
        let minutes = (total_seconds % 3_600) / 60;
        let seconds = total_seconds % 60;
        Some(format!("{hours:02}:{minutes:02}:{seconds:02}"))
    }

    fn show_about_dialog(&mut self, ctx: &egui::Context, palette: &ThemePalette) {
        if !self.about_dialog_open {
            return;
        }

        let mut open = self.about_dialog_open;
        let original_visuals = ctx.style_of(egui::Theme::Dark).visuals.clone();
        let mut dialog_visuals = original_visuals.clone();
        dialog_visuals.widgets.inactive.fg_stroke.color = palette.primary_text;
        dialog_visuals.widgets.hovered.fg_stroke.color = palette.accent;
        dialog_visuals.widgets.active.fg_stroke.color = palette.accent;
        ctx.set_visuals(dialog_visuals);

        egui::Window::new("ABOUT OFFTIMER")
            .id(egui::Id::new("offtimer-about"))
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                *ui.visuals_mut() = original_visuals.clone();
                ui.set_width(300.0); // fixed content width so wrapping and centering are consistent

                ui.vertical_centered(|ui| {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("OffTimer")
                            .size(22.0)
                            .strong()
                            .color(palette.primary_text),
                    );
                    ui.add_space(8.0);
                    ui.label("OffTimer uses native Windows system APIs to perform power actions.");
                    ui.add_space(4.0);
                    ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                    ui.add_space(16.0);
                    ui.hyperlink_to(
                        "© 2026 Amel Okicic",
                        "https://github.com/Amel-Okicic",
                    );
                    ui.add_space(4.0);
                });
            });

        ctx.set_visuals(original_visuals);
        self.about_dialog_open = open;
    }

    fn show_custom_tooltip(&mut self, ctx: &egui::Context, response: &egui::Response) {
        if response.hovered() {
            let started = self.custom_hover_started.get_or_insert_with(Instant::now);
            if started.elapsed() >= Duration::from_millis(1_500) {
                if let Some(position) = response.hover_pos() {
                    egui::Area::new(egui::Id::new("custom-preset-tooltip"))
                        .order(egui::Order::Tooltip)
                        .fixed_pos(position + egui::vec2(12.0, 18.0))
                        .show(ctx, |ui| {
                            egui::Frame::popup(ui.style()).show(ui, |ui| {
                                ui.label("Right click for customization.");
                            });
                        });
                }
            }
        } else {
            self.custom_hover_started = None;
        }
    }

    fn show_custom_dialog(&mut self, ctx: &egui::Context, palette: &ThemePalette) {
        if !self.custom_dialog_open {
            return;
        }

        let mut open = self.custom_dialog_open;
        let mut hours = self.custom_hours_draft.clone();
        let mut minutes = self.custom_minutes_draft.clone();
        let mut save_clicked = false;
        let mut cancel_clicked = false;
        let mut error = self.custom_dialog_error.clone();
        let editing_quick_preset = self.editing_quick_preset;
        let (window_title, instruction) = if let Some(index) = editing_quick_preset {
            let label = QUICK_PRESETS
                .get(index)
                .map(|preset| preset.0)
                .unwrap_or("QUICK PRESET");
            (
                "CUSTOMIZE QUICK PRESET",
                format!("Choose a duration for the {label} preset."),
            )
        } else {
            (
                "CUSTOM PRESET",
                "Choose a duration for your CUSTOM preset.".to_owned(),
            )
        };

        let dialog_id = egui::Id::new("offtimer-custom-preset");
        let previous_rect = ctx.memory(|memory| memory.area_rect(dialog_id));
        let recenter_position = if self.custom_dialog_recenter_on_open {
            self.custom_dialog_recenter_on_open = false;
            let content_rect = ctx.content_rect();
            let dialog_width = previous_rect.map_or(340.0, |rect| rect.width());
            let dialog_top = previous_rect.map_or(content_rect.top() + 16.0, |rect| rect.top());
            Some(egui::pos2(
                content_rect.center().x - dialog_width * 0.5,
                dialog_top,
            ))
        } else {
            None
        };

        let window = egui::Window::new(window_title)
            .id(dialog_id)
            .collapsible(false)
            .resizable(false)
            .open(&mut open);

        let window = match (recenter_position, previous_rect) {
            (Some(position), Some(_)) => window.current_pos(position),
            (Some(position), None) => window.default_pos(position),
            (None, _) => window,
        };

        // egui draws the title-bar close icon using the interactive widget
        // foreground colors. Change those only while this dialog is being drawn.
        let original_visuals = ctx.style_of(egui::Theme::Dark).visuals.clone();
        let mut dialog_visuals = original_visuals.clone();
        dialog_visuals.widgets.inactive.fg_stroke.color = palette.primary_text;
        dialog_visuals.widgets.hovered.fg_stroke.color = palette.accent;
        dialog_visuals.widgets.active.fg_stroke.color = palette.accent;
        ctx.set_visuals(dialog_visuals);

        window.show(ctx, |ui| {
            // Keep the body controls on their normal theme; the title-bar X
            // above uses the temporary light/accent colors.
            *ui.visuals_mut() = original_visuals.clone();

            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new(instruction).color(palette.secondary_text));
                ui.add_space(8.0);

                let column_width = 112.0;
                let column_gap = ui.spacing().item_spacing.x;
                let fields_group_width = column_width * 2.0 + column_gap;
                ui.horizontal(|ui| {
                    ui.add_space(((ui.available_width() - fields_group_width) * 0.5).max(0.0));
                    ui.horizontal(|ui| {
                        ui.allocate_ui_with_layout(
                            egui::vec2(column_width, 64.0),
                            egui::Layout::top_down(egui::Align::Center),
                            |ui| {
                                ui.label(egui::RichText::new("HOURS").size(11.0).color(palette.secondary_text).strong());
                                ui.add_sized(
                                    [column_width, 40.0],
                                    egui::TextEdit::singleline(&mut hours)
                                        .char_limit(3)
                                        .horizontal_align(egui::Align::Center),
                                );
                            },
                        );
                        ui.allocate_ui_with_layout(
                            egui::vec2(column_width, 64.0),
                            egui::Layout::top_down(egui::Align::Center),
                            |ui| {
                                ui.label(egui::RichText::new("MINUTES").size(11.0).color(palette.secondary_text).strong());
                                ui.add_sized(
                                    [column_width, 40.0],
                                    egui::TextEdit::singleline(&mut minutes)
                                        .char_limit(2)
                                        .horizontal_align(egui::Align::Center),
                                );
                            },
                        );
                    });
                });

                if let Some(message) = &error {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(message).color(palette.error_text));
                }

                ui.add_space(12.0);
                ui.scope(|ui| {
                    Self::apply_dialog_button_hover_style(ui, palette);
                    let buttons_group_width = 112.0 * 2.0 + ui.spacing().item_spacing.x;
                    ui.horizontal(|ui| {
                        ui.add_space(((ui.available_width() - buttons_group_width) * 0.5).max(0.0));
                        ui.horizontal(|ui| {
                            if ui.add_sized([112.0, 38.0], egui::Button::new("SAVE")).clicked() {
                                save_clicked = true;
                            }
                            if ui.add_sized([112.0, 38.0], egui::Button::new("CANCEL")).clicked() {
                                cancel_clicked = true;
                            }
                        });
                    });
                });
            });
        });

        ctx.set_visuals(original_visuals);

        self.custom_hours_draft = hours.clone();
        self.custom_minutes_draft = minutes.clone();
        self.custom_dialog_open = open && !cancel_clicked;

        if cancel_clicked || !open {
            self.custom_dialog_error = None;
            self.editing_quick_preset = None;
        } else if save_clicked {
            match parse_duration(&hours, &minutes) {
                Ok(_) => match (parse_optional_number(&hours), parse_optional_number(&minutes)) {
                    (Ok(h), Ok(m)) => {
                        if let Some(index) = editing_quick_preset {
                            let (label, default_hours, default_minutes) = QUICK_PRESETS[index];
                            self.quick_preset_custom_durations[index] =
                                if (h, m) == (default_hours, default_minutes) {
                                    None
                                } else {
                                    Some((h, m))
                                };
                            self.status_message = Some(format!("{label} preset updated."));
                        } else {
                            self.custom_duration = Some((h, m));
                            self.status_message = Some("Custom preset saved.".to_owned());
                        }
                        self.custom_dialog_open = false;
                        self.custom_dialog_error = None;
                        self.editing_quick_preset = None;
                        self.status_is_error = false;
                        self.settings_dirty = true;
                    }
                    _ => {
                        self.custom_dialog_error = Some("Enter whole numbers only.".to_owned());
                    }
                },
                Err(message) => {
                    self.custom_dialog_error = Some(message);
                }
            }
        } else {
            self.custom_dialog_error = error.take();
        }
    }
}

impl eframe::App for OffTimerApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.centered_on_launch {
            if let Some(command) = egui::ViewportCommand::center_on_screen(ctx) {
                ctx.send_viewport_cmd(command);
                self.centered_on_launch = true;
            }
        }

        let close_requested = ctx.input(|input| input.viewport().close_requested());
        if close_requested && self.close_to_tray && !self.exit_requested {
            // Keep the process alive, but hide the native window immediately.
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            #[cfg(windows)]
            if let Err(error) = actions::hide_main_window() {
                self.status_message = Some(format!("Could not hide OffTimer: {error}"));
                self.status_is_error = true;
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }

        #[cfg(windows)]
        self.check_for_sleep_resume();

        self.update_countdown();
        ctx.request_repaint_after(if self.is_running() {
            Duration::from_millis(200)
        } else {
            Duration::from_millis(500)
        });
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let selected_action_before_ui = self.selected_action;
        let palette = self.theme.palette();
        Self::apply_theme(&ctx, &palette);
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(palette.main_background).inner_margin(egui::Margin::same(24)))
            .show(ui, |ui| {
                let menu_available = ui.available_rect_before_wrap();
                // The right-side controls are scaled up by 80%; keep the menu row tall enough.
                let menu_height = 50.4;
                let menu_rect = egui::Rect::from_min_size(
                    egui::pos2(menu_available.min.x, menu_available.min.y - 20.0),
                    egui::vec2(menu_available.width(), menu_height),
                );
                let left_menu_rect = egui::Rect::from_min_size(
                    menu_rect.min,
                    egui::vec2(125.0_f32.min(menu_rect.width()), menu_height),
                );
                let right_menu_rect = egui::Rect::from_min_size(
                    egui::pos2(left_menu_rect.max.x, menu_rect.min.y),
                    egui::vec2((menu_rect.max.x - left_menu_rect.max.x - 12.0).max(0.0), menu_height),
                );

                let mut left_menu_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .id_salt("offtimer-main-menu")
                        .max_rect(left_menu_rect)
                        .layout(egui::Layout::left_to_right(egui::Align::Center)),
                );
                left_menu_ui.scope(|ui| {
                    {
                        let visuals = &mut ui.style_mut().visuals;
                        visuals.override_text_color = None;
                        visuals.widgets.inactive.fg_stroke.color = palette.secondary_text;
                        visuals.widgets.hovered.fg_stroke.color = palette.contrast_background;
                        visuals.widgets.active.fg_stroke.color = palette.contrast_background;
                        visuals.widgets.open.fg_stroke.color = palette.contrast_background;
                    }

                    egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("FILE", |ui| {
                        Self::apply_menu_popup_style(ui, &palette);

                        ui.menu_button("Theme", |ui| {
                            Self::apply_menu_popup_style(ui, &palette);

                            for theme in Theme::ALL {
                                let label = theme.label();
                                let item_color = if self.theme == theme {
                                    palette.accent
                                } else {
                                    palette.primary_text
                                };
                                let item_text = egui::RichText::new(label).color(item_color);
                                let item_text = if self.theme == theme {
                                    item_text.strong()
                                } else {
                                    item_text
                                };

                                if ui.button(item_text).clicked() {
                                    if self.theme != theme {
                                        self.theme = theme;
                                        self.settings_dirty = true;
                                    }
                                    ui.close();
                                }
                            }
                        });

                        ui.separator();

                        if ui.button("Exit").clicked() {
                            self.exit_requested = true;
                            self.close_to_tray = false;
                            #[cfg(windows)]
                            {
                                self.tray_icon = None;
                            }
                            ui.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });

                    ui.menu_button("HELP", |ui| {
                        Self::apply_menu_popup_style(ui, &palette);
                        if ui.button("About OffTimer").clicked() {
                            self.about_dialog_open = true;
                            ui.close();
                        }
                    });
                    });
                });
                
                let mut right_menu_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .id_salt("offtimer-menu-right")
                        .max_rect(right_menu_rect)
                        .layout(egui::Layout::right_to_left(egui::Align::Center)),
                );

                let close_to_tray_response = draw_menu_toggle(
                    &mut right_menu_ui,
                    "CLOSE TO TRAY",
                    if self.close_to_tray { "ON" } else { "OFF" },
                    self.close_to_tray,
                    &palette,
                );
                let close_to_tray_before = self.close_to_tray;
                if close_to_tray_response.clicked() {
                    let enable = !self.close_to_tray;
                    #[cfg(windows)]
                    {
                        if enable {
                            match create_offtimer_tray_icon(&ctx) {
                                Ok(icon) => {
                                    self.tray_icon = Some(icon);
                                    self.close_to_tray = true;
                                    self.status_message = Some(
                                        "Close to Tray is enabled. The X button will hide OffTimer."
                                            .to_owned(),
                                    );
                                    self.status_is_error = false;
                                }
                                Err(error) => {
                                    self.close_to_tray = false;
                                    self.status_message = Some(format!(
                                        "Could not enable Close to Tray: {error}"
                                    ));
                                    self.status_is_error = true;
                                }
                            }
                        } else {
                            self.close_to_tray = false;
                            self.tray_icon = None;
                            self.status_message =
                                Some("Close to Tray is disabled. The X button will exit OffTimer.".to_owned());
                            self.status_is_error = false;
                        }
                    }
                    #[cfg(not(windows))]
                    {
                        let _ = enable;
                        self.status_message =
                            Some("Close to Tray is available on Windows only.".to_owned());
                        self.status_is_error = true;
                    }
                }
                if self.close_to_tray != close_to_tray_before {
                    self.settings_dirty = true;
                }

                right_menu_ui.add_space(30.0);
                let clock_format_response = draw_menu_toggle(
                    &mut right_menu_ui,
                    "CLOCK FORMAT",
                    if self.use_24_hour_clock { "24H" } else { "12H" },
                    self.use_24_hour_clock,
                    &palette,
                );
                if clock_format_response.clicked() {
                    self.use_24_hour_clock = !self.use_24_hour_clock;
                    self.settings_dirty = true;
                }

                ui.advance_cursor_after_rect(menu_rect);
                ui.add_space(20.0);

                let header_height = 88.0;
                let header_available = ui.available_rect_before_wrap();
                let header_rect = egui::Rect::from_min_size(
                    header_available.min,
                    egui::vec2(header_available.width(), header_height),
                );

                let mut clock_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .id_salt("offtimer-live-clock")
                        .max_rect(header_rect)
                        .layout(egui::Layout::centered_and_justified(
                            egui::Direction::TopDown,
                        )),
                );
                clock_ui.label(
                    egui::RichText::new(self.clock_text())
                        .size(72.0)
                        .strong()
                        .color(palette.primary_text),
                );

                ui.advance_cursor_after_rect(header_rect);
                ui.add_space(26.0);

                let cards_rect = ui.available_rect_before_wrap();
                let gap = 16.0;
                let left_width = (cards_rect.width() - gap) * 0.35;
                let right_width = cards_rect.width() - gap - left_width;
                let panel_height = cards_rect.height();
                let left_rect = egui::Rect::from_min_size(
                    cards_rect.min,
                    egui::vec2(left_width, panel_height),
                );
                let right_rect = egui::Rect::from_min_size(
                    egui::pos2(left_rect.max.x + gap, cards_rect.min.y),
                    egui::vec2(right_width, panel_height),
                );
                let running = self.is_running();
                let mut timer_input_has_focus = false;

                let mut left_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .id_salt("offtimer-quick-presets")
                        .max_rect(left_rect)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                );
                egui::Frame::new()
                                .fill(palette.surface_background)
                                .corner_radius(palette.card_corner_radius)
                                .stroke(palette.card_border)
                                .inner_margin(egui::Margin::same(22))
                                .show(&mut left_ui, |ui| {
                                    ui.set_min_width(left_width - 44.0);
                                    ui.set_min_height(panel_height - 44.0);
                                    ui.with_layout(
                                        egui::Layout::top_down(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                egui::RichText::new("QUICK PRESETS")
                                                    .size(16.0)
                                                    .strong()
                                                    .color(palette.secondary_text),
                                            );
                                        },
                                    );
                                    ui.add_space(14.0);

                                    let button_width = ui.available_width().max(120.0);
                                    let mut hovered_quick_preset: Option<egui::Response> = None;
                                    for (index, (label, default_hours, default_minutes)) in
                                        QUICK_PRESETS.iter().enumerate()
                                    {
                                        let custom_duration = self.quick_preset_custom_durations[index];
                                        let (hours, minutes) = custom_duration
                                            .unwrap_or((*default_hours, *default_minutes));
                                        let button_label = if custom_duration.is_some() {
                                            format_custom_label(hours, minutes)
                                        } else {
                                            (*label).to_owned()
                                        };

                                        let response = ui
                                            .add_enabled_ui(!running, |ui| {
                                                ui.add_sized(
                                                    [button_width, 54.0],
                                                    egui::Button::new(
                                                        egui::RichText::new(button_label)
                                                            .size(14.0)
                                                            .strong(),
                                                    ),
                                                )
                                            })
                                            .inner;

                                        if response.secondary_clicked() {
                                            self.open_quick_preset_dialog(index);
                                        } else if response.clicked() {
                                            self.set_preset(hours, minutes);
                                        }

                                        if response.hovered() {
                                            hovered_quick_preset = Some(response);
                                        }
                                    }

                                    let custom_label = self
                                        .custom_duration
                                        .map(|(h, m)| format_custom_label(h, m))
                                        .unwrap_or_else(|| "CUSTOM".to_owned());

                                    let custom_response = ui
                                        .add_enabled_ui(!running, |ui| {
                                            ui.add_sized(
                                                [button_width, 54.0],
                                                egui::Button::new(
                                                    egui::RichText::new(custom_label)
                                                        .size(14.0)
                                                        .strong(),
                                                ),
                                            )
                                        })
                                        .inner;

                                    if custom_response.secondary_clicked() {
                                        self.open_custom_dialog();
                                    } else if custom_response.clicked() {
                                        if let Some((hours, minutes)) = self.custom_duration {
                                            self.set_preset(hours, minutes);
                                        } else {
                                            self.open_custom_dialog();
                                        }
                                    }

                                    if custom_response.hovered() {
                                        self.show_custom_tooltip(&ctx, &custom_response);
                                    } else if let Some(response) = hovered_quick_preset.as_ref() {
                                        self.show_custom_tooltip(&ctx, response);
                                    } else {
                                        self.custom_hover_started = None;
                                    }
                                });

                let mut right_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .id_salt("offtimer-set-timer")
                        .max_rect(right_rect)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                );
                egui::Frame::new()
                                .fill(palette.surface_background)
                                .corner_radius(palette.card_corner_radius)
                                .stroke(palette.card_border)
                                .inner_margin(egui::Margin::same(22))
                                .show(&mut right_ui, |ui| {
                                    ui.set_min_width(right_width - 44.0);
                                    ui.set_min_height(panel_height - 44.0);

                                    ui.with_layout(
                                        egui::Layout::top_down(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                egui::RichText::new("SET TIMER")
                                                    .size(16.0)
                                                    .strong()
                                                    .color(palette.secondary_text),
                                            );
                                        },
                                    );
                                    ui.add_space(10.0);

                                    let field_width = ((ui.available_width() - 14.0) / 2.0)
                                        .clamp(92.0, 190.0);

                                    ui.horizontal(|ui| {
                                        let input_row_width =
                                            field_width * 2.0 + ui.spacing().item_spacing.x;
                                        ui.add_space(
                                            ((ui.available_width() - input_row_width).max(0.0))
                                                / 2.0,
                                        );

                                        ui.vertical(|ui| {
                                            ui.label(
                                                egui::RichText::new("HOURS")
                                                    .size(11.0)
                                                    .strong()
                                                    .color(palette.secondary_text),
                                            );
                                            let response = ui
                                                .add_enabled_ui(!running, |ui| {
                                                    ui.add_sized(
                                                        [field_width, 48.0],
                                                        egui::TextEdit::singleline(&mut self.hours_input)
                                                            .font(egui::FontId::proportional(22.0))
                                                            .char_limit(3)
                                                            .horizontal_align(egui::Align::Center),
                                                    )
                                                })
                                                .inner;
                                            timer_input_has_focus |= response.has_focus() || response.lost_focus();
                                        });

                                        ui.vertical(|ui| {
                                            ui.label(
                                                egui::RichText::new("MINUTES")
                                                    .size(11.0)
                                                    .strong()
                                                    .color(palette.secondary_text),
                                            );
                                            let response = ui
                                                .add_enabled_ui(!running, |ui| {
                                                    ui.add_sized(
                                                        [field_width, 48.0],
                                                        egui::TextEdit::singleline(&mut self.minutes_input)
                                                            .font(egui::FontId::proportional(22.0))
                                                            .char_limit(2)
                                                            .horizontal_align(egui::Align::Center),
                                                    )
                                                })
                                                .inner;
                                            timer_input_has_focus |= response.has_focus() || response.lost_focus();
                                        });
                                    });

                                    ui.add_space(18.0);

                                    if let Some(countdown) = self.countdown_text() {
                                        ui.with_layout(
                                            egui::Layout::top_down(egui::Align::Center),
                                            |ui| {
                                                ui.label(
                                                    egui::RichText::new(countdown)
                                                        .size(40.0)
                                                        .strong()
                                                        .color(palette.accent),
                                                );
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "{} will run when the countdown reaches zero.",
                                                        self.pending_action
                                                            .map(PowerAction::label)
                                                            .unwrap_or("Selected action")
                                                    ))
                                                    .size(11.0)
                                                    .color(palette.secondary_text),
                                                );
                                            },
                                        );
                                    } else {
                                        ui.with_layout(
                                            egui::Layout::top_down(egui::Align::Center),
                                            |ui| {
                                                ui.label(
                                                    egui::RichText::new("No timer set.")
                                                        .size(14.0)
                                                        .color(palette.secondary_text),
                                                );
                                            },
                                        );
                                        ui.add_space(22.0);
                                    }

                                    ui.add_space(12.0);

                                    let button_width = ((ui.available_width() - 12.0) / 2.0)
                                        .max(110.0);
                                    ui.horizontal(|ui| {
                                        if ui
                                            .add_enabled(
                                                !running,
                                                egui::Button::new(
                                                    egui::RichText::new("CONFIRM")
                                                        .strong()
                                                        .size(14.0),
                                                )
                                                .min_size(egui::vec2(button_width, 48.0)),
                                            )
                                            .clicked()
                                        {
                                            self.start_timer();
                                        }

                                        if ui
                                            .add_sized(
                                                [button_width, 48.0],
                                                egui::Button::new(
                                                    egui::RichText::new("CANCEL")
                                                        .strong()
                                                        .size(14.0),
                                                ),
                                            )
                                            .clicked()
                                        {
                                            self.cancel_timer();
                                        }
                                    });

                                    ui.add_space(20.0);
                                    ui.vertical_centered(|ui| {
                                        ui.label(
                                            egui::RichText::new("ACTION")
                                                .size(11.0)
                                                .strong()
                                                .color(palette.secondary_text),
                                        );
                                    });

                                    ui.add_enabled_ui(!running, |ui| {
                                        ui.spacing_mut().item_spacing.x += 5.0;
                                        ui.vertical_centered(|ui| {
                                            ui.horizontal_wrapped(|ui| {
                                                let selected = self.selected_action;
                                                ui.radio_value(
                                                    &mut self.selected_action,
                                                    PowerAction::Shutdown,
                                                    egui::RichText::new("SHUT DOWN").color(
                                                        if selected == PowerAction::Shutdown {
                                                            palette.accent
                                                        } else {
                                                            palette.secondary_text
                                                        },
                                                    ),
                                                );
                                                let selected = self.selected_action;
                                                ui.radio_value(
                                                    &mut self.selected_action,
                                                    PowerAction::Sleep,
                                                    egui::RichText::new("SLEEP").color(
                                                        if selected == PowerAction::Sleep {
                                                            palette.accent
                                                        } else {
                                                            palette.secondary_text
                                                        },
                                                    ),
                                                );
                                                let selected = self.selected_action;
                                                ui.radio_value(
                                                    &mut self.selected_action,
                                                    PowerAction::SignOut,
                                                    egui::RichText::new("SIGN OUT").color(
                                                        if selected == PowerAction::SignOut {
                                                            palette.accent
                                                        } else {
                                                            palette.secondary_text
                                                        },
                                                    ),
                                                );
                                                let selected = self.selected_action;
                                                ui.radio_value(
                                                    &mut self.selected_action,
                                                    PowerAction::Restart,
                                                    egui::RichText::new("RESTART").color(
                                                        if selected == PowerAction::Restart {
                                                            palette.accent
                                                        } else {
                                                            palette.secondary_text
                                                        },
                                                    ),
                                                );
                                            });
                                        });
                                    });

                                    if let Some(message) = &self.status_message {
                                        ui.add_space(16.0);
                                        ui.label(
                                            egui::RichText::new(message)
                                                .size(11.0)
                                                .color(if self.status_is_error { palette.error_text } else { palette.secondary_text }),
                                        );
                                    }

                                    if let Some(error) = &self.settings_save_error {
                                        ui.add_space(if self.status_message.is_some() { 4.0 } else { 16.0 });
                                        ui.label(
                                            egui::RichText::new(error)
                                                .size(11.0)
                                                .color(palette.error_text),
                                        );
                                    }
                                });

                ui.advance_cursor_after_rect(cards_rect);

                let enter_pressed = ui.input(|input| input.key_pressed(egui::Key::Enter));
                if enter_pressed && timer_input_has_focus && !self.is_running() && !self.custom_dialog_open {
                    self.start_timer();
                }
            });

        self.show_custom_dialog(&ctx, &palette);
        self.show_about_dialog(&ctx, &palette);

        if self.selected_action != selected_action_before_ui {
            self.settings_dirty = true;
        }
        self.save_settings_if_dirty();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_settings_before_exit();
    }
}

fn draw_menu_toggle(
    ui: &mut egui::Ui,
    label: &str,
    value_text: &str,
    enabled: bool,
    palette: &ThemePalette,
) -> egui::Response {
    const CONTROL_SCALE: f32 = 1.0;
    const LABEL_BUTTON_GAP: f32 = 12.0;
    const SWITCH_WIDTH: f32 = 60.0;

    let menu_font = ui.style().text_styles[&egui::TextStyle::Button].clone();
    // Match the native FILE/HELP menu-button height: button text.
    let menu_element_height = menu_font.size + ui.spacing().button_padding.y * 2.0 * 0.5;
    let switch_radius = menu_element_height / 2.0;
    let thumb_radius = (menu_element_height * 0.37).min(12.5);
    let control_font = egui::FontId::proportional(menu_font.size * CONTROL_SCALE);
    let label_width = ui
        .painter()
        .layout_no_wrap(label.to_owned(), control_font.clone(), palette.secondary_text)
        .size()
        .x;

    let total_width = label_width + LABEL_BUTTON_GAP + SWITCH_WIDTH;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(total_width, menu_element_height),
        egui::Sense::hover(),
    );
    let mut content_ui = ui.new_child(
        egui::UiBuilder::new()
            .id_salt(label)
            .max_rect(rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );

    content_ui.label(
        egui::RichText::new(label)
            .font(control_font.clone())
            .color(palette.secondary_text),
    );
    content_ui.add_space(LABEL_BUTTON_GAP);

    let (switch_rect, _) = content_ui.allocate_exact_size(
        egui::vec2(SWITCH_WIDTH, menu_element_height),
        egui::Sense::hover(),
    );
    let switch_id = content_ui.id().with("switch");
    let response = content_ui.interact(switch_rect, switch_id, egui::Sense::click());

    // Draw a pill-shaped track and a separate sliding thumb: this is a real
    // toggle switch, not a rectangular push button.
    let track_fill = if enabled { palette.accent } else { palette.surface_background };
    let thumb_fill = if enabled { palette.contrast_background } else { palette.accent };
    let text_color = if enabled { palette.contrast_background } else { palette.primary_text };
    let painter = content_ui.painter();
    painter.rect_filled(
        switch_rect,
        egui::CornerRadius::same(switch_radius as u8),
        track_fill,
    );

    let thumb_x = if enabled {
        switch_rect.right() - switch_radius
    } else {
        switch_rect.left() + switch_radius
    };
    painter.circle_filled(
        egui::pos2(thumb_x, switch_rect.center().y),
        thumb_radius,
        thumb_fill,
    );

    // Center the state text exactly halfway between the outer track edge
    // and the nearest edge of the thumb, in the half opposite the thumb.
    let text_x = if enabled {
        (switch_rect.left() + thumb_x - thumb_radius) / 2.0
    } else {
        (switch_rect.right() + thumb_x + thumb_radius) / 2.0
    };
    painter.text(
        egui::pos2(text_x, switch_rect.center().y),
        egui::Align2::CENTER_CENTER,
        value_text,
        control_font,
        text_color,
    );

    response
}

#[cfg(windows)]
fn create_offtimer_tray_icon(ctx: &egui::Context) -> Result<tray_icon::TrayIcon, String> {
    use tray_icon::{Icon, MouseButton, TrayIconBuilder, TrayIconEvent};

    // Use the exact same generated OffTimer logo asset as the application window icon.
    const SIDE: u32 = 256;
    let rgba = include_bytes!(concat!(env!("OUT_DIR"), "/offtimer_icon.rgba")).to_vec();
    let icon = Icon::from_rgba(rgba, SIDE, SIDE)
        .map_err(|error| format!("Could not create the tray icon: {error}"))?;

    let restore_context = ctx.clone();
    TrayIconEvent::set_event_handler(Some(move |event| {
        let restore = matches!(
            event,
            TrayIconEvent::Click { button: MouseButton::Left, .. }
                | TrayIconEvent::DoubleClick { button: MouseButton::Left, .. }
        );

        if restore {
            restore_context.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            restore_context.send_viewport_cmd(egui::ViewportCommand::Focus);
            restore_context.request_repaint();
        }
    }));

    TrayIconBuilder::new()
        .with_tooltip("OffTimer - click to restore")
        .with_icon(icon)
        .with_menu_on_left_click(false)
        .with_menu_on_right_click(false)
        .build()
        .map_err(|error| format!("Could not create the Windows tray icon: {error}"))
}

fn parse_optional_number(value: &str) -> Result<u64, String> {
    let value = value.trim();
    if value.is_empty() {
        Ok(0)
    } else {
        value
            .parse::<u64>()
            .map_err(|_| "Enter whole numbers only.".to_owned())
    }
}

fn parse_duration(hours: &str, minutes: &str) -> Result<Duration, String> {
    let hours = parse_optional_number(hours)
        .map_err(|_| "Enter a whole number of hours.".to_owned())?;
    let minutes = parse_optional_number(minutes)
        .map_err(|_| "Enter a whole number of minutes.".to_owned())?;

    if hours > 999 {
        return Err("Hours must be 999 or less.".to_owned());
    }
    if minutes > 59 {
        return Err("Minutes must be between 0 and 59.".to_owned());
    }

    let total_seconds = hours * 3_600 + minutes * 60;
    if total_seconds == 0 {
        return Err("Choose a duration greater than zero.".to_owned());
    }

    Ok(Duration::from_secs(total_seconds))
}

fn format_custom_label(hours: u64, minutes: u64) -> String {
    match (hours, minutes) {
        (0, m) => format!("{m} MIN"),
        (h, 0) => format!("{h} H"),
        (h, m) => format!("{h} H {m} MIN"),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_five_theme_palettes_are_defined() {
        assert_eq!(Theme::ALL.len(), 5);
        for theme in Theme::ALL {
            let palette = theme.palette();
            assert_ne!(palette.main_background, egui::Color32::TRANSPARENT);
            assert_ne!(palette.surface_background, egui::Color32::TRANSPARENT);
            assert_ne!(palette.primary_text, egui::Color32::TRANSPARENT);
            assert_ne!(palette.control_background, egui::Color32::TRANSPARENT);
        }
    }

    #[test]
    fn default_theme_preserves_original_palette_values_and_widget_defaults() {
        let palette = Theme::Default.palette();
        assert_eq!(palette.main_background, egui::Color32::from_rgb(0x00, 0x00, 0x00));
        assert_eq!(palette.surface_background, egui::Color32::from_rgb(0x47, 0x47, 0x47));
        assert_eq!(palette.contrast_background, egui::Color32::from_rgb(0x00, 0x00, 0x00));
        assert_eq!(palette.secondary_text, egui::Color32::from_rgb(0x87, 0x87, 0x87));
        assert_eq!(palette.accent, egui::Color32::from_rgb(0xf5, 0xf5, 0xf5));
        assert_eq!(palette.control_background, egui::Color32::from_rgb(0xb5, 0xb5, 0xb5));
        assert_eq!(palette.primary_text, egui::Color32::from_rgb(0xff, 0xff, 0xff));
        assert_eq!(palette.selection_background, egui::Color32::from_rgb(0x21, 0x21, 0x21));
        assert_eq!(palette.error_text, egui::Color32::from_rgb(0xff, 0x8b, 0x8b));
        assert!(palette.button_corner_radius.is_none());
        assert!(palette.control_border.is_none());
        assert!(palette.window_border.is_none());
        assert_eq!(palette.card_corner_radius, egui::CornerRadius::same(12));
        assert!(palette.use_dark_visuals);
    }

    #[test]
    fn red_moon_and_classic_have_distinct_widget_shapes() {
        let red_moon = Theme::RedMoon.palette();
        let classic = Theme::WindowsClassic.palette();
        assert_eq!(red_moon.main_background, egui::Color32::from_rgb(0x12, 0x0c, 0x11));
        assert_eq!(red_moon.accent, egui::Color32::from_rgb(0xe0, 0x4b, 0x68));
        assert_eq!(red_moon.button_corner_radius, Some(egui::CornerRadius::same(8)));
        assert_eq!(red_moon.card_corner_radius, egui::CornerRadius::same(14));
        assert!(red_moon.control_border.is_some());
        assert!(red_moon.use_dark_visuals);
        assert_eq!(classic.button_corner_radius, Some(egui::CornerRadius::ZERO));
        assert_eq!(classic.card_corner_radius, egui::CornerRadius::ZERO);
        assert!(classic.control_border.is_some());
        assert_eq!(classic.accent, egui::Color32::from_rgb(0x00, 0x00, 0x80));
    }

    #[test]
    fn light_and_midnight_use_their_respective_visual_bases() {
        assert!(!Theme::Light.palette().use_dark_visuals);
        assert!(Theme::Midnight.palette().use_dark_visuals);
    }

    #[test]
    fn parses_valid_duration() {
        assert_eq!(parse_duration("1", "30").unwrap(), Duration::from_secs(5_400));
        assert_eq!(parse_duration("0", "10").unwrap(), Duration::from_secs(600));
    }

    #[test]
    fn accepts_blank_fields_as_zero() {
        assert_eq!(parse_duration("1", "").unwrap(), Duration::from_secs(3_600));
        assert_eq!(parse_duration("", "15").unwrap(), Duration::from_secs(900));
        assert!(parse_duration("", "").is_err());
    }

    #[test]
    fn rejects_invalid_minutes_and_zero_duration() {
        assert!(parse_duration("1", "60").is_err());
        assert!(parse_duration("0", "0").is_err());
    }

    #[test]
    fn formats_custom_label_compactly() {
        assert_eq!(format_custom_label(1, 30), "1 H 30 MIN");
        assert_eq!(format_custom_label(2, 0), "2 H");
        assert_eq!(format_custom_label(0, 10), "10 MIN");
    }
}

use cosmic::app::{Core, Task};
use cosmic::iced::time;
use cosmic::iced::Subscription;
use cosmic::iced::Color;
use cosmic::widget::{button, text, row, column, mouse_area, container};
use cosmic::{Application, Element};
use std::time::Duration;
use cosmic::iced::window;
use zbus::blocking::{Connection, Proxy};


#[derive(Default)]
pub struct BatteryApplet {
    core: Core,
    battery_percentage: u32,
    is_charging: bool,
    time_remaining: Option<f32>,
    active_profile: String,
    brightness_percent: u32,
    accent_color: Option<Color>,
    popup: Option<window::Id>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    ToggleMenu,
    SetProfile(String),
    SetBrightness(u32),
    SetAccentColor(Option<Color>),
}

impl Application for BatteryApplet {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "com.github.pewmoe.cosmic-ascii-battery";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let mut applet = BatteryApplet {
            core,
            battery_percentage: 100,
            is_charging: false,
            time_remaining: None,
            active_profile: String::from("balanced"),
            brightness_percent: 100,
            accent_color: None,
            popup: None,
        };

        applet.update_battery();
        applet.update_profile();
        applet.update_brightness();

        (applet, Task::none())
    }
   fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
    match message {
        Message::Tick => {
           //(nice)
            self.update_battery();
            self.update_profile();
            self.update_brightness();
        }

        Message::ToggleMenu => {
            if let Some(popup_id) = self.popup.take() {
                return cosmic::iced::platform_specific::shell::commands::popup::destroy_popup::<Message>(
              //nice
                    popup_id,
                )
                .map(cosmic::Action::from);
            }

            let new_id = window::Id::unique();
            self.popup = Some(new_id);

            let mut popup_settings = self.core.applet.get_popup_settings(
                self.core.main_window_id().unwrap(),
                new_id,
                Some((500, 300)),
                None,
                None,
            );

            popup_settings.positioner.size_limits = cosmic::iced::Limits::NONE
                .min_width(360.0)
                .min_height(180.0)
                .max_width(500.0)
                .max_height(400.0);

            return cosmic::iced::platform_specific::shell::commands::popup::get_popup::<Message>(
                popup_settings,
            )
            .map(cosmic::Action::from);
        }

        Message::SetProfile(profile) => {
            set_active_profile(&profile).ok();
            self.active_profile = profile;

            if let Some(popup_id) = self.popup.take() {
                return cosmic::iced::platform_specific::shell::commands::popup::destroy_popup::<Message>(
                    popup_id,
                )
                .map(cosmic::Action::from);
            }
        }

        Message::SetBrightness(pct) => {
            let pct = pct.min(100);
            set_brightness_percent(pct).ok();
            self.brightness_percent = pct;
        }

        Message::SetAccentColor(color) => {
            self.accent_color = color;
        }
    }

    Task::none()
}
    fn subscription(&self) -> Subscription<Self::Message> {
        time::every(Duration::from_secs(5)).map(|_| Message::Tick)
    }

    // 1. Panel view: Compact block bar, sized off the panel's own configured size
   fn view(&self) -> Element<'_, Self::Message> {
    let pct = self.battery_percentage.min(100);

    let text_color = self.accent_color.unwrap_or_else(|| match pct {
        0..=20 => Color::from_rgb8(255, 60, 60),
        21..=50 => Color::from_rgb8(255, 165, 0),
        51..=70 => Color::from_rgb8(255, 220, 0),
        _ => Color::from_rgb8(180, 220, 255),
    });

    let total_blocks: usize = 8;
    let filled_count =
        ((pct as f32 / 100.0) * total_blocks as f32).round() as usize;
    let empty_count = total_blocks.saturating_sub(filled_count);

    let filled = "█".repeat(filled_count);
    let empty = "░".repeat(empty_count);

 let status_char = if self.is_charging { "⚡" } else { " " };

let display_str = format!(
    "{}% {}[{}{}]",
    pct, status_char, filled, empty
);

// Scale off the panel's configured size (COSMIC_PANEL_SIZE) instead of a
// fixed constant, so the applet grows/shrinks when the user changes panel
// size in Settings > Desktop > Panel.
let (_, icon_height) = self.core.applet.suggested_size(true);
let font_size = (icon_height as f32 * 0.6).clamp(9.0, 22.0);

let styled_text = text(display_str)
    .size(font_size)
    //(nice)
    .wrapping(cosmic::iced::widget::text::Wrapping::None)
    .class(cosmic::theme::Text::Color(text_color));

let clickable = mouse_area(styled_text)
    .on_press(Message::ToggleMenu);

let content = container(clickable)
    .width(cosmic::iced::Length::Shrink)
    .height(cosmic::iced::Length::Shrink)
    .clip(true);

self.core.applet.autosize_window(content).into()
}

    // 2. Popup window view when clicked
    fn view_window(&self, _id: cosmic::iced::window::Id) -> Element<'_, Self::Message> {
        let time_str = match self.time_remaining {
            Some(secs) if secs > 0.0 => {
                let h = (secs / 3600.0).floor() as u32;
                let m = ((secs % 3600.0) / 60.0).floor() as u32;
                if self.is_charging {
                    format!("Time to full: {}h {}m", h, m)
                } else {
                    format!("Time remaining: {}h {}m", h, m)
                }
            }
            _ => "Calculating time...".to_string(),
        };

        let time_label = text(time_str).size(14);
        let active_label = text(format!("Current Profile: {}", self.active_profile)).size(12);

        let btn_save = button::custom(text("Power Saver"))
            .on_press(Message::SetProfile("power-saver".to_string()))
            .padding(8);
        let btn_bal = button::custom(text("Balanced"))
            .on_press(Message::SetProfile("balanced".to_string()))
            .padding(8);
        let btn_perf = button::custom(text("Performance"))
            .on_press(Message::SetProfile("performance".to_string()))
            .padding(8);

        let profile_row = row![btn_save, btn_bal, btn_perf].spacing(8);

        // ASCII brightness "slider": 10 clickable segments, each worth 10%.
        // Clicking a segment sets brightness to that segment's threshold.
        let brightness_segments: usize = 10;
        let filled_segments = ((self.brightness_percent as f32 / 100.0)
            * brightness_segments as f32)
            .round() as usize;

        let mut slider_row = row![].spacing(2);
        for i in 1..=brightness_segments {
            let glyph = if i <= filled_segments { "█" } else { "░" };
            let target_pct = (i * 100 / brightness_segments) as u32;
            let segment =
                mouse_area(text(glyph).size(16)).on_press(Message::SetBrightness(target_pct));
            slider_row = slider_row.push(segment);
        }

        let brightness_label =
            text(format!("Brightness: {}%", self.brightness_percent)).size(12);
        let brightness_block = column![brightness_label, slider_row].spacing(4);

        // Accent color picker: Auto (percentage-tiered) or a fixed preset.
        let color_options: [(&str, Option<Color>); 8] = [
            ("Auto", None),
            ("Blue", Some(Color::from_rgb8(100, 170, 255))),
            ("Green", Some(Color::from_rgb8(120, 220, 120))),
            ("Purple", Some(Color::from_rgb8(190, 130, 255))),
            ("Pink", Some(Color::from_rgb8(255, 130, 200))),
            ("Orange", Some(Color::from_rgb8(255, 165, 60))),
            ("White", Some(Color::from_rgb8(230, 230, 230))),
            ("idk", Some(Color::from_rgb8(170, 69, 36))),
        ];

        let mut color_row = row![].spacing(6);
        for (label, color) in color_options {
            let swatch_color = color.unwrap_or(Color::from_rgb8(200, 200, 200));
            let btn = button::custom(
                text(label)
                    .size(11)
                    .class(cosmic::theme::Text::Color(swatch_color)),
            )
            .on_press(Message::SetAccentColor(color))
            .padding(6);
            color_row = color_row.push(btn);
        }
        let color_block = column![text("Accent color:").size(12), color_row].spacing(4);

       let content = column![
    time_label,
    active_label,
    profile_row,
    brightness_block,
    color_block,
]
.spacing(12)
.padding(16);
//(nice)
self.core
    .applet
    .popup_container(content)
    .max_width(900.0)
    .max_height(400.0)
    .into()
    }
}

impl BatteryApplet {
    fn update_battery(&mut self) {
        if let Ok(manager) = starship_battery::Manager::new() {
            if let Ok(mut batteries) = manager.batteries() {
                if let Some(Ok(bat)) = batteries.next() {
                    self.battery_percentage = (bat.state_of_charge().value * 100.0) as u32;
                    
                    let state = bat.state();
                    self.is_charging = matches!(
                        state,
                        starship_battery::State::Charging | starship_battery::State::Full
                    );

                    self.time_remaining = match state {
                        starship_battery::State::Charging => bat.time_to_full().map(|t| t.value),
                        starship_battery::State::Discharging => bat.time_to_empty().map(|t| t.value),
                        _ => None,
                    };
                }
            }
        }
    }

    fn update_profile(&mut self) {
        if let Some(profile) = get_active_profile() {
            self.active_profile = profile;
        }
    }

    fn update_brightness(&mut self) {
        if let Some(pct) = get_brightness_percent() {
            self.brightness_percent = pct;
        }
    }
}

// --- power-profiles-daemon, over D-Bus (sandbox-safe replacement for powerprofilesctl boiiiiiii) ---

fn power_profiles_proxy(connection: &Connection) -> zbus::Result<Proxy<'_>> {
    Proxy::new(
        connection,
        "net.hadess.PowerProfiles",
        "/net/hadess/PowerProfiles",
        "net.hadess.PowerProfiles",
    )
}

fn get_active_profile() -> Option<String> {
    let connection = Connection::system().ok()?;
    let proxy = power_profiles_proxy(&connection).ok()?;
    proxy.get_property("ActiveProfile").ok()
}

fn set_active_profile(profile: &str) -> zbus::Result<()> {
    let connection = Connection::system()?;
    let proxy = power_profiles_proxy(&connection)?;
    proxy.set_property("ActiveProfile", profile)?;
    Ok(())
}

// --- screen brightness: read via sysfs, write via logind (sandbox-safe, no brightnessctl) ---

fn backlight_device_name() -> Option<String> {
    let entries = std::fs::read_dir("/sys/class/backlight").ok()?;
    entries
        .filter_map(|e| e.ok())
        .next()
        .map(|e| e.file_name().to_string_lossy().into_owned())
}

fn get_brightness_percent() -> Option<u32> {
    let name = backlight_device_name()?;
    let base = format!("/sys/class/backlight/{name}");
    let current: u32 = std::fs::read_to_string(format!("{base}/brightness"))
        .ok()?
        .trim()
        .parse()
        .ok()?;
    let max: u32 = std::fs::read_to_string(format!("{base}/max_brightness"))
        .ok()?
        .trim()
        .parse()
        .ok()?;
    if max == 0 {
        return None;
    }
    Some(((current as f32 / max as f32) * 100.0).round() as u32)
}

fn set_brightness_percent(pct: u32) -> zbus::Result<()> {
//(nice)
    let name = backlight_device_name()
        .ok_or_else(|| zbus::Error::Failure("no backlight device found".into()))?;
    let base = format!("/sys/class/backlight/{name}");
    let max: u32 = std::fs::read_to_string(format!("{base}/max_brightness"))
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(100);
    let target = ((pct.min(100) as f32 / 100.0) * max as f32).round() as u32;

    let connection = Connection::system()?;
    let manager = Proxy::new(
        &connection,
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
    )?;
    let session_path: zbus::zvariant::OwnedObjectPath =
        manager.call("GetSessionByPID", &std::process::id())?;

    let session = Proxy::new(
        &connection,
        "org.freedesktop.login1",
        session_path,
        "org.freedesktop.login1.Session",
    )?;
 session.call::<_, _, ()>("SetBrightness", &("backlight", name.as_str(), target))?;
    Ok(())
}

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<BatteryApplet>(())
}
// version 1.1 will include a more aesthetic power profile picker and 
// more than a 10block indicator for battery indicator/brightness adjuster no promises tho cause this is hard.
//anyways thanks for reading the code
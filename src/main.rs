use cosmic::app::{Core, Task};
use cosmic::iced::time;
use cosmic::iced::window;
use cosmic::iced::{Color, Alignment};
use cosmic::iced::Subscription;
use cosmic::widget::{button, column, container, mouse_area, row, text};
use cosmic::{Application, Element};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use zbus::blocking::{Connection, Proxy};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PopupTab {
    Main,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DotSize {
    Standard,
    Small,
    Tiny,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanelModule {
    Battery,
    Brightness,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)] // Ensures older saved configs won't crash when missing new fields
pub struct AppletConfig {
    pub show_panel_battery: bool,
    pub show_panel_brightness: bool,
    pub panel_order: Vec<PanelModule>,
    pub show_dot_grid: bool,
    pub dot_size: DotSize,
    pub accent_color_rgba: Option<[f32; 4]>,
    
    // Custom Dimensions
    pub panel_font_size: u16,
    pub panel_block_count: usize,
    pub panel_spacing: u16,
}

impl Default for AppletConfig {
    fn default() -> Self {
        Self {
            show_panel_battery: true,
            show_panel_brightness: false,
            panel_order: vec![PanelModule::Battery, PanelModule::Brightness],
            show_dot_grid: true,
            dot_size: DotSize::Standard,
            accent_color_rgba: None,
            panel_font_size: 13,
            panel_block_count: 10,
            panel_spacing: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatteryState {
    pub percentage: u32,
    pub is_charging: bool,
    pub time_remaining: Option<f32>,
}

pub struct BatteryApplet {
    core: Core,
    active_tab: PopupTab,
    config: AppletConfig,

    battery_percentage: u32,
    is_charging: bool,
    time_remaining: Option<f32>,
    active_profile: String,
    brightness_percent: u32,

    popup: Option<window::Id>,
    is_dragging_brightness: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    BatteryFetched(Option<BatteryState>),
    ProfileFetched(Option<String>),
    BrightnessFetched(Option<u32>),

    ToggleMenu,
    SwitchTab(PopupTab),
    SetProfile(String),

    BrightnessPress(u32),
    BrightnessDragOver(u32),
    BrightnessDragEnd,

    SetAccentColor(Option<Color>),
    TogglePanelBattery(bool),
    TogglePanelBrightness(bool),
    MoveModuleLeft(usize),
    MoveModuleRight(usize),
    ToggleDotGrid(bool),
    SetDotSize(DotSize),

    // Dimension Adjustments
    AdjustFontSize(i16),
    AdjustBlockCount(i16),
    AdjustSpacing(i16),
}

impl Default for PopupTab {
    fn default() -> Self {
        PopupTab::Main
    }
}

impl Default for DotSize {
    fn default() -> Self {
        DotSize::Standard
    }
}

impl Default for BatteryApplet {
    fn default() -> Self {
        Self {
            core: Core::default(),
            active_tab: PopupTab::Main,
            config: load_config(),
            battery_percentage: 100,
            is_charging: false,
            time_remaining: None,
            active_profile: String::from("balanced"),
            brightness_percent: 100,
            popup: None,
            is_dragging_brightness: false,
        }
    }
}

impl Application for BatteryApplet {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "com.github.pewmoe.cosmic-ext-ASCII-deck";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let applet = BatteryApplet { core, ..Default::default() };
        (applet, fetch_all_states())
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        let mut save_needed = false;

        match message {
            Message::Tick => return fetch_all_states(),
            Message::BatteryFetched(Some(state)) => {
                self.battery_percentage = state.percentage;
                self.is_charging = state.is_charging;
                self.time_remaining = state.time_remaining;
            }
            Message::BatteryFetched(None) => {}
            Message::ProfileFetched(Some(profile)) => self.active_profile = profile,
            Message::ProfileFetched(None) => {}
            Message::BrightnessFetched(Some(pct)) => {
                if !self.is_dragging_brightness { self.brightness_percent = pct; }
            }
            Message::BrightnessFetched(None) => {}

            Message::ToggleMenu => {
                if let Some(popup_id) = self.popup.take() {
                    return cosmic::iced::platform_specific::shell::commands::popup::destroy_popup::<Message>(popup_id)
                        .map(cosmic::Action::from);
                }

                let new_id = window::Id::unique();
                self.popup = Some(new_id);

                let popup_settings = self.core.applet.get_popup_settings(
                    self.core.main_window_id().unwrap(),
                    new_id,
                    Some((420, 480)), // Made popup slightly taller to fit new settings
                    None, None,
                );

                return cosmic::iced::platform_specific::shell::commands::popup::get_popup::<Message>(popup_settings)
                    .map(cosmic::Action::from);
            }

            Message::SwitchTab(tab) => self.active_tab = tab,

            Message::SetProfile(profile) => {
                self.active_profile = profile.clone();
                return Task::perform(
                    async move { tokio::task::spawn_blocking(move || set_active_profile(&profile)).await.ok(); },
                    |_| cosmic::Action::App(Message::Tick),
                );
            }

            Message::BrightnessPress(pct) => {
                let pct = pct.min(100);
                self.is_dragging_brightness = true;
                self.brightness_percent = pct;
                return Task::perform(
                    async move { tokio::task::spawn_blocking(move || set_brightness_percent(pct)).await.ok(); },
                    move |_| cosmic::Action::App(Message::BrightnessFetched(Some(pct))),
                );
            }
            Message::BrightnessDragOver(pct) => {
                if self.is_dragging_brightness {
                    let pct = pct.min(100);
                    self.brightness_percent = pct;
                    return Task::perform(
                        async move { tokio::task::spawn_blocking(move || set_brightness_percent(pct)).await.ok(); },
                        move |_| cosmic::Action::App(Message::BrightnessFetched(Some(pct))),
                    );
                }
            }
            Message::BrightnessDragEnd => self.is_dragging_brightness = false,

            Message::SetAccentColor(color) => {
                self.config.accent_color_rgba = color.map(|c| [c.r, c.g, c.b, c.a]);
                save_needed = true;
            }
            Message::TogglePanelBattery(val) => { self.config.show_panel_battery = val; save_needed = true; }
            Message::TogglePanelBrightness(val) => { self.config.show_panel_brightness = val; save_needed = true; }
            Message::MoveModuleLeft(idx) => {
                if idx > 0 && idx < self.config.panel_order.len() {
                    self.config.panel_order.swap(idx, idx - 1);
                    save_needed = true;
                }
            }
            Message::MoveModuleRight(idx) => {
                if idx + 1 < self.config.panel_order.len() {
                    self.config.panel_order.swap(idx, idx + 1);
                    save_needed = true;
                }
            }
            Message::ToggleDotGrid(val) => { self.config.show_dot_grid = val; save_needed = true; }
            Message::SetDotSize(size) => { self.config.dot_size = size; save_needed = true; }
            
            Message::AdjustFontSize(delta) => {
                self.config.panel_font_size = (self.config.panel_font_size as i16 + delta).clamp(6, 24) as u16;
                save_needed = true;
            }
            Message::AdjustBlockCount(delta) => {
                self.config.panel_block_count = (self.config.panel_block_count as i16 + delta).clamp(1, 30) as usize;
                save_needed = true;
            }
            Message::AdjustSpacing(delta) => {
                self.config.panel_spacing = (self.config.panel_spacing as i16 + delta).clamp(0, 32) as u16;
                save_needed = true;
            }
        }

        if save_needed { save_config(&self.config); }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        time::every(Duration::from_secs(3)).map(|_| Message::Tick)
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let active_theme = cosmic::theme::active();
        let fallback_accent: Color = Color::from(active_theme.cosmic().accent.base);
        let accent_color = self.config.accent_color_rgba.map(|[r, g, b, a]| Color::from_rgba(r, g, b, a));

        let mut panel_items = row![].spacing(self.config.panel_spacing);
        let mut visible_count = 0;
        let blocks = self.config.panel_block_count;

        for module in &self.config.panel_order {
            match module {
                PanelModule::Battery if self.config.show_panel_battery => {
                    visible_count += 1;
                    let pct = self.battery_percentage.min(100);
                    let bat_color = accent_color.unwrap_or_else(|| get_tier_color(pct));
                    let filled_count = ((pct as f32 / 100.0) * blocks as f32).round() as usize;
                    let status_char = if self.is_charging { "⚡" } else { " " };
                    
                    let str_bat = format!(
                        "{:>3}% {}[{}{}]",
                        pct, status_char, "█".repeat(filled_count), "░".repeat(blocks.saturating_sub(filled_count))
                    );

                    let txt = text(str_bat)
                        .size(self.config.panel_font_size)
                        .wrapping(cosmic::iced::widget::text::Wrapping::None)
                        .class(cosmic::theme::Text::Color(bat_color));
                    panel_items = panel_items.push(txt);
                }

                PanelModule::Brightness if self.config.show_panel_brightness => {
                    visible_count += 1;
                    let pct = self.brightness_percent.min(100);
                    let bri_color = accent_color.unwrap_or_else(|| get_tier_color(pct));
                    let filled_count = ((pct as f32 / 100.0) * blocks as f32).round() as usize;
                    
                    let str_bri = format!(
                        "☼{:>3}%[{}{}]",
                        pct, "█".repeat(filled_count), "░".repeat(blocks.saturating_sub(filled_count))
                    );

                    let txt = text(str_bri)
                        .size(self.config.panel_font_size)
                        .wrapping(cosmic::iced::widget::text::Wrapping::None)
                        .class(cosmic::theme::Text::Color(bri_color));
                    panel_items = panel_items.push(txt);
                }
                _ => {}
            }
        }

        if visible_count == 0 {
            let txt = text("ASCII")
                .size(self.config.panel_font_size)
                .class(cosmic::theme::Text::Color(accent_color.unwrap_or(fallback_accent)));
            panel_items = panel_items.push(txt);
        }

        let clickable = mouse_area(panel_items).on_press(Message::ToggleMenu);
        let content = container(clickable)
            .width(cosmic::iced::Length::Shrink)
            .height(cosmic::iced::Length::Shrink)
            .clip(true);

        self.core.applet.autosize_window(content).into()
    }

    fn view_window(&self, _id: cosmic::iced::window::Id) -> Element<'_, Self::Message> {
        let active_theme = cosmic::theme::active();
        let fallback_accent: Color = Color::from(active_theme.cosmic().accent.base);

        let btn_main = button::custom(text("Control Deck").size(12))
            .on_press(Message::SwitchTab(PopupTab::Main)).padding(6);
        let btn_settings = button::custom(text("Customization").size(12))
            .on_press(Message::SwitchTab(PopupTab::Settings)).padding(6);
        let header_row = row![btn_main, btn_settings].spacing(8);

        let body_content: Element<'_, Self::Message> = match self.active_tab {
            PopupTab::Main => self.view_main_deck(fallback_accent),
            PopupTab::Settings => self.view_settings_deck(fallback_accent),
        };

        let content = column![header_row, body_content].spacing(12).padding(16);

        container(content)
            .class(cosmic::theme::Container::Dropdown)
            .width(cosmic::iced::Length::Fill)
            .height(cosmic::iced::Length::Fill)
            .into()
    }
}

impl BatteryApplet {
    fn view_main_deck(&self, _fallback_accent: Color) -> Element<'_, Message> {
        let pct = self.battery_percentage.min(100);
        let accent_color = self.config.accent_color_rgba.map(|[r, g, b, a]| Color::from_rgba(r, g, b, a));
        let bat_color = accent_color.unwrap_or_else(|| get_tier_color(pct));

        let time_str = match self.time_remaining {
            Some(secs) if secs > 0.0 => {
                let h = (secs / 3600.0).floor() as u32;
                let m = ((secs % 3600.0) / 60.0).floor() as u32;
                if self.is_charging { format!("Time to full: {}h {}m", h, m) } 
                else { format!("Time remaining: {}h {}m", h, m) }
            }
            _ => "Calculating time...".to_string(),
        };
        let time_label = text(time_str).size(13);

        let active_label = text(format!("Profile: {}", self.active_profile)).size(12);
        let btn_save = button::custom(text("Power Saver").size(11))
            .on_press(Message::SetProfile("power-saver".to_string())).padding(6);
        let btn_bal = button::custom(text("Balanced").size(11))
            .on_press(Message::SetProfile("balanced".to_string())).padding(6);
        let btn_perf = button::custom(text("Performance").size(11))
            .on_press(Message::SetProfile("performance".to_string())).padding(6);
        let profile_row = row![btn_save, btn_bal, btn_perf].spacing(6);

        let bri_color = accent_color.unwrap_or_else(|| get_tier_color(self.brightness_percent));
        let mut brightness_slider_row = row![].spacing(2);
        let filled_bri_segments = ((self.brightness_percent as f32 / 100.0) * 10.0).round() as usize;

        for i in 1..=10 {
            let glyph = if i <= filled_bri_segments { "█" } else { "░" };
            let target_pct = (i * 10) as u32;
            let segment = mouse_area(text(glyph).size(15).class(cosmic::theme::Text::Color(bri_color)))
                .on_press(Message::BrightnessPress(target_pct))
                .on_enter(Message::BrightnessDragOver(target_pct))
                .on_release(Message::BrightnessDragEnd);
            brightness_slider_row = brightness_slider_row.push(segment);
        }

        let brightness_block = column![
            text(format!("Brightness: {}%", self.brightness_percent)).size(12),
            brightness_slider_row
        ].spacing(4);

        let mut main_col = column![time_label, active_label, profile_row, brightness_block].spacing(10);

        if self.config.show_dot_grid {
            let filled_dots = pct.min(100) as usize;

            let font_sz = match self.config.dot_size {
                DotSize::Standard => 13,
                DotSize::Small => 9,
                DotSize::Tiny => 5,
            };

            let mut dot_grid = column![].spacing(1);

            for row_index in 0..10 {
                let mut dot_row = row![].spacing(2);

                for col_index in 0..10 {
                    let i = row_index * 10 + col_index;
                    let glyph = if i < filled_dots { '●' } else { '○' };

                    let dot = text(glyph.to_string())
                        .size(font_sz)
                        .class(cosmic::theme::Text::Color(bat_color));

                    dot_row = dot_row.push(dot);
                }

                dot_grid = dot_grid.push(dot_row);
            }

            let grid_block = column![
                text(format!("Battery State: {pct}%")).size(12),
                dot_grid,
            ]
            .spacing(4);

            main_col = main_col.push(grid_block);
        }

        container(main_col).into()
    }

    fn view_settings_deck(&self, fallback_accent: Color) -> Element<'_, Message> {
        let btn_tog_bat = button::custom(text(if self.config.show_panel_battery { "[x] Battery" } else { "[ ] Battery" }).size(11))
            .on_press(Message::TogglePanelBattery(!self.config.show_panel_battery)).padding(4);
        let btn_tog_bri = button::custom(text(if self.config.show_panel_brightness { "[x] Brightness" } else { "[ ] Brightness" }).size(11))
            .on_press(Message::TogglePanelBrightness(!self.config.show_panel_brightness)).padding(4);

        let panel_toggles = column![
            text("Top Panel Elements:").size(12),
            row![btn_tog_bat, btn_tog_bri].spacing(6)
        ].spacing(4);

        // Dimensional Sliders / Buttons
       let font_row = row![
            text(format!("Text Size: {}", self.config.panel_font_size)).size(11),
            button::custom(text("-").size(10)).on_press(Message::AdjustFontSize(-1)).padding(4),
            button::custom(text("+").size(10)).on_press(Message::AdjustFontSize(1)).padding(4)
        ].spacing(8).align_y(Alignment::Center);

        let block_row = row![
            text(format!("ASCII Blocks: {}", self.config.panel_block_count)).size(11),
            button::custom(text("-").size(10)).on_press(Message::AdjustBlockCount(-1)).padding(4),
            button::custom(text("+").size(10)).on_press(Message::AdjustBlockCount(1)).padding(4)
        ].spacing(8).align_y(Alignment::Center);

        let space_row = row![
            text(format!("Spacing: {}", self.config.panel_spacing)).size(11),
            button::custom(text("-").size(10)).on_press(Message::AdjustSpacing(-1)).padding(4),
            button::custom(text("+").size(10)).on_press(Message::AdjustSpacing(1)).padding(4)
        ].spacing(8).align_y(Alignment::Center);
        
        let dims_col = column![
            text("Panel Dimensions:").size(12),
            font_row, block_row, space_row
        ].spacing(4);

        let mut reorder_col = column![text("Panel Element Order:").size(12)].spacing(4);
        for (idx, module) in self.config.panel_order.iter().enumerate() {
            let name = match module {
                PanelModule::Battery => "Battery",
                PanelModule::Brightness => "Brightness",
            };
            let btn_left = button::custom(text("<").size(10)).on_press(Message::MoveModuleLeft(idx)).padding(3);
            let btn_right = button::custom(text(">").size(10)).on_press(Message::MoveModuleRight(idx)).padding(3);
            reorder_col = reorder_col.push(row![text(format!("{}. {}", idx + 1, name)).size(11), btn_left, btn_right].spacing(6));
        }

        let color_options: [(&str, Option<Color>); 8] = [
            ("Auto", None),
            ("Blue", Some(Color::from_rgb8(100, 170, 255))),
            ("Green", Some(Color::from_rgb8(120, 220, 120))),
            ("Purple", Some(Color::from_rgb8(190, 130, 255))),
            ("Pink", Some(Color::from_rgb8(255, 130, 200))),
            ("Orange", Some(Color::from_rgb8(255, 165, 60))),
            ("White", Some(Color::from_rgb8(230, 230, 230))),
            ("Rust", Some(Color::from_rgb8(170, 69, 36))),
        ];

        let mut color_row = row![].spacing(4);
        for (label, color) in color_options {
            let swatch_color = color.unwrap_or(fallback_accent);
            let btn = button::custom(text(label).size(10).class(cosmic::theme::Text::Color(swatch_color)))
                .on_press(Message::SetAccentColor(color)).padding(5);
            color_row = color_row.push(btn);
        }
        let color_block = column![text("Global Color Accent Override:").size(12), color_row].spacing(4);

        container(column![panel_toggles, dims_col, reorder_col, color_block].spacing(12)).into()
    }
}

fn get_config_path() -> PathBuf {
    let mut path = PathBuf::from(std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        format!("{}/.config", std::env::var("HOME").unwrap_or_default())
    }));
    path.push("com.github.pewmoe.cosmic-ext-ASCII-deck");
    std::fs::create_dir_all(&path).ok();
    path.push("config.json");
    path
}

fn load_config() -> AppletConfig {
    let path = get_config_path();
    if let Ok(data) = std::fs::read_to_string(path) {
        if let Ok(config) = serde_json::from_str(&data) {
            return config;
        }
    }
    AppletConfig::default()
}

fn save_config(config: &AppletConfig) {
    let path = get_config_path();
    if let Ok(data) = serde_json::to_string_pretty(config) {
        std::fs::write(path, data).ok();
    }
}

fn fetch_all_states() -> Task<Message> {
    Task::batch(vec![
        Task::perform(
            async { tokio::task::spawn_blocking(get_battery_info).await.unwrap_or(None) },
            |res| cosmic::Action::App(Message::BatteryFetched(res)),
        ),
        Task::perform(
            async { tokio::task::spawn_blocking(get_active_profile).await.unwrap_or(None) },
            |res| cosmic::Action::App(Message::ProfileFetched(res)),
        ),
        Task::perform(
            async { tokio::task::spawn_blocking(get_brightness_percent).await.unwrap_or(None) },
            |res| cosmic::Action::App(Message::BrightnessFetched(res)),
        ),
    ])
}

fn get_tier_color(pct: u32) -> Color {
    match pct {
        0..=20 => Color::from_rgb8(255, 60, 60),
        21..=50 => Color::from_rgb8(255, 165, 0),
        51..=70 => Color::from_rgb8(255, 220, 0),
        _ => Color::from_rgb8(180, 220, 255),
    }
}

fn get_battery_info() -> Option<BatteryState> {
    let manager = starship_battery::Manager::new().ok()?;
    let mut batteries = manager.batteries().ok()?;
    let bat = batteries.next()?.ok()?;
    let percentage = (bat.state_of_charge().value * 100.0) as u32;
    let state = bat.state();
    let is_charging = matches!(state, starship_battery::State::Charging | starship_battery::State::Full);
    let time_remaining = match state {
        starship_battery::State::Charging => bat.time_to_full().map(|t| t.value),
        starship_battery::State::Discharging => bat.time_to_empty().map(|t| t.value),
        _ => None,
    };
    Some(BatteryState { percentage, is_charging, time_remaining })
}

fn power_profiles_proxy(connection: &Connection) -> zbus::Result<Proxy<'_>> {
    Proxy::new(connection, "net.hadess.PowerProfiles", "/net/hadess/PowerProfiles", "net.hadess.PowerProfiles")
}

fn get_active_profile() -> Option<String> {
    let connection = Connection::system().ok()?;
    let proxy = power_profiles_proxy(&connection).ok()?;
    proxy.get_property("ActiveProfile").ok()
}

fn set_active_profile(profile: &str) -> zbus::Result<()> {
    let connection = Connection::system()?;
    power_profiles_proxy(&connection)?.set_property("ActiveProfile", profile)?;
    Ok(())
}

fn backlight_device_name() -> Option<String> {
    std::fs::read_dir("/sys/class/backlight").ok()?
        .filter_map(|e| e.ok())
        .next()
        .map(|e| e.file_name().to_string_lossy().into_owned())
}

fn get_brightness_percent() -> Option<u32> {
    let name = backlight_device_name()?;
    let base = format!("/sys/class/backlight/{name}");
    let current: u32 = std::fs::read_to_string(format!("{base}/brightness")).ok()?.trim().parse().ok()?;
    let max: u32 = std::fs::read_to_string(format!("{base}/max_brightness")).ok()?.trim().parse().ok()?;
    if max == 0 { return None; }
    Some(((current as f32 / max as f32) * 100.0).round() as u32)
}

fn get_session_path(connection: &Connection) -> zbus::Result<zbus::zvariant::OwnedObjectPath> {
    let manager = Proxy::new(connection, "org.freedesktop.login1", "/org/freedesktop/login1", "org.freedesktop.login1.Manager")?;
    type SessionEntry = (String, u32, String, String, zbus::zvariant::OwnedObjectPath);
    let sessions: Vec<SessionEntry> = manager.call("ListSessions", &())?;
    
    if let Ok(session_id) = std::env::var("XDG_SESSION_ID") {
        if let Some((_, _, _, _, path)) = sessions.iter().find(|(id, ..)| *id == session_id) {
            return Ok(path.clone());
        }
    }
    
    sessions.into_iter().next().map(|(_, _, _, _, path)| path)
        .ok_or_else(|| zbus::Error::Failure("no active logind session found".into()))
}

fn set_brightness_percent(pct: u32) -> zbus::Result<()> {
    let name = backlight_device_name().ok_or_else(|| zbus::Error::Failure("no backlight device found".into()))?;
    let base = format!("/sys/class/backlight/{name}");
    let max: u32 = std::fs::read_to_string(format!("{base}/max_brightness")).ok().and_then(|s| s.trim().parse().ok()).unwrap_or(100);
    let target = ((pct.min(100) as f32 / 100.0) * max as f32).round() as u32;

    let connection = Connection::system()?;
    let session_path = get_session_path(&connection)?;
    let session = Proxy::new(&connection, "org.freedesktop.login1", session_path, "org.freedesktop.login1.Session")?;
    session.call::<_, _, ()>("SetBrightness", &("backlight", name.as_str(), target))?;
    Ok(())
}

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<BatteryApplet>(())
}
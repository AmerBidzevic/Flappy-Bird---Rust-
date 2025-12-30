use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};
use bevy::color::palettes::css::AQUA;
use bevy::audio::Volume;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

// ---------------------------- STATES ----------------------------
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    MainMenu,
    SaveSelect,
    ModeSelect,
    DifficultySelect,
    ThemeSelect,
    Playing,
//    Paused,
    GameOver,
    Leaderboard,
}

// ---------------------------- GAME SETTINGS ----------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameMode {Endless, TimeAttack, Checkpoints}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Difficulty {Easy, Normal, Hard}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Theme {Classic, HighContrast, Minimal}

#[derive(Resource, Serialize, Deserialize, Clone)]
struct PlayerProfile {
    name: String,
    high_score: u32,
    total_games: u32,
    average_score: f32,
    longest_survival: f32,
}

impl Default for PlayerProfile {
    fn default() -> Self {
        Self {
            name: String::from("Player"),
            high_score: 0,
            total_games: 0,
            average_score: 0.0,
            longest_survival: 0.0,
        }
    }
}

#[derive(Resource, Serialize, Deserialize, Clone)]
struct SaveSlot {
    slot_number: u8,
        profile: PlayerProfile,
        mode: GameMode,
        difficulty: Difficulty,
        theme: Theme,
    score: u32,
    survival_time: f32,
}

#[derive(Resource)]
struct GameSettings {
    current_slot: Option<u8>,
    selected_mode: GameMode,
    selected_difficulty: Difficulty,
    selected_theme: Theme,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            current_slot: None,
            selected_mode: GameMode::Endless,
            selected_difficulty: Difficulty::Normal,
            selected_theme: Theme::Classic,
        }
    }
}

// ---------------------------- SERIALIZATION ----------------------------
impl Serialize for GameMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            GameMode::Endless => "Endless",
            GameMode::TimeAttack => "TimeAttack",
            GameMode::Checkpoints => "Checkpoints",
        })
    }
}

impl<'de> Deserialize<'de> for GameMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Endless" => Ok(GameMode::Endless),
            "TimeAttack" => Ok(GameMode::TimeAttack),
            "Checkpoints" => Ok(GameMode::Checkpoints),
            _ => Err(serde::de::Error::custom("Invalid game mode")),
        }
    }
}

impl Serialize for Difficulty {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            Difficulty::Easy => "Easy",
            Difficulty::Normal => "Normal",
            Difficulty::Hard => "Hard",
        })
    }
}

impl<'de> Deserialize<'de> for Difficulty {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Easy" => Ok(Difficulty::Easy),
            "Normal" => Ok(Difficulty::Normal),
            "Hard" => Ok(Difficulty::Hard),
            _ => Err(serde::de::Error::custom("Invalid difficulty")),
        }
    }
}

impl Serialize for Theme {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            Theme::Classic => "Classic",
            Theme::HighContrast => "HighContrast",
            Theme::Minimal => "Minimal",
        })
    }
}

impl<'de> Deserialize<'de> for Theme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Classic" => Ok(Theme::Classic),
            "HighContrast" => Ok(Theme::HighContrast),
            "Minimal" => Ok(Theme::Minimal),
            _ => Err(serde::de::Error::custom("Invalid theme")),
        }
    }
}

// ---------------------------- MAIN ----------------------------
fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Flappy Bird"),
                        position: WindowPosition::Centered(MonitorSelection::Primary),
                        resolution: (800, 600).into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .init_state::<GameState>()
        .init_resource::<GameSettings>()
        .add_systems(Startup, (setup_save_system, setup_main_menu))
        .add_systems(OnEnter(GameState::MainMenu), setup_main_menu_ui)
        .add_systems(OnExit(GameState::MainMenu), cleanup_menu::<MainMenuMarker>)
        .add_systems(OnEnter(GameState::SaveSelect), setup_save_select_ui)
        .add_systems(OnExit(GameState::SaveSelect), cleanup_menu::<SaveSelectMarker>)
        .add_systems(OnEnter(GameState::Leaderboard), setup_leaderboard_ui)
        .add_systems(OnExit(GameState::Leaderboard), cleanup_menu::<LeaderboardMarker>)
        .add_systems(OnEnter(GameState::ModeSelect), setup_mode_select_ui)
        .add_systems(OnExit(GameState::ModeSelect), cleanup_menu::<ModeSelectMarker>)
        .add_systems(OnEnter(GameState::DifficultySelect), setup_difficulty_select_ui)
        .add_systems(OnExit(GameState::DifficultySelect), cleanup_menu::<DifficultySelectMarker>)
        .add_systems(OnEnter(GameState::ThemeSelect), setup_theme_select_ui)
        .add_systems(OnExit(GameState::ThemeSelect), cleanup_menu::<ThemeSelectMarker>)
        .add_systems(OnEnter(GameState::Playing), (setup_level, reset_on_play_start).chain())
        .add_systems(OnExit(GameState::Playing), cleanup_game)
        .add_systems(OnEnter(GameState::GameOver), setup_game_over_ui)
        .add_systems(OnExit(GameState::GameOver), cleanup_menu::<GameOverMarker>)
        .add_systems(Update, (
            main_menu_system.run_if(in_state(GameState::MainMenu)),
            save_select_system.run_if(in_state(GameState::SaveSelect)),
            mode_select_system.run_if(in_state(GameState::ModeSelect)),
            difficulty_select_system.run_if(in_state(GameState::DifficultySelect)),
            theme_select_system.run_if(in_state(GameState::ThemeSelect)),
            update_bird.run_if(in_state(GameState::Playing)),
            update_obstacles.run_if(in_state(GameState::Playing)),
            update_ui.run_if(in_state(GameState::Playing)),
            update_time_attack.run_if(in_state(GameState::Playing)),
            handle_game_over.run_if(in_state(GameState::GameOver)),
            leaderboard_system.run_if(in_state(GameState::Leaderboard)),
        ))
        .run();
}

#[derive(Serialize, Deserialize, Clone)]
struct LeaderboardEntry {
    name: String,
    score: u32,
    mode: GameMode,
    difficulty: Difficulty,
}

// Marker components for menu cleanup
#[derive(Component)]
struct MainMenuMarker;

#[derive(Component)]
struct SaveSelectMarker;

#[derive(Component)]
struct ModeSelectMarker;

#[derive(Component)]
struct DifficultySelectMarker;

#[derive(Component)]
struct ThemeSelectMarker;

#[derive(Component)]
struct GameOverMarker;

fn load_leaderboard() -> Vec<LeaderboardEntry> {
    let mut entries = Vec::new();

    for slot in 1..=3 {
        if let Some(save) = load_save_slot(slot) {
            entries.push(LeaderboardEntry {
                name: save.profile.name.clone(),
                score: save.score,
                mode: save.mode,
                difficulty: save.difficulty,
            });
        }
    }

    // Sort descending by score
    entries.sort_by(|a, b| b.score.cmp(&a.score));
    entries
}

#[derive(Component)]
struct LeaderboardMarker;

fn setup_leaderboard_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let entries = load_leaderboard();

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        LeaderboardMarker,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("LEADERBOARD"),
            TextFont {
                font_size: 48.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node { margin: UiRect::all(Val::Px(20.0)), ..default() },
        ));

        for (i, entry) in entries.iter().enumerate() {
            parent.spawn((
                Text::new(format!(
                    "{}. {} - {} pts [{:?} {:?}]",
                    i + 1,
                    entry.name,
                    entry.score,
                    entry.mode,
                    entry.difficulty
                )),
                TextFont { font_size: 28.0, ..default() },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node { margin: UiRect::all(Val::Px(5.0)), ..default() },
            ));
        }

        parent.spawn((
            Text::new("Press ESC to return"),
            TextFont { font_size: 24.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
            Node { margin: UiRect::top(Val::Px(20.0)), ..default() },
        ));
    });
}

fn leaderboard_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::MainMenu);
    }
}


// Save system setup
fn setup_save_system(_commands: Commands) {
    // Create saves directory if it doesn't exist
    if let Err(e) = fs::create_dir_all("saves") {
        eprintln!("Failed to create saves directory: {}", e);
    }
}

fn setup_main_menu(mut commands: Commands) {
    // Basic 2D camera for UI
    commands.spawn(Camera2d);
}

fn load_save_slot(slot: u32) -> Option<SaveSlot> {
    let path = format!("saves/slot_{}.json", slot);
    if Path::new(&path).exists() {
        if let Ok(contents) = fs::read_to_string(&path) {
            serde_json::from_str(&contents).ok()
        } else {
            None
        }
    } else {
        None
    }
}

fn save_to_slot(slot: &SaveSlot) -> Result<(), Box<dyn std::error::Error>> {
    let path = format!("saves/slot_{}.json", slot.slot_number);
    let json = serde_json::to_string_pretty(slot)?;
    fs::write(&path, json)?;
    Ok(())
}

fn cleanup_menu<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn cleanup_game(
    mut commands: Commands,
    bird_query: Query<Entity, With<Bird>>,
    obstacle_query: Query<Entity, With<Obstacle>>,
    ui_query: Query<Entity, Or<(With<ScoreDisplay>, With<BestScoreDisplay>, With<TimeDisplay>)>>,
    background_query: Query<Entity, With<Background>>,
) {
    // Tear down everything that belongs to a run before returning to menus
    for entity in &bird_query {
        commands.entity(entity).despawn();
    }
    for entity in &obstacle_query {
        commands.entity(entity).despawn();
    }
    for entity in &ui_query {
        commands.entity(entity).despawn();
    }
    for entity in &background_query {
        commands.entity(entity).despawn();
    }

    commands.remove_resource::<TimeAttackState>();
}

// Main Menu UI
fn setup_main_menu_ui(mut commands: Commands, asset_server: Res<AssetServer>, window_query: Query<&Window, With<PrimaryWindow>>,) {
    // Neutral background for menus so theme colors from gameplay don't stick
    let window = window_query.single().expect("Missing primary window");
    let window_width = window.width();
    let window_height = window.height();

    commands.spawn((
                Sprite {
                    image: asset_server.load("Background2.png"),
                    custom_size: Some(Vec2::new(window_width, window_height)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, -50.0)),
                Background,
                MainMenuMarker,
            ));

    // Loop menu music
    commands.spawn((
    AudioPlayer::new(asset_server.load("35-Lost-Woods.ogg")),
    PlaybackSettings {
        volume: Volume::Linear(0.1),
        ..PlaybackSettings::LOOP
    },
    MainMenuMarker,
    ));

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        MainMenuMarker,
    ))
    .with_children(|parent| {

        parent.spawn((
            Text::new("FLAPPY BIRD"),
            TextFont {
                font: asset_server.load("fonts/BBHHegarty-Regular.ttf"),
                font_size: 80.0,
                ..default()
            },
            TextShadow::default(),
            TextColor(Color::srgb(1.0, 0.992, 0.816)),
            Node {
                margin: UiRect::all(Val::Px(20.0)),
                ..default()
            },
        ));
        
        parent.spawn((
            Text::new("Start Game [Space]"),
            TextFont {
                font: asset_server.load("fonts/BBHHegarty-Regular.ttf"),
                font_size: 32.0,
                ..default()
            },
            TextShadow::default(),
            TextColor(AQUA.into()),
        ));

        parent.spawn((
            Text::new("Leaderboard [F1]"),
            TextFont {
                font: asset_server.load("fonts/BBHHegarty-Regular.ttf"),
                font_size: 32.0,
                ..default()
            },
            TextShadow::default(),
            TextColor(AQUA.into()),
        ));

    });
}

fn main_menu_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(GameState::SaveSelect);
    }

    if keyboard.just_pressed(KeyCode::F1) {
    next_state.set(GameState::Leaderboard);
    }
}

// Save Select UI
fn setup_save_select_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        SaveSelectMarker,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("SELECT SAVE SLOT"),
            TextFont {
                font_size: 48.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::all(Val::Px(20.0)),
                ..default()
            },
        ));
        
        for slot_num in 1..=3 {
            let save_data = load_save_slot(slot_num);
            let text = if let Some(save) = save_data {
                format!("Slot {}: {} - High Score: {}", 
                    slot_num, save.profile.name, save.profile.high_score)
            } else {
                format!("Slot {}: Empty", slot_num)
            };
            
            parent.spawn((
                Text::new(text),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
            ));
        }
        
        parent.spawn((
            Text::new("\nPress 1, 2, or 3 to select a slot\nHold CTRL + (1/2/3) to delete a slot\nPress ESC to return"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
            Node {
                margin: UiRect::top(Val::Px(30.0)),
                ..default()
            },
        ));
    });
}

fn save_select_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut settings: ResMut<GameSettings>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::MainMenu);
        return;
    }
    
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    for (key, slot) in [(KeyCode::Digit1, 1), (KeyCode::Digit2, 2), (KeyCode::Digit3, 3)] {
        if keyboard.just_pressed(key) {
            if ctrl {
                delete_save_slot(slot as u32);
                settings.current_slot = None;
                continue;
            }

            settings.current_slot = Some(slot);
            
            // Load existing save or use defaults
            if let Some(save_data) = load_save_slot(slot as u32) {
                settings.selected_mode = save_data.mode;
                settings.selected_difficulty = save_data.difficulty;
                settings.selected_theme = save_data.theme;
            }
            
            next_state.set(GameState::ModeSelect);
            return;
        }
    }
}

fn delete_save_slot(slot: u32) {
    let path = format!("saves/slot_{}.json", slot);
    let _ = fs::remove_file(path);
}

// Mode Select UI
fn setup_mode_select_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ModeSelectMarker,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("SELECT GAME MODE"),
            TextFont {
                font_size: 48.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::all(Val::Px(20.0)),
                ..default()
            },
        ));
        
        parent.spawn((
            Text::new("1. Endless - Classic mode, go as far as you can"),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            Node {
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            },
        ));
        
        parent.spawn((
            Text::new("2. Time Attack - Score as much as possible in 60 seconds"),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            Node {
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            },
        ));
        
        parent.spawn((
            Text::new("3. Checkpoints - Reach checkpoints to save progress"),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            Node {
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            },
        ));
        
        parent.spawn((
            Text::new("\nPress ESC to return"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
            Node {
                margin: UiRect::top(Val::Px(20.0)),
                ..default()
            },
        ));
    });
}

fn mode_select_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut settings: ResMut<GameSettings>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::SaveSelect);
        return;
    }
    
    for (key, mode) in [
        (KeyCode::Digit1, GameMode::Endless),
        (KeyCode::Digit2, GameMode::TimeAttack),
        (KeyCode::Digit3, GameMode::Checkpoints),
    ] {
        if keyboard.just_pressed(key) {
            settings.selected_mode = mode;
            next_state.set(GameState::DifficultySelect);
            return;
        }
    }
}

// Difficulty Select UI
fn setup_difficulty_select_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        DifficultySelectMarker,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("SELECT DIFFICULTY"),
            TextFont {
                font_size: 48.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::all(Val::Px(20.0)),
                ..default()
            },
        ));
        
        parent.spawn((
            Text::new("1. Easy - Larger gaps, slower pipes, less gravity"),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgb(0.5, 1.0, 0.5)),
            Node {
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            },
        ));
        
        parent.spawn((
            Text::new("2. Normal - Standard game settings"),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 0.5)),
            Node {
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            },
        ));
        
        parent.spawn((
            Text::new("3. Hard - Smaller gaps, faster pipes, more gravity"),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.5, 0.5)),
            Node {
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            },
        ));
        
        parent.spawn((
            Text::new("\nPress ESC to return"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
            Node {
                margin: UiRect::top(Val::Px(20.0)),
                ..default()
            },
        ));
    });
}

fn difficulty_select_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut settings: ResMut<GameSettings>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::ModeSelect);
        return;
    }
    
    for (key, difficulty) in [
        (KeyCode::Digit1, Difficulty::Easy),
        (KeyCode::Digit2, Difficulty::Normal),
        (KeyCode::Digit3, Difficulty::Hard),
    ] {
        if keyboard.just_pressed(key) {
            settings.selected_difficulty = difficulty;
            next_state.set(GameState::ThemeSelect);
            return;
        }
    }
}

// Theme Select UI
fn setup_theme_select_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ThemeSelectMarker,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("SELECT THEME"),
            TextFont {
                font_size: 48.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::all(Val::Px(20.0)),
                ..default()
            },
        ));
        
        parent.spawn((
            Text::new("1. Classic - Original Flappy Bird style"),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            Node {
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            },
        ));
        
        parent.spawn((
            Text::new("2. High Contrast - Enhanced visibility"),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            Node {
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            },
        ));
        
        parent.spawn((
            Text::new("3. Minimal - Clean, simple aesthetics"),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            Node {
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            },
        ));
        
        parent.spawn((
            Text::new("\nPress ESC to return"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
            Node {
                margin: UiRect::top(Val::Px(20.0)),
                ..default()
            },
        ));
    });
}

fn theme_select_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut settings: ResMut<GameSettings>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::DifficultySelect);
        return;
    }
    
    for (key, theme) in [
        (KeyCode::Digit1, Theme::Classic),
        (KeyCode::Digit2, Theme::HighContrast),
        (KeyCode::Digit3, Theme::Minimal),
    ] {
        if keyboard.just_pressed(key) {
            settings.selected_theme = theme;
            next_state.set(GameState::Playing);
            return;
        }
    }
}
// BIRD
const PIXEL_RATIO: f32 = 4.;
const FLAP_FORCE: f32 = 500.;
const GRAVITY: f32 = 2000.;
const VELOCITY_TO_ROTATION_RATIO: f32 = 7.5;
//OBSTACLE
const OBSTACLE_AMOUNT: i32 = 5;
const OBSTACLE_WIDTH: f32 = 32.;
const OBSTACLE_HEIGHT: f32 = 144.;
const OBSTACLE_VERTICAL_OFFSET: f32 = 8.;  // Reduced to ensure gap stays passable
const OBSTACLE_GAP_SIZE: f32 = 25.;  // Increased from 15 to make gap larger
const OBSTACLE_SPACING: f32 = 60.;
const OBSTACLE_SCROLL_SPEED: f32 = 150.;

#[derive(Resource)]
pub struct Score {
    pub current: u32,
    pub best: u32,
    pub scored_pipes: Vec<Entity>,
}

impl Default for Score {
    fn default() -> Self {
        Self {
            current: 0,
            best: 0,
            scored_pipes: Vec::new(),
        }
    }
}

#[derive(Resource)]
pub struct GameManager {
    pub pipe_image: Handle<Image>,
    pub window_dimensions: Vec2,
}

#[derive(Resource)]
pub struct SoundEffects {
    pub flap: Handle<AudioSource>,
    pub point: Handle<AudioSource>,
    pub die: Handle<AudioSource>,
    pub swoosh: Handle<AudioSource>,
}

#[derive(Resource)]
struct TimeAttackState {
    remaining: f32,
}

#[derive(Resource, Clone, Copy)]
struct DifficultyTuning {
    gap_size: f32,
    scroll_speed: f32,
    gravity_mult: f32,
    flap_mult: f32,
    vertical_offset: f32,
}

fn difficulty_tuning(difficulty: Difficulty) -> DifficultyTuning {
    match difficulty {
        Difficulty::Easy => DifficultyTuning {
            gap_size: OBSTACLE_GAP_SIZE * 1.3,
            scroll_speed: OBSTACLE_SCROLL_SPEED * 0.85,
            gravity_mult: 0.75,
            flap_mult: 1.2,
            vertical_offset: OBSTACLE_VERTICAL_OFFSET * 0.7,
        },
        Difficulty::Normal => DifficultyTuning {
            gap_size: OBSTACLE_GAP_SIZE,
            scroll_speed: OBSTACLE_SCROLL_SPEED,
            gravity_mult: 1.0,
            flap_mult: 1.0,
            vertical_offset: OBSTACLE_VERTICAL_OFFSET,
        },
        Difficulty::Hard => DifficultyTuning {
            gap_size: OBSTACLE_GAP_SIZE * 0.75,
            scroll_speed: OBSTACLE_SCROLL_SPEED * 1.25,
            gravity_mult: 1.3,
            flap_mult: 1.05,
            vertical_offset: OBSTACLE_VERTICAL_OFFSET * 1.2,
        },
    }
}

#[derive(Component)]
struct Bird {
    pub velocity: f32,
}

#[derive(Component)]
struct ScoreDisplay;

#[derive(Component)]
struct BestScoreDisplay;

#[derive(Component)]
struct TimeDisplay;

#[derive(Component)]
struct Background;

#[derive(Component)]
struct Obstacle {
    pipe_direction: f32,
    scored: bool,
}

fn setup_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    settings: Res<GameSettings>,
) {
    // Load core assets and cache window info used by obstacle wrap logic
    let pipe_image = asset_server.load("pipe.png");
    let window = window_query.single().expect("Missing primary window");
    let window_width = window.width();
    let window_height = window.height();
    commands.insert_resource(GameManager {
        pipe_image: pipe_image.clone(),
        window_dimensions: Vec2::new(window_width, window_height),
    });
    
    // Load sound effects (OGG format)
    commands.insert_resource(SoundEffects {
        flap: asset_server.load("flap.ogg"),
        point: asset_server.load("point.ogg"),
        die: asset_server.load("die.ogg"),
        swoosh: asset_server.load("swoosh.ogg"),
    });
    
    let tuning = difficulty_tuning(settings.selected_difficulty);
    commands.insert_resource(tuning);

    commands.insert_resource(Score::default());

    // Time Attack setup: start a 60s countdown and show UI
    if settings.selected_mode == GameMode::TimeAttack {
        commands.insert_resource(TimeAttackState { remaining: 60.0 });

        commands.spawn((
            Text::new("Time: 60"),
            TextFont {
                font_size: 22.0,
                ..default()
            },
            TextColor(Color::linear_rgba(1.0, 0.9, 0.9, 0.95)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(15.0),
                left: Val::Percent(45.0),
                ..default()
            },
            TimeDisplay,
        ));
    }
    
    // Apply theme background; Classic uses a full-screen texture instead of a flat color
    match settings.selected_theme {
        Theme::Classic => {
            commands.insert_resource(ClearColor(Color::BLACK));
            commands.spawn((
                Sprite {
                    image: asset_server.load("Background2.png"),
                    custom_size: Some(Vec2::new(window_width, window_height)),
                    ..Default::default()
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, -50.0)),
                Background,
            ));
        }
        Theme::HighContrast => {
            commands.insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)));
        }
        Theme::Minimal => {
            commands.insert_resource(ClearColor(Color::srgb(0.95, 0.95, 0.95)));
        }
    }

    commands.spawn((
        Sprite {
            image: asset_server.load("bird.png"),
            ..Default::default()
        },
        Transform::IDENTITY.with_scale(Vec3::splat(PIXEL_RATIO)),
        Bird { velocity: 0. },
    ));

    // Best Score UI - Top Right
    commands.spawn((
        Text::new("Best: 0"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::linear_rgba(1.0, 1.0, 1.0, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(15.0),
            right: Val::Px(15.0),
            ..default()
        },
        BestScoreDisplay,
    ));

    // Current Score UI - Top Left
    commands.spawn((
        Text::new("Score: 0"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::linear_rgba(1.0, 1.0, 1.0, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(15.0),
            left: Val::Px(15.0),
            ..default()
        },
        ScoreDisplay,
    ));

    let mut rand = thread_rng();
    spawn_obstacles(&mut commands, &mut rand, window_width, &pipe_image, tuning);
}

fn get_centered_pipe_position(gap_size: f32) -> f32 {
    return (OBSTACLE_HEIGHT / 2. + gap_size) * PIXEL_RATIO;
}

fn spawn_obstacles(
    commands: &mut Commands,
    rand: &mut ThreadRng,
    window_width: f32,
    pipe_image: &Handle<Image>,
    tuning: DifficultyTuning,
) {
    // Spawn paired top/bottom pipes spaced across the screen
    for i in 0..OBSTACLE_AMOUNT {
        let y_offset = generate_offset(rand, tuning.vertical_offset);
        let x_pos = window_width / 2. + (OBSTACLE_SPACING * PIXEL_RATIO * i as f32);
        spawn_obstacle(
            Vec3::X * x_pos + Vec3::Y * (get_centered_pipe_position(tuning.gap_size) + y_offset),
            1.,
            commands,
            pipe_image,
        );

        spawn_obstacle(
            Vec3::X * x_pos + Vec3::Y * (-get_centered_pipe_position(tuning.gap_size) + y_offset),
            -1.,
            commands,
            pipe_image,
        );
    }
}

fn spawn_obstacle(
    translation: Vec3,
    pipe_direction: f32,
    commands: &mut Commands,
    pipe_image: &Handle<Image>,
) {
    commands.spawn((
        Sprite {
            image: pipe_image.clone(),
            ..Default::default()
        },
        Transform::from_translation(translation).with_scale(Vec3::new(
            PIXEL_RATIO,
            PIXEL_RATIO * -pipe_direction,
            PIXEL_RATIO,
        )),
        Obstacle { 
            pipe_direction,
            scored: false,
        },
    ));
}

fn generate_offset(rand: &mut ThreadRng, vertical_offset: f32) -> f32 {
    return rand.gen_range(-vertical_offset..vertical_offset) * PIXEL_RATIO;
}

fn update_obstacles(
    time: Res<Time>,
    game_manager: Res<GameManager>,
    tuning: Res<DifficultyTuning>,
    mut obstacle_query: Query<(&mut Obstacle, &mut Transform)>,
) {
    // Scroll pipes and recycle them when they exit left
    for (mut obstacle, mut transform) in obstacle_query.iter_mut() {
        transform.translation.x -= time.delta_secs() * tuning.scroll_speed;

        if transform.translation.x + OBSTACLE_WIDTH * PIXEL_RATIO / 2.
            < -game_manager.window_dimensions.x / 2.
        {
            transform.translation.x += OBSTACLE_AMOUNT as f32 * OBSTACLE_SPACING * PIXEL_RATIO;
            let mut rand = thread_rng();
            let y_offset = generate_offset(&mut rand, tuning.vertical_offset);
            transform.translation.y =
                get_centered_pipe_position(tuning.gap_size) * obstacle.pipe_direction + y_offset;
            obstacle.scored = false;
        }
    }
}

fn update_bird(
    mut commands: Commands,
    mut bird_query: Query<(&mut Bird, &mut Transform), Without<Obstacle>>,
    mut obstacle_query: Query<(&mut Obstacle, &Transform, Entity)>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    game_manager: Res<GameManager>,
    sound_effects: Res<SoundEffects>,
    mut score: ResMut<Score>,
    mut state: ResMut<NextState<GameState>>,
    settings: Res<GameSettings>,
    tuning: Res<DifficultyTuning>,
) {
    if let Ok((mut bird, mut transform)) = bird_query.single_mut() {
        // Input + physics
        if keys.just_pressed(KeyCode::Space) {
            bird.velocity = FLAP_FORCE * tuning.flap_mult;
            commands.spawn((
            AudioPlayer::new(sound_effects.flap.clone()),
            PlaybackSettings {
                volume: Volume::Linear(0.1),
                ..PlaybackSettings::DESPAWN
        }
        ));
        }

        bird.velocity -= time.delta_secs() * GRAVITY * tuning.gravity_mult;
        transform.translation.y += bird.velocity * time.delta_secs();

        transform.rotation = Quat::from_axis_angle(
            Vec3::Z,
            f32::clamp(bird.velocity / VELOCITY_TO_ROTATION_RATIO, -90., 90.).to_radians(),
        );

        // Collision and scoring
        let mut dead = false;
        if transform.translation.y <= -game_manager.window_dimensions.y / 2. {
            dead = true;
        } else {
            for (mut obstacle, pipe_transform, _) in obstacle_query.iter_mut() {
                if !obstacle.scored && transform.translation.x > pipe_transform.translation.x {
                    if obstacle.pipe_direction == 1.0 {
                        score.current += 1;
                        if score.current > score.best {
                            score.best = score.current;
                        }
                        obstacle.scored = true;
                        commands.spawn((
                            AudioPlayer::new(sound_effects.point.clone()),
                            PlaybackSettings {
                                volume: Volume::Linear(0.1),
                                ..PlaybackSettings::DESPAWN // Fix for overlapping sounds and volume adjusted
                            }     
                        ));
                    }
                }

                if (pipe_transform.translation.y - transform.translation.y).abs()
                    < OBSTACLE_HEIGHT * PIXEL_RATIO / 2.
                    && (pipe_transform.translation.x - transform.translation.x).abs()
                        < OBSTACLE_WIDTH * PIXEL_RATIO / 2.
                {
                    dead = true;
                    break;
                }
            }
        }
        
        if dead {
            commands.spawn((
                AudioPlayer::new(sound_effects.die.clone()),
                PlaybackSettings {
                    volume: Volume::Linear(0.1),
                    ..PlaybackSettings::DESPAWN
            }
            ));

            // Save game data
            if let Some(slot_num) = settings.current_slot {
                 let save_data = load_save_slot(slot_num as u32);
                    let mut profile = save_data.map(|s| s.profile).unwrap_or_else(|| PlayerProfile {
                    name: format!("Player {}", slot_num),
                    high_score: 0,
                    total_games: 0,
                    average_score: 0.0,
                    longest_survival: 0.0,
                });
                
                profile.total_games += 1;
                if score.current > profile.high_score {
                    profile.high_score = score.current;
                }
                profile.average_score = ((profile.average_score * (profile.total_games - 1) as f32) 
                    + score.current as f32) / profile.total_games as f32;
                
                let save_slot = SaveSlot {
                    slot_number: slot_num,
                    profile,
                    mode: settings.selected_mode,
                    difficulty: settings.selected_difficulty,
                    theme: settings.selected_theme,
                    score: score.current,
                    survival_time: 0.0,
                };
                
                let _ = save_to_slot(&save_slot);
            }
            
            state.set(GameState::GameOver);
        }
    }
}

fn update_ui(
    mut score_query: Query<&mut Text, (With<ScoreDisplay>, Without<BestScoreDisplay>)>,
    mut best_score_query: Query<&mut Text, With<BestScoreDisplay>>,
    score: Res<Score>,
) {
    for mut text in score_query.iter_mut() {
        text.0 = format!("Score: {}", score.current);
    }

    for mut text in best_score_query.iter_mut() {
        text.0 = format!("Best: {}", score.best);
    }
}

fn update_time_attack(
    time: Res<Time>,
    settings: Res<GameSettings>,
    timer: Option<ResMut<TimeAttackState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut time_ui: Query<&mut Text, With<TimeDisplay>>,
) {
    if settings.selected_mode != GameMode::TimeAttack {
        return;
    }

    // Count down and end the run at zero
    let Some(mut timer) = timer else { return; };
    timer.remaining -= time.delta_secs();
    if let Some(mut txt) = time_ui.iter_mut().next() {
        txt.0 = format!("Time: {:.0}", timer.remaining.max(0.0));
    }

    if timer.remaining <= 0.0 {
        next_state.set(GameState::GameOver);
    }
}

fn handle_game_over(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(GameState::MainMenu);
    }
}

fn setup_game_over_ui(mut commands: Commands, score: Res<Score>) {
    // Simple summary screen after a run ends
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        GameOverMarker,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("GAME OVER"),
            TextFont {
                font_size: 54.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::all(Val::Px(16.0)),
                ..default()
            },
        ));

        parent.spawn((
            Text::new(format!("Score: {}", score.current)),
            TextFont {
                font_size: 32.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            Node {
                margin: UiRect::all(Val::Px(8.0)),
                ..default()
            },
        ));

        parent.spawn((
            Text::new(format!("Best: {}", score.best)),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgb(0.85, 0.95, 1.0)),
            Node {
                margin: UiRect::all(Val::Px(4.0)),
                ..default()
            },
        ));

        parent.spawn((
            Text::new("Press SPACE to return to Main Menu"),
            TextFont {
                font_size: 22.0,
                ..default()
            },
            TextColor(Color::srgb(0.75, 0.75, 0.75)),
            Node {
                margin: UiRect::top(Val::Px(24.0)),
                ..default()
            },
        ));
    });
}

fn reset_on_play_start(
    mut commands: Commands,
    mut bird_query: Query<(&mut Bird, &mut Transform)>,
    obstacle_query: Query<Entity, With<Obstacle>>,
    game_manager: Option<Res<GameManager>>,
    tuning: Option<Res<DifficultyTuning>>,
    sound_effects: Res<SoundEffects>,
    mut score: ResMut<Score>,
) {
    // Reset player state and respawn pipes before a new run
    commands.spawn((
        AudioPlayer::new(sound_effects.swoosh.clone()),
        PlaybackSettings {
            volume: Volume::Linear(0.1),
            ..PlaybackSettings::DESPAWN
        },
    ));

    score.current = 0;
    score.scored_pipes.clear();
    
    if let Ok((mut bird, mut transform)) = bird_query.single_mut() {
        bird.velocity = 0.;
        transform.translation = Vec3::ZERO;
        transform.rotation = Quat::IDENTITY;
    }

    let Some(game_manager) = game_manager else { return; };
    let Some(tuning) = tuning else { return; };

    for entity in obstacle_query.iter() {
        commands.entity(entity).despawn();
    }

    let mut rand = thread_rng();
    spawn_obstacles(
        &mut commands,
        &mut rand,
        game_manager.window_dimensions.x,
        &game_manager.pipe_image,
        *tuning,
    );
}
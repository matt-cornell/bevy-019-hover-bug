#[cfg(feature = "bevy-018")]
use bevy_018::{self as bevy, ecs as bevy_ecs};
#[cfg(feature = "bevy-019")]
use bevy_019::{self as bevy, ecs as bevy_ecs};

use bevy::picking::hover::Hovered;
use bevy::prelude::*;

#[cfg(not(any(feature = "bevy-018", feature = "bevy-019")))]
compile_error!("One of bevy-018 or bevy-019 must be enabled");

#[cfg(all(feature = "bevy-018", feature = "bevy-019"))]
compile_error!("One of bevy-018 or bevy-019 must be enabled");

#[derive(Component)]
struct Button;

const BUTTON_INERT_BG: Color = Color::srgb(0.2, 0.2, 0.2);
const BUTTON_HOVER_BG: Color = Color::srgb(0.4, 0.4, 0.4);

fn main() {
    let mut app = App::new();
    #[cfg(feature = "bevy-018")]
    app.add_plugins(bevy::input_focus::InputDispatchPlugin);
    app.add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, button_updates)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    let ui_root = commands
        .spawn(Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        })
        .id();
    let centered = commands
        .spawn((
            ChildOf(ui_root),
            BackgroundColor(Color::BLACK),
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(10)),
                ..default()
            },
        ))
        .id();
    commands.spawn((ChildOf(centered), button("Before")));
    let scroll_area = commands.spawn((
        ChildOf(centered),
        Node {
            height: px(100),
            width: percent(100),
            ..default()
        }
    )).id();
    let scroll_container = commands.spawn((
        ChildOf(scroll_area),
        Node {
            overflow: Overflow::scroll_y(),
            width: percent(100),
            ..default()
        },
        ScrollPosition(Vec2::new(0.0, 20.0)),
    )).id();
    commands.spawn((
        ChildOf(scroll_container),
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(5),
            ..default()
        }
    )).with_children(|parent| {
        for i in 1..=5 {
            parent.spawn(button(&i.to_string()));
        }
    });
}

fn button(text: &str) -> impl Bundle {
    (
        Node {
            border_radius: BorderRadius::all(px(6)),
            border: UiRect::all(px(1)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(px(6)),
            width: percent(100),
            ..default()
        },
        Button,
        Hovered::default(),
        BackgroundColor(BUTTON_INERT_BG),
        Children::spawn(Spawn(Text::new(text))),
    )
}

fn button_updates(mut interactions: Query<(Ref<Hovered>, &mut BackgroundColor), With<Button>>) {
    for (hover, mut background) in &mut interactions {
        if hover.is_changed() {
            background.0 = if hover.0 {
                BUTTON_HOVER_BG
            } else {
                BUTTON_INERT_BG
            };
        }
    }
}

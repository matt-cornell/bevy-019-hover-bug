#[cfg(feature = "bevy-018")]
extern crate bevy_018 as bevy;
#[cfg(feature = "bevy-019")]
extern crate bevy_019 as bevy;

use bevy::ecs::relationship::RelatedSpawner;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

mod scroll;

const BUTTON_INERT_BG: Color = Color::srgb(0.2, 0.2, 0.2);
const BUTTON_HOVER_BG: Color = Color::srgb(0.4, 0.4, 0.4);

fn main() {
    let mut app = App::new();
    #[cfg(feature = "bevy-018")]
    app.add_plugins((
        bevy::ui_widgets::UiWidgetsPlugins,
        bevy::input_focus::InputDispatchPlugin,
    ));
    app.add_plugins((DefaultPlugins, scroll::plugin))
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
    commands.spawn((
        ChildOf(centered),
        scroll::scrollable(
            Node {
                height: px(100),
                ..default()
            },
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(5),
                    ..default()
                },
                Children::spawn(SpawnWith(|parent: &mut RelatedSpawner<ChildOf>| {
                    for i in 1..=10 {
                        parent.spawn(button(&i.to_string()));
                    }
                })),
            ),
        ),
    ));
}

fn button(text: &str) -> impl Bundle {
    (
        Node {
            border_radius: BorderRadius::all(px(6)),
            border: UiRect::all(px(1)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(px(6)),
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

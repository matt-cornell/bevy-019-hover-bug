use super::*;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::HoverMap;
use bevy::ui_widgets::{ControlOrientation, Scrollbar};

#[cfg(feature = "bevy-018")]
use bevy::ui_widgets::{
    CoreScrollbarDragState as ScrollbarDragState, CoreScrollbarThumb as ScrollbarThumb,
};
#[cfg(feature = "bevy-019")]
use bevy::ui_widgets::{ScrollbarDragState, ScrollbarThumb};

#[cfg(feature = "bevy-018")]
fn thumb() -> ScrollbarThumb {
    ScrollbarThumb
}
#[cfg(feature = "bevy-019")]
fn thumb() -> ScrollbarThumb {
    ScrollbarThumb::default()
}

/// Marker component for entities that should be scrollable via mouse.
#[derive(Component)]
pub struct MouseScrollable;

/// UI scrolling event
#[derive(EntityEvent)]
#[entity_event(propagate, auto_propagate)]
struct Scroll {
    entity: Entity,
    /// Scroll delta in logical coordinates.
    delta: Vec2,
}

/// Wrap content in a grid with scroll bars.
///
/// The root node should have max width/height or fixed width/height fields set, and the content should
/// have visible overflow.
pub fn scrollable(root: Node, content: impl Bundle) -> impl Bundle {
    let grid_template_columns = vec![RepeatedGridTrack::flex(1, 1.)];
    let grid_template_rows = vec![RepeatedGridTrack::flex(1, 1.), RepeatedGridTrack::auto(1)];
    (
        Node {
            display: Display::Grid,
            grid_template_columns,
            grid_template_rows,
            row_gap: px(5),
            column_gap: px(5),
            ..root
        },
        Children::spawn(SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
            let scroll_area = parent
                .spawn((
                    Node {
                        overflow: Overflow::scroll_y(),
                        grid_row: GridPlacement::start(1),
                        grid_column: GridPlacement::start(1),
                        ..default()
                    },
                    MouseScrollable,
                    ScrollPosition(Vec2::ZERO),
                    Children::spawn_one(content),
                ))
                .id();
            parent.spawn((
                Node {
                    min_width: px(8),
                    grid_row: GridPlacement::start(1),
                    grid_column: GridPlacement::start(2),
                    ..default()
                },
                Scrollbar {
                    orientation: ControlOrientation::Vertical,
                    target: scroll_area,
                    min_thumb_length: 8.0,
                },
                Children::spawn(Spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        border_radius: BorderRadius::all(px(4)),
                        ..default()
                    },
                    Hovered::default(),
                    BackgroundColor(BUTTON_INERT_BG),
                    thumb(),
                ))),
            ));
        })),
    )
}

/// This is different from our typical hover code because we want to support dragging and hiding too
fn update_scrollbar_thumb(
    mut thumbs: Query<
        (
            &mut BackgroundColor,
            &ChildOf,
            &Hovered,
            &ScrollbarDragState,
        ),
        (
            With<ScrollbarThumb>,
            Or<(
                Changed<Hovered>,
                Changed<ScrollbarDragState>,
                Changed<ComputedNode>,
            )>,
        ),
    >,
    mut scrollbars: Query<(&Scrollbar, &mut Node, &ChildOf)>,
    mut parents: Query<&mut Node, Without<Scrollbar>>,
    nodes: Query<&ComputedNode>,
) {
    for (mut thumb_bg, &ChildOf(parent), Hovered(is_hovering), drag) in thumbs.iter_mut() {
        if let Ok((scrollbar, mut node, &ChildOf(parent))) = scrollbars.get_mut(parent)
            && let Ok(comp) = nodes.get(scrollbar.target)
        {
            let no_scroll = comp.size != Vec2::ZERO
                && match scrollbar.orientation {
                    ControlOrientation::Horizontal => comp.size.x >= comp.content_size.x,
                    ControlOrientation::Vertical => comp.size.y >= comp.content_size.y,
                };
            if no_scroll {
                if node.display != Display::None {
                    node.display = Display::None;
                }
                if let Ok(mut parent) = parents.get_mut(parent) {
                    match scrollbar.orientation {
                        ControlOrientation::Horizontal => parent.row_gap = px(0),
                        ControlOrientation::Vertical => parent.column_gap = px(0),
                    }
                }
                continue;
            } else if node.display == Display::None {
                node.display = Display::Flex;
                if let Ok(mut parent) = parents.get_mut(parent) {
                    match scrollbar.orientation {
                        ControlOrientation::Horizontal => parent.row_gap = px(5),
                        ControlOrientation::Vertical => parent.column_gap = px(5),
                    }
                }
            }
        }
        let color = if *is_hovering || drag.dragging {
            BUTTON_HOVER_BG
        } else {
            BUTTON_INERT_BG
        };

        if thumb_bg.0 != color {
            thumb_bg.0 = color;
        }
    }
}

const LINE_HEIGHT: f32 = 21.;

/// Injects scroll events into the UI hierarchy.
fn send_scroll_events(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);

        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta *= LINE_HEIGHT;
        }

        if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            std::mem::swap(&mut delta.x, &mut delta.y);
        }

        for pointer_map in hover_map.values() {
            for entity in pointer_map.keys().copied() {
                commands.trigger(Scroll { entity, delta });
            }
        }
    }
}

/// Event handler for scroll events.
fn on_scroll_handler(
    mut scroll: On<Scroll>,
    mut query: Query<(
        &mut ScrollPosition,
        &Node,
        &ComputedNode,
        Has<MouseScrollable>,
    )>,
) {
    let Ok((mut scroll_position, node, computed, scrollable)) = query.get_mut(scroll.entity) else {
        return;
    };

    let max_offset = (computed.content_size() - computed.size()) * computed.inverse_scale_factor();

    let delta = &mut scroll.delta;
    if scrollable {
        if node.overflow.x == OverflowAxis::Scroll && delta.x != 0. {
            // Is this node already scrolled all the way in the direction of the scroll?
            let max = if delta.x > 0. {
                scroll_position.x >= max_offset.x
            } else {
                scroll_position.x <= 0.
            };

            if !max {
                scroll_position.x += delta.x;
                // Consume the X portion of the scroll delta.
                delta.x = 0.;
            }
        }

        if node.overflow.y == OverflowAxis::Scroll && delta.y != 0. {
            // Is this node already scrolled all the way in the direction of the scroll?
            let max = if delta.y > 0. {
                scroll_position.y >= max_offset.y
            } else {
                scroll_position.y <= 0.
            };

            if !max {
                scroll_position.y += delta.y;
                // Consume the Y portion of the scroll delta.
                delta.y = 0.;
            }
        }
    }

    // Stop propagating when the delta is fully consumed.
    if *delta == Vec2::ZERO {
        scroll.propagate(false);
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (update_scrollbar_thumb, send_scroll_events))
        .add_observer(on_scroll_handler);
}

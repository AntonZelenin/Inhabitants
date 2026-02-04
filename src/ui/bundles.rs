use crate::ui::components::*;
use bevy::color::Color;
use bevy::prelude::*;

#[derive(Bundle)]
pub struct LabelBundle {
    pub text: Text,
    pub font: TextFont,
    pub color: TextColor,
    pub node: Node,
}

impl LabelBundle {
    pub fn new(text: &str, font_size: f32, color: Color) -> Self {
        Self {
            text: Text::new(text),
            font: TextFont {
                font_size,
                ..default()
            },
            color: TextColor(color),
            node: Node::default(),
        }
    }
}

#[derive(Bundle)]
pub struct ButtonBundle {
    pub button: Button,
    pub node: Node,
    pub background: BackgroundColor,
    pub interaction: Interaction,
    pub ui_button: UIButton,
    pub button_config: ButtonConfig,
}

impl ButtonBundle {
    pub fn new(
        width: f32,
        height: f32,
        normal_color: Color,
        hover_color: Color,
        pressed_color: Color,
        border_radius: f32,
    ) -> Self {
        Self {
            button: Button,
            node: Node {
                width: Val::Px(width),
                height: Val::Px(height),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(border_radius)),
                ..default()
            },
            background: BackgroundColor(normal_color),
            interaction: Interaction::None,
            ui_button: UIButton,
            button_config: ButtonConfig {
                normal_color,
                hover_color,
                pressed_color,
            },
        }
    }
}

#[derive(Bundle)]
pub struct SliderTrackBundle {
    pub node: Node,
    pub background: BackgroundColor,
}

impl SliderTrackBundle {
    pub fn new(width: f32, height: f32, color: Color) -> Self {
        Self {
            node: Node {
                width: Val::Px(width),
                height: Val::Px(height),
                position_type: PositionType::Relative,
                border_radius: BorderRadius::all(Val::Px(height / 2.0)),
                ..default()
            },
            background: BackgroundColor(color),
        }
    }
}

#[derive(Bundle)]
pub struct SliderHandleBundle {
    pub button: Button,
    pub node: Node,
    pub background: BackgroundColor,
}

impl SliderHandleBundle {
    pub fn new(size: f32, color: Color) -> Self {
        Self {
            button: Button,
            node: Node {
                width: Val::Px(size),
                height: Val::Px(size),
                position_type: PositionType::Absolute,
                top: Val::Px(-5.0), // Center vertically on track
                left: Val::Px(0.0), // Will be updated based on value
                border_radius: BorderRadius::all(Val::Px(size / 2.0)),
                ..default()
            },
            background: BackgroundColor(color),
        }
    }

    pub fn with_position(mut self, left: f32) -> Self {
        self.node.left = Val::Px(left);
        self
    }
}

#[derive(Bundle)]
pub struct ToggleBundle {
    pub button: Button,
    pub node: Node,
    pub background: BackgroundColor,
    pub interaction: Interaction,
    pub ui_toggle: UIToggle,
    pub toggle_state: ToggleState,
}

impl ToggleBundle {
    pub fn new(width: f32, height: f32, initial_state: bool, border_radius: f32) -> Self {
        let background_color = if initial_state {
            Color::srgb(0.3, 0.7, 0.3)
        } else {
            Color::srgb(0.6, 0.6, 0.6)
        };

        Self {
            button: Button,
            node: Node {
                width: Val::Px(width),
                height: Val::Px(height),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(border_radius)),
                ..default()
            },
            background: BackgroundColor(background_color),
            interaction: Interaction::None,
            ui_toggle: UIToggle,
            toggle_state: ToggleState {
                is_on: initial_state,
            },
        }
    }
}

#[derive(Bundle)]
pub struct SliderWidgetBundle {
    pub node: Node,
    pub slider: Slider,
    pub slider_value_display: SliderValueDisplay,
}

impl SliderWidgetBundle {
    pub fn new(
        width: f32,
        initial_value: f32,
        min_value: f32,
        max_value: f32,
        is_integer: bool,
    ) -> Self {
        let slider = Slider {
            current_value: initial_value,
            min_value,
            max_value,
            is_integer,
        };

        Self {
            node: Node {
                flex_direction: FlexDirection::Column,
                width: Val::Px(width),
                margin: UiRect::bottom(Val::Px(15.0)), // Margin for the whole widget
                ..default()
            },
            slider,
            slider_value_display: SliderValueDisplay,
        }
    }
}

#[derive(Bundle)]
pub struct SliderTitleRowBundle {
    pub node: Node,
}

impl SliderTitleRowBundle {
    pub fn new() -> Self {
        Self {
            node: Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
        }
    }
}

#[derive(Bundle)]
pub struct SliderTrackRowBundle {
    pub node: Node,
}

impl SliderTrackRowBundle {
    pub fn new(width: f32) -> Self {
        Self {
            node: Node {
                width: Val::Px(width),
                height: Val::Px(24.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(4.0)),
                ..default()
            },
        }
    }
}
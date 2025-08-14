use crate::ui::components::*;
use bevy::color::Color;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

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

    pub fn with_margin(mut self, margin: UiRect) -> Self {
        self.node.margin = margin;
        self
    }

    pub fn with_node(mut self, node: Node) -> Self {
        self.node = node;
        self
    }
}

#[derive(Bundle)]
pub struct SmallButtonBundle {
    pub button: Button,
    pub node: Node,
    pub border_radius: BorderRadius,
    pub interaction: Interaction,
    pub config: ButtonConfig,
}

impl SmallButtonBundle {
    pub fn new(size: f32, normal_color: Color, hover_color: Color, pressed_color: Color) -> Self {
        Self {
            button: Button,
            node: Node {
                width: Val::Px(size),
                height: Val::Px(size),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            border_radius: BorderRadius::all(Val::Px(5.0)),
            interaction: Interaction::None,
            config: ButtonConfig {
                normal_color,
                hover_color,
                pressed_color,
            },
        }
    }
}

#[derive(Bundle)]
pub struct SliderBundle {
    pub node: Node,
    pub slider: Slider,
}

impl SliderBundle {
    pub fn new(width: f32, height: f32, slider: Slider) -> Self {
        Self {
            node: Node {
                flex_direction: FlexDirection::Column,
                width: Val::Px(width),
                height: Val::Px(height),
                margin: UiRect::bottom(Val::Px(15.0)), // Margin for the whole widget
                ..default()
            },
            slider,
        }
    }
}

#[derive(Bundle)]
pub struct SliderTrackBundle {
    pub node: Node,
    pub background: BackgroundColor,
    pub border_radius: BorderRadius,
}

impl SliderTrackBundle {
    pub fn new(width: f32, height: f32, color: Color) -> Self {
        Self {
            node: Node {
                width: Val::Px(width),
                height: Val::Px(height),
                position_type: PositionType::Relative,
                ..default()
            },
            background: BackgroundColor(color),
            border_radius: BorderRadius::all(Val::Px(height / 2.0)),
        }
    }
}

#[derive(Bundle)]
pub struct SliderHandleBundle {
    pub button: Button,
    pub node: Node,
    pub background: BackgroundColor,
    pub border_radius: BorderRadius,
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
                ..default()
            },
            background: BackgroundColor(color),
            border_radius: BorderRadius::all(Val::Px(size / 2.0)),
        }
    }

    pub fn with_position(mut self, left: f32) -> Self {
        self.node.left = Val::Px(left);
        self
    }
}